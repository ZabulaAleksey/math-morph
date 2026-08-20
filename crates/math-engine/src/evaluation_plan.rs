//! Deterministic worksheet evaluation order for a dependency graph.
//!
//! Graph edges point from a dependent definition to its dependency. This
//! module reverses those edges and applies a bounded Kahn traversal, keeping
//! source ordinal order among simultaneously ready definitions.

use crate::{DefinitionId, DependencyGraph};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    fmt,
};

/// Resource limits for one evaluation-plan build.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct EvaluationPlanLimits {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_ready: usize,
    pub max_output: usize,
}

impl EvaluationPlanLimits {
    pub const HARD_MAX_NODES: usize = 1_000_000;
    pub const HARD_MAX_EDGES: usize = 1_000_000;
    pub const HARD_MAX_READY: usize = 1_000_000;
    pub const HARD_MAX_OUTPUT: usize = 1_000_000;

    pub const fn new(
        max_nodes: usize,
        max_edges: usize,
        max_ready: usize,
        max_output: usize,
    ) -> Self {
        Self {
            max_nodes,
            max_edges,
            max_ready,
            max_output,
        }
    }

    fn validate(self) -> Result<(), EvaluationPlanError> {
        if self.max_nodes == 0
            || self.max_nodes > Self::HARD_MAX_NODES
            || self.max_edges == 0
            || self.max_edges > Self::HARD_MAX_EDGES
            || self.max_ready == 0
            || self.max_ready > Self::HARD_MAX_READY
            || self.max_output == 0
            || self.max_output > Self::HARD_MAX_OUTPUT
        {
            return Err(EvaluationPlanError::InvalidLimits);
        }
        Ok(())
    }
}

impl Default for EvaluationPlanLimits {
    fn default() -> Self {
        Self::new(
            Self::HARD_MAX_NODES,
            Self::HARD_MAX_EDGES,
            Self::HARD_MAX_READY,
            Self::HARD_MAX_OUTPUT,
        )
    }
}

impl fmt::Debug for EvaluationPlanLimits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EvaluationPlanLimits")
            .field("max_nodes", &self.max_nodes)
            .field("max_edges", &self.max_edges)
            .field("max_ready", &self.max_ready)
            .field("max_output", &self.max_output)
            .finish()
    }
}

/// Typed, redacted failures while producing an evaluation plan.
#[derive(Clone, Eq, PartialEq)]
pub enum EvaluationPlanError {
    InvalidLimits,
    NodeLimitExceeded { limit: usize },
    EdgeLimitExceeded { limit: usize },
    ReadyLimitExceeded { limit: usize },
    OutputLimitExceeded { limit: usize },
    UnresolvedDependencies { count: usize },
    CyclePresent { remaining: usize },
    DuplicateNodeId { source_ordinal: usize },
    DanglingEdge { source_ordinal: usize },
    ArithmeticOverflow,
}

impl EvaluationPlanError {
    const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidLimits => "InvalidLimits",
            Self::NodeLimitExceeded { .. } => "NodeLimitExceeded",
            Self::EdgeLimitExceeded { .. } => "EdgeLimitExceeded",
            Self::ReadyLimitExceeded { .. } => "ReadyLimitExceeded",
            Self::OutputLimitExceeded { .. } => "OutputLimitExceeded",
            Self::UnresolvedDependencies { .. } => "UnresolvedDependencies",
            Self::CyclePresent { .. } => "CyclePresent",
            Self::DuplicateNodeId { .. } => "DuplicateNodeId",
            Self::DanglingEdge { .. } => "DanglingEdge",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
        }
    }
}

impl fmt::Debug for EvaluationPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple(self.kind()).finish()
    }
}

impl fmt::Display for EvaluationPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLimits => "evaluation-plan limits are invalid",
            Self::NodeLimitExceeded { .. } => "evaluation-plan node limit exceeded",
            Self::EdgeLimitExceeded { .. } => "evaluation-plan edge limit exceeded",
            Self::ReadyLimitExceeded { .. } => "evaluation-plan ready-set limit exceeded",
            Self::OutputLimitExceeded { .. } => "evaluation-plan output limit exceeded",
            Self::UnresolvedDependencies { .. } => "evaluation graph has unresolved dependencies",
            Self::CyclePresent { .. } => "evaluation graph contains a cycle",
            Self::DuplicateNodeId { .. } => "evaluation graph contains a duplicate node",
            Self::DanglingEdge { .. } => "evaluation graph contains a dangling edge",
            Self::ArithmeticOverflow => "evaluation-plan accounting overflow",
        })
    }
}

impl std::error::Error for EvaluationPlanError {}

