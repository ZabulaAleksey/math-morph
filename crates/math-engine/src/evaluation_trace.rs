//! Redacted deterministic evaluation trace.
use std::fmt;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationTraceKind {
    ReferenceObserved,
    BindingSelected,
    SubstitutionApplied,
    BranchSkipped,
    Completed,
    Failed,
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct EvaluationTraceStep {
    source_ordinal: usize,
    kind: EvaluationTraceKind,
    depth: usize,
    count: usize,
    binding_source_ordinal: Option<usize>,
}
impl EvaluationTraceStep {
    pub const fn new(source_ordinal: usize, kind: EvaluationTraceKind, depth: usize) -> Self {
        Self::with_count(source_ordinal, kind, depth, 0)
    }
    pub const fn with_count(
        source_ordinal: usize,
        kind: EvaluationTraceKind,
        depth: usize,
        count: usize,
    ) -> Self {
        Self {
            source_ordinal,
            kind,
            depth,
            count,
            binding_source_ordinal: None,
        }
    }
    pub const fn with_binding_source(
        source_ordinal: usize,
        binding_source_ordinal: usize,
        depth: usize,
        count: usize,
    ) -> Self {
        Self {
            source_ordinal,
            kind: EvaluationTraceKind::BindingSelected,
            depth,
            count,
            binding_source_ordinal: Some(binding_source_ordinal),
        }
    }
    pub const fn source_ordinal(self) -> usize {
        self.source_ordinal
    }
    pub const fn kind(self) -> EvaluationTraceKind {
        self.kind
    }
    pub const fn depth(self) -> usize {
        self.depth
    }
    pub const fn count(self) -> usize {
        self.count
    }
    pub const fn binding_source_ordinal(self) -> Option<usize> {
        self.binding_source_ordinal
    }
}
impl fmt::Debug for EvaluationTraceStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvaluationTraceStep")
            .field("source_ordinal", &self.source_ordinal)
            .field("kind", &self.kind)
            .field("depth", &self.depth)
            .field("count", &self.count)
            .field("binding_source_ordinal", &self.binding_source_ordinal)
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq, Default)]
pub struct EvaluationTrace {
    steps: Vec<EvaluationTraceStep>,
}
impl EvaluationTrace {
    pub const fn empty() -> Self {
        Self { steps: Vec::new() }
    }
    pub fn steps(&self) -> &[EvaluationTraceStep] {
        &self.steps
    }
    pub(crate) fn from_steps(steps: Vec<EvaluationTraceStep>) -> Self {
        Self { steps }
    }
}
impl fmt::Debug for EvaluationTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvaluationTrace")
            .field("step_count", &self.steps.len())
            .finish()
    }
}
