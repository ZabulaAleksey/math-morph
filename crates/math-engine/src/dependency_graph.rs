//! Deterministic, backend-neutral dependency graph construction.
//!
//! This stage resolves references against the immutable [`SymbolTable`] only.
//! Missing and forward references are retained as unresolved records for later
//! diagnostic stages; no fallback or heuristic node is created.

use crate::symbol_table::{
    FunctionSymbolDefinition, SymbolDefinition, SymbolTable, VariableDefinition,
};
use crate::{
    ReferenceAnalyzer, ReferenceError, ReferenceIdentity, ReferenceInput, ReferenceLimits,
};
use math_model::{ExpressionOrigin, MathExpression};
use std::{collections::HashMap, fmt};

/// Graph resource limits. Every limit is cumulative for one build.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct DependencyGraphLimits {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_unresolved: usize,
}

impl DependencyGraphLimits {
    pub const HARD_MAX_NODES: usize = 1_000_000;
    pub const HARD_MAX_EDGES: usize = 1_000_000;
    pub const HARD_MAX_UNRESOLVED: usize = 1_000_000;

    pub const fn new(max_nodes: usize, max_edges: usize, max_unresolved: usize) -> Self {
        Self {
            max_nodes,
            max_edges,
            max_unresolved,
        }
    }

    fn validate(self) -> Result<(), DependencyGraphError> {
        if self.max_nodes == 0
            || self.max_nodes > Self::HARD_MAX_NODES
            || self.max_edges == 0
            || self.max_edges > Self::HARD_MAX_EDGES
            || self.max_unresolved == 0
            || self.max_unresolved > Self::HARD_MAX_UNRESOLVED
        {
            return Err(DependencyGraphError::InvalidLimits);
        }
        Ok(())
    }
}

impl Default for DependencyGraphLimits {
    fn default() -> Self {
        Self::new(
            Self::HARD_MAX_NODES,
            Self::HARD_MAX_EDGES,
            Self::HARD_MAX_UNRESOLVED,
        )
    }
}

impl fmt::Debug for DependencyGraphLimits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DependencyGraphLimits")
            .field("max_nodes", &self.max_nodes)
            .field("max_edges", &self.max_edges)
            .field("max_unresolved", &self.max_unresolved)
            .finish()
    }
}

/// Namespace carried by a stable definition identity.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DefinitionNamespace {
    Variable,
    Function { arity: usize },
}

impl DefinitionNamespace {
    pub const fn is_variable(self) -> bool {
        matches!(self, Self::Variable)
    }

    pub const fn function_arity(self) -> Option<usize> {
        match self {
            Self::Variable => None,
            Self::Function { arity } => Some(arity),
        }
    }
}

impl fmt::Debug for DefinitionNamespace {
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

/// Stable node identity. Names and identifier payloads are intentionally not
/// part of the public debug representation; source ordinals are unique in a
/// [`SymbolTable`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DefinitionId {
    source_ordinal: usize,
    namespace: DefinitionNamespace,
}

impl DefinitionId {
    pub const fn new(source_ordinal: usize, namespace: DefinitionNamespace) -> Self {
        Self {
            source_ordinal,
            namespace,
        }
    }

    pub const fn source_ordinal(self) -> usize {
        self.source_ordinal
    }

    pub const fn namespace(self) -> DefinitionNamespace {
        self.namespace
    }
}

impl fmt::Debug for DefinitionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DefinitionId")
            .field("source_ordinal", &self.source_ordinal)
            .field("namespace", &self.namespace)
            .finish()
    }
}

/// A graph node in source order.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct DependencyNode {
    id: DefinitionId,
}

impl DependencyNode {
    pub const fn id(self) -> DefinitionId {
        self.id
    }

    pub const fn source_ordinal(self) -> usize {
        self.id.source_ordinal()
    }

    pub const fn namespace(self) -> DefinitionNamespace {
        self.id.namespace()
    }
}

