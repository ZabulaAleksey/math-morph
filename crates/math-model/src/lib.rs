//! Source-neutral structural math model shared by parsers and later pipeline stages.

use std::fmt;
use std::num::NonZeroI64;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    pub const fn new(start: usize, end: usize) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub const fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpandedName {
    pub namespace_uri: Option<Arc<str>>,
    pub local_name: String,
}

impl fmt::Debug for ExpandedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExpandedName")
            .field("has_namespace", &self.namespace_uri.is_some())
            .field("local_name_bytes", &self.local_name.len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpressionOrigin {
    Source(SourceSpan),
    Derived,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum NumericBase {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl NumericBase {
    pub const fn value(self) -> u8 {
        match self {
            Self::Binary => 2,
            Self::Octal => 8,
            Self::Decimal => 10,
            Self::Hexadecimal => 16,
        }
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RealLiteral {
    pub lexeme: String,
    pub base: NumericBase,
}

impl fmt::Debug for RealLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RealLiteral")
            .field("base", &self.base)
            .field("lexeme_bytes", &self.lexeme.len())
            .finish()
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Identifier {
    pub name: String,
    pub subscript: Option<String>,
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identifier")
            .field("name_bytes", &self.name.len())
            .field("has_subscript", &self.subscript.is_some())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum MultiplicationStyle {
    Default,
    AutoSelect,
    Dot,
    NarrowDot,
    LargeDot,
    X,
    ThinSpace,
    NoSpace,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryExpression {
    pub operator: BinaryOperator,
    pub multiplication_style: Option<MultiplicationStyle>,
    pub left: Box<MathExpression>,
    pub right: Box<MathExpression>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum DefinitionKind {
    Define,
    GlobalDefine,
    LocalDefine,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum DefinitionStyle {
    Default,
    ColonEqual,
    Equal,
    TripleEqual,
    LeftArrow,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub kind: DefinitionKind,
    pub style: DefinitionStyle,
    pub target: Box<MathExpression>,
    pub value: Box<MathExpression>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    pub expression: Box<MathExpression>,
    pub unit_override: Option<Box<MathExpression>>,
    pub saved_result: Option<Box<MathExpression>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionCall {
    pub callee: Box<MathExpression>,
    pub arguments: Vec<MathExpression>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionDefinition {
    pub style: DefinitionStyle,
    pub name: Box<MathExpression>,
    pub parameters: Vec<MathExpression>,
    pub body: Box<MathExpression>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum UnaryOperator {
    AbsoluteValue,
    Conjugate,
    Factorial,
    Negate,
    SquareRoot,
    Transpose,
    Vectorize,
    VectorSum,
    Determinant,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<MathExpression>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Grouping {
    pub expression: Box<MathExpression>,
    pub unpaired: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArrayIndex {
    pub target: Box<MathExpression>,
    pub indices: Vec<MathExpression>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Matrix {
    pub rows: usize,
    pub columns: usize,
    pub elements: Vec<MathExpression>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorOrientation {
    Row,
    Column,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Vector {
    pub orientation: VectorOrientation,
    pub elements: Vec<MathExpression>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RangeExpression {
    pub start: Box<MathExpression>,
    pub next: Option<Box<MathExpression>>,
    pub end: Box<MathExpression>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub lower: Box<MathExpression>,
    pub upper: Box<MathExpression>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum IntegralAlgorithm {
    EqualInterval,
    Adaptive,
    Infinite,
    Oscillating,
    LimitEndPoints,
    Romberg,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Integral {
    pub bound_variable: Box<MathExpression>,
    pub integrand: Box<MathExpression>,
    pub bounds: Option<Bounds>,
    pub algorithm: Option<IntegralAlgorithm>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum DerivativeStyle {
    Default,
    Derivative,
    Partial,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Derivative {
    pub bound_variable: Box<MathExpression>,
    pub expression: Box<MathExpression>,
    pub degree: Option<Box<MathExpression>>,
    pub style: DerivativeStyle,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AggregateOperator {
    Summation,
    Product,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateExpression {
    pub operator: AggregateOperator,
    pub bound_variable: Box<MathExpression>,
    pub body: Box<MathExpression>,
    pub bounds: Option<Bounds>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterOrEqual,
    GreaterThan,
    LessOrEqual,
    LessThan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonExpression {
    pub operator: ComparisonOperator,
    pub left: Box<MathExpression>,
    pub right: Box<MathExpression>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum BooleanOperator {
    And,
    Or,
    Xor,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanExpression {
    pub operator: BooleanOperator,
    pub left: Box<MathExpression>,
    pub right: Box<MathExpression>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalNot {
    pub operand: Box<MathExpression>,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnitReference {
    pub unit: String,
    pub power_numerator: i64,
    pub power_denominator: NonZeroI64,
}

impl fmt::Debug for UnitReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnitReference")
            .field("unit_bytes", &self.unit.len())
            .field("power_numerator", &self.power_numerator)
            .field("power_denominator", &self.power_denominator)
            .finish()
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnitMonomial {
    pub system: Option<String>,
    pub factors: Vec<UnitReference>,
}

impl fmt::Debug for UnitMonomial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnitMonomial")
            .field("has_system", &self.system.is_some())
            .field("factor_count", &self.factors.len())
            .finish()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnitedValue {
    pub value: Box<MathExpression>,
    pub units: UnitMonomial,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum UnsupportedReason {
    UnknownExpression,
    UnknownOperator,
    UnsupportedBaseValue,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedNode {
    pub name: ExpandedName,
    pub feature: Option<ExpandedName>,
    pub span: SourceSpan,
    pub reason: UnsupportedReason,
}

impl fmt::Debug for UnsupportedNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnsupportedNode")
            .field("has_feature", &self.feature.is_some())
            .field("span", &self.span)
            .field("reason", &self.reason)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum MathExpressionKind {
    Real(RealLiteral),
    Identifier(Identifier),
    Binary(BinaryExpression),
    Definition(Definition),
    Evaluation(Evaluation),
    FunctionCall(FunctionCall),
    FunctionDefinition(FunctionDefinition),
    Unary(UnaryExpression),
    Grouping(Grouping),
    ArrayIndex(ArrayIndex),
    Matrix(Matrix),
    Vector(Vector),
    Range(RangeExpression),
    Integral(Integral),
    Derivative(Derivative),
    Aggregate(AggregateExpression),
    Comparison(ComparisonExpression),
    Boolean(BooleanExpression),
    LogicalNot(LogicalNot),
    UnitedValue(UnitedValue),
    Unsupported(UnsupportedNode),
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MathExpression {
    pub kind: MathExpressionKind,
    pub origin: ExpressionOrigin,
}

impl MathExpression {
    pub const fn source_span(&self) -> Option<SourceSpan> {
        match self.origin {
            ExpressionOrigin::Source(span) => Some(span),
            ExpressionOrigin::Derived => None,
        }
    }
}

impl fmt::Debug for MathExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match &self.kind {
            MathExpressionKind::Real(_) => "Real",
            MathExpressionKind::Identifier(_) => "Identifier",
            MathExpressionKind::Binary(_) => "Binary",
            MathExpressionKind::Definition(_) => "Definition",
            MathExpressionKind::Evaluation(_) => "Evaluation",
            MathExpressionKind::FunctionCall(_) => "FunctionCall",
            MathExpressionKind::FunctionDefinition(_) => "FunctionDefinition",
            MathExpressionKind::Unary(_) => "Unary",
            MathExpressionKind::Grouping(_) => "Grouping",
            MathExpressionKind::ArrayIndex(_) => "ArrayIndex",
            MathExpressionKind::Matrix(_) => "Matrix",
            MathExpressionKind::Vector(_) => "Vector",
            MathExpressionKind::Range(_) => "Range",
            MathExpressionKind::Integral(_) => "Integral",
            MathExpressionKind::Derivative(_) => "Derivative",
            MathExpressionKind::Aggregate(_) => "Aggregate",
            MathExpressionKind::Comparison(_) => "Comparison",
            MathExpressionKind::Boolean(_) => "Boolean",
            MathExpressionKind::LogicalNot(_) => "LogicalNot",
            MathExpressionKind::UnitedValue(_) => "UnitedValue",
            MathExpressionKind::Unsupported(_) => "Unsupported",
        };
        f.debug_struct("MathExpression")
            .field("kind", &kind)
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_and_derived_origins_serialize_round_trip() {
        for origin in [
            ExpressionOrigin::Source(SourceSpan { start: 3, end: 8 }),
            ExpressionOrigin::Derived,
        ] {
            let expression = MathExpression {
                kind: MathExpressionKind::Identifier(Identifier {
                    name: "x".into(),
                    subscript: None,
                }),
                origin,
            };
            let json = serde_json::to_string(&expression).expect("serialize AST");
            let decoded: MathExpression = serde_json::from_str(&json).expect("deserialize AST");
            assert_eq!(decoded, expression);
        }
    }

    #[test]
    fn debug_redacts_identifier_unit_and_qname_payloads() {
        let unsupported = UnsupportedNode {
            name: ExpandedName {
                namespace_uri: Some(Arc::from("urn:secret")),
                local_name: "secret-node".into(),
            },
            feature: None,
            span: SourceSpan { start: 0, end: 9 },
            reason: UnsupportedReason::UnknownExpression,
        };
        let unit = UnitReference {
            unit: "secret-unit".into(),
            power_numerator: 1,
            power_denominator: NonZeroI64::new(1).unwrap(),
        };
        let debug = format!("{unsupported:?} {unit:?}");
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn serde_wire_names_are_stable_snake_case() {
        assert_eq!(
            serde_json::to_string(&MultiplicationStyle::AutoSelect).unwrap(),
            r#""auto_select""#
        );
        let expression = MathExpression {
            kind: MathExpressionKind::Identifier(Identifier {
                name: "x".into(),
                subscript: None,
            }),
            origin: ExpressionOrigin::Derived,
        };
        let json = serde_json::to_string(&expression).unwrap();
        assert!(json.contains(r#""identifier""#));
        assert!(json.contains(r#""derived""#));
    }

    #[test]
    fn serde_rejects_unknown_fields_and_zero_denominators() {
        let unknown = serde_json::from_str::<Identifier>(
            r#"{"name":"x","subscript":null,"unexpected":"payload"}"#,
        );
        assert!(unknown.is_err());

        let zero = serde_json::from_str::<UnitReference>(
            r#"{"unit":"m","power_numerator":1,"power_denominator":0}"#,
        );
        assert!(zero.is_err());
    }
}
