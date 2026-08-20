//! Bounded, backend-neutral free-reference collection for math expressions.
//!
//! This stage intentionally stops before dependency graph construction or
//! evaluation. Input expressions are borrowed; a complete validation pass is
//! performed before any identifier is cloned into the result.

use crate::{FunctionKey, SymbolKey};
use math_model::{
    AggregateExpression, ArrayIndex, BinaryExpression, BooleanExpression, ComparisonExpression,
    Definition, Derivative, Evaluation, ExpressionOrigin, FunctionCall, FunctionDefinition,
    Grouping, Identifier, Integral, MathExpression, MathExpressionKind, Matrix, RangeExpression,
    UnaryExpression, UnitedValue, Vector,
};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ReferenceLimits {
    pub max_input_expressions: usize,
    pub max_ast_depth: usize,
    pub max_ast_nodes: usize,
    pub max_text_bytes: usize,
    pub max_identifier_bytes: usize,
    pub max_collection_elements: usize,
    pub max_references: usize,
}

impl ReferenceLimits {
    pub const HARD_MAX_INPUT_EXPRESSIONS: usize = 1_000_000;
    pub const HARD_MAX_AST_DEPTH: usize = 256;
    pub const HARD_MAX_AST_NODES: usize = 100_000;
    pub const HARD_MAX_TEXT_BYTES: usize = 64 * 1024 * 1024;
    pub const HARD_MAX_IDENTIFIER_BYTES: usize = 1024 * 1024;
    pub const HARD_MAX_COLLECTION_ELEMENTS: usize = 1_000_000;
    pub const HARD_MAX_REFERENCES: usize = 1_000_000;

    pub const fn new(
        max_input_expressions: usize,
        max_ast_depth: usize,
        max_ast_nodes: usize,
        max_text_bytes: usize,
        max_identifier_bytes: usize,
        max_collection_elements: usize,
        max_references: usize,
    ) -> Self {
        Self {
            max_input_expressions,
            max_ast_depth,
            max_ast_nodes,
            max_text_bytes,
            max_identifier_bytes,
            max_collection_elements,
            max_references,
        }
    }

    pub(crate) fn validate(self) -> Result<(), ReferenceError> {
        if self.max_input_expressions == 0
            || self.max_input_expressions > Self::HARD_MAX_INPUT_EXPRESSIONS
            || self.max_ast_depth == 0
            || self.max_ast_depth > Self::HARD_MAX_AST_DEPTH
            || self.max_ast_nodes == 0
            || self.max_ast_nodes > Self::HARD_MAX_AST_NODES
            || self.max_text_bytes == 0
            || self.max_text_bytes > Self::HARD_MAX_TEXT_BYTES
            || self.max_identifier_bytes == 0
            || self.max_identifier_bytes > Self::HARD_MAX_IDENTIFIER_BYTES
            || self.max_collection_elements == 0
            || self.max_collection_elements > Self::HARD_MAX_COLLECTION_ELEMENTS
            || self.max_references == 0
            || self.max_references > Self::HARD_MAX_REFERENCES
        {
            Err(ReferenceError::InvalidLimits)
        } else {
            Ok(())
        }
    }
}

impl Default for ReferenceLimits {
    fn default() -> Self {
        Self::new(
            100_000,
            256,
            100_000,
            16 * 1024 * 1024,
            64 * 1024,
            1_000_000,
            100_000,
        )
    }
}

impl fmt::Debug for ReferenceLimits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReferenceLimits")
            .field("max_input_expressions", &self.max_input_expressions)
            .field("max_ast_depth", &self.max_ast_depth)
            .field("max_ast_nodes", &self.max_ast_nodes)
            .field("max_text_bytes", &self.max_text_bytes)
            .field("max_identifier_bytes", &self.max_identifier_bytes)
            .field("max_collection_elements", &self.max_collection_elements)
            .field("max_references", &self.max_references)
            .finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ReferenceInput<'a> {
    pub source_ordinal: usize,
    pub expression: &'a MathExpression,
}

impl<'a> ReferenceInput<'a> {
    pub const fn new(source_ordinal: usize, expression: &'a MathExpression) -> Self {
        Self {
            source_ordinal,
            expression,
        }
    }
}

impl<'a> From<(usize, &'a MathExpression)> for ReferenceInput<'a> {
    fn from((source_ordinal, expression): (usize, &'a MathExpression)) -> Self {
        Self::new(source_ordinal, expression)
    }
}

impl fmt::Debug for ReferenceInput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReferenceInput")
            .field("source_ordinal", &self.source_ordinal)
            .field("expression_present", &true)
            .finish()
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ReferenceIdentity {
    Variable(SymbolKey),
    Function(FunctionKey),
}

impl fmt::Debug for ReferenceIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReferenceIdentity")
            .field(
                "kind",
                &match self {
                    Self::Variable(_) => "variable",
                    Self::Function(_) => "function",
                },
            )
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ReferenceOccurrence {
    pub source_ordinal: usize,
    pub occurrence_index: usize,
    pub provenance: ExpressionOrigin,
    pub identity: ReferenceIdentity,
}

