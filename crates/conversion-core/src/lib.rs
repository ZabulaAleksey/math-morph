//! Application core for deterministic, fail-closed XMCD to DOCX conversion.

use document_ir::{
    BlockContentIr, BlockId, BlockIr, DocumentIrV1, DocumentIrV3, FidelityIr, FormulaDisplayModeIr,
    FormulaIr, MetadataIr, PageIr, PageMarginsIr, PageOrientationIr, ParagraphIr, PhysicalSizeIr,
    PlotIr, PlotMetadataIrV3, ProvenanceIr, SourceKindIr, TextBlockIr, TextRunIr, TextStyleIr,
    VersionedDocumentIr, VerticalAlignIr,
};
use exporter_docx::{
    DocxExporter, DocxLimits, DocxValidator, EquationBackend, OmmlError, OmmlLimits,
    WordEquationExporter,
};
use math_engine::{
    ComplexOutputMode, PrecisionPolicy, TransformationLimits, TransformationPipeline,
};
use math_model::{MathExpression, MathExpressionKind};
use mathcad_parser::{
    DiagnosticCode as ParserDiagnosticCode, FormatDetector, FormatError, InlineKind, InputFormat,
    MathParseOutcome, RegionContent, TextRun, WorksheetError, WorksheetLimits, WorksheetParser,
};
use serde::Serialize;
use std::fmt;
use std::io::{self, Write};
use thiserror::Error;

/// Requested output format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetFormat {
    Docx,
    Markdown,
    Latex,
    Html,
    Pdf,
    Json,
    Typst,
}

impl TargetFormat {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "docx" => Some(Self::Docx),
            "markdown" | "md" => Some(Self::Markdown),
            "latex" | "tex" => Some(Self::Latex),
            "html" => Some(Self::Html),
            "pdf" => Some(Self::Pdf),
            "json" => Some(Self::Json),
            "typst" => Some(Self::Typst),
            _ => None,
        }
    }
    pub const fn is_available(self) -> bool {
        matches!(self, Self::Docx)
    }
}

/// Policy for unsupported regions and malformed math.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PartialPolicy {
    #[default]
    Strict,
    AllowSafePartial,
}

/// Resource limits owned by the application boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConversionLimits {
    pub max_diagnostics: usize,
    pub max_items: usize,
    pub worksheet: WorksheetLimits,
    pub transformation: TransformationLimits,
    pub docx: DocxLimits,
}

