use std::fmt;

use thiserror::Error;

use crate::SourceSpan;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Eq, PartialEq)]
pub struct RealLiteral {
    pub lexeme: String,
    pub base: NumericBase,
}

impl fmt::Debug for RealLiteral {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RealLiteral")
            .field("base", &self.base)
            .field("lexeme_bytes", &self.lexeme.len())
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Identifier {
    pub name: String,
    pub subscript: Option<String>,
}

impl fmt::Debug for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Identifier")
            .field("name_bytes", &self.name.len())
            .field("has_subscript", &self.subscript.is_some())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefinitionKind {
    Define,
    GlobalDefine,
    LocalDefine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefinitionStyle {
    Default,
    ColonEqual,
    Equal,
    TripleEqual,
    LeftArrow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Definition {
    pub kind: DefinitionKind,
    pub style: DefinitionStyle,
    pub target: Box<MathExpression>,
    pub value: Box<MathExpression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub expression: Box<MathExpression>,
    pub unit_override: Option<Box<MathExpression>>,
    pub saved_result: Option<Box<MathExpression>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionCall {
    pub callee: Box<MathExpression>,
    pub arguments: Vec<MathExpression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDefinition {
    pub style: DefinitionStyle,
    pub name: Box<MathExpression>,
    pub parameters: Vec<MathExpression>,
    pub body: Box<MathExpression>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<MathExpression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grouping {
    pub expression: Box<MathExpression>,
    pub unpaired: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayIndex {
    pub target: Box<MathExpression>,
    pub indices: Vec<MathExpression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub columns: usize,
    pub elements: Vec<MathExpression>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorOrientation {
    Row,
    Column,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vector {
    pub orientation: VectorOrientation,
    pub elements: Vec<MathExpression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RangeExpression {
    pub start: Box<MathExpression>,
    pub next: Option<Box<MathExpression>>,
    pub end: Box<MathExpression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bounds {
    pub lower: Box<MathExpression>,
    pub upper: Box<MathExpression>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegralAlgorithm {
    EqualInterval,
    Adaptive,
    Infinite,
    Oscillating,
    LimitEndPoints,
    Romberg,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Integral {
    pub bound_variable: Box<MathExpression>,
    pub integrand: Box<MathExpression>,
    pub bounds: Option<Bounds>,
    pub algorithm: Option<IntegralAlgorithm>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DerivativeStyle {
    Default,
    Derivative,
    Partial,
}

impl fmt::Debug for Integral {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Integral")
            .field("bound_variable", &self.bound_variable)
            .field("integrand", &self.integrand)
            .field("bounds", &self.bounds)
            .field("has_algorithm", &self.algorithm.is_some())
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Derivative {
    pub bound_variable: Box<MathExpression>,
    pub expression: Box<MathExpression>,
    pub degree: Option<Box<MathExpression>>,
    pub style: DerivativeStyle,
}

impl fmt::Debug for Derivative {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Derivative")
            .field("bound_variable", &self.bound_variable)
            .field("expression", &self.expression)
            .field("degree", &self.degree)
            .field("style", &self.style)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AggregateOperator {
    Summation,
    Product,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateExpression {
    pub operator: AggregateOperator,
    pub bound_variable: Box<MathExpression>,
    pub body: Box<MathExpression>,
    pub bounds: Option<Bounds>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterOrEqual,
    GreaterThan,
    LessOrEqual,
    LessThan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComparisonExpression {
    pub operator: ComparisonOperator,
    pub left: Box<MathExpression>,
    pub right: Box<MathExpression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BinaryExpression {
    pub operator: BinaryOperator,
    pub left: Box<MathExpression>,
    pub right: Box<MathExpression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
}

#[derive(Clone, Eq, PartialEq)]
pub struct MathExpression {
    pub kind: MathExpressionKind,
    pub span: SourceSpan,
}

impl fmt::Debug for MathExpression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        };
        formatter
            .debug_struct("MathExpression")
            .field("kind", &kind)
            .field("span", &self.span)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum MathAstError {
    #[error("math AST node limit exceeded")]
    NodeLimitExceeded,
    #[error("math real literal has an unsupported radix")]
    InvalidRadix,
    #[error("math real literal is malformed for its radix")]
    MalformedReal,
    #[error("math literal content is malformed")]
    MalformedLiteral,
    #[error("math binary operation has invalid arity")]
    WrongBinaryArity {
        operator: BinaryOperator,
        actual: usize,
    },
    #[error("math definition target is invalid")]
    InvalidDefinitionTarget,
    #[error("math definition style is invalid")]
    InvalidDefinitionStyle,
    #[error("math evaluation structure is malformed")]
    MalformedEvaluation,
    #[error("math function call has invalid arity")]
    WrongFunctionArity { actual: usize },
    #[error("math function name is invalid")]
    InvalidFunctionName,
    #[error("math function parameter is invalid")]
    InvalidFunctionParameter,
    #[error("math function definition structure is malformed")]
    MalformedFunctionDefinition,
    #[error("math unary operation has invalid arity")]
    WrongUnaryArity {
        operator: UnaryOperator,
        actual: usize,
    },
    #[error("math array index has invalid arity")]
    WrongArrayIndexArity { actual: usize },
    #[error("math grouping structure is malformed")]
    MalformedGrouping,
    #[error("math boolean attribute is malformed")]
    InvalidBooleanAttribute,
    #[error("math array index structure is malformed")]
    MalformedArrayIndex,
    #[error("math matrix dimensions are malformed")]
    InvalidMatrixDimensions,
    #[error("math matrix element count does not match its dimensions")]
    MatrixElementCountMismatch { expected: usize, actual: usize },
    #[error("math matrix element limit exceeded")]
    MatrixElementLimitExceeded,
    #[error("math range structure is malformed")]
    MalformedRange,
    #[error("math calculus structure is malformed")]
    MalformedCalculus,
    #[error("math integral algorithm is invalid")]
    InvalidIntegralAlgorithm,
    #[error("math derivative style is invalid")]
    InvalidDerivativeStyle,
    #[error("math bound variable is invalid")]
    InvalidBoundVariable,
    #[error("math comparison has invalid arity")]
    WrongComparisonArity {
        operator: ComparisonOperator,
        actual: usize,
    },
}