impl fmt::Debug for ReferenceOccurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReferenceOccurrence")
            .field("source_ordinal", &self.source_ordinal)
            .field("occurrence_index", &self.occurrence_index)
            .field("provenance", &self.provenance)
            .field("identity", &self.identity)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceDedupPolicy {
    FirstOccurrence,
}

/// Array indexing always analyzes both the target expression and each index;
/// no target-shape heuristic is applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrayIndexTargetPolicy {
    TargetAndIndices,
}

pub const ARRAY_INDEX_TARGET_POLICY: ArrayIndexTargetPolicy =
    ArrayIndexTargetPolicy::TargetAndIndices;

#[derive(Clone, Eq, PartialEq)]
pub struct ReferenceAnalysis {
    references: Vec<ReferenceOccurrence>,
    input_count: usize,
}

impl fmt::Debug for ReferenceAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReferenceAnalysis")
            .field("input_count", &self.input_count)
            .field("reference_count", &self.references.len())
            .finish()
    }
}

impl ReferenceAnalysis {
    pub fn references(&self) -> &[ReferenceOccurrence] {
        &self.references
    }
    pub fn len(&self) -> usize {
        self.references.len()
    }
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }
    pub fn input_count(&self) -> usize {
        self.input_count
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ReferenceError {
    InvalidLimits,
    InputLimitExceeded {
        limit: usize,
    },
    DepthLimitExceeded {
        source_ordinal: usize,
        limit: usize,
    },
    NodeLimitExceeded {
        source_ordinal: usize,
        limit: usize,
    },
    TextLimitExceeded {
        source_ordinal: usize,
        limit: usize,
    },
    IdentifierLimitExceeded {
        source_ordinal: usize,
        limit: usize,
    },
    CollectionLimitExceeded {
        source_ordinal: usize,
        limit: usize,
    },
    ReferenceLimitExceeded {
        source_ordinal: usize,
        limit: usize,
    },
    MaterializedReferenceLimitExceeded {
        source_ordinal: usize,
        limit: usize,
    },
    NonIncreasingSourceOrdinal {
        previous: usize,
        current: usize,
    },
    InvalidDefinitionTarget {
        source_ordinal: usize,
    },
    InvalidFunctionName {
        source_ordinal: usize,
    },
    InvalidFunctionParameter {
        source_ordinal: usize,
        parameter_index: usize,
    },
    InvalidBinder {
        source_ordinal: usize,
    },
    InvalidReferenceIdentifier {
        source_ordinal: usize,
    },
    InvalidFunctionCallee {
        source_ordinal: usize,
    },
    AmbiguousFunctionCallee {
        source_ordinal: usize,
    },
    UnsupportedExpression {
        source_ordinal: usize,
    },
    ArithmeticOverflow,
}

impl ReferenceError {
    const fn kind(self) -> &'static str {
        match self {
            Self::InvalidLimits => "InvalidLimits",
            Self::InputLimitExceeded { .. } => "InputLimitExceeded",
            Self::DepthLimitExceeded { .. } => "DepthLimitExceeded",
            Self::NodeLimitExceeded { .. } => "NodeLimitExceeded",
            Self::TextLimitExceeded { .. } => "TextLimitExceeded",
            Self::IdentifierLimitExceeded { .. } => "IdentifierLimitExceeded",
            Self::CollectionLimitExceeded { .. } => "CollectionLimitExceeded",
            Self::ReferenceLimitExceeded { .. } => "ReferenceLimitExceeded",
            Self::MaterializedReferenceLimitExceeded { .. } => "MaterializedReferenceLimitExceeded",
            Self::NonIncreasingSourceOrdinal { .. } => "NonIncreasingSourceOrdinal",
            Self::InvalidDefinitionTarget { .. } => "InvalidDefinitionTarget",
            Self::InvalidFunctionName { .. } => "InvalidFunctionName",
            Self::InvalidFunctionParameter { .. } => "InvalidFunctionParameter",
            Self::InvalidBinder { .. } => "InvalidBinder",
            Self::InvalidReferenceIdentifier { .. } => "InvalidReferenceIdentifier",
            Self::InvalidFunctionCallee { .. } => "InvalidFunctionCallee",
            Self::AmbiguousFunctionCallee { .. } => "AmbiguousFunctionCallee",
            Self::UnsupportedExpression { .. } => "UnsupportedExpression",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
        }
    }
}

impl fmt::Debug for ReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(self.kind()).finish()
    }
}

