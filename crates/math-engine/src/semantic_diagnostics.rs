//! Typed, redacted semantic diagnostics derived from a dependency graph.
//!
//! This stage turns graph-level unresolved references and cycles into
//! caller-bounded diagnostics. It deliberately does not retain a symbol identity: names,
//! subscripts, literals, and source AST payloads must not cross this boundary.

use crate::{DefinitionId, DependencyGraph, ReferenceIdentity};
use math_model::ExpressionOrigin;
use std::{collections::HashMap, fmt};

/// Maximum number of semantic diagnostics materialized by one analysis.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SemanticDiagnosticsLimits {
    pub max_diagnostics: usize,
    pub max_nodes: usize,
    pub max_edges: usize,
}

impl SemanticDiagnosticsLimits {
    pub const HARD_MAX_DIAGNOSTICS: usize = 1_000_000;
    pub const HARD_MAX_NODES: usize = 1_000_000;
    pub const HARD_MAX_EDGES: usize = 1_000_000;

    pub const fn new(max_diagnostics: usize) -> Self {
        Self::with_graph_limits(max_diagnostics, Self::HARD_MAX_NODES, Self::HARD_MAX_EDGES)
    }

    pub const fn with_graph_limits(
        max_diagnostics: usize,
        max_nodes: usize,
        max_edges: usize,
    ) -> Self {
        Self {
            max_diagnostics,
            max_nodes,
            max_edges,
        }
    }

    fn validate(self) -> Result<(), SemanticDiagnosticsError> {
        if self.max_diagnostics == 0
            || self.max_diagnostics > Self::HARD_MAX_DIAGNOSTICS
            || self.max_nodes == 0
            || self.max_nodes > Self::HARD_MAX_NODES
            || self.max_edges == 0
            || self.max_edges > Self::HARD_MAX_EDGES
        {
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
            .field("max_nodes", &self.max_nodes)
            .field("max_edges", &self.max_edges)
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

/// One member of a circular dependency, identified without symbol payload.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CircularDependencyDiagnostic {
    definition_id: DefinitionId,
    cycle_leader: DefinitionId,
    cycle_size: usize,
}

impl CircularDependencyDiagnostic {
    pub const fn definition_id(self) -> DefinitionId {
        self.definition_id
    }
    pub const fn cycle_leader(self) -> DefinitionId {
        self.cycle_leader
    }
    pub const fn cycle_size(self) -> usize {
        self.cycle_size
    }
}

impl fmt::Debug for CircularDependencyDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CircularDependencyDiagnostic")
            .field("definition_id", &self.definition_id)
            .field("cycle_leader", &self.cycle_leader)
            .field("cycle_size", &self.cycle_size)
            .finish()
    }
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

/// Semantic diagnostic output for unresolved references and circular dependencies.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SemanticDiagnostic {
    UndefinedReference(UndefinedReferenceDiagnostic),
    CircularDependency(CircularDependencyDiagnostic),
}