/// Complete source-compatible order. Every graph node occurs exactly once.
#[derive(Clone, Eq, PartialEq)]
pub struct EvaluationPlan {
    order: Vec<DefinitionId>,
    limits: EvaluationPlanLimits,
}

impl fmt::Debug for EvaluationPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EvaluationPlan")
            .field("node_count", &self.order.len())
            .field("order", &self.order)
            .field("limits", &self.limits)
            .finish()
    }
}

impl EvaluationPlan {
    pub fn build(
        graph: &DependencyGraph,
        limits: EvaluationPlanLimits,
    ) -> Result<Self, EvaluationPlanError> {
        limits.validate()?;

        let node_count = graph.node_count();
        if node_count > limits.max_nodes {
            return Err(EvaluationPlanError::NodeLimitExceeded {
                limit: limits.max_nodes,
            });
        }
        if graph.edge_count() > limits.max_edges {
            return Err(EvaluationPlanError::EdgeLimitExceeded {
                limit: limits.max_edges,
            });
        }
        if node_count > limits.max_output {
            return Err(EvaluationPlanError::OutputLimitExceeded {
                limit: limits.max_output,
            });
        }
        if !graph.unresolved().is_empty() {
            return Err(EvaluationPlanError::UnresolvedDependencies {
                count: graph.unresolved_count(),
            });
        }

        let mut node_indices = HashMap::with_capacity(node_count);
        for (index, node) in graph.nodes().iter().enumerate() {
            if node_indices.insert(node.id(), index).is_some() {
                return Err(EvaluationPlanError::DuplicateNodeId {
                    source_ordinal: node.source_ordinal(),
                });
            }
        }

        let mut reverse_adjacency = vec![Vec::new(); node_count];
        let mut indegree = vec![0usize; node_count];
        for edge in graph.edges() {
            let Some(&dependent) = node_indices.get(&edge.from()) else {
                return Err(EvaluationPlanError::DanglingEdge {
                    source_ordinal: edge.source().source_ordinal(),
                });
            };
            let Some(&dependency) = node_indices.get(&edge.to()) else {
                return Err(EvaluationPlanError::DanglingEdge {
                    source_ordinal: edge.target().source_ordinal(),
                });
            };
            indegree[dependent] = indegree[dependent]
                .checked_add(1)
                .ok_or(EvaluationPlanError::ArithmeticOverflow)?;
            reverse_adjacency[dependency].push(dependent);
        }

        let mut ready = BinaryHeap::new();
        for (index, node) in graph.nodes().iter().enumerate() {
            if indegree[index] == 0 {
                push_ready(&mut ready, node.id(), index, limits.max_ready)?;
            }
        }

        let mut order = Vec::with_capacity(node_count);
        while let Some(Reverse((id, index))) = ready.pop() {
            let next_output = order
                .len()
                .checked_add(1)
                .ok_or(EvaluationPlanError::ArithmeticOverflow)?;
            if next_output > limits.max_output {
                return Err(EvaluationPlanError::OutputLimitExceeded {
                    limit: limits.max_output,
                });
            }
            order.push(id);
            for dependent in &reverse_adjacency[index] {
                indegree[*dependent] = indegree[*dependent]
                    .checked_sub(1)
                    .ok_or(EvaluationPlanError::ArithmeticOverflow)?;
                if indegree[*dependent] == 0 {
                    let dependent_id = graph.nodes()[*dependent].id();
                    push_ready(&mut ready, dependent_id, *dependent, limits.max_ready)?;
                }
            }
        }

        if order.len() != node_count {
            return Err(EvaluationPlanError::CyclePresent {
                remaining: node_count - order.len(),
            });
        }

        Ok(Self { order, limits })
    }

    pub fn new(
        graph: &DependencyGraph,
        limits: EvaluationPlanLimits,
    ) -> Result<Self, EvaluationPlanError> {
        Self::build(graph, limits)
    }

    pub fn order(&self) -> &[DefinitionId] {
        &self.order
    }

    pub fn definition_ids(&self) -> &[DefinitionId] {
        self.order()
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub const fn limits(&self) -> EvaluationPlanLimits {
        self.limits
    }
}

fn push_ready(
    ready: &mut BinaryHeap<Reverse<(DefinitionId, usize)>>,
    id: DefinitionId,
    index: usize,
    max_ready: usize,
) -> Result<(), EvaluationPlanError> {
    let next_ready = ready
        .len()
        .checked_add(1)
        .ok_or(EvaluationPlanError::ArithmeticOverflow)?;
    if next_ready > max_ready {
        return Err(EvaluationPlanError::ReadyLimitExceeded { limit: max_ready });
    }
    ready.push(Reverse((id, index)));
    Ok(())
}