impl fmt::Display for ReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLimits => "reference limits are invalid",
            Self::InputLimitExceeded { .. } => "reference input limit exceeded",
            Self::DepthLimitExceeded { .. } => "reference AST depth limit exceeded",
            Self::NodeLimitExceeded { .. } => "reference AST node limit exceeded",
            Self::TextLimitExceeded { .. } => "reference text budget exceeded",
            Self::IdentifierLimitExceeded { .. } => "reference identifier budget exceeded",
            Self::CollectionLimitExceeded { .. } => "reference collection budget exceeded",
            Self::ReferenceLimitExceeded { .. } => "reference count limit exceeded",
            Self::MaterializedReferenceLimitExceeded { .. } => {
                "materialized reference output limit exceeded"
            }
            Self::NonIncreasingSourceOrdinal { .. } => {
                "reference source ordinals are not increasing"
            }
            Self::InvalidDefinitionTarget { .. } => "definition target is invalid",
            Self::InvalidFunctionName { .. } => "function name is invalid",
            Self::InvalidFunctionParameter { .. } => "function parameter is invalid",
            Self::InvalidBinder { .. } => "expression binder is invalid",
            Self::InvalidReferenceIdentifier { .. } => "reference identifier is invalid",
            Self::InvalidFunctionCallee { .. } => "function callee identifier is invalid",
            Self::AmbiguousFunctionCallee { .. } => "function callee is ambiguous",
            Self::UnsupportedExpression { .. } => "expression is unsupported",
            Self::ArithmeticOverflow => "reference accounting overflow",
        })
    }
}
impl std::error::Error for ReferenceError {}

pub struct ReferenceAnalyzer {
    limits: ReferenceLimits,
    dedup_policy: ReferenceDedupPolicy,
}

impl ReferenceAnalyzer {
    pub const fn new(limits: ReferenceLimits) -> Self {
        Self {
            limits,
            dedup_policy: ReferenceDedupPolicy::FirstOccurrence,
        }
    }

    pub const fn with_dedup_policy(mut self, policy: ReferenceDedupPolicy) -> Self {
        self.dedup_policy = policy;
        self
    }

    pub const fn limits(&self) -> ReferenceLimits {
        self.limits
    }

    pub fn analyze(
        &self,
        inputs: &[ReferenceInput<'_>],
    ) -> Result<ReferenceAnalysis, ReferenceError> {
        self.analyze_indexed(inputs.len(), |index| inputs[index])
    }

    pub(crate) fn analyze_indexed<'a, F>(
        &self,
        input_count: usize,
        input_at: F,
    ) -> Result<ReferenceAnalysis, ReferenceError>
    where
        F: FnMut(usize) -> ReferenceInput<'a>,
    {
        self.analyze_indexed_with_output_limit(input_count, input_at, None)
    }

    pub(crate) fn analyze_indexed_with_output_limit<'a, F>(
        &self,
        input_count: usize,
        mut input_at: F,
        materialized_output_limit: Option<usize>,
    ) -> Result<ReferenceAnalysis, ReferenceError>
    where
        F: FnMut(usize) -> ReferenceInput<'a>,
    {
        self.limits.validate()?;
        let mut state = PreflightState::default();
        let mut previous = None;
        for index in 0..input_count {
            let input = input_at(index);
            state.input_count = state
                .input_count
                .checked_add(1)
                .ok_or(ReferenceError::ArithmeticOverflow)?;
            if state.input_count > self.limits.max_input_expressions {
                return Err(ReferenceError::InputLimitExceeded {
                    limit: self.limits.max_input_expressions,
                });
            }
            if let Some(previous) = previous {
                if input.source_ordinal <= previous {
                    return Err(ReferenceError::NonIncreasingSourceOrdinal {
                        previous,
                        current: input.source_ordinal,
                    });
                }
            }
            previous = Some(input.source_ordinal);
            let mut bound = HashMap::new();
            preflight(
                input.expression,
                input.source_ordinal,
                &self.limits,
                0,
                &mut state,
                &mut bound,
            )?;
        }

        let output_capacity_limit = materialized_output_limit
            .unwrap_or(self.limits.max_references)
            .min(self.limits.max_references);
        let mut references = Vec::with_capacity(state.reference_count.min(output_capacity_limit));
        let mut seen = HashSet::with_capacity(references.capacity());
        for index in 0..input_count {
            let input = input_at(index);
            seen.clear();
            let mut bound = HashMap::new();
            let mut collector = Collector {
                references: &mut references,
                seen: &mut seen,
                policy: self.dedup_policy,
                materialized_output_limit,
            };
            collect(
                input.expression,
                input.source_ordinal,
                0,
                &mut bound,
                &mut collector,
            )?;
        }
        Ok(ReferenceAnalysis {
            references,
            input_count: state.input_count,
        })
    }

    pub fn analyze_expressions(
        &self,
        expressions: &[MathExpression],
    ) -> Result<ReferenceAnalysis, ReferenceError> {
        self.analyze_indexed(expressions.len(), |index| {
            ReferenceInput::new(index, &expressions[index])
        })
    }
}

#[derive(Default)]
struct PreflightState {
    input_count: usize,
    nodes: usize,
    text_bytes: usize,
    collection_elements: usize,
    reference_count: usize,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct IdentifierRef<'a> {
    name: &'a str,
    subscript: Option<&'a str>,
}

impl<'a> IdentifierRef<'a> {
    fn from_identifier(identifier: &'a Identifier) -> Self {
        Self {
            name: &identifier.name,
            subscript: identifier.subscript.as_deref(),
        }
    }
}

