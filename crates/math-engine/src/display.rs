//! Explicit display-mode boundary without numeric evaluation.
use crate::{SubstitutionEngine, SubstitutionError, SubstitutionResult, SymbolTable};
use math_model::MathExpression;
use std::fmt;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayMode {
    Substitution,
    DetailedTrace,
    ResultOnly,
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum DisplayError {
    Substitution(SubstitutionError),
    ResultUnavailable,
}
impl fmt::Debug for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(match self {
            Self::Substitution(_) => "Substitution",
            Self::ResultUnavailable => "ResultUnavailable",
        })
        .finish()
    }
}
impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Substitution(_) => "display substitution failed",
            Self::ResultUnavailable => "numeric result is unavailable",
        })
    }
}
impl std::error::Error for DisplayError {}
pub fn display(
    mode: DisplayMode,
    engine: &SubstitutionEngine,
    expression: &MathExpression,
    source_ordinal: usize,
    symbols: &SymbolTable,
) -> Result<SubstitutionResult, DisplayError> {
    match mode {
        DisplayMode::Substitution | DisplayMode::DetailedTrace => engine
            .once(expression, source_ordinal, symbols)
            .map_err(DisplayError::Substitution),
        DisplayMode::ResultOnly => Err(DisplayError::ResultUnavailable),
    }
}
