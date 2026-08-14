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
}