impl fmt::Debug for DependencyNode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DependencyNode")
            .field("id", &self.id)
            .finish()
    }
}

/// A resolved reference edge in source/reference traversal order.
#[derive(Clone, Eq, PartialEq)]
pub struct DependencyEdge {
    from: DefinitionId,
    to: DefinitionId,
    reference: ReferenceIdentity,
    source_ordinal: usize,
    occurrence_index: usize,
}

impl DependencyEdge {
    pub const fn source(&self) -> DefinitionId {
        self.from
    }

    pub const fn target(&self) -> DefinitionId {
        self.to
    }

    pub const fn from(&self) -> DefinitionId {
        self.from
    }

    pub const fn to(&self) -> DefinitionId {
        self.to
    }

    pub fn reference(&self) -> &ReferenceIdentity {
        &self.reference
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }
}

impl fmt::Debug for DependencyEdge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DependencyEdge")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("reference", &self.reference)
            .field("source_ordinal", &self.source_ordinal)
            .field("occurrence_index", &self.occurrence_index)
            .finish()
    }
}

/// A reference with no visible definition. It is deliberately not converted
/// into a guessed node or edge.
#[derive(Clone, Eq, PartialEq)]
pub struct UnresolvedReference {
    from: DefinitionId,
    source_ordinal: usize,
    occurrence_index: usize,
    provenance: ExpressionOrigin,
    identity: ReferenceIdentity,
}

impl UnresolvedReference {
    pub const fn source(&self) -> DefinitionId {
        self.from
    }

    pub const fn from(&self) -> DefinitionId {
        self.from
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub const fn provenance(&self) -> ExpressionOrigin {
        self.provenance
    }

    pub fn identity(&self) -> &ReferenceIdentity {
        &self.identity
    }
}

impl fmt::Debug for UnresolvedReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UnresolvedReference")
            .field("from", &self.from)
            .field("source_ordinal", &self.source_ordinal)
            .field("occurrence_index", &self.occurrence_index)
            .field("provenance", &self.provenance)
            .field("identity", &self.identity)
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum DependencyGraphError {
    InvalidLimits,
    NodeLimitExceeded { limit: usize },
    EdgeLimitExceeded { limit: usize },
    UnresolvedLimitExceeded { limit: usize },
    ReferenceOutputLimitExceeded { limit: usize },
    MissingDefinitionNode { source_ordinal: usize },
    ReferenceAnalysis(ReferenceError),
    ArithmeticOverflow,
}

impl DependencyGraphError {
    const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidLimits => "InvalidLimits",
            Self::NodeLimitExceeded { .. } => "NodeLimitExceeded",
            Self::EdgeLimitExceeded { .. } => "EdgeLimitExceeded",
            Self::UnresolvedLimitExceeded { .. } => "UnresolvedLimitExceeded",
            Self::ReferenceOutputLimitExceeded { .. } => "ReferenceOutputLimitExceeded",
            Self::MissingDefinitionNode { .. } => "MissingDefinitionNode",
            Self::ReferenceAnalysis(_) => "ReferenceAnalysis",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
        }
    }
}

impl fmt::Debug for DependencyGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple(self.kind()).finish()
    }
}

impl fmt::Display for DependencyGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLimits => "dependency graph limits are invalid",
            Self::NodeLimitExceeded { .. } => "dependency graph node limit exceeded",
            Self::EdgeLimitExceeded { .. } => "dependency graph edge limit exceeded",
            Self::UnresolvedLimitExceeded { .. } => {
                "dependency graph unresolved-reference limit exceeded"
            }
            Self::ReferenceOutputLimitExceeded { .. } => {
                "dependency graph reference-output limit exceeded"
            }
            Self::MissingDefinitionNode { .. } => "dependency graph source node is missing",
            Self::ReferenceAnalysis(_) => "dependency graph reference analysis failed",
            Self::ArithmeticOverflow => "dependency graph accounting overflow",
        })
    }
}