type BoundMap<'a> = HashMap<IdentifierRef<'a>, usize>;

fn is_bound(scope: &BoundMap<'_>, identifier: &Identifier) -> bool {
    scope.contains_key(&IdentifierRef::from_identifier(identifier))
}

fn push_binding<'a>(scope: &mut BoundMap<'a>, identifier: &'a Identifier) -> IdentifierRef<'a> {
    let key = IdentifierRef::from_identifier(identifier);
    *scope.entry(key).or_insert(0) += 1;
    key
}

fn pop_binding<'a>(scope: &mut BoundMap<'a>, key: IdentifierRef<'a>) {
    if let Some(count) = scope.get_mut(&key) {
        if *count > 1 {
            *count -= 1;
        } else {
            scope.remove(&key);
        }
    }
}

trait AsIdentifier {
    fn as_identifier(&self) -> Option<&Identifier>;
}

impl AsIdentifier for MathExpression {
    fn as_identifier(&self) -> Option<&Identifier> {
        match &self.kind {
            MathExpressionKind::Identifier(identifier) => Some(identifier),
            _ => None,
        }
    }
}

fn add_text(
    state: &mut PreflightState,
    bytes: usize,
    ordinal: usize,
    limits: &ReferenceLimits,
) -> Result<(), ReferenceError> {
    state.text_bytes = state
        .text_bytes
        .checked_add(bytes)
        .ok_or(ReferenceError::ArithmeticOverflow)?;
    if state.text_bytes > limits.max_text_bytes {
        return Err(ReferenceError::TextLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_text_bytes,
        });
    }
    Ok(())
}

fn validate_identifier(id: &Identifier, invalid: ReferenceError) -> Result<(), ReferenceError> {
    if id.name.is_empty() || id.subscript.as_deref() == Some("") {
        Err(invalid)
    } else {
        Ok(())
    }
}

fn add_identifier(
    state: &mut PreflightState,
    id: &Identifier,
    ordinal: usize,
    limits: &ReferenceLimits,
) -> Result<(), ReferenceError> {
    let bytes = id
        .name
        .len()
        .checked_add(id.subscript.as_ref().map_or(0, String::len))
        .ok_or(ReferenceError::ArithmeticOverflow)?;
    if bytes > limits.max_identifier_bytes {
        return Err(ReferenceError::IdentifierLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_identifier_bytes,
        });
    }
    add_text(state, id.name.len(), ordinal, limits)?;
    if let Some(subscript) = &id.subscript {
        add_text(state, subscript.len(), ordinal, limits)?;
    }
    Ok(())
}

fn add_collection(
    state: &mut PreflightState,
    count: usize,
    ordinal: usize,
    limits: &ReferenceLimits,
) -> Result<(), ReferenceError> {
    state.collection_elements = state
        .collection_elements
        .checked_add(count)
        .ok_or(ReferenceError::ArithmeticOverflow)?;
    if state.collection_elements > limits.max_collection_elements {
        return Err(ReferenceError::CollectionLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_collection_elements,
        });
    }
    Ok(())
}

fn add_reference(
    state: &mut PreflightState,
    ordinal: usize,
    limits: &ReferenceLimits,
) -> Result<(), ReferenceError> {
    state.reference_count = state
        .reference_count
        .checked_add(1)
        .ok_or(ReferenceError::ArithmeticOverflow)?;
    if state.reference_count > limits.max_references {
        return Err(ReferenceError::ReferenceLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_references,
        });
    }
    Ok(())
}

