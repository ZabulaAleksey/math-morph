use thiserror::Error;

/// A bounded resource category enforced while producing MathML.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathMlLimit {
    Depth,
    Nodes,
    OutputBytes,
}

/// A fail-closed MathML export error that never includes formula content.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum MathMlError {
    #[error("math expression is not supported by this MathML stage")]
    UnsupportedExpression,
    #[error("math literal is invalid")]
    InvalidLiteral,
    #[error("math expression is structurally invalid")]
    InvalidExpression,
    #[error("math text is not valid XML 1.0 content")]
    InvalidXmlText,
    #[error("MathML limit exceeded: {0:?}")]
    LimitExceeded(MathMlLimit),
}
