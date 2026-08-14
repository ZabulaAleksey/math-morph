//! Безопасная граница входных форматов Mathcad.

mod ast;
mod diagnostic;
mod format;
mod math_xml;
mod mcdx;
mod region;
mod source;
mod worksheet;
mod xml_metadata;
mod xml_worksheet;

pub use ast::{
    AggregateExpression, AggregateOperator, ArrayIndex, BinaryExpression, BinaryOperator, Bounds,
    ComparisonExpression, ComparisonOperator, Definition, DefinitionKind, DefinitionStyle,
    Derivative, DerivativeStyle, Evaluation, FunctionCall, FunctionDefinition, Grouping,
    Identifier, Integral, IntegralAlgorithm, MathAstError, MathExpression, MathExpressionKind,
    Matrix, NumericBase, RangeExpression, RealLiteral, UnaryExpression, UnaryOperator, Vector,
    VectorOrientation,
};
pub use diagnostic::{Diagnostic, DiagnosticCode, Severity};
pub use format::{FormatDetection, FormatDetector, FormatError, InputFormat};
pub use mcdx::{
    ContainerError, ContainerLimit, ContainerLimits, ContainerManifest, ContainerPart,
    ContainerPartKind, SafeMcdxReader,
};
pub use region::{
    InlineAttribute, InlineKind, MathParseOutcome, MathRegion, OpaqueTableResult, PictureKind,
    PictureRegion, PlotRegion, Region, RegionContent, RegionLayout, ResultFormat, SourceNumber,
    TextParagraph, TextRegion, TextRun, TextValue,
};
pub use source::{ExpandedName, OpaqueFragment, SourceDocument, SourceSpan};
pub use worksheet::{
    CoordinateError, CustomValueKind, Worksheet, WorksheetCustomValue, WorksheetError,
    WorksheetIdentityInfo, WorksheetLimit, WorksheetLimits, WorksheetMetadata, WorksheetParser,
    WorksheetUserData,
};
pub use xml_metadata::{
    NamespaceBinding, SchemaLocation, XmlMetadata, XmlMetadataError, XmlMetadataLimits,
    inspect_xml_metadata,
};