fn preflight<'a>(
    expr: &'a MathExpression,
    ordinal: usize,
    limits: &ReferenceLimits,
    depth: usize,
    state: &mut PreflightState,
    scope: &mut BoundMap<'a>,
) -> Result<(), ReferenceError> {
    if depth > limits.max_ast_depth {
        return Err(ReferenceError::DepthLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_ast_depth,
        });
    }
    state.nodes = state
        .nodes
        .checked_add(1)
        .ok_or(ReferenceError::ArithmeticOverflow)?;
    if state.nodes > limits.max_ast_nodes {
        return Err(ReferenceError::NodeLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_ast_nodes,
        });
    }
    match &expr.kind {
        MathExpressionKind::Real(real) => add_text(state, real.lexeme.len(), ordinal, limits)?,
        MathExpressionKind::Identifier(id) => {
            validate_identifier(
                id,
                ReferenceError::InvalidReferenceIdentifier {
                    source_ordinal: ordinal,
                },
            )?;
            add_identifier(state, id, ordinal, limits)?;
            if !is_bound(scope, id) {
                add_reference(state, ordinal, limits)?;
            }
        }
        MathExpressionKind::Unsupported(node) => {
            add_text(state, node.name.local_name.len(), ordinal, limits)?;
            if let Some(ns) = &node.name.namespace_uri {
                add_text(state, ns.len(), ordinal, limits)?;
            }
            if let Some(feature) = &node.feature {
                add_text(state, feature.local_name.len(), ordinal, limits)?;
                if let Some(ns) = &feature.namespace_uri {
                    add_text(state, ns.len(), ordinal, limits)?;
                }
            }
            return Err(ReferenceError::UnsupportedExpression {
                source_ordinal: ordinal,
            });
        }
        MathExpressionKind::UnitedValue(UnitedValue { value, units }) => {
            add_collection(state, units.factors.len(), ordinal, limits)?;
            if let Some(system) = &units.system {
                add_text(state, system.len(), ordinal, limits)?;
            }
            for factor in &units.factors {
                add_text(state, factor.unit.len(), ordinal, limits)?;
            }
            preflight(value, ordinal, limits, depth + 1, state, scope)?;
        }
        MathExpressionKind::Definition(Definition { target, value, .. }) => {
            preflight_definition_target(target, ordinal, limits, depth + 1, state)?;
            preflight(value, ordinal, limits, depth + 1, state, scope)?;
        }
        MathExpressionKind::FunctionDefinition(FunctionDefinition {
            name,
            parameters,
            body,
            ..
        }) => {
            preflight_function_binder(name, parameters, ordinal, limits, depth + 1, state)?;
            for (parameter_index, parameter) in parameters.iter().enumerate() {
                let identifier =
                    parameter
                        .as_identifier()
                        .ok_or(ReferenceError::InvalidFunctionParameter {
                            source_ordinal: ordinal,
                            parameter_index,
                        })?;
                let _ = push_binding(scope, identifier);
            }
            let body_result = preflight(body, ordinal, limits, depth + 1, state, scope);
            for parameter in parameters.iter().rev() {
                if let Some(identifier) = parameter.as_identifier() {
                    pop_binding(scope, IdentifierRef::from_identifier(identifier));
                }
            }
            body_result?;
        }
        MathExpressionKind::FunctionCall(FunctionCall { callee, arguments }) => {
            let callee_id = preflight_function_callee(callee, ordinal, limits, depth + 1, state)?;
            if !is_bound(scope, callee_id) {
                add_reference(state, ordinal, limits)?;
            }
            add_collection(state, arguments.len(), ordinal, limits)?;
            for argument in arguments {
                preflight(argument, ordinal, limits, depth + 1, state, scope)?;
            }
        }
        MathExpressionKind::Integral(Integral {
            bound_variable,
            integrand,
            bounds,
            ..
        }) => {
            let id = preflight_binder(
                bound_variable,
                ordinal,
                limits,
                depth + 1,
                state,
                ReferenceError::InvalidBinder {
                    source_ordinal: ordinal,
                },
            )?;
            if let Some(bounds) = bounds {
                preflight(&bounds.lower, ordinal, limits, depth + 1, state, scope)?;
                preflight(&bounds.upper, ordinal, limits, depth + 1, state, scope)?;
            }
            let binding = push_binding(scope, id);
            let body_result = preflight(integrand, ordinal, limits, depth + 1, state, scope);
            pop_binding(scope, binding);
            body_result?;
        }
        MathExpressionKind::Aggregate(AggregateExpression {
            bound_variable,
            body,
            bounds,
            ..
        }) => {
            let id = preflight_binder(
                bound_variable,
                ordinal,
                limits,
                depth + 1,
                state,
                ReferenceError::InvalidBinder {
                    source_ordinal: ordinal,
                },
            )?;
            if let Some(bounds) = bounds {
                preflight(&bounds.lower, ordinal, limits, depth + 1, state, scope)?;
                preflight(&bounds.upper, ordinal, limits, depth + 1, state, scope)?;
            }
            let binding = push_binding(scope, id);
            let body_result = preflight(body, ordinal, limits, depth + 1, state, scope);
            pop_binding(scope, binding);
            body_result?;
        }
        MathExpressionKind::Derivative(Derivative {
            bound_variable,
            expression,
            degree,
            ..
        }) => {
            let id = preflight_binder(
                bound_variable,
                ordinal,
                limits,
                depth + 1,
                state,
                ReferenceError::InvalidBinder {
                    source_ordinal: ordinal,
                },
            )?;
            if let Some(degree) = degree {
                preflight(degree, ordinal, limits, depth + 1, state, scope)?;
            }
            let binding = push_binding(scope, id);
            let body_result = preflight(expression, ordinal, limits, depth + 1, state, scope);
            pop_binding(scope, binding);
            body_result?;
        }
        _ => preflight_children(expr, ordinal, limits, depth, state, scope)?,
    }
    Ok(())
}

fn preflight_definition_target(
    target: &MathExpression,
    ordinal: usize,
    limits: &ReferenceLimits,
    depth: usize,
    state: &mut PreflightState,
) -> Result<(), ReferenceError> {
    if depth > limits.max_ast_depth {
        return Err(ReferenceError::DepthLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_ast_depth,
        });
    }
    state.nodes = state
        .nodes
        .checked_add(1)
        .ok_or(ReferenceError::ArithmeticOverflow)?;
    if state.nodes > limits.max_ast_nodes {
        return Err(ReferenceError::NodeLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_ast_nodes,
        });
    }
    let Some(id) = target.as_identifier() else {
        return Err(ReferenceError::InvalidDefinitionTarget {
            source_ordinal: ordinal,
        });
    };
    validate_identifier(
        id,
        ReferenceError::InvalidDefinitionTarget {
            source_ordinal: ordinal,
        },
    )?;
    add_identifier(state, id, ordinal, limits)
}