impl Default for ConversionLimits {
    fn default() -> Self {
        Self {
            max_diagnostics: 1_024,
            max_items: 100_000,
            worksheet: WorksheetLimits::default(),
            transformation: TransformationLimits::default(),
            docx: DocxLimits::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionOptions {
    pub partial_policy: PartialPolicy,
    pub limits: ConversionLimits,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            partial_policy: PartialPolicy::Strict,
            limits: ConversionLimits::default(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ConversionRequest {
    pub bytes: Vec<u8>,
    pub file_name: Option<String>,
    pub target: TargetFormat,
    pub options: ConversionOptions,
}

impl fmt::Debug for ConversionRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConversionRequest")
            .field("byte_len", &self.bytes.len())
            .field("file_name_present", &self.file_name.is_some())
            .field("target", &self.target)
            .field("options", &self.options)
            .finish()
    }
}

impl ConversionRequest {
    pub fn new(
        bytes: Vec<u8>,
        file_name: Option<String>,
        target: TargetFormat,
        options: ConversionOptions,
    ) -> Self {
        Self {
            bytes,
            file_name,
            target,
            options,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Warning,
    RecoverableError,
    FatalError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticCode {
    FileExtensionMismatch,
    McdxContentUnsupported,
    InvalidInput,
    ParserFailure,
    UnsupportedRegion,
    InvalidMath,
    TransformationFailure,
    ExportFailure,
    ValidationFailure,
    NoExportableContent,
    DiagnosticLimitExceeded,
    ItemLimitExceeded,
    UnsupportedTarget,
    UnknownRegionContent,
    UnknownInlineNode,
    UnsupportedMathNode,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileExtensionMismatch => "FILE_EXTENSION_MISMATCH",
            Self::McdxContentUnsupported => "MCDX_CONTENT_UNSUPPORTED",
            Self::InvalidInput => "INVALID_INPUT",
            Self::ParserFailure => "PARSER_FAILURE",
            Self::UnsupportedRegion => "UNSUPPORTED_REGION",
            Self::InvalidMath => "INVALID_MATH",
            Self::TransformationFailure => "TRANSFORMATION_FAILURE",
            Self::ExportFailure => "EXPORT_FAILURE",
            Self::ValidationFailure => "VALIDATION_FAILURE",
            Self::NoExportableContent => "NO_EXPORTABLE_CONTENT",
            Self::DiagnosticLimitExceeded => "DIAGNOSTIC_LIMIT_EXCEEDED",
            Self::ItemLimitExceeded => "ITEM_LIMIT_EXCEEDED",
            Self::UnsupportedTarget => "UNSUPPORTED_TARGET",
            Self::UnknownRegionContent => "UNKNOWN_REGION_CONTENT",
            Self::UnknownInlineNode => "UNKNOWN_INLINE_NODE",
            Self::UnsupportedMathNode => "UNSUPPORTED_MATH_NODE",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A bounded, payload-free diagnostic. It intentionally contains no filename or source text.
#[derive(Clone, Copy, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub region_id: Option<u64>,
    pub source_ordinal: Option<usize>,
}

impl Diagnostic {
    pub const fn new(
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        region_id: Option<u64>,
        source_ordinal: Option<usize>,
    ) -> Self {
        Self {
            code,
            severity,
            region_id,
            source_ordinal,
        }
    }
}

impl fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Diagnostic")
            .field("code", &self.code)
            .field("severity", &self.severity)
            .field("region_id", &self.region_id)
            .field("source_ordinal", &self.source_ordinal)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticsLimits {
    pub max_diagnostics: usize,
}

impl Default for DiagnosticsLimits {
    fn default() -> Self {
        Self {
            max_diagnostics: ConversionLimits::default().max_diagnostics,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticsCollector {
    limits: DiagnosticsLimits,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
#[error("diagnostic limit exceeded")]
pub struct DiagnosticLimitExceeded;

impl DiagnosticsCollector {
    pub fn new(limits: DiagnosticsLimits) -> Self {
        Self {
            limits,
            diagnostics: Vec::new(),
        }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) -> Result<(), DiagnosticLimitExceeded> {
        if self.diagnostics.len() >= self.limits.max_diagnostics {
            return Err(DiagnosticLimitExceeded);
        }
        self.diagnostics.push(diagnostic);
        Ok(())
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Completed,
    CompletedWithWarnings,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct DiagnosticCounts {
    pub warnings: usize,
    pub recoverable_errors: usize,
    pub fatal_errors: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Fidelity {
    Exact,
    Approximate,
    Unsupported,
    FallbackRendered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct ReportItem {
    pub region_id: u64,
    pub source_ordinal: usize,
    pub fidelity: Fidelity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConversionReport {
    pub status: ReportStatus,
    pub counts: DiagnosticCounts,
    pub diagnostics: Vec<Diagnostic>,
    pub items: Vec<ReportItem>,
    pub numeric_options: NumericOptionsReport,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NumericOptionsReport {
    pub complex_mode: &'static str,
    pub computation_digits: u16,
    pub display_digits: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InspectionReport {
    pub detected_format: String,
    pub region_count: usize,
    pub diagnostics: Vec<DiagnosticCode>,
}
impl InspectionReport {
    pub fn to_json(&self, max_bytes: usize) -> Result<Vec<u8>, ReportSerializationError> {
        serialize_envelope("inspection_report", self, max_bytes)
    }
}

impl ConversionReport {
    pub fn new(diagnostics: Vec<Diagnostic>, items: Vec<ReportItem>) -> Self {
        Self::new_with_numeric_options(diagnostics, items, NumericConversionOptions::default())
    }

    pub fn new_with_numeric_options(
        diagnostics: Vec<Diagnostic>,
        items: Vec<ReportItem>,
        numeric_options: NumericConversionOptions,
    ) -> Self {
        let mut counts = DiagnosticCounts::default();
        for diagnostic in &diagnostics {
            match diagnostic.severity {
                DiagnosticSeverity::Warning => counts.warnings += 1,
                DiagnosticSeverity::RecoverableError => counts.recoverable_errors += 1,
                DiagnosticSeverity::FatalError => counts.fatal_errors += 1,
            }
        }
        let status = if counts.warnings + counts.recoverable_errors > 0 {
            ReportStatus::CompletedWithWarnings
        } else {
            ReportStatus::Completed
        };
        Self {
            status,
            counts,
            diagnostics,
            items,
            numeric_options: NumericOptionsReport {
                complex_mode: match numeric_options.complex_mode {
                    ComplexOutputMode::Algebraic => "algebraic",
                    ComplexOutputMode::Polar => "polar",
                    ComplexOutputMode::Both => "both",
                },
                computation_digits: numeric_options.precision.computation_digits(),
                display_digits: numeric_options.precision.display_digits(),
            },
        }
    }
    pub fn to_json(&self, max_bytes: usize) -> Result<Vec<u8>, ReportSerializationError> {
        serialize_envelope("conversion_report", self, max_bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReportSerializationError {
    InvalidLimit,
    OutputLimitExceeded,
    SerializationFailure,
}

#[derive(Serialize)]
struct ReportEnvelope<'a, T> {
    schema_version: u16,
    kind: &'static str,
    report: &'a T,
}

fn serialize_envelope<T: Serialize>(
    kind: &'static str,
    value: &T,
    max_bytes: usize,
) -> Result<Vec<u8>, ReportSerializationError> {
    if max_bytes == 0 || max_bytes > 64 * 1024 * 1024 {
        return Err(ReportSerializationError::InvalidLimit);
    }
    struct Limited {
        bytes: Vec<u8>,
        max: usize,
        exceeded: bool,
    }
    impl Write for Limited {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let Some(next) = self.bytes.len().checked_add(bytes.len()) else {
                self.exceeded = true;
                return Err(io::Error::other("limit"));
            };
            if next > self.max {
                self.exceeded = true;
                return Err(io::Error::other("limit"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut output = Limited {
        bytes: Vec::new(),
        max: max_bytes,
        exceeded: false,
    };
    let envelope = ReportEnvelope {
        schema_version: 1,
        kind,
        report: value,
    };
    if serde_json::to_writer(&mut output, &envelope).is_err() {
        return Err(if output.exceeded {
            ReportSerializationError::OutputLimitExceeded
        } else {
            ReportSerializationError::SerializationFailure
        });
    }
    Ok(output.bytes)
}

#[derive(Clone, Eq, PartialEq)]
pub struct ConversionOutcome {
    pub artifact: Vec<u8>,
    pub report: ConversionReport,
    pub document: VersionedDocumentIr,
}

impl fmt::Debug for ConversionOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConversionOutcome")
            .field("artifact_len", &self.artifact.len())
            .field("report", &self.report)
            .field("document", &self.document)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureCode {
    UnsupportedTarget,
    InvalidInput,
    McdxContentUnsupported,
    ParserFailure,
    StrictUnsupportedContent,
    NoExportableContent,
    DiagnosticLimitExceeded,
    ItemLimitExceeded,
    TransformationFailure,
    IrValidationFailure,
    ExportFailure,
    DocxValidationFailure,
}

/// Conversion errors deliberately expose only stable machine-level codes.
#[derive(Clone, Eq, PartialEq)]
pub struct ConversionFailure {
    pub code: FailureCode,
    pub diagnostics: Vec<Diagnostic>,
}

impl fmt::Debug for ConversionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConversionFailure")
            .field("code", &self.code)
            .field("diagnostic_count", &self.diagnostics.len())
            .finish()
    }
}

impl fmt::Display for ConversionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.code {
            FailureCode::UnsupportedTarget => "unsupported target format",
            FailureCode::InvalidInput => "invalid input",
            FailureCode::McdxContentUnsupported => "MCDX content is unsupported",
            FailureCode::ParserFailure => "worksheet parsing failed",
            FailureCode::StrictUnsupportedContent => "unsupported content under strict policy",
            FailureCode::NoExportableContent => "no exportable content",
            FailureCode::DiagnosticLimitExceeded => "diagnostic limit exceeded",
            FailureCode::ItemLimitExceeded => "item limit exceeded",
            FailureCode::TransformationFailure => "math transformation failed",
            FailureCode::IrValidationFailure => "document IR validation failed",
            FailureCode::ExportFailure => "DOCX export failed",
            FailureCode::DocxValidationFailure => "DOCX validation failed",
        })
    }
}

impl std::error::Error for ConversionFailure {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumericConversionOptions {
    pub complex_mode: ComplexOutputMode,
    pub precision: PrecisionPolicy,
}

impl Default for NumericConversionOptions {
    fn default() -> Self {
        Self {
            complex_mode: ComplexOutputMode::Algebraic,
            precision: PrecisionPolicy::new(15, 15).expect("valid default precision"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ConversionPipeline {
    numeric_options: NumericConversionOptions,
}

impl ConversionPipeline {
    pub const fn new() -> Self {
        Self {
            numeric_options: NumericConversionOptions {
                complex_mode: ComplexOutputMode::Algebraic,
                precision: match PrecisionPolicy::new(15, 15) {
                    Ok(value) => value,
                    Err(_) => panic!("valid default precision"),
                },
            },
        }
    }

    pub const fn with_numeric_options(
        complex_mode: ComplexOutputMode,
        precision: PrecisionPolicy,
    ) -> Self {
        Self {
            numeric_options: NumericConversionOptions {
                complex_mode,
                precision,
            },
        }
    }

    pub const fn numeric_options(&self) -> NumericConversionOptions {
        self.numeric_options
    }

    pub fn convert_with_numeric_options(
        &self,
        request: ConversionRequest,
        complex_mode: ComplexOutputMode,
        precision: PrecisionPolicy,
    ) -> Result<ConversionOutcome, ConversionFailure> {
        Self::with_numeric_options(complex_mode, precision).convert(request)
    }

    pub fn inspect(
        &self,
        bytes: &[u8],
        file_name: Option<&str>,
        limits: ConversionLimits,
    ) -> Result<InspectionReport, ConversionFailure> {
        let detection = FormatDetector::default()
            .detect(bytes, file_name)
            .map_err(|error| {
                bounded_fatal_failure(
                    format_error_code(error),
                    DiagnosticCode::InvalidInput,
                    Vec::new(),
                    limits.max_diagnostics,
                )
            })?;
        let mut diagnostics = detection
            .diagnostics
            .iter()
            .map(|_| DiagnosticCode::FileExtensionMismatch)
            .collect::<Vec<_>>();
        if diagnostics.len() > limits.max_diagnostics {
            return Err(bounded_fatal_failure(
                FailureCode::DiagnosticLimitExceeded,
                DiagnosticCode::DiagnosticLimitExceeded,
                Vec::new(),
                limits.max_diagnostics,
            ));
        }
        let (detected_format, region_count) = match detection.detected {
            InputFormat::Xmcd => {
                let worksheet = WorksheetParser::new(limits.worksheet)
                    .parse(bytes)
                    .map_err(|error| {
                        bounded_fatal_failure(
                            worksheet_error_code(error),
                            DiagnosticCode::ParserFailure,
                            Vec::new(),
                            limits.max_diagnostics,
                        )
                    })?;
                for diagnostic in &worksheet.diagnostics {
                    if diagnostics.len() >= limits.max_diagnostics {
                        return Err(bounded_fatal_failure(
                            FailureCode::DiagnosticLimitExceeded,
                            DiagnosticCode::DiagnosticLimitExceeded,
                            Vec::new(),
                            limits.max_diagnostics,
                        ));
                    }
                    diagnostics.push(match diagnostic.code {
                        ParserDiagnosticCode::FileExtensionMismatch => {
                            DiagnosticCode::FileExtensionMismatch
                        }
                        ParserDiagnosticCode::UnknownRegionContent => {
                            DiagnosticCode::UnknownRegionContent
                        }
                        ParserDiagnosticCode::UnknownInlineNode => {
                            DiagnosticCode::UnknownInlineNode
                        }
                        ParserDiagnosticCode::UnsupportedMathNode => {
                            DiagnosticCode::UnsupportedMathNode
                        }
                        ParserDiagnosticCode::UnknownContainerPart => {
                            DiagnosticCode::UnsupportedRegion
                        }
                    });
                }
                ("xmcd".to_owned(), worksheet.regions.len())
            }
            InputFormat::Mcdx => {
                return Err(bounded_fatal_failure(
                    FailureCode::McdxContentUnsupported,
                    DiagnosticCode::McdxContentUnsupported,
                    Vec::new(),
                    limits.max_diagnostics,
                ));
            }
            InputFormat::Unknown => {
                return Err(bounded_fatal_failure(
                    FailureCode::InvalidInput,
                    DiagnosticCode::InvalidInput,
                    Vec::new(),
                    limits.max_diagnostics,
                ));
            }
        };
        Ok(InspectionReport {
            detected_format,
            region_count,
            diagnostics,
        })
    }

    pub fn convert(
        &self,
        request: ConversionRequest,
    ) -> Result<ConversionOutcome, ConversionFailure> {
        if request.target != TargetFormat::Docx {
            return Err(bounded_fatal_failure(
                FailureCode::UnsupportedTarget,
                DiagnosticCode::UnsupportedTarget,
                Vec::new(),
                request.options.limits.max_diagnostics,
            ));
        }
        let limits = request.options.limits;
        let mut diagnostics = DiagnosticsCollector::new(DiagnosticsLimits {
            max_diagnostics: limits.max_diagnostics,
        });
        let detector = FormatDetector::default();
        let detection = detector
            .detect(&request.bytes, request.file_name.as_deref())
            .map_err(|error| {
                bounded_fatal_failure(
                    format_error_code(error),
                    DiagnosticCode::InvalidInput,
                    Vec::new(),
                    request.options.limits.max_diagnostics,
                )
            })?;
        if detection.detected == InputFormat::Mcdx {
            return Err(bounded_fatal_failure(
                FailureCode::McdxContentUnsupported,
                DiagnosticCode::McdxContentUnsupported,
                Vec::new(),
                request.options.limits.max_diagnostics,
            ));
        }
        if detection.detected != InputFormat::Xmcd {
            return Err(bounded_fatal_failure(
                FailureCode::InvalidInput,
                DiagnosticCode::InvalidInput,
                Vec::new(),
                request.options.limits.max_diagnostics,
            ));
        }
        for diagnostic in detection.diagnostics {
            push_diag(
                &mut diagnostics,
                Diagnostic::new(
                    DiagnosticCode::FileExtensionMismatch,
                    DiagnosticSeverity::Warning,
                    None,
                    None,
                ),
            )?;
            let _ = diagnostic;
        }

        let worksheet = WorksheetParser::new(limits.worksheet)
            .parse(&request.bytes)
            .map_err(|error| {
                bounded_fatal_failure(
                    worksheet_error_code(error),
                    DiagnosticCode::ParserFailure,
                    diagnostics.diagnostics().to_vec(),
                    limits.max_diagnostics,
                )
            })?;
        for diagnostic in worksheet.diagnostics.iter().copied() {
            let mapped = match diagnostic.code {
                ParserDiagnosticCode::FileExtensionMismatch => {
                    DiagnosticCode::FileExtensionMismatch
                }
                ParserDiagnosticCode::UnknownRegionContent => DiagnosticCode::UnknownRegionContent,
                ParserDiagnosticCode::UnknownInlineNode => DiagnosticCode::UnknownInlineNode,
                ParserDiagnosticCode::UnsupportedMathNode => DiagnosticCode::UnsupportedMathNode,
                ParserDiagnosticCode::UnknownContainerPart => DiagnosticCode::UnsupportedRegion,
            };
            push_diag(
                &mut diagnostics,
                Diagnostic::new(
                    mapped,
                    DiagnosticSeverity::Warning,
                    None,
                    diagnostic.entry_index,
                ),
            )?;
        }

        let transform = TransformationPipeline::with_limits(
            math_engine::NotationProfile::faithful(),
            limits.transformation,
        );
        let mut blocks = Vec::new();
        let mut plot_metadata = Vec::new();
        let mut items = Vec::new();
        for region in worksheet.visual_order() {
            if items.len() >= limits.max_items {
                return Err(bounded_fatal_failure(
                    FailureCode::ItemLimitExceeded,
                    DiagnosticCode::ItemLimitExceeded,
                    diagnostics.into_diagnostics(),
                    limits.max_diagnostics,
                ));
            }
            let provenance = ProvenanceIr {
                source_kind: SourceKindIr::Xmcd,
                region_id: Some(region.id),
                source_ordinal: Some(region.source_ordinal),
                span: Some(region.span),
            };
            match &region.content {
                RegionContent::Text(text) => {
                    let mut unsupported = false;
                    let mut paragraphs = Vec::new();
                    for paragraph in &text.paragraphs {
                        let mut runs = Vec::new();
                        let mut style = TextStyleIr::default();
                        for run in &paragraph.runs {
                            flatten_text_run(run, &mut style, &mut runs, &mut unsupported);
                        }
                        paragraphs.push(ParagraphIr { runs });
                    }
                    if unsupported
                        && handle_unsupported(
                            &request.options,
                            &mut diagnostics,
                            &mut items,
                            region.id,
                            region.source_ordinal,
                        )?
                    {
                        continue;
                    }
                    blocks.push(BlockIr {
                        id: BlockId(format!("region-{}", region.id)),
                        provenance,
                        fidelity: FidelityIr::Approximate,
                        placement: None,
                        content: BlockContentIr::Text(TextBlockIr { paragraphs }),
                    });
                    items.push(ReportItem {
                        region_id: region.id,
                        source_ordinal: region.source_ordinal,
                        fidelity: Fidelity::Approximate,
                    });
                }
                RegionContent::Math(math) => match &math.outcome {
                    MathParseOutcome::Parsed { expression, .. }
                        if !contains_unsupported(expression) =>
                    {
                        let transformed = transform.transform(expression).map_err(|_| {
                            bounded_fatal_failure(
                                FailureCode::TransformationFailure,
                                DiagnosticCode::TransformationFailure,
                                diagnostics.diagnostics().to_vec(),
                                limits.max_diagnostics,
                            )
                        })?;
                        let omml = WordEquationExporter::new(OmmlLimits {
                            max_depth: limits.docx.max_equation_depth,
                            max_nodes: limits.docx.max_equation_nodes,
                            max_output_bytes: limits.docx.max_equation_output_bytes,
                        });
                        if let Err(error) = omml.export_expression(&transformed.display) {
                            if is_recoverable_omml_error(error)
                                && handle_unsupported(
                                    &request.options,
                                    &mut diagnostics,
                                    &mut items,
                                    region.id,
                                    region.source_ordinal,
                                )?
                            {
                                continue;
                            }
                            return Err(bounded_fatal_failure(
                                FailureCode::ExportFailure,
                                DiagnosticCode::ExportFailure,
                                diagnostics.diagnostics().to_vec(),
                                limits.max_diagnostics,
                            ));
                        }
                        blocks.push(BlockIr {
                            id: BlockId(format!("region-{}", region.id)),
                            provenance,
                            fidelity: FidelityIr::Approximate,
                            placement: None,
                            content: BlockContentIr::Equation(FormulaIr {
                                original: Some(expression.clone()),
                                display: transformed.display,
                                mode: FormulaDisplayModeIr::Display,
                            }),
                        });
                        items.push(ReportItem {
                            region_id: region.id,
                            source_ordinal: region.source_ordinal,
                            fidelity: Fidelity::Approximate,
                        });
                    }
                    _ => {
                        if handle_unsupported(
                            &request.options,
                            &mut diagnostics,
                            &mut items,
                            region.id,
                            region.source_ordinal,
                        )? {
                            continue;
                        }
                    }
                },
                RegionContent::Plot(plot) => {
                    let _ = handle_unsupported(
                        &request.options,
                        &mut diagnostics,
                        &mut items,
                        region.id,
                        region.source_ordinal,
                    )?;
                    let block_id = BlockId(format!("region-{}", region.id));
                    blocks.push(BlockIr {
                        id: block_id.clone(),
                        provenance,
                        fidelity: FidelityIr::Unsupported,
                        placement: None,
                        content: BlockContentIr::Plot(PlotIr { preview: None }),
                    });
                    plot_metadata.push(PlotMetadataIrV3 {
                        block_id,
                        item_idref: plot.item_idref.clone(),
                        disable_calc: plot.disable_calc,
                    });
                }
                _ => {
                    if handle_unsupported(
                        &request.options,
                        &mut diagnostics,
                        &mut items,
                        region.id,
                        region.source_ordinal,
                    )? {
                        continue;
                    }
                }
            }
        }
        if !blocks.iter().any(|block| {
            matches!(
                block.content,
                BlockContentIr::Text(_) | BlockContentIr::Image(_) | BlockContentIr::Equation(_)
            )
        }) {
            return Err(bounded_fatal_failure(
                FailureCode::NoExportableContent,
                DiagnosticCode::NoExportableContent,
                diagnostics.diagnostics().to_vec(),
                limits.max_diagnostics,
            ));
        }
        let document = DocumentIrV1 {
            metadata: worksheet_metadata(worksheet.metadata.as_ref()),
            pages: vec![PageIr {
                size: PhysicalSizeIr::a4_portrait(),
                orientation: PageOrientationIr::Portrait,
                margins: PageMarginsIr {
                    top_um: 12_700,
                    right_um: 12_700,
                    bottom_um: 12_700,
                    left_um: 12_700,
                },
                blocks,
            }],
        };
        document.validate().map_err(|_| {
            bounded_fatal_failure(
                FailureCode::IrValidationFailure,
                DiagnosticCode::ValidationFailure,
                diagnostics.diagnostics().to_vec(),
                limits.max_diagnostics,
            )
        })?;
        let export_document = DocumentIrV1 {
            metadata: document.metadata.clone(),
            pages: document
                .pages
                .iter()
                .map(|page| PageIr {
                    size: page.size,
                    orientation: page.orientation,
                    margins: page.margins,
                    blocks: page
                        .blocks
                        .iter()
                        .filter(|block| {
                            matches!(
                                block.content,
                                BlockContentIr::Text(_)
                                    | BlockContentIr::Image(_)
                                    | BlockContentIr::Equation(_)
                            )
                        })
                        .cloned()
                        .collect(),
                })
                .collect(),
        };
        let exporter = DocxExporter::with_config(
            limits.docx,
            exporter_docx::DocxExportConfig {
                equation_backend: EquationBackend::WordOmml,
            },
        );
        let artifact = exporter
            .export(&export_document, &EmptyAssetResolver)
            .map_err(|_| {
                bounded_fatal_failure(
                    FailureCode::ExportFailure,
                    DiagnosticCode::ExportFailure,
                    diagnostics.diagnostics().to_vec(),
                    limits.max_diagnostics,
                )
            })?;
        DocxValidator::new(limits.docx)
            .validate(&artifact)
            .map_err(|_| {
                bounded_fatal_failure(
                    FailureCode::DocxValidationFailure,
                    DiagnosticCode::ValidationFailure,
                    diagnostics.diagnostics().to_vec(),
                    limits.max_diagnostics,
                )
            })?;
        let document = if plot_metadata.is_empty() {
            VersionedDocumentIr::v1(document)
        } else {
            VersionedDocumentIr::v3(DocumentIrV3 {
                document,
                plot_metadata,
            })
        };
        document.validate().map_err(|_| {
            bounded_fatal_failure(
                FailureCode::IrValidationFailure,
                DiagnosticCode::ValidationFailure,
                diagnostics.diagnostics().to_vec(),
                limits.max_diagnostics,
            )
        })?;
        Ok(ConversionOutcome {
            artifact,
            report: ConversionReport::new_with_numeric_options(
                diagnostics.into_diagnostics(),
                items,
                self.numeric_options,
            ),
            document,
        })
    }
}

impl Default for ConversionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct EmptyAssetResolver;

impl document_ir::ports::AssetResolver for EmptyAssetResolver {
    fn resolve(
        &self,
        _reference: &document_ir::AssetRefIr,
    ) -> Result<document_ir::ports::ResolvedAsset, document_ir::ports::AssetResolveError> {
        Err(document_ir::ports::AssetResolveError::Unavailable)
    }
}

fn handle_unsupported(
    options: &ConversionOptions,
    diagnostics: &mut DiagnosticsCollector,
    items: &mut Vec<ReportItem>,
    region_id: u64,
    source_ordinal: usize,
) -> Result<bool, ConversionFailure> {
    let diagnostic = Diagnostic::new(
        DiagnosticCode::UnsupportedRegion,
        DiagnosticSeverity::RecoverableError,
        Some(region_id),
        Some(source_ordinal),
    );
    if options.partial_policy == PartialPolicy::Strict {
        push_diag(diagnostics, diagnostic)?;
        return Err(failure(
            FailureCode::StrictUnsupportedContent,
            diagnostics.clone().into_diagnostics(),
        ));
    }
    push_diag(diagnostics, diagnostic)?;
    items.push(ReportItem {
        region_id,
        source_ordinal,
        fidelity: Fidelity::Unsupported,
    });
    Ok(true)
}

fn push_diag(
    collector: &mut DiagnosticsCollector,
    diagnostic: Diagnostic,
) -> Result<(), ConversionFailure> {
    collector.push(diagnostic).map_err(|_| {
        bounded_fatal_failure(
            FailureCode::DiagnosticLimitExceeded,
            DiagnosticCode::DiagnosticLimitExceeded,
            collector.clone().into_diagnostics(),
            collector.limits.max_diagnostics,
        )
    })
}

fn failure(code: FailureCode, diagnostics: Vec<Diagnostic>) -> ConversionFailure {
    ConversionFailure { code, diagnostics }
}

fn bounded_fatal_failure(
    code: FailureCode,
    diagnostic_code: DiagnosticCode,
    mut diagnostics: Vec<Diagnostic>,
    max_diagnostics: usize,
) -> ConversionFailure {
    let terminal = Diagnostic::new(diagnostic_code, DiagnosticSeverity::FatalError, None, None);
    if max_diagnostics == 0 {
        diagnostics.clear();
    } else if diagnostics.len() >= max_diagnostics {
        diagnostics.truncate(max_diagnostics);
        diagnostics[max_diagnostics - 1] = terminal;
    } else {
        diagnostics.push(terminal);
    }
    failure(code, diagnostics)
}

fn format_error_code(error: FormatError) -> FailureCode {
    match error {
        FormatError::Xml(_) | FormatError::Container(_) => FailureCode::InvalidInput,
    }
}

fn worksheet_error_code(error: WorksheetError) -> FailureCode {
    match error {
        WorksheetError::LimitExceeded(_) => FailureCode::ParserFailure,
        WorksheetError::DoctypeForbidden
        | WorksheetError::UnsupportedEncoding
        | WorksheetError::MalformedXml
        | WorksheetError::UnknownNamespacePrefix
        | WorksheetError::UnsupportedRoot
        | WorksheetError::UnsupportedVersion
        | WorksheetError::MissingRegionId
        | WorksheetError::DuplicateRegionId
        | WorksheetError::MalformedRegionId
        | WorksheetError::InvalidCoordinate { .. }
        | WorksheetError::MalformedZOrder
        | WorksheetError::InvalidMathExpressionCount
        | WorksheetError::UnsupportedMathNamespace
        | WorksheetError::MalformedPicture
        | WorksheetError::MalformedResultFormat
        | WorksheetError::MalformedCustomValue
        | WorksheetError::MissingTextStyle
        | WorksheetError::MalformedBoolean => FailureCode::ParserFailure,
    }
}

fn is_recoverable_omml_error(error: OmmlError) -> bool {
    matches!(
        error,
        OmmlError::UnsupportedExpression
            | OmmlError::IdentifierSubscriptUnsupported
            | OmmlError::SemanticGroupingRequired
    )
}

fn worksheet_metadata(metadata: Option<&mathcad_parser::WorksheetMetadata>) -> MetadataIr {
    let Some(metadata) = metadata else {
        return MetadataIr::default();
    };
    let user = &metadata.user_data;
    MetadataIr {
        title: user.title.clone(),
        creator: user.author.clone(),
        description: user.description.clone(),
        language: None,
        keywords: user
            .keywords
            .as_deref()
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn flatten_text_run(
    run: &TextRun,
    style: &mut TextStyleIr,
    output: &mut Vec<TextRunIr>,
    unsupported: &mut bool,
) {
    match run {
        TextRun::Text { value, .. } => output.push(TextRunIr {
            text: value.0.clone(),
            style: style.clone(),
        }),
        TextRun::Opaque(_) => *unsupported = true,
        TextRun::Inline {
            kind,
            attributes,
            children,
            ..
        } => {
            let previous = style.clone();
            match kind {
                InlineKind::Bold => style.bold = true,
                InlineKind::Italic => style.italic = true,
                InlineKind::Underline => style.underline = true,
                InlineKind::StrikeOut => style.strike = true,
                InlineKind::Subscript => style.vertical_align = VerticalAlignIr::Subscript,
                InlineKind::Superscript => style.vertical_align = VerticalAlignIr::Superscript,
                InlineKind::Break => output.push(TextRunIr {
                    text: "\n".to_owned(),
                    style: style.clone(),
                }),
                InlineKind::Tab => output.push(TextRunIr {
                    text: "\t".to_owned(),
                    style: style.clone(),
                }),
                InlineKind::Space => output.push(TextRunIr {
                    text: " ".to_owned(),
                    style: style.clone(),
                }),
                InlineKind::Color
                | InlineKind::Font
                | InlineKind::InlineAttribute
                | InlineKind::Link => {
                    // These style/link payloads are not yet mapped losslessly to Document IR.
                    *unsupported = true;
                }
            }
            if !matches!(
                kind,
                InlineKind::Break | InlineKind::Tab | InlineKind::Space
            ) {
                for child in children {
                    flatten_text_run(child, style, output, unsupported);
                }
            }
            *style = previous;
            let _ = attributes;
        }
    }
}

fn contains_unsupported(expression: &MathExpression) -> bool {
    match &expression.kind {
        MathExpressionKind::Unsupported(_) => true,
        MathExpressionKind::Real(_) | MathExpressionKind::Identifier(_) => false,
        MathExpressionKind::Binary(value) => {
            contains_unsupported(&value.left) || contains_unsupported(&value.right)
        }
        MathExpressionKind::Definition(value) => {
            contains_unsupported(&value.target) || contains_unsupported(&value.value)
        }
        MathExpressionKind::Evaluation(value) => {
            contains_unsupported(&value.expression)
                || value
                    .unit_override
                    .as_deref()
                    .is_some_and(contains_unsupported)
                || value
                    .saved_result
                    .as_deref()
                    .is_some_and(contains_unsupported)
        }
        MathExpressionKind::FunctionCall(value) => {
            contains_unsupported(&value.callee) || value.arguments.iter().any(contains_unsupported)
        }
        MathExpressionKind::FunctionDefinition(value) => {
            contains_unsupported(&value.name)
                || value.parameters.iter().any(contains_unsupported)
                || contains_unsupported(&value.body)
        }
        MathExpressionKind::Unary(value) => contains_unsupported(&value.operand),
        MathExpressionKind::Grouping(value) => contains_unsupported(&value.expression),
        MathExpressionKind::ArrayIndex(value) => {
            contains_unsupported(&value.target) || value.indices.iter().any(contains_unsupported)
        }
        MathExpressionKind::Matrix(value) => value.elements.iter().any(contains_unsupported),
        MathExpressionKind::Vector(value) => value.elements.iter().any(contains_unsupported),
        MathExpressionKind::Range(value) => {
            contains_unsupported(&value.start)
                || value.next.as_deref().is_some_and(contains_unsupported)
                || contains_unsupported(&value.end)
        }
        MathExpressionKind::Integral(value) => {
            contains_unsupported(&value.bound_variable)
                || contains_unsupported(&value.integrand)
                || value.bounds.as_ref().is_some_and(|b| {
                    contains_unsupported(&b.lower) || contains_unsupported(&b.upper)
                })
        }
        MathExpressionKind::Derivative(value) => {
            contains_unsupported(&value.bound_variable)
                || contains_unsupported(&value.expression)
                || value.degree.as_deref().is_some_and(contains_unsupported)
        }
        MathExpressionKind::Aggregate(value) => {
            contains_unsupported(&value.bound_variable)
                || contains_unsupported(&value.body)
                || value.bounds.as_ref().is_some_and(|b| {
                    contains_unsupported(&b.lower) || contains_unsupported(&b.upper)
                })
        }
        MathExpressionKind::Comparison(value) => {
            contains_unsupported(&value.left) || contains_unsupported(&value.right)
        }
        MathExpressionKind::Boolean(value) => {
            contains_unsupported(&value.left) || contains_unsupported(&value.right)
        }
        MathExpressionKind::LogicalNot(value) => contains_unsupported(&value.operand),
        MathExpressionKind::UnitedValue(value) => contains_unsupported(&value.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_are_bounded_and_payload_free() {
        let mut collector = DiagnosticsCollector::new(DiagnosticsLimits { max_diagnostics: 1 });
        collector
            .push(Diagnostic::new(
                DiagnosticCode::InvalidInput,
                DiagnosticSeverity::FatalError,
                None,
                None,
            ))
            .unwrap();
        assert!(
            collector
                .push(Diagnostic::new(
                    DiagnosticCode::InvalidInput,
                    DiagnosticSeverity::FatalError,
                    Some(2),
                    Some(3)
                ))
                .is_err()
        );
        assert!(!format!("{collector:?}").contains("secret"));
    }

    #[test]
    fn report_counts_recoverable_diagnostics() {
        let report = ConversionReport::new(
            vec![Diagnostic::new(
                DiagnosticCode::UnsupportedRegion,
                DiagnosticSeverity::RecoverableError,
                Some(1),
                Some(0),
            )],
            Vec::new(),
        );
        assert_eq!(report.status, ReportStatus::CompletedWithWarnings);
        assert_eq!(report.counts.recoverable_errors, 1);
    }

    #[test]
    fn numeric_options_cross_the_conversion_boundary_and_are_reported() {
        let precision = PrecisionPolicy::new(40, 12).expect("precision");
        let pipeline = ConversionPipeline::with_numeric_options(ComplexOutputMode::Both, precision);
        let configured = pipeline.numeric_options();
        assert_eq!(configured.complex_mode, ComplexOutputMode::Both);
        assert_eq!(configured.precision, precision);

        let report = ConversionReport::new_with_numeric_options(Vec::new(), Vec::new(), configured);
        assert_eq!(report.numeric_options.complex_mode, "both");
        assert_eq!(report.numeric_options.computation_digits, 40);
        assert_eq!(report.numeric_options.display_digits, 12);
    }

    #[test]
    fn report_preserves_diagnostic_order() {
        let ordered = vec![
            Diagnostic::new(
                DiagnosticCode::FileExtensionMismatch,
                DiagnosticSeverity::Warning,
                None,
                None,
            ),
            Diagnostic::new(
                DiagnosticCode::UnsupportedRegion,
                DiagnosticSeverity::RecoverableError,
                Some(7),
                Some(2),
            ),
        ];
        let report = ConversionReport::new(ordered.clone(), Vec::new());
        assert_eq!(report.diagnostics, ordered);
    }

    #[test]
    fn failure_debug_does_not_include_source_payload() {
        let failure = ConversionFailure {
            code: FailureCode::InvalidInput,
            diagnostics: Vec::new(),
        };
        assert!(!format!("{failure:?}").contains("worksheet"));
    }

    #[test]
    fn target_format_is_explicit() {
        assert_eq!(TargetFormat::Docx, TargetFormat::Docx);
    }

    #[test]
    fn conversion_request_debug_is_redacted_to_bounded_metadata() {
        let request = ConversionRequest::new(
            b"SECRET_FORMULA".to_vec(),
            Some("/absolute/private/worksheet.xmcd".to_owned()),
            TargetFormat::Docx,
            ConversionOptions::default(),
        );
        let rendered = format!("{request:?}");
        assert!(rendered.contains("byte_len: 14"));
        assert!(rendered.contains("file_name_present: true"));
        assert!(!rendered.contains("SECRET_FORMULA"));
        assert!(!rendered.contains("worksheet.xmcd"));
        assert!(!rendered.contains("/absolute"));
    }

    #[test]
    fn conversion_outcome_debug_is_redacted_to_artifact_length_and_report() {
        let outcome = ConversionOutcome {
            artifact: b"SECRET_DOCX".to_vec(),
            report: ConversionReport::new(Vec::new(), Vec::new()),
            document: VersionedDocumentIr::v1(DocumentIrV1 {
                metadata: MetadataIr::default(),
                pages: vec![PageIr {
                    size: PhysicalSizeIr::a4_portrait(),
                    orientation: PageOrientationIr::Portrait,
                    margins: PageMarginsIr {
                        top_um: 1,
                        right_um: 1,
                        bottom_um: 1,
                        left_um: 1,
                    },
                    blocks: Vec::new(),
                }],
            }),
        };
        let rendered = format!("{outcome:?}");
        assert!(rendered.contains("artifact_len: 11"));
        assert!(rendered.contains("report:"));
        assert!(!rendered.contains("SECRET_DOCX"));
    }
}
