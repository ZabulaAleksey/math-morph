//! Typed, redacted semantic diagnostics derived from a dependency graph.
//!
//! This stage turns graph-level unresolved references into caller-bounded
//! diagnostics. It deliberately does not retain a symbol identity: names,
//! subscripts, literals, and source AST payloads must not cross this boundary.

use crate::{DependencyGraph, ReferenceIdentity};
use math_model::ExpressionOrigin;
use std::fmt;

/// Maximum number of semantic diagnostics materialized by one analysis.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SemanticDiagnosticsLimits {
    pub max_diagnostics: usize,
}

impl SemanticDiagnosticsLimits {
    pub const HARD_MAX_DIAGNOSTICS: usize = 1_000_000;

    pub const fn new(max_diagnostics: usize) -> Self {
        Self { max_diagnostics }
    }

    fn validate(self) -> Result<(), SemanticDiagnosticsError> {
        if self.max_diagnostics == 0 || self.max_diagnostics > Self::HARD_MAX_DIAGNOSTICS {
            return Err(SemanticDiagnosticsError::InvalidLimits);
        }
        Ok(())
    }
}

impl Default for SemanticDiagnosticsLimits {
    fn default() -> Self {
        Self::new(Self::HARD_MAX_DIAGNOSTICS)
    }
}

impl fmt::Debug for SemanticDiagnosticsLimits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SemanticDiagnosticsLimits")
            .field("max_diagnostics", &self.max_diagnostics)
            .finish()
    }
}

/// Redacted category of a missing free reference.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum UndefinedReferenceCategory {
    Variable,
    Function { arity: usize },
}

impl fmt::Debug for UndefinedReferenceCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable => formatter.write_str("Variable"),
            Self::Function { arity } => formatter
                .debug_struct("Function")
                .field("arity", arity)
                .finish(),
        }
    }
}

/// One undefined free reference, with no symbol payload.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct UndefinedReferenceDiagnostic {
    category: UndefinedReferenceCategory,
    source_ordinal: usize,
    occurrence_index: usize,
    has_source_provenance: bool,
}

impl UndefinedReferenceDiagnostic {
    pub const fn category(self) -> UndefinedReferenceCategory {
        self.category
    }

    pub const fn source_ordinal(self) -> usize {
        self.source_ordinal
    }

    pub const fn occurrence_index(self) -> usize {
        self.occurrence_index
    }

    pub const fn has_source_provenance(self) -> bool {
        self.has_source_provenance
    }
}

impl fmt::Debug for UndefinedReferenceDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UndefinedReferenceDiagnostic")
            .field("category", &self.category)
            .field("source_ordinal", &self.source_ordinal)
            .field("occurrence_index", &self.occurrence_index)
            .field("has_source_provenance", &self.has_source_provenance)
            .finish()
    }
}

/// Stage-104 semantic diagnostic output.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SemanticDiagnostic {
    UndefinedReference(UndefinedReferenceDiagnostic),
}

impl SemanticDiagnostic {
    pub const fn source_ordinal(self) -> usize {
        match self {
            Self::UndefinedReference(diagnostic) => diagnostic.source_ordinal(),
        }
    }
}

impl fmt::Debug for SemanticDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndefinedReference(diagnostic) => formatter
                .debug_tuple("UndefinedReference")
                .field(diagnostic)
                .finish(),
        }
    }
}

/// Typed failures while materializing semantic diagnostics.
#[derive(Clone, Eq, PartialEq)]
pub enum SemanticDiagnosticsError {
    InvalidLimits,
    DiagnosticLimitExceeded { limit: usize },
    ArithmeticOverflow,
}

impl SemanticDiagnosticsError {
    const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidLimits => "InvalidLimits",
            Self::DiagnosticLimitExceeded { .. } => "DiagnosticLimitExceeded",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
        }
    }
}

impl fmt::Debug for SemanticDiagnosticsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple(self.kind()).finish()
    }
}

impl fmt::Display for SemanticDiagnosticsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLimits => "semantic-diagnostic limits are invalid",
            Self::DiagnosticLimitExceeded { .. } => "semantic-diagnostic limit exceeded",
            Self::ArithmeticOverflow => "semantic-diagnostic accounting overflow",
        })
    }
}

impl std::error::Error for SemanticDiagnosticsError {}

/// Deterministic, bounded diagnostic output derived from one immutable graph.
#[derive(Clone, Eq, PartialEq)]
pub struct SemanticDiagnostics {
    diagnostics: Vec<SemanticDiagnostic>,
    limits: SemanticDiagnosticsLimits,
}

impl fmt::Debug for SemanticDiagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SemanticDiagnostics")
            .field("diagnostic_count", &self.diagnostics.len())
            .field("limits", &self.limits)
            .finish()
    }
}

impl SemanticDiagnostics {
    pub fn from_graph(
        graph: &DependencyGraph,
        limits: SemanticDiagnosticsLimits,
    ) -> Result<Self, SemanticDiagnosticsError> {
        limits.validate()?;
        let unresolved = graph.unresolved();
        if unresolved.len() > limits.max_diagnostics {
            return Err(SemanticDiagnosticsError::DiagnosticLimitExceeded {
                limit: limits.max_diagnostics,
            });
        }

        let mut diagnostics = Vec::with_capacity(unresolved.len());
        for reference in unresolved {
            let next_count = diagnostics
                .len()
                .checked_add(1)
                .ok_or(SemanticDiagnosticsError::ArithmeticOverflow)?;
            if next_count > limits.max_diagnostics {
                return Err(SemanticDiagnosticsError::DiagnosticLimitExceeded {
                    limit: limits.max_diagnostics,
                });
            }
            diagnostics.push(SemanticDiagnostic::UndefinedReference(
                UndefinedReferenceDiagnostic {
                    category: undefined_category(reference.identity()),
                    source_ordinal: reference.source_ordinal(),
                    occurrence_index: reference.occurrence_index(),
                    has_source_provenance: matches!(
                        reference.provenance(),
                        ExpressionOrigin::Source(_)
                    ),
                },
            ));
        }

        Ok(Self {
            diagnostics,
            limits,
        })
    }

    pub fn diagnostics(&self) -> &[SemanticDiagnostic] {
        &self.diagnostics
    }

    pub fn undefined_references(&self) -> impl Iterator<Item = UndefinedReferenceDiagnostic> + '_ {
        self.diagnostics.iter().map(|diagnostic| match diagnostic {
            SemanticDiagnostic::UndefinedReference(diagnostic) => *diagnostic,
        })
    }

    pub const fn limits(&self) -> SemanticDiagnosticsLimits {
        self.limits
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

fn undefined_category(identity: &ReferenceIdentity) -> UndefinedReferenceCategory {
    match identity {
        ReferenceIdentity::Variable(_) => UndefinedReferenceCategory::Variable,
        ReferenceIdentity::Function(key) => {
            UndefinedReferenceCategory::Function { arity: key.arity }
        }
    }
}