fn preflight_binder<'a>(
    binder: &'a MathExpression,
    ordinal: usize,
    limits: &ReferenceLimits,
    depth: usize,
    state: &mut PreflightState,
    invalid: ReferenceError,
) -> Result<&'a Identifier, ReferenceError> {
    if depth > limits.max_ast_depth {
        return Err(ReferenceError::DepthLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_ast_depth,
        });
    }
    state.nodes = state
        .nodes
        .checked_add(1)
        .ok_or(ReferenceError::ArithmeticOverflow)?;
    if state.nodes > limits.max_ast_nodes {
        return Err(ReferenceError::NodeLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_ast_nodes,
        });
    }
    let Some(id) = binder.as_identifier() else {
        return Err(invalid);
    };
    validate_identifier(id, invalid)?;
    add_identifier(state, id, ordinal, limits)?;
    Ok(id)
}

fn preflight_function_callee<'a>(
    callee: &'a MathExpression,
    ordinal: usize,
    limits: &ReferenceLimits,
    depth: usize,
    state: &mut PreflightState,
) -> Result<&'a Identifier, ReferenceError> {
    if depth > limits.max_ast_depth {
        return Err(ReferenceError::DepthLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_ast_depth,
        });
    }
    state.nodes = state
        .nodes
        .checked_add(1)
        .ok_or(ReferenceError::ArithmeticOverflow)?;
    if state.nodes > limits.max_ast_nodes {
        return Err(ReferenceError::NodeLimitExceeded {
            source_ordinal: ordinal,
            limit: limits.max_ast_nodes,
        });
    }
    let Some(identifier) = callee.as_identifier() else {
        return Err(ReferenceError::AmbiguousFunctionCallee {
            source_ordinal: ordinal,
        });
    };
    validate_identifier(
        identifier,
        ReferenceError::InvalidFunctionCallee {
            source_ordinal: ordinal,
        },
    )?;
    add_identifier(state, identifier, ordinal, limits)?;
    Ok(identifier)
}

fn preflight_function_binder(
    name: &MathExpression,
    parameters: &[MathExpression],
    ordinal: usize,
    limits: &ReferenceLimits,
    depth: usize,
    state: &mut PreflightState,
) -> Result<(), ReferenceError> {
    preflight_binder(
        name,
        ordinal,
        limits,
        depth,
        state,
        ReferenceError::InvalidFunctionName {
            source_ordinal: ordinal,
        },
    )?;
    add_collection(state, parameters.len(), ordinal, limits)?;
    for (index, parameter) in parameters.iter().enumerate() {
        preflight_binder(
            parameter,
            ordinal,
            limits,
            depth,
            state,
            ReferenceError::InvalidFunctionParameter {
                source_ordinal: ordinal,
                parameter_index: index,
            },
        )?;
    }
    Ok(())
}

fn preflight_children<'a>(
    expr: &'a MathExpression,
    ordinal: usize,
    limits: &ReferenceLimits,
    depth: usize,
    state: &mut PreflightState,
    scope: &mut BoundMap<'a>,
) -> Result<(), ReferenceError> {
    match &expr.kind {
        MathExpressionKind::Binary(BinaryExpression { left, right, .. })
        | MathExpressionKind::Comparison(ComparisonExpression { left, right, .. })
        | MathExpressionKind::Boolean(BooleanExpression { left, right, .. }) => {
            preflight(left, ordinal, limits, depth + 1, state, scope)?;
            preflight(right, ordinal, limits, depth + 1, state, scope)?;
        }
        MathExpressionKind::Evaluation(Evaluation {
            expression,
            unit_override,
            saved_result,
        }) => {
            preflight(expression, ordinal, limits, depth + 1, state, scope)?;
            if let Some(v) = unit_override {
                preflight(v, ordinal, limits, depth + 1, state, scope)?;
            }
            if let Some(v) = saved_result {
                preflight(v, ordinal, limits, depth + 1, state, scope)?;
            }
        }
        MathExpressionKind::Unary(UnaryExpression { operand, .. })
        | MathExpressionKind::Grouping(Grouping {
            expression: operand,
            ..
        })
        | MathExpressionKind::LogicalNot(math_model::LogicalNot { operand }) => {
            preflight(operand, ordinal, limits, depth + 1, state, scope)?
        }
        MathExpressionKind::ArrayIndex(ArrayIndex { target, indices }) => {
            preflight(target, ordinal, limits, depth + 1, state, scope)?;
            add_collection(state, indices.len(), ordinal, limits)?;
            for index in indices {
                preflight(index, ordinal, limits, depth + 1, state, scope)?;
            }
        }
        MathExpressionKind::Matrix(Matrix { elements, .. })
        | MathExpressionKind::Vector(Vector { elements, .. }) => {
            add_collection(state, elements.len(), ordinal, limits)?;
            for element in elements {
                preflight(element, ordinal, limits, depth + 1, state, scope)?;
            }
        }
        MathExpressionKind::Range(RangeExpression { start, next, end }) => {
            preflight(start, ordinal, limits, depth + 1, state, scope)?;
            if let Some(v) = next {
                preflight(v, ordinal, limits, depth + 1, state, scope)?;
            }
            preflight(end, ordinal, limits, depth + 1, state, scope)?;
        }
        MathExpressionKind::UnitedValue(UnitedValue { value, .. }) => {
            preflight(value, ordinal, limits, depth + 1, state, scope)?
        }
        MathExpressionKind::Real(_)
        | MathExpressionKind::Identifier(_)
        | MathExpressionKind::Unsupported(_)
        | MathExpressionKind::Definition(_)
        | MathExpressionKind::FunctionDefinition(_)
        | MathExpressionKind::FunctionCall(_)
        | MathExpressionKind::Integral(_)
        | MathExpressionKind::Aggregate(_)
        | MathExpressionKind::Derivative(_) => {}
    }
    Ok(())
}

