use std::collections::BTreeSet;
use std::fmt;

use math_model::{ExpressionOrigin, MathExpression, MathExpressionKind, SourceSpan};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DOCUMENT_IR_SCHEMA_VERSION: u16 = 1;
const MAX_PAGES: usize = 1_024;
const MAX_BLOCKS: usize = 100_000;
const MAX_TABLE_DEPTH: usize = 16;
const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024;
const MAX_SINGLE_VALUE_BYTES: usize = 1024 * 1024;
const MAX_ID_BYTES: usize = 128;
const MAX_EXPRESSION_DEPTH: usize = 256;
const MAX_EXPRESSION_NODES: usize = 100_000;

#[derive(Clone, Eq, PartialEq)]
pub enum VersionedDocumentIr {
    V1(DocumentIrV1),
}

impl VersionedDocumentIr {
    pub const fn v1(document: DocumentIrV1) -> Self {
        Self::V1(document)
    }

    pub const fn schema_version(&self) -> u16 {
        match self {
            Self::V1(_) => DOCUMENT_IR_SCHEMA_VERSION,
        }
    }

    pub const fn as_v1(&self) -> &DocumentIrV1 {
        match self {
            Self::V1(document) => document,
        }
    }

    pub fn validate(&self) -> Result<(), DocumentIrValidationError> {
        self.as_v1().validate()
    }
}

