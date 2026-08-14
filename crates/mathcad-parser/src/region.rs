use std::cmp::Ordering;
use std::fmt;

use crate::{Diagnostic, ExpandedName, MathAstError, MathExpression, OpaqueFragment, SourceSpan};

#[derive(Clone, Debug, PartialEq)]
pub struct SourceNumber {
    pub value: f64,
    pub lexeme: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegionLayout {
    pub top: SourceNumber,
    pub left: SourceNumber,
    pub height: SourceNumber,
    pub width: SourceNumber,
    pub z_order: i64,
}

#[derive(Clone, Eq, PartialEq)]
pub struct TextValue(pub String);

impl fmt::Debug for TextValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("TextValue")
            .field(&format_args!("<redacted:{} bytes>", self.0.len()))
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct InlineAttribute {
    pub name: ExpandedName,
    pub value: TextValue,
}

impl fmt::Debug for InlineAttribute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InlineAttribute")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineKind {
    Bold,
    Italic,
    Underline,
    StrikeOut,
    Subscript,
    Superscript,
    Color,
    Font,
    InlineAttribute,
    Link,
    Break,
    Tab,
    Space,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextRun {
    Text {
        value: TextValue,
        span: SourceSpan,
    },
    Inline {
        kind: InlineKind,
        attributes: Vec<InlineAttribute>,
        children: Vec<TextRun>,
        span: SourceSpan,
    },
    Opaque(OpaqueFragment),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextParagraph {
    pub style: TextValue,
    pub runs: Vec<TextRun>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextRegion {
    pub paragraphs: Vec<TextParagraph>,
    pub span: SourceSpan,
}

#[derive(Clone, Eq, PartialEq)]
pub struct OpaqueTableResult {
    pub span: SourceSpan,
    pub item_idref: String,
}

impl fmt::Debug for OpaqueTableResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpaqueTableResult")
            .field("span", &self.span)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultFormat {
    pub span: SourceSpan,
    pub table: Option<OpaqueTableResult>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MathParseOutcome {
    Parsed(MathExpression),
    Invalid(MathAstError),
    /// The node is intentionally outside the implemented AST subset.
    Unsupported(Diagnostic),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MathRegion {
    pub disable_calc: bool,
    pub optimize: bool,
    pub span: SourceSpan,
    pub expression_span: SourceSpan,
    pub result_format: Option<ResultFormat>,
    pub outcome: MathParseOutcome,
}

#[derive(Clone, Eq, PartialEq)]
pub struct PlotRegion {
    pub item_idref: Option<String>,
    pub disable_calc: bool,
    pub span: SourceSpan,
}

impl fmt::Debug for PlotRegion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlotRegion")
            .field("has_item_idref", &self.item_idref.is_some())
            .field("disable_calc", &self.disable_calc)
            .field("span", &self.span)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PictureKind {
    Png,
    Jpg,
    Metafile,
}

#[derive(Clone, PartialEq)]
pub struct PictureRegion {
    pub kind: PictureKind,
    pub item_idref: String,
    pub display_width: Option<SourceNumber>,
    pub display_height: Option<SourceNumber>,
    pub x_extent: Option<SourceNumber>,
    pub y_extent: Option<SourceNumber>,
    pub quality: Option<u8>,
    pub mapping_mode: Option<i64>,
    pub span: SourceSpan,
}

impl fmt::Debug for PictureRegion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PictureRegion")
            .field("kind", &self.kind)
            .field("display_width", &self.display_width)
            .field("display_height", &self.display_height)
            .field("x_extent", &self.x_extent)
            .field("y_extent", &self.y_extent)
            .field("quality", &self.quality)
            .field("mapping_mode", &self.mapping_mode)
            .field("span", &self.span)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RegionContent {
    Text(TextRegion),
    Math(MathRegion),
    Plot(PlotRegion),
    Picture(PictureRegion),
    Area(OpaqueFragment),
    Opaque(OpaqueFragment),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    pub id: u64,
    pub source_ordinal: usize,
    pub span: SourceSpan,
    pub layout: RegionLayout,
    pub content: RegionContent,
}

impl Region {
    pub(crate) fn visual_cmp(&self, other: &Self) -> Ordering {
        self.layout
            .top
            .value
            .total_cmp(&other.layout.top.value)
            .then_with(|| self.layout.left.value.total_cmp(&other.layout.left.value))
            .then_with(|| self.source_ordinal.cmp(&other.source_ordinal))
    }

    pub(crate) fn z_cmp(&self, other: &Self) -> Ordering {
        self.layout
            .z_order
            .cmp(&other.layout.z_order)
            .then_with(|| self.source_ordinal.cmp(&other.source_ordinal))
    }
}