struct Collector<'a> {
    references: &'a mut Vec<ReferenceOccurrence>,
    seen: &'a mut HashSet<ReferenceIdentity>,
    policy: ReferenceDedupPolicy,
    materialized_output_limit: Option<usize>,
}

fn collect<'a>(
    expr: &'a MathExpression,
    ordinal: usize,
    depth: usize,
    scope: &mut BoundMap<'a>,
    collector: &mut Collector<'_>,
) -> Result<(), ReferenceError> {
    match &expr.kind {
        MathExpressionKind::Identifier(id) => {
            validate_identifier(
                id,
                ReferenceError::InvalidReferenceIdentifier {
                    source_ordinal: ordinal,
                },
            )?;
            if !is_bound(scope, id) {
                record(
                    ReferenceIdentity::Variable(SymbolKey::from_identifier(id)),
                    ordinal,
                    expr.origin,
                    collector,
                )?;
            }
        }
        MathExpressionKind::Definition(Definition { target, value, .. }) => {
            collect(value, ordinal, depth + 1, scope, collector)?;
            let _ = target;
        }
        MathExpressionKind::FunctionDefinition(FunctionDefinition {
            name,
            parameters,
            body,
            ..
        }) => {
            let name = name
                .as_identifier()
                .ok_or(ReferenceError::InvalidFunctionName {
                    source_ordinal: ordinal,
                })?;
            validate_identifier(
                name,
                ReferenceError::InvalidFunctionName {
                    source_ordinal: ordinal,
                },
            )?;
            for (parameter_index, parameter) in parameters.iter().enumerate() {
                let identifier =
                    parameter
                        .as_identifier()
                        .ok_or(ReferenceError::InvalidFunctionParameter {
                            source_ordinal: ordinal,
                            parameter_index,
                        })?;
                validate_identifier(
                    identifier,
                    ReferenceError::InvalidFunctionParameter {
                        source_ordinal: ordinal,
                        parameter_index,
                    },
                )?;
                let _ = push_binding(scope, identifier);
            }
            let body_result = collect(body, ordinal, depth + 1, scope, collector);
            for parameter in parameters.iter().rev() {
                if let Some(identifier) = parameter.as_identifier() {
                    pop_binding(scope, IdentifierRef::from_identifier(identifier));
                }
            }
            body_result?;
        }
        MathExpressionKind::FunctionCall(FunctionCall { callee, arguments }) => {
            let id = callee
                .as_identifier()
                .ok_or(ReferenceError::AmbiguousFunctionCallee {
                    source_ordinal: ordinal,
                })?;
            validate_identifier(
                id,
                ReferenceError::InvalidFunctionCallee {
                    source_ordinal: ordinal,
                },
            )?;
            if !is_bound(scope, id) {
                record(
                    ReferenceIdentity::Function(FunctionKey::from_identifier(id, arguments.len())),
                    ordinal,
                    callee.origin,
                    collector,
                )?;
            }
            for arg in arguments {
                collect(arg, ordinal, depth + 1, scope, collector)?;
            }
        }
        MathExpressionKind::Integral(Integral {
            bound_variable,
            integrand,
            bounds,
            ..
        }) => {
            if let Some(bounds) = bounds {
                collect(&bounds.lower, ordinal, depth + 1, scope, collector)?;
                collect(&bounds.upper, ordinal, depth + 1, scope, collector)?;
            }
            let id = bound_variable
                .as_identifier()
                .ok_or(ReferenceError::InvalidBinder {
                    source_ordinal: ordinal,
                })?;
            validate_identifier(
                id,
                ReferenceError::InvalidBinder {
                    source_ordinal: ordinal,
                },
            )?;
            let binding = push_binding(scope, id);
            let body_result = collect(integrand, ordinal, depth + 1, scope, collector);
            pop_binding(scope, binding);
            body_result?;
        }
        MathExpressionKind::Aggregate(AggregateExpression {
            bound_variable,
            body,
            bounds,
            ..
        }) => {
            if let Some(bounds) = bounds {
                collect(&bounds.lower, ordinal, depth + 1, scope, collector)?;
                collect(&bounds.upper, ordinal, depth + 1, scope, collector)?;
            }
            let id = bound_variable
                .as_identifier()
                .ok_or(ReferenceError::InvalidBinder {
                    source_ordinal: ordinal,
                })?;
            validate_identifier(
                id,
                ReferenceError::InvalidBinder {
                    source_ordinal: ordinal,
                },
            )?;
            let binding = push_binding(scope, id);
            let body_result = collect(body, ordinal, depth + 1, scope, collector);
            pop_binding(scope, binding);
            body_result?;
        }
        MathExpressionKind::Derivative(Derivative {
            bound_variable,
            expression,
            degree,
            ..
        }) => {
            if let Some(degree) = degree {
                collect(degree, ordinal, depth + 1, scope, collector)?;
            }
            let id = bound_variable
                .as_identifier()
                .ok_or(ReferenceError::InvalidBinder {
                    source_ordinal: ordinal,
                })?;
            validate_identifier(
                id,
                ReferenceError::InvalidBinder {
                    source_ordinal: ordinal,
                },
            )?;
            let binding = push_binding(scope, id);
            let body_result = collect(expression, ordinal, depth + 1, scope, collector);
            pop_binding(scope, binding);
            body_result?;
        }
        _ => collect_children(expr, ordinal, depth, scope, collector)?,
    }
    Ok(())
}