impl std::error::Error for DependencyGraphError {}

/// Immutable dependency graph with deterministic node, edge, and unresolved
/// record order.
#[derive(Clone, Eq, PartialEq)]
pub struct DependencyGraph {
    nodes: Vec<DependencyNode>,
    edges: Vec<DependencyEdge>,
    unresolved: Vec<UnresolvedReference>,
    limits: DependencyGraphLimits,
}

impl fmt::Debug for DependencyGraph {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DependencyGraph")
            .field("node_count", &self.nodes.len())
            .field("edge_count", &self.edges.len())
            .field("unresolved_count", &self.unresolved.len())
            .field("limits", &self.limits)
            .finish()
    }
}

impl DependencyGraph {
    pub fn new(
        table: &SymbolTable,
        analyzer: &ReferenceAnalyzer,
        limits: DependencyGraphLimits,
    ) -> Result<Self, DependencyGraphError> {
        Self::build(table, analyzer, limits)
    }

    /// Builds one graph-wide reference analysis over every ordered definition.
    pub fn build(
        table: &SymbolTable,
        analyzer: &ReferenceAnalyzer,
        limits: DependencyGraphLimits,
    ) -> Result<Self, DependencyGraphError> {
        limits.validate()?;
        let definitions = table.definitions();
        if definitions.len() > limits.max_nodes {
            return Err(DependencyGraphError::NodeLimitExceeded {
                limit: limits.max_nodes,
            });
        }

        analyzer
            .limits()
            .validate()
            .map_err(DependencyGraphError::ReferenceAnalysis)?;
        if definitions.len() > analyzer.limits().max_input_expressions {
            return Err(DependencyGraphError::ReferenceAnalysis(
                ReferenceError::InputLimitExceeded {
                    limit: analyzer.limits().max_input_expressions,
                },
            ));
        }
        let materialized_output_limit = limits
            .max_edges
            .checked_add(limits.max_unresolved)
            .unwrap_or(ReferenceLimits::HARD_MAX_REFERENCES)
            .min(ReferenceLimits::HARD_MAX_REFERENCES);
        let analysis = analyzer
            .analyze_indexed_with_output_limit(
                definitions.len(),
                |index| {
                    let definition = &definitions[index];
                    ReferenceInput::new(
                        definition.source_ordinal(),
                        definition_expression(definition),
                    )
                },
                Some(materialized_output_limit),
            )
            .map_err(|error| match error {
                ReferenceError::MaterializedReferenceLimitExceeded { limit, .. } => {
                    DependencyGraphError::ReferenceOutputLimitExceeded { limit }
                }
                other => DependencyGraphError::ReferenceAnalysis(other),
            })?;

        let mut nodes = Vec::with_capacity(definitions.len());
        let mut node_indices = HashMap::with_capacity(definitions.len());
        for definition in definitions {
            let source_ordinal = definition.source_ordinal();
            let id = definition_id(definition);
            let next_node_count = nodes
                .len()
                .checked_add(1)
                .ok_or(DependencyGraphError::ArithmeticOverflow)?;
            if next_node_count > limits.max_nodes {
                return Err(DependencyGraphError::NodeLimitExceeded {
                    limit: limits.max_nodes,
                });
            }
            let index = nodes.len();
            node_indices.insert(source_ordinal, index);
            nodes.push(DependencyNode { id });
        }

        let mut edges = Vec::new();
        let mut unresolved = Vec::new();
        for reference in analysis.references() {
            let Some(&from_index) = node_indices.get(&reference.source_ordinal) else {
                return Err(DependencyGraphError::MissingDefinitionNode {
                    source_ordinal: reference.source_ordinal,
                });
            };
            let from = nodes[from_index].id;
            let target = resolve_reference(table, definitions, from_index, &reference.identity);
            if let Some(to) = target {
                let next_edge_count = edges
                    .len()
                    .checked_add(1)
                    .ok_or(DependencyGraphError::ArithmeticOverflow)?;
                if next_edge_count > limits.max_edges {
                    return Err(DependencyGraphError::EdgeLimitExceeded {
                        limit: limits.max_edges,
                    });
                }
                edges.push(DependencyEdge {
                    from,
                    to,
                    reference: reference.identity.clone(),
                    source_ordinal: reference.source_ordinal,
                    occurrence_index: reference.occurrence_index,
                });
            } else {
                let next_unresolved_count = unresolved
                    .len()
                    .checked_add(1)
                    .ok_or(DependencyGraphError::ArithmeticOverflow)?;
                if next_unresolved_count > limits.max_unresolved {
                    return Err(DependencyGraphError::UnresolvedLimitExceeded {
                        limit: limits.max_unresolved,
                    });
                }
                unresolved.push(UnresolvedReference {
                    from,
                    source_ordinal: reference.source_ordinal,
                    occurrence_index: reference.occurrence_index,
                    provenance: reference.provenance,
                    identity: reference.identity.clone(),
                });
            }
        }

        Ok(Self {
            nodes,
            edges,
            unresolved,
            limits,
        })
    }

