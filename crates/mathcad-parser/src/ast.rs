pub use math_model::*;

use thiserror::Error;

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
    #[error("math multiplication style is invalid")]
    InvalidMultiplicationStyle,
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
    #[error("math boolean operation has invalid arity")]
    WrongBooleanArity {
        operator: BooleanOperator,
        actual: usize,
    },
    #[error("math logical not has invalid arity")]
    WrongLogicalNotArity { actual: usize },
    #[error("math boolean operator marker must be empty")]
    NonEmptyBooleanMarker,
    #[error("math boolean operator QName is invalid")]
    InvalidBooleanOperatorQName,
    #[error("math united value structure is malformed")]
    MalformedUnitedValue,
    #[error("math unit monomial structure is malformed")]
    MalformedUnitMonomial,
    #[error("math unit reference is missing a non-empty unit")]
    MissingUnitName,
    #[error("math unit power is malformed")]
    InvalidUnitPower,
    #[error("math unit power denominator is zero")]
    ZeroUnitPowerDenominator,
    #[error("math unit factor limit exceeded")]
    UnitFactorLimitExceeded,
    #[error("math unit QName is invalid")]
    InvalidUnitQName,
}