fn collect_children<'a>(
    expr: &'a MathExpression,
    ordinal: usize,
    depth: usize,
    scope: &mut BoundMap<'a>,
    collector: &mut Collector<'_>,
) -> Result<(), ReferenceError> {
    macro_rules! visit {
        ($child:expr) => {
            collect($child, ordinal, depth + 1, scope, collector)?
        };
    }
    match &expr.kind {
        MathExpressionKind::Binary(BinaryExpression { left, right, .. })
        | MathExpressionKind::Comparison(ComparisonExpression { left, right, .. })
        | MathExpressionKind::Boolean(BooleanExpression { left, right, .. }) => {
            visit!(left);
            visit!(right);
        }
        MathExpressionKind::Evaluation(Evaluation {
            expression,
            unit_override,
            saved_result,
        }) => {
            visit!(expression);
            if let Some(v) = unit_override {
                visit!(v);
            }
            if let Some(v) = saved_result {
                visit!(v);
            }
        }
        MathExpressionKind::FunctionCall(_)
        | MathExpressionKind::FunctionDefinition(_)
        | MathExpressionKind::Definition(_)
        | MathExpressionKind::Integral(_)
        | MathExpressionKind::Aggregate(_)
        | MathExpressionKind::Derivative(_)
        | MathExpressionKind::Identifier(_)
        | MathExpressionKind::Real(_)
        | MathExpressionKind::Unsupported(_) => {}
        MathExpressionKind::Unary(UnaryExpression { operand, .. })
        | MathExpressionKind::Grouping(Grouping {
            expression: operand,
            ..
        })
        | MathExpressionKind::LogicalNot(math_model::LogicalNot { operand }) => visit!(operand),
        MathExpressionKind::ArrayIndex(ArrayIndex { target, indices }) => {
            visit!(target);
            for index in indices {
                visit!(index);
            }
        }
        MathExpressionKind::Matrix(Matrix { elements, .. })
        | MathExpressionKind::Vector(Vector { elements, .. }) => {
            for element in elements {
                visit!(element);
            }
        }
        MathExpressionKind::Range(RangeExpression { start, next, end }) => {
            visit!(start);
            if let Some(v) = next {
                visit!(v);
            }
            visit!(end);
        }
        MathExpressionKind::UnitedValue(UnitedValue { value, .. }) => visit!(value),
    }
    Ok(())
}

fn record(
    identity: ReferenceIdentity,
    ordinal: usize,
    provenance: ExpressionOrigin,
    collector: &mut Collector<'_>,
) -> Result<(), ReferenceError> {
    if collector.policy == ReferenceDedupPolicy::FirstOccurrence
        && collector.seen.contains(&identity)
    {
        return Ok(());
    }
    let next_count = collector
        .references
        .len()
        .checked_add(1)
        .ok_or(ReferenceError::ArithmeticOverflow)?;
    if let Some(limit) = collector.materialized_output_limit {
        if next_count > limit {
            return Err(ReferenceError::MaterializedReferenceLimitExceeded {
                source_ordinal: ordinal,
                limit,
            });
        }
    }
    if collector.policy == ReferenceDedupPolicy::FirstOccurrence {
        collector.seen.insert(identity.clone());
    }
    let occurrence_index = next_count - 1;
    collector.references.push(ReferenceOccurrence {
        source_ordinal: ordinal,
        occurrence_index,
        provenance,
        identity,
    });
    Ok(())
}
