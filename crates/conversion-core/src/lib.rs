//! Application core for deterministic, fail-closed XMCD to DOCX conversion.

use document_ir::{
    BlockContentIr, BlockId, BlockIr, DocumentIrV1, FidelityIr, FormulaDisplayModeIr, FormulaIr,
    MetadataIr, PageIr, PageMarginsIr, PageOrientationIr, ParagraphIr, PhysicalSizeIr,
    ProvenanceIr, SourceKindIr, TextBlockIr, TextRunIr, TextStyleIr, VerticalAlignIr,
};
use exporter_docx::{
    DocxExporter, DocxLimits, DocxValidator, EquationBackend, OmmlError, OmmlLimits,
    WordEquationExporter,
};
use math_engine::{TransformationLimits, TransformationPipeline};
use math_model::{MathExpression, MathExpressionKind};
use mathcad_parser::{
    DiagnosticCode as ParserDiagnosticCode, FormatDetector, FormatError, InlineKind, InputFormat,
    MathParseOutcome, RegionContent, TextRun, WorksheetError, WorksheetLimits, WorksheetParser,
};
use std::fmt;
use thiserror::Error;

/// Requested output format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetFormat {
    Docx,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionRequest {
    pub bytes: Vec<u8>,
    pub file_name: Option<String>,
    pub target: TargetFormat,
    pub options: ConversionOptions,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Warning,
    RecoverableError,
    FatalError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
#[derive(Clone, Copy, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReportStatus {
    Completed,
    CompletedWithWarnings,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticCounts {
    pub warnings: usize,
    pub recoverable_errors: usize,
    pub fatal_errors: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fidelity {
    Exact,
    Approximate,
    Unsupported,
    FallbackRendered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReportItem {
    pub region_id: u64,
    pub source_ordinal: usize,
    pub fidelity: Fidelity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionReport {
    pub status: ReportStatus,
    pub counts: DiagnosticCounts,
    pub diagnostics: Vec<Diagnostic>,
    pub items: Vec<ReportItem>,
}

impl ConversionReport {
    pub fn new(diagnostics: Vec<Diagnostic>, items: Vec<ReportItem>) -> Self {
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
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionOutcome {
    pub artifact: Vec<u8>,
    pub report: ConversionReport,
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

#[derive(Clone, Copy, Debug)]
pub struct ConversionPipeline;

impl ConversionPipeline {
    pub const fn new() -> Self {
        Self
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
        if blocks.is_empty() {
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
        let exporter = DocxExporter::with_config(
            limits.docx,
            exporter_docx::DocxExportConfig {
                equation_backend: EquationBackend::WordOmml,
            },
        );
        let artifact = exporter
            .export(&document, &EmptyAssetResolver)
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
        Ok(ConversionOutcome {
            artifact,
            report: ConversionReport::new(diagnostics.into_diagnostics(), items),
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
}