impl SemanticDiagnostic {
    pub const fn source_ordinal(self) -> usize {
        match self {
            Self::UndefinedReference(diagnostic) => diagnostic.source_ordinal(),
            Self::CircularDependency(diagnostic) => diagnostic.definition_id().source_ordinal(),
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
            Self::CircularDependency(diagnostic) => formatter
                .debug_tuple("CircularDependency")
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
    NodeLimitExceeded { limit: usize },
    EdgeLimitExceeded { limit: usize },
    GraphInvariantViolation { source_ordinal: usize },
    ArithmeticOverflow,
}

impl SemanticDiagnosticsError {
    const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidLimits => "InvalidLimits",
            Self::DiagnosticLimitExceeded { .. } => "DiagnosticLimitExceeded",
            Self::NodeLimitExceeded { .. } => "NodeLimitExceeded",
            Self::EdgeLimitExceeded { .. } => "EdgeLimitExceeded",
            Self::GraphInvariantViolation { .. } => "GraphInvariantViolation",
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
            Self::NodeLimitExceeded { .. } => "semantic-diagnostic node limit exceeded",
            Self::EdgeLimitExceeded { .. } => "semantic-diagnostic edge limit exceeded",
            Self::GraphInvariantViolation { .. } => "semantic-diagnostic graph invariant failed",
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
        if graph.node_count() > limits.max_nodes {
            return Err(SemanticDiagnosticsError::NodeLimitExceeded {
                limit: limits.max_nodes,
            });
        }
        if graph.edge_count() > limits.max_edges {
            return Err(SemanticDiagnosticsError::EdgeLimitExceeded {
                limit: limits.max_edges,
            });
        }
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
        for component in cyclic_components(graph)? {
            let cycle_leader = component[0];
            let cycle_size = component.len();
            for definition_id in component {
                push_diagnostic(
                    &mut diagnostics,
                    limits.max_diagnostics,
                    SemanticDiagnostic::CircularDependency(CircularDependencyDiagnostic {
                        definition_id,
                        cycle_leader,
                        cycle_size,
                    }),
                )?;
            }
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
        self.diagnostics
            .iter()
            .filter_map(|diagnostic| match diagnostic {
                SemanticDiagnostic::UndefinedReference(diagnostic) => Some(*diagnostic),
                SemanticDiagnostic::CircularDependency(_) => None,
            })
    }

    pub fn circular_dependencies(&self) -> impl Iterator<Item = CircularDependencyDiagnostic> + '_ {
        self.diagnostics
            .iter()
            .filter_map(|diagnostic| match diagnostic {
                SemanticDiagnostic::UndefinedReference(_) => None,
                SemanticDiagnostic::CircularDependency(diagnostic) => Some(*diagnostic),
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

fn push_diagnostic(
    diagnostics: &mut Vec<SemanticDiagnostic>,
    max_diagnostics: usize,
    diagnostic: SemanticDiagnostic,
) -> Result<(), SemanticDiagnosticsError> {
    let next_count = diagnostics
        .len()
        .checked_add(1)
        .ok_or(SemanticDiagnosticsError::ArithmeticOverflow)?;
    if next_count > max_diagnostics {
        return Err(SemanticDiagnosticsError::DiagnosticLimitExceeded {
            limit: max_diagnostics,
        });
    }
    diagnostics.push(diagnostic);
    Ok(())
}

fn cyclic_components(
    graph: &DependencyGraph,
) -> Result<Vec<Vec<DefinitionId>>, SemanticDiagnosticsError> {
    let nodes: Vec<_> = graph.nodes().iter().map(|node| node.id()).collect();
    cyclic_components_from_parts(
        &nodes,
        graph
            .edges()
            .iter()
            .map(|edge| (edge.from(), edge.to(), edge.source_ordinal())),
    )
}

fn cyclic_components_from_parts<I>(
    nodes: &[DefinitionId],
    edges: I,
) -> Result<Vec<Vec<DefinitionId>>, SemanticDiagnosticsError>
where
    I: IntoIterator<Item = (DefinitionId, DefinitionId, usize)>,
{
    let mut indices = HashMap::with_capacity(nodes.len());
    for (index, node) in nodes.iter().copied().enumerate() {
        indices.insert(node, index);
    }
    let mut forward = vec![Vec::new(); nodes.len()];
    let mut reverse = vec![Vec::new(); nodes.len()];
    for (from_id, to_id, source_ordinal) in edges {
        let from = *indices
            .get(&from_id)
            .ok_or(SemanticDiagnosticsError::GraphInvariantViolation { source_ordinal })?;
        let to = *indices
            .get(&to_id)
            .ok_or(SemanticDiagnosticsError::GraphInvariantViolation { source_ordinal })?;
        forward[from].push(to);
        reverse[to].push(from);
    }
    let mut visited = vec![false; nodes.len()];
    let mut finish = Vec::with_capacity(nodes.len());
    for start in 0..nodes.len() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut stack = vec![(start, 0usize)];
        while let Some((node, next)) = stack.last_mut() {
            if *next < forward[*node].len() {
                let child = forward[*node][*next];
                *next += 1;
                if !visited[child] {
                    visited[child] = true;
                    stack.push((child, 0));
                }
            } else {
                finish.push(*node);
                stack.pop();
            }
        }
    }
    visited.fill(false);
    let mut components = Vec::new();
    while let Some(start) = finish.pop() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut stack = vec![start];
        let mut component = Vec::new();
        while let Some(node) = stack.pop() {
            component.push(node);
            for &parent in &reverse[node] {
                if !visited[parent] {
                    visited[parent] = true;
                    stack.push(parent);
                }
            }
        }
        let self_loop = component.len() == 1 && forward[component[0]].contains(&component[0]);
        if component.len() > 1 || self_loop {
            let mut ids: Vec<_> = component.into_iter().map(|index| nodes[index]).collect();
            ids.sort_unstable();
            components.push(ids);
        }
    }
    components.sort_unstable_by_key(|component| component[0]);
    Ok(components)
}

fn undefined_category(identity: &ReferenceIdentity) -> UndefinedReferenceCategory {
    match identity {
        ReferenceIdentity::Variable(_) => UndefinedReferenceCategory::Variable,
        ReferenceIdentity::Function(key) => {
            UndefinedReferenceCategory::Function { arity: key.arity }
        }
    }
}

#[cfg(test)]
mod cycle_tests {
    use super::*;
    use crate::DefinitionNamespace;

    fn node(ordinal: usize) -> DefinitionId {
        DefinitionId::new(ordinal, DefinitionNamespace::Variable)
    }

    #[test]
    fn synthetic_multi_node_sccs_are_exact_and_deterministic() {
        let nodes = [node(1), node(2), node(3), node(4), node(5)];
        let edges = [
            (node(1), node(2), 1),
            (node(2), node(1), 2),
            (node(3), node(4), 3),
            (node(4), node(5), 4),
            (node(5), node(3), 5),
        ];
        assert_eq!(
            cyclic_components_from_parts(&nodes, edges).expect("synthetic SCCs"),
            vec![vec![node(1), node(2)], vec![node(3), node(4), node(5)]]
        );
    }
}