    pub fn nodes(&self) -> &[DependencyNode] {
        &self.nodes
    }

    pub fn edges(&self) -> &[DependencyEdge] {
        &self.edges
    }

    pub fn unresolved(&self) -> &[UnresolvedReference] {
        &self.unresolved
    }

    pub fn unresolved_references(&self) -> &[UnresolvedReference] {
        self.unresolved()
    }

    pub const fn limits(&self) -> DependencyGraphLimits {
        self.limits
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn unresolved_count(&self) -> usize {
        self.unresolved.len()
    }
}

/// Short compatibility alias for callers that refer to graph budgets simply
/// as `GraphLimits`.
pub type GraphLimits = DependencyGraphLimits;

fn definition_expression(definition: &SymbolDefinition) -> &MathExpression {
    match definition {
        SymbolDefinition::Variable(definition) => definition.expression.as_ref(),
        SymbolDefinition::Function(definition) => definition.expression.as_ref(),
    }
}

fn definition_id(definition: &SymbolDefinition) -> DefinitionId {
    match definition {
        SymbolDefinition::Variable(definition) => DefinitionId {
            source_ordinal: definition.source_ordinal,
            namespace: DefinitionNamespace::Variable,
        },
        SymbolDefinition::Function(definition) => DefinitionId {
            source_ordinal: definition.source_ordinal,
            namespace: DefinitionNamespace::Function {
                arity: definition.key.arity,
            },
        },
    }
}

fn resolve_reference(
    table: &SymbolTable,
    definitions: &[SymbolDefinition],
    from_index: usize,
    identity: &ReferenceIdentity,
) -> Option<DefinitionId> {
    let from_definition = definitions.get(from_index)?;
    match identity {
        ReferenceIdentity::Variable(key) => table
            .visible_variable_before(key, from_definition.source_ordinal())
            .map(variable_id),
        ReferenceIdentity::Function(key) => {
            if let SymbolDefinition::Function(current) = from_definition {
                if current.key == *key {
                    return Some(definition_id(from_definition));
                }
            }
            table
                .visible_function_before(key, from_definition.source_ordinal())
                .map(function_id)
        }
    }
}

fn variable_id(definition: &VariableDefinition) -> DefinitionId {
    DefinitionId {
        source_ordinal: definition.source_ordinal,
        namespace: DefinitionNamespace::Variable,
    }
}

fn function_id(definition: &FunctionSymbolDefinition) -> DefinitionId {
    DefinitionId {
        source_ordinal: definition.source_ordinal,
        namespace: DefinitionNamespace::Function {
            arity: definition.key.arity,
        },
    }
}