impl fmt::Debug for VersionedDocumentIr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VersionedDocumentIr")
            .field("schema_version", &self.schema_version())
            .field("page_count", &self.as_v1().pages.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentIrV1 {
    pub metadata: MetadataIr,
    pub pages: Vec<PageIr>,
}

impl DocumentIrV1 {
    pub fn validate(&self) -> Result<(), DocumentIrValidationError> {
        if self.pages.is_empty() {
            return Err(DocumentIrValidationError::MissingPage);
        }
        if self.pages.len() > MAX_PAGES {
            return Err(DocumentIrValidationError::PageLimitExceeded);
        }
        validate_metadata(&self.metadata)?;
        let mut state = ValidationState::default();
        for page in &self.pages {
            validate_page(page, &mut state)?;
        }
        Ok(())
    }
}

impl fmt::Debug for DocumentIrV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DocumentIrV1")
            .field("metadata_present", &self.metadata.present_field_count())
            .field("page_count", &self.pages.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataIr {
    pub title: Option<String>,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub keywords: Vec<String>,
}

impl MetadataIr {
    fn present_field_count(&self) -> usize {
        [
            &self.title,
            &self.creator,
            &self.description,
            &self.language,
        ]
        .into_iter()
        .filter(|value| value.is_some())
        .count()
            + usize::from(!self.keywords.is_empty())
    }
}

impl fmt::Debug for MetadataIr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MetadataIr")
            .field("present_fields", &self.present_field_count())
            .finish()
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PageIr {
    pub size: PhysicalSizeIr,
    pub orientation: PageOrientationIr,
    pub margins: PageMarginsIr,
    pub blocks: Vec<BlockIr>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageOrientationIr {
    Portrait,
    Landscape,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalSizeIr {
    pub width_um: u64,
    pub height_um: u64,
}

impl PhysicalSizeIr {
    pub const fn new(width_um: u64, height_um: u64) -> Option<Self> {
        if width_um == 0 || height_um == 0 {
            None
        } else {
            Some(Self {
                width_um,
                height_um,
            })
        }
    }

    pub const fn letter_portrait() -> Self {
        Self {
            width_um: 215_900,
            height_um: 279_400,
        }
    }

    pub const fn a4_portrait() -> Self {
        Self {
            width_um: 210_000,
            height_um: 297_000,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PageMarginsIr {
    pub top_um: u64,
    pub right_um: u64,
    pub bottom_um: u64,
    pub left_um: u64,
}

#[derive(Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct BlockId(pub String);

impl fmt::Debug for BlockId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("BlockId")
            .field(&format_args!("{} bytes", self.0.len()))
            .finish()
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BlockIr {
    pub id: BlockId,
    pub provenance: ProvenanceIr,
    pub fidelity: FidelityIr,
    pub placement: Option<BlockPlacementIr>,
    pub content: BlockContentIr,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockContentIr {
    Text(TextBlockIr),
    Equation(FormulaIr),
    Table(TableIr),
    Image(ImageIr),
    Plot(PlotIr),
    Diagram(DiagramIr),
    Unsupported(UnsupportedBlockIr),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FidelityIr {
    Exact,
    Approximate,
    Unsupported,
    FallbackRendered,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKindIr {
    Xmcd,
    Mcdx,
    Derived,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceIr {
    pub source_kind: SourceKindIr,
    pub region_id: Option<u64>,
    pub source_ordinal: Option<usize>,
    pub span: Option<SourceSpan>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BlockPlacementIr {
    pub x_um: i64,
    pub y_um: i64,
    pub width_um: u64,
    pub height_um: u64,
    pub z_index: i64,
    pub visual_ordinal: Option<usize>,
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TextBlockIr {
    pub paragraphs: Vec<ParagraphIr>,
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParagraphIr {
    pub runs: Vec<TextRunIr>,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TextRunIr {
    pub text: String,
    pub style: TextStyleIr,
}

impl fmt::Debug for TextRunIr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TextRunIr")
            .field("text_bytes", &self.text.len())
            .field("style", &self.style)
            .finish()
    }
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TextStyleIr {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub vertical_align: VerticalAlignIr,
    pub font_family: Option<String>,
    pub font_size_half_points: Option<u16>,
    pub color: Option<RgbColorIr>,
}

impl fmt::Debug for TextStyleIr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TextStyleIr")
            .field("bold", &self.bold)
            .field("italic", &self.italic)
            .field("underline", &self.underline)
            .field("strike", &self.strike)
            .field("vertical_align", &self.vertical_align)
            .field("has_font_family", &self.font_family.is_some())
            .field("font_size_half_points", &self.font_size_half_points)
            .field("has_color", &self.color.is_some())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlignIr {
    #[default]
    Baseline,
    Subscript,
    Superscript,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RgbColorIr {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormulaIr {
    pub original: Option<MathExpression>,
    pub display: MathExpression,
    pub mode: FormulaDisplayModeIr,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormulaDisplayModeIr {
    Inline,
    Display,
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TableIr {
    pub rows: Vec<TableRowIr>,
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TableRowIr {
    pub cells: Vec<TableCellIr>,
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TableCellIr {
    pub blocks: Vec<BlockIr>,
}

#[derive(Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct AssetId(pub String);

impl fmt::Debug for AssetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("AssetId")
            .field(&format_args!("{} bytes", self.0.len()))
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaTypeIr {
    Png,
    Jpeg,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetRefIr {
    pub id: AssetId,
    pub media_type: MediaTypeIr,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImageIr {
    pub asset: AssetRefIr,
    pub alt_text: Option<String>,
    pub size: Option<PhysicalSizeIr>,
}

impl fmt::Debug for ImageIr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ImageIr")
            .field("media_type", &self.asset.media_type)
            .field("has_alt_text", &self.alt_text.is_some())
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlotIr {
    pub preview: Option<ImageIr>,
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagramIr {
    pub preview: Option<ImageIr>,
    pub primitives: Vec<DiagramPrimitiveIr>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagramPrimitiveIr {
    pub kind: DiagramPrimitiveKindIr,
    pub bounds: Option<BlockPlacementIr>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramPrimitiveKindIr {
    Shape,
    Connector,
    Text,
    Group,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedBlockIr {
    pub kind: String,
}

impl fmt::Debug for UnsupportedBlockIr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UnsupportedBlockIr")
            .field("kind_bytes", &self.kind.len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum DocumentIrValidationError {
    #[error("document IR must contain at least one page")]
    MissingPage,
    #[error("document IR page limit exceeded")]
    PageLimitExceeded,
    #[error("document IR contains an invalid physical size")]
    InvalidPhysicalSize,
    #[error("document IR contains invalid page margins")]
    InvalidPageMargins,
    #[error("document IR contains an invalid machine identifier")]
    InvalidIdentifier,
    #[error("document IR contains duplicate block identifiers")]
    DuplicateBlockId,
    #[error("document IR block limit exceeded")]
    BlockLimitExceeded,
    #[error("document IR table nesting limit exceeded")]
    TableDepthLimitExceeded,
    #[error("document IR text limit exceeded")]
    TextLimitExceeded,
    #[error("document IR contains invalid text formatting")]
    InvalidTextStyle,
    #[error("document IR contains invalid block placement")]
    InvalidPlacement,
    #[error("document IR contains invalid source provenance")]
    InvalidProvenance,
    #[error("document IR contains invalid formula provenance")]
    InvalidFormula,
}

#[derive(Default)]
struct ValidationState {
    block_ids: BTreeSet<String>,
    block_count: usize,
    text_bytes: usize,
}

fn validate_metadata(metadata: &MetadataIr) -> Result<(), DocumentIrValidationError> {
    for value in [
        metadata.title.as_deref(),
        metadata.creator.as_deref(),
        metadata.description.as_deref(),
        metadata.language.as_deref(),
    ]
    .into_iter()
    .flatten()
    .chain(metadata.keywords.iter().map(String::as_str))
    {
        validate_value_length(value)?;
    }
    Ok(())
}

fn validate_page(
    page: &PageIr,
    state: &mut ValidationState,
) -> Result<(), DocumentIrValidationError> {
    validate_physical_size(page.size)?;
    let horizontal = page
        .margins
        .left_um
        .checked_add(page.margins.right_um)
        .ok_or(DocumentIrValidationError::InvalidPageMargins)?;
    let vertical = page
        .margins
        .top_um
        .checked_add(page.margins.bottom_um)
        .ok_or(DocumentIrValidationError::InvalidPageMargins)?;
    if horizontal >= page.size.width_um || vertical >= page.size.height_um {
        return Err(DocumentIrValidationError::InvalidPageMargins);
    }
    for block in &page.blocks {
        validate_block(block, 0, state)?;
    }
    Ok(())
}

fn validate_block(
    block: &BlockIr,
    table_depth: usize,
    state: &mut ValidationState,
) -> Result<(), DocumentIrValidationError> {
    state.block_count = state
        .block_count
        .checked_add(1)
        .ok_or(DocumentIrValidationError::BlockLimitExceeded)?;
    if state.block_count > MAX_BLOCKS {
        return Err(DocumentIrValidationError::BlockLimitExceeded);
    }
    validate_machine_id(&block.id.0)?;
    if !state.block_ids.insert(block.id.0.clone()) {
        return Err(DocumentIrValidationError::DuplicateBlockId);
    }
    validate_provenance(block.provenance)?;
    if let Some(placement) = block.placement {
        if placement.width_um == 0 || placement.height_um == 0 {
            return Err(DocumentIrValidationError::InvalidPlacement);
        }
    }
    match &block.content {
        BlockContentIr::Text(text) => validate_text(text, state),
        BlockContentIr::Equation(formula) => validate_formula(formula),
        BlockContentIr::Table(table) => validate_table(table, table_depth, state),
        BlockContentIr::Image(image) => validate_image(image),
        BlockContentIr::Plot(plot) => plot.preview.as_ref().map_or(Ok(()), validate_image),
        BlockContentIr::Diagram(diagram) => {
            if let Some(preview) = &diagram.preview {
                validate_image(preview)?;
            }
            for primitive in &diagram.primitives {
                if primitive
                    .bounds
                    .is_some_and(|bounds| bounds.width_um == 0 || bounds.height_um == 0)
                {
                    return Err(DocumentIrValidationError::InvalidPlacement);
                }
            }
            Ok(())
        }
        BlockContentIr::Unsupported(unsupported) => validate_value_length(&unsupported.kind),
    }
}

fn validate_table(
    table: &TableIr,
    table_depth: usize,
    state: &mut ValidationState,
) -> Result<(), DocumentIrValidationError> {
    let next_depth = table_depth
        .checked_add(1)
        .ok_or(DocumentIrValidationError::TableDepthLimitExceeded)?;
    if next_depth > MAX_TABLE_DEPTH {
        return Err(DocumentIrValidationError::TableDepthLimitExceeded);
    }
    for row in &table.rows {
        for cell in &row.cells {
            for block in &cell.blocks {
                validate_block(block, next_depth, state)?;
            }
        }
    }
    Ok(())
}

fn validate_text(
    text: &TextBlockIr,
    state: &mut ValidationState,
) -> Result<(), DocumentIrValidationError> {
    for paragraph in &text.paragraphs {
        for run in &paragraph.runs {
            validate_value_length(&run.text)?;
            state.text_bytes = state
                .text_bytes
                .checked_add(run.text.len())
                .ok_or(DocumentIrValidationError::TextLimitExceeded)?;
            if state.text_bytes > MAX_TEXT_BYTES {
                return Err(DocumentIrValidationError::TextLimitExceeded);
            }
            if run.style.font_size_half_points == Some(0) {
                return Err(DocumentIrValidationError::InvalidTextStyle);
            }
            if let Some(font) = &run.style.font_family {
                validate_value_length(font)?;
            }
        }
    }
    Ok(())
}

fn validate_formula(formula: &FormulaIr) -> Result<(), DocumentIrValidationError> {
    let mut nodes = 0;
    if let Some(original) = &formula.original {
        validate_expression(original, 0, &mut nodes)?;
    }
    validate_expression(&formula.display, 0, &mut nodes)
}

fn validate_expression(
    expression: &MathExpression,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), DocumentIrValidationError> {
    if depth > MAX_EXPRESSION_DEPTH {
        return Err(DocumentIrValidationError::InvalidFormula);
    }
    *nodes = nodes
        .checked_add(1)
        .ok_or(DocumentIrValidationError::InvalidFormula)?;
    if *nodes > MAX_EXPRESSION_NODES {
        return Err(DocumentIrValidationError::InvalidFormula);
    }
    if let ExpressionOrigin::Source(span) = expression.origin {
        validate_span(span)?;
    }
    let next = depth
        .checked_add(1)
        .ok_or(DocumentIrValidationError::InvalidFormula)?;
    let mut child = |value: &MathExpression| validate_expression(value, next, nodes);
    match &expression.kind {
        MathExpressionKind::Real(value) => validate_value_length(&value.lexeme),
        MathExpressionKind::Identifier(value) => {
            validate_value_length(&value.name)?;
            if let Some(subscript) = &value.subscript {
                validate_value_length(subscript)?;
            }
            Ok(())
        }
        MathExpressionKind::Binary(value) => {
            child(&value.left)?;
            child(&value.right)
        }
        MathExpressionKind::Definition(value) => {
            child(&value.target)?;
            child(&value.value)
        }
        MathExpressionKind::Evaluation(value) => {
            child(&value.expression)?;
            if let Some(unit) = &value.unit_override {
                child(unit)?;
            }
            if let Some(result) = &value.saved_result {
                child(result)?;
            }
            Ok(())
        }
        MathExpressionKind::FunctionCall(value) => {
            child(&value.callee)?;
            for argument in &value.arguments {
                child(argument)?;
            }
            Ok(())
        }
        MathExpressionKind::FunctionDefinition(value) => {
            child(&value.name)?;
            for parameter in &value.parameters {
                child(parameter)?;
            }
            child(&value.body)
        }
        MathExpressionKind::Unary(value) => child(&value.operand),
        MathExpressionKind::Grouping(value) => child(&value.expression),
        MathExpressionKind::ArrayIndex(value) => {
            child(&value.target)?;
            for index in &value.indices {
                child(index)?;
            }
            Ok(())
        }
        MathExpressionKind::Matrix(value) => {
            for element in &value.elements {
                child(element)?;
            }
            Ok(())
        }
        MathExpressionKind::Vector(value) => {
            for element in &value.elements {
                child(element)?;
            }
            Ok(())
        }
        MathExpressionKind::Range(value) => {
            child(&value.start)?;
            if let Some(next_value) = &value.next {
                child(next_value)?;
            }
            child(&value.end)
        }
        MathExpressionKind::Integral(value) => {
            child(&value.bound_variable)?;
            child(&value.integrand)?;
            if let Some(bounds) = &value.bounds {
                child(&bounds.lower)?;
                child(&bounds.upper)?;
            }
            Ok(())
        }
        MathExpressionKind::Derivative(value) => {
            child(&value.bound_variable)?;
            child(&value.expression)?;
            if let Some(degree) = &value.degree {
                child(degree)?;
            }
            Ok(())
        }
        MathExpressionKind::Aggregate(value) => {
            child(&value.bound_variable)?;
            child(&value.body)?;
            if let Some(bounds) = &value.bounds {
                child(&bounds.lower)?;
                child(&bounds.upper)?;
            }
            Ok(())
        }
        MathExpressionKind::Comparison(value) => {
            child(&value.left)?;
            child(&value.right)
        }
        MathExpressionKind::Boolean(value) => {
            child(&value.left)?;
            child(&value.right)
        }
        MathExpressionKind::LogicalNot(value) => child(&value.operand),
        MathExpressionKind::UnitedValue(value) => {
            child(&value.value)?;
            if let Some(system) = &value.units.system {
                validate_value_length(system)?;
            }
            for factor in &value.units.factors {
                validate_value_length(&factor.unit)?;
            }
            Ok(())
        }
        MathExpressionKind::Unsupported(value) => validate_span(value.span),
    }
}

fn validate_image(image: &ImageIr) -> Result<(), DocumentIrValidationError> {
    validate_machine_id(&image.asset.id.0)?;
    if let Some(alt_text) = &image.alt_text {
        validate_value_length(alt_text)?;
    }
    if let Some(size) = image.size {
        validate_physical_size(size)?;
    }
    Ok(())
}

fn validate_physical_size(size: PhysicalSizeIr) -> Result<(), DocumentIrValidationError> {
    if size.width_um == 0 || size.height_um == 0 {
        Err(DocumentIrValidationError::InvalidPhysicalSize)
    } else {
        Ok(())
    }
}

fn validate_provenance(provenance: ProvenanceIr) -> Result<(), DocumentIrValidationError> {
    if let Some(span) = provenance.span {
        validate_span(span).map_err(|_| DocumentIrValidationError::InvalidProvenance)?;
    }
    Ok(())
}

fn validate_span(span: SourceSpan) -> Result<(), DocumentIrValidationError> {
    if span.start <= span.end {
        Ok(())
    } else {
        Err(DocumentIrValidationError::InvalidFormula)
    }
}

fn validate_value_length(value: &str) -> Result<(), DocumentIrValidationError> {
    if value.len() > MAX_SINGLE_VALUE_BYTES {
        Err(DocumentIrValidationError::TextLimitExceeded)
    } else {
        Ok(())
    }
}

fn validate_machine_id(value: &str) -> Result<(), DocumentIrValidationError> {
    if value.is_empty()
        || value.len() > MAX_ID_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        Err(DocumentIrValidationError::InvalidIdentifier)
    } else {
        Ok(())
    }
}
