//! Visibility-safe, resource-bounded scalar substitution.

use crate::{EvaluationTrace, EvaluationTraceKind, EvaluationTraceStep, SymbolKey, SymbolTable};
use math_model::*;
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SubstitutionLimits {
    pub max_input_depth: usize,
    pub max_input_nodes: usize,
    pub max_input_text_bytes: usize,
    pub max_output_depth: usize,
    pub max_output_nodes: usize,
    pub max_output_text_bytes: usize,
    pub max_substitutions: usize,
    pub max_substitution_depth: usize,
    pub max_recursive_steps: usize,
    pub max_expansion_steps: usize,
    pub max_trace_steps: usize,
}

impl SubstitutionLimits {
    pub const HARD_MAX_DEPTH: usize = 256;
    pub const HARD_MAX_NODES: usize = 100_000;
    pub const HARD_MAX_TEXT_BYTES: usize = 64 * 1024 * 1024;
    pub const HARD_MAX_SUBSTITUTIONS: usize = 1_000_000;
    pub const HARD_MAX_TRACE_STEPS: usize = 1_000_000;

    pub const fn new(max_substitutions: usize) -> Self {
        Self {
            max_input_depth: Self::HARD_MAX_DEPTH,
            max_input_nodes: Self::HARD_MAX_NODES,
            max_input_text_bytes: 16 * 1024 * 1024,
            max_output_depth: Self::HARD_MAX_DEPTH,
            max_output_nodes: Self::HARD_MAX_NODES,
            max_output_text_bytes: 16 * 1024 * 1024,
            max_substitutions,
            max_substitution_depth: Self::HARD_MAX_DEPTH,
            max_recursive_steps: max_substitutions,
            max_expansion_steps: max_substitutions,
            max_trace_steps: Self::HARD_MAX_TRACE_STEPS,
        }
    }

    fn validate(self) -> Result<(), SubstitutionError> {
        let depth = |v| v > 0 && v <= Self::HARD_MAX_DEPTH;
        let nodes = |v| v > 0 && v <= Self::HARD_MAX_NODES;
        let text = |v| v > 0 && v <= Self::HARD_MAX_TEXT_BYTES;
        let expansions = |v| v > 0 && v <= Self::HARD_MAX_SUBSTITUTIONS;
        if depth(self.max_input_depth)
            && nodes(self.max_input_nodes)
            && text(self.max_input_text_bytes)
            && depth(self.max_output_depth)
            && nodes(self.max_output_nodes)
            && text(self.max_output_text_bytes)
            && expansions(self.max_substitutions)
            && depth(self.max_substitution_depth)
            && expansions(self.max_recursive_steps)
            && expansions(self.max_expansion_steps)
            && self.max_trace_steps > 0
            && self.max_trace_steps <= Self::HARD_MAX_TRACE_STEPS
        {
            Ok(())
        } else {
            Err(SubstitutionError::InvalidLimits)
        }
    }
}

impl Default for SubstitutionLimits {
    fn default() -> Self {
        Self::new(100_000)
    }
}

impl fmt::Debug for SubstitutionLimits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubstitutionLimits")
            .field("max_input_depth", &self.max_input_depth)
            .field("max_input_nodes", &self.max_input_nodes)
            .field("max_input_text_bytes", &self.max_input_text_bytes)
            .field("max_output_depth", &self.max_output_depth)
            .field("max_output_nodes", &self.max_output_nodes)
            .field("max_output_text_bytes", &self.max_output_text_bytes)
            .field("max_substitutions", &self.max_substitutions)
            .field("max_substitution_depth", &self.max_substitution_depth)
            .field("max_recursive_steps", &self.max_recursive_steps)
            .field("max_expansion_steps", &self.max_expansion_steps)
            .field("max_trace_steps", &self.max_trace_steps)
            .finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SubstitutionError {
    InvalidLimits,
    UnknownVariable { source_ordinal: usize },
    UnsupportedCallable { source_ordinal: usize },
    UnsupportedExpression { source_ordinal: usize },
    CycleDetected { source_ordinal: usize },
    InputDepthLimitExceeded { limit: usize },
    InputNodeLimitExceeded { limit: usize },
    InputTextLimitExceeded { limit: usize },
    OutputDepthLimitExceeded { limit: usize },
    OutputNodeLimitExceeded { limit: usize },
    OutputTextLimitExceeded { limit: usize },
    SubstitutionDepthLimitExceeded { limit: usize },
    RecursiveStepLimitExceeded { limit: usize },
    ExpansionStepLimitExceeded { limit: usize },
    SubstitutionLimitExceeded { limit: usize },
    TraceLimitExceeded { limit: usize },
    ArithmeticOverflow,
}

impl SubstitutionError {
    fn name(self) -> &'static str {
        match self {
            Self::InvalidLimits => "InvalidLimits",
            Self::UnknownVariable { .. } => "UnknownVariable",
            Self::UnsupportedCallable { .. } => "UnsupportedCallable",
            Self::UnsupportedExpression { .. } => "UnsupportedExpression",
            Self::CycleDetected { .. } => "CycleDetected",
            Self::InputDepthLimitExceeded { .. } => "InputDepthLimitExceeded",
            Self::InputNodeLimitExceeded { .. } => "InputNodeLimitExceeded",
            Self::InputTextLimitExceeded { .. } => "InputTextLimitExceeded",
            Self::OutputDepthLimitExceeded { .. } => "OutputDepthLimitExceeded",
            Self::OutputNodeLimitExceeded { .. } => "OutputNodeLimitExceeded",
            Self::OutputTextLimitExceeded { .. } => "OutputTextLimitExceeded",
            Self::SubstitutionDepthLimitExceeded { .. } => "SubstitutionDepthLimitExceeded",
            Self::RecursiveStepLimitExceeded { .. } => "RecursiveStepLimitExceeded",
            Self::ExpansionStepLimitExceeded { .. } => "ExpansionStepLimitExceeded",
            Self::SubstitutionLimitExceeded { .. } => "SubstitutionLimitExceeded",
            Self::TraceLimitExceeded { .. } => "TraceLimitExceeded",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
        }
    }
}

impl fmt::Debug for SubstitutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(self.name()).finish()
    }
}
impl fmt::Display for SubstitutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "substitution failed: {}", self.name())
    }
}
impl std::error::Error for SubstitutionError {}

#[derive(Clone, Eq, PartialEq)]
pub struct SubstitutionFailure {
    error: SubstitutionError,
    trace: EvaluationTrace,
}
impl SubstitutionFailure {
    pub const fn error(&self) -> SubstitutionError {
        self.error
    }
    pub fn trace(&self) -> &EvaluationTrace {
        &self.trace
    }
    fn new(error: SubstitutionError, source_ordinal: usize) -> Self {
        Self {
            error,
            trace: EvaluationTrace::from_steps(vec![EvaluationTraceStep::with_count(
                source_ordinal,
                EvaluationTraceKind::Failed,
                0,
                0,
            )]),
        }
    }
}
impl fmt::Debug for SubstitutionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubstitutionFailure")
            .field("error", &self.error)
            .field("trace_steps", &self.trace.steps().len())
            .finish()
    }
}
impl fmt::Display for SubstitutionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}
impl std::error::Error for SubstitutionFailure {}

#[derive(Clone, Eq, PartialEq)]
pub struct SubstitutionResult {
    expression: MathExpression,
    substitution_count: usize,
    trace: EvaluationTrace,
}
impl SubstitutionResult {
    pub fn expression(&self) -> &MathExpression {
        &self.expression
    }
    pub const fn substitution_count(&self) -> usize {
        self.substitution_count
    }
    pub fn trace(&self) -> &EvaluationTrace {
        &self.trace
    }
}
impl fmt::Debug for SubstitutionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubstitutionResult")
            .field("substitution_count", &self.substitution_count)
            .field("trace_steps", &self.trace.steps().len())
            .finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SubstitutionEngine {
    limits: SubstitutionLimits,
}
impl SubstitutionEngine {
    pub const fn new(limits: SubstitutionLimits) -> Self {
        Self { limits }
    }
    pub fn once(
        &self,
        expression: &MathExpression,
        source_ordinal: usize,
        symbols: &SymbolTable,
    ) -> Result<SubstitutionResult, SubstitutionError> {
        self.run(expression, source_ordinal, symbols, false)
    }
    pub fn recursive(
        &self,
        expression: &MathExpression,
        source_ordinal: usize,
        symbols: &SymbolTable,
    ) -> Result<SubstitutionResult, SubstitutionError> {
        self.run(expression, source_ordinal, symbols, true)
    }
    pub fn once_with_failure_trace(
        &self,
        expression: &MathExpression,
        source_ordinal: usize,
        symbols: &SymbolTable,
    ) -> Result<SubstitutionResult, SubstitutionFailure> {
        self.once(expression, source_ordinal, symbols)
            .map_err(|error| SubstitutionFailure::new(error, source_ordinal))
    }
    pub fn recursive_with_failure_trace(
        &self,
        expression: &MathExpression,
        source_ordinal: usize,
        symbols: &SymbolTable,
    ) -> Result<SubstitutionResult, SubstitutionFailure> {
        self.recursive(expression, source_ordinal, symbols)
            .map_err(|error| SubstitutionFailure::new(error, source_ordinal))
    }
    fn run(
        &self,
        expression: &MathExpression,
        source_ordinal: usize,
        symbols: &SymbolTable,
        recursive: bool,
    ) -> Result<SubstitutionResult, SubstitutionError> {
        self.limits.validate()?;
        measure(
            expression,
            self.limits.max_input_depth,
            self.limits.max_input_nodes,
            self.limits.max_input_text_bytes,
            true,
        )?;
        let mut state = State {
            limits: self.limits,
            symbols,
            source_ordinal,
            recursive,
            output_nodes: 0,
            output_text: 0,
            substitutions: 0,
            expansion_steps: 0,
            trace: Vec::new(),
            active: Vec::new(),
        };
        let expression = transform(expression, 0, &mut Vec::new(), &mut state)?;
        state.trace(EvaluationTraceKind::Completed, 0)?;
        Ok(SubstitutionResult {
            expression,
            substitution_count: state.substitutions,
            trace: EvaluationTrace::from_steps(state.trace),
        })
    }
}
impl Default for SubstitutionEngine {
    fn default() -> Self {
        Self::new(SubstitutionLimits::default())
    }
}

struct State<'a> {
    limits: SubstitutionLimits,
    symbols: &'a SymbolTable,
    source_ordinal: usize,
    recursive: bool,
    output_nodes: usize,
    output_text: usize,
    substitutions: usize,
    expansion_steps: usize,
    trace: Vec<EvaluationTraceStep>,
    active: Vec<(SymbolKey, usize)>,
}
impl State<'_> {
    fn trace(&mut self, kind: EvaluationTraceKind, depth: usize) -> Result<(), SubstitutionError> {
        if self.trace.len() >= self.limits.max_trace_steps {
            return Err(SubstitutionError::TraceLimitExceeded {
                limit: self.limits.max_trace_steps,
            });
        }
        self.trace.push(EvaluationTraceStep::with_count(
            self.source_ordinal,
            kind,
            depth,
            self.substitutions,
        ));
        Ok(())
    }
    fn binding_trace(
        &mut self,
        depth: usize,
        binding_source_ordinal: usize,
    ) -> Result<(), SubstitutionError> {
        if self.trace.len() >= self.limits.max_trace_steps {
            return Err(SubstitutionError::TraceLimitExceeded {
                limit: self.limits.max_trace_steps,
            });
        }
        self.trace.push(EvaluationTraceStep::with_binding_source(
            self.source_ordinal,
            binding_source_ordinal,
            depth,
            self.substitutions,
        ));
        Ok(())
    }
    fn node(&mut self, depth: usize, text: usize) -> Result<(), SubstitutionError> {
        if depth > self.limits.max_output_depth {
            return Err(SubstitutionError::OutputDepthLimitExceeded {
                limit: self.limits.max_output_depth,
            });
        }
        self.output_nodes = add(self.output_nodes, 1)?;
        if self.output_nodes > self.limits.max_output_nodes {
            return Err(SubstitutionError::OutputNodeLimitExceeded {
                limit: self.limits.max_output_nodes,
            });
        }
        self.output_text = add(self.output_text, text)?;
        if self.output_text > self.limits.max_output_text_bytes {
            return Err(SubstitutionError::OutputTextLimitExceeded {
                limit: self.limits.max_output_text_bytes,
            });
        }
        Ok(())
    }
    fn collection(&mut self, count: usize) -> Result<(), SubstitutionError> {
        self.output_nodes = add(self.output_nodes, count)?;
        if self.output_nodes > self.limits.max_output_nodes {
            return Err(SubstitutionError::OutputNodeLimitExceeded {
                limit: self.limits.max_output_nodes,
            });
        }
        Ok(())
    }
    fn subtree(&mut self, value: &MathExpression, depth: usize) -> Result<(), SubstitutionError> {
        let m = measure(
            value,
            self.limits.max_output_depth,
            self.limits.max_output_nodes,
            self.limits.max_output_text_bytes,
            false,
        )?;
        if add(depth, m.depth)? > self.limits.max_output_depth {
            return Err(SubstitutionError::OutputDepthLimitExceeded {
                limit: self.limits.max_output_depth,
            });
        }
        self.output_nodes = add(self.output_nodes, m.nodes)?;
        if self.output_nodes > self.limits.max_output_nodes {
            return Err(SubstitutionError::OutputNodeLimitExceeded {
                limit: self.limits.max_output_nodes,
            });
        }
        self.output_text = add(self.output_text, m.text)?;
        if self.output_text > self.limits.max_output_text_bytes {
            return Err(SubstitutionError::OutputTextLimitExceeded {
                limit: self.limits.max_output_text_bytes,
            });
        }
        Ok(())
    }
}

fn transform(
    expression: &MathExpression,
    depth: usize,
    scope: &mut Vec<SymbolKey>,
    state: &mut State<'_>,
) -> Result<MathExpression, SubstitutionError> {
    if depth > state.limits.max_output_depth {
        return Err(SubstitutionError::OutputDepthLimitExceeded {
            limit: state.limits.max_output_depth,
        });
    }
    let next = add(depth, 1)?;
    let origin = expression.origin;
    let kind = match &expression.kind {
        MathExpressionKind::Identifier(id) => {
            return substitute_identifier(id, origin, depth, scope, state);
        }
        MathExpressionKind::Real(v) => {
            state.node(depth, v.lexeme.len())?;
            MathExpressionKind::Real(v.clone())
        }
        MathExpressionKind::Binary(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Binary(BinaryExpression {
                operator: v.operator,
                multiplication_style: v.multiplication_style,
                left: Box::new(transform(&v.left, next, scope, state)?),
                right: Box::new(transform(&v.right, next, scope, state)?),
            })
        }
        MathExpressionKind::Definition(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Definition(Definition {
                kind: v.kind,
                style: v.style,
                target: Box::new(clone_bounded(&v.target, next, state)?),
                value: Box::new(transform(&v.value, next, scope, state)?),
            })
        }
        MathExpressionKind::Evaluation(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Evaluation(Evaluation {
                expression: Box::new(transform(&v.expression, next, scope, state)?),
                unit_override: optional(&v.unit_override, next, scope, state)?,
                saved_result: optional(&v.saved_result, next, scope, state)?,
            })
        }
        MathExpressionKind::FunctionCall(_) => {
            return Err(SubstitutionError::UnsupportedCallable {
                source_ordinal: state.source_ordinal,
            });
        }
        MathExpressionKind::FunctionDefinition(v) => {
            state.node(depth, 0)?;
            let name = clone_bounded(&v.name, next, state)?;
            let parameters = clone_vec(&v.parameters, next, state)?;
            let scope_len = scope.len();
            for p in &v.parameters {
                scope.push(binder(p, state.source_ordinal)?);
            }
            let body = transform(&v.body, next, scope, state);
            scope.truncate(scope_len);
            MathExpressionKind::FunctionDefinition(FunctionDefinition {
                style: v.style,
                name: Box::new(name),
                parameters,
                body: Box::new(body?),
            })
        }
        MathExpressionKind::Unary(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Unary(UnaryExpression {
                operator: v.operator,
                operand: Box::new(transform(&v.operand, next, scope, state)?),
            })
        }
        MathExpressionKind::Grouping(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Grouping(Grouping {
                expression: Box::new(transform(&v.expression, next, scope, state)?),
                unpaired: v.unpaired,
            })
        }
        MathExpressionKind::ArrayIndex(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::ArrayIndex(ArrayIndex {
                target: Box::new(transform(&v.target, next, scope, state)?),
                indices: vector(&v.indices, next, scope, state)?,
            })
        }
        MathExpressionKind::Matrix(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Matrix(Matrix {
                rows: v.rows,
                columns: v.columns,
                elements: vector(&v.elements, next, scope, state)?,
            })
        }
        MathExpressionKind::Vector(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Vector(Vector {
                orientation: v.orientation,
                elements: vector(&v.elements, next, scope, state)?,
            })
        }
        MathExpressionKind::Range(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Range(RangeExpression {
                start: Box::new(transform(&v.start, next, scope, state)?),
                next: optional(&v.next, next, scope, state)?,
                end: Box::new(transform(&v.end, next, scope, state)?),
            })
        }
        MathExpressionKind::Integral(v) => {
            state.node(depth, 0)?;
            let variable = clone_bounded(&v.bound_variable, next, state)?;
            let bound = binder(&v.bound_variable, state.source_ordinal)?;
            scope.push(bound);
            let integrand = transform(&v.integrand, next, scope, state);
            scope.pop();
            MathExpressionKind::Integral(Integral {
                bound_variable: Box::new(variable),
                integrand: Box::new(integrand?),
                bounds: bounds(&v.bounds, next, scope, state)?,
                algorithm: v.algorithm,
            })
        }
        MathExpressionKind::Derivative(v) => {
            state.node(depth, 0)?;
            let variable = clone_bounded(&v.bound_variable, next, state)?;
            let bound = binder(&v.bound_variable, state.source_ordinal)?;
            scope.push(bound);
            let inner_expression = transform(&v.expression, next, scope, state);
            scope.pop();
            MathExpressionKind::Derivative(Derivative {
                bound_variable: Box::new(variable),
                expression: Box::new(inner_expression?),
                degree: optional(&v.degree, next, scope, state)?,
                style: v.style,
            })
        }
        MathExpressionKind::Aggregate(v) => {
            state.node(depth, 0)?;
            let variable = clone_bounded(&v.bound_variable, next, state)?;
            let bound = binder(&v.bound_variable, state.source_ordinal)?;
            scope.push(bound);
            let body = transform(&v.body, next, scope, state);
            scope.pop();
            MathExpressionKind::Aggregate(AggregateExpression {
                operator: v.operator,
                bound_variable: Box::new(variable),
                body: Box::new(body?),
                bounds: bounds(&v.bounds, next, scope, state)?,
            })
        }
        MathExpressionKind::Comparison(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Comparison(ComparisonExpression {
                operator: v.operator,
                left: Box::new(transform(&v.left, next, scope, state)?),
                right: Box::new(transform(&v.right, next, scope, state)?),
            })
        }
        MathExpressionKind::Boolean(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::Boolean(BooleanExpression {
                operator: v.operator,
                left: Box::new(transform(&v.left, next, scope, state)?),
                right: Box::new(transform(&v.right, next, scope, state)?),
            })
        }
        MathExpressionKind::LogicalNot(v) => {
            state.node(depth, 0)?;
            MathExpressionKind::LogicalNot(LogicalNot {
                operand: Box::new(transform(&v.operand, next, scope, state)?),
            })
        }
        MathExpressionKind::UnitedValue(v) => {
            state.node(depth, units_text(&v.units)?)?;
            state.collection(v.units.factors.len())?;
            MathExpressionKind::UnitedValue(UnitedValue {
                value: Box::new(transform(&v.value, next, scope, state)?),
                units: v.units.clone(),
            })
        }
        MathExpressionKind::Unsupported(_) => {
            return Err(SubstitutionError::UnsupportedExpression {
                source_ordinal: state.source_ordinal,
            });
        }
    };
    Ok(MathExpression { kind, origin })
}

fn substitute_identifier(
    id: &Identifier,
    origin: ExpressionOrigin,
    depth: usize,
    scope: &mut Vec<SymbolKey>,
    state: &mut State<'_>,
) -> Result<MathExpression, SubstitutionError> {
    state.trace(EvaluationTraceKind::ReferenceObserved, depth)?;
    let key = SymbolKey::from_identifier(id);
    if scope.iter().rev().any(|v| v == &key) {
        state.trace(EvaluationTraceKind::BranchSkipped, depth)?;
        state.node(depth, identifier_text(id)?)?;
        return Ok(MathExpression {
            kind: MathExpressionKind::Identifier(id.clone()),
            origin,
        });
    }
    let definition = state
        .symbols
        .visible_variable_before(&key, state.source_ordinal)
        .ok_or(SubstitutionError::UnknownVariable {
            source_ordinal: state.source_ordinal,
        })?;
    state.binding_trace(depth, definition.source_ordinal)?;
    state.substitutions = add(state.substitutions, 1)?;
    if state.substitutions > state.limits.max_substitutions {
        return Err(SubstitutionError::SubstitutionLimitExceeded {
            limit: state.limits.max_substitutions,
        });
    }
    let MathExpressionKind::Definition(value) = &definition.expression.kind else {
        return Err(SubstitutionError::UnsupportedExpression {
            source_ordinal: state.source_ordinal,
        });
    };
    state.trace(EvaluationTraceKind::SubstitutionApplied, depth)?;
    if !state.recursive {
        state.subtree(&value.value, depth)?;
        return Ok((*value.value).clone());
    }
    state.expansion_steps = add(state.expansion_steps, 1)?;
    if state.expansion_steps > state.limits.max_recursive_steps {
        return Err(SubstitutionError::RecursiveStepLimitExceeded {
            limit: state.limits.max_recursive_steps,
        });
    }
    if state.expansion_steps > state.limits.max_expansion_steps {
        return Err(SubstitutionError::ExpansionStepLimitExceeded {
            limit: state.limits.max_expansion_steps,
        });
    }
    if state.active.len() >= state.limits.max_substitution_depth {
        return Err(SubstitutionError::SubstitutionDepthLimitExceeded {
            limit: state.limits.max_substitution_depth,
        });
    }
    let identity = (definition.key.clone(), definition.source_ordinal);
    if state.active.contains(&identity) {
        return Err(SubstitutionError::CycleDetected {
            source_ordinal: state.source_ordinal,
        });
    }
    state.active.push(identity);
    let saved_scope = std::mem::take(scope);
    let expanded = transform(&value.value, depth, scope, state);
    *scope = saved_scope;
    state.active.pop();
    expanded
}

fn binder(v: &MathExpression, source_ordinal: usize) -> Result<SymbolKey, SubstitutionError> {
    match &v.kind {
        MathExpressionKind::Identifier(id) => Ok(SymbolKey::from_identifier(id)),
        _ => Err(SubstitutionError::UnsupportedExpression { source_ordinal }),
    }
}
fn clone_bounded(
    v: &MathExpression,
    depth: usize,
    state: &mut State<'_>,
) -> Result<MathExpression, SubstitutionError> {
    state.subtree(v, depth)?;
    Ok(v.clone())
}
fn clone_vec(
    values: &[MathExpression],
    depth: usize,
    state: &mut State<'_>,
) -> Result<Vec<MathExpression>, SubstitutionError> {
    values
        .iter()
        .map(|v| clone_bounded(v, depth, state))
        .collect()
}
fn vector(
    values: &[MathExpression],
    depth: usize,
    scope: &mut Vec<SymbolKey>,
    state: &mut State<'_>,
) -> Result<Vec<MathExpression>, SubstitutionError> {
    if values.len()
        > state
            .limits
            .max_output_nodes
            .saturating_sub(state.output_nodes)
    {
        return Err(SubstitutionError::OutputNodeLimitExceeded {
            limit: state.limits.max_output_nodes,
        });
    }
    values
        .iter()
        .map(|v| transform(v, depth, scope, state))
        .collect()
}
fn optional(
    v: &Option<Box<MathExpression>>,
    depth: usize,
    scope: &mut Vec<SymbolKey>,
    state: &mut State<'_>,
) -> Result<Option<Box<MathExpression>>, SubstitutionError> {
    v.as_deref()
        .map(|v| transform(v, depth, scope, state).map(Box::new))
        .transpose()
}
fn bounds(
    v: &Option<Bounds>,
    depth: usize,
    scope: &mut Vec<SymbolKey>,
    state: &mut State<'_>,
) -> Result<Option<Bounds>, SubstitutionError> {
    v.as_ref()
        .map(|v| {
            Ok(Bounds {
                lower: Box::new(transform(&v.lower, depth, scope, state)?),
                upper: Box::new(transform(&v.upper, depth, scope, state)?),
            })
        })
        .transpose()
}

#[derive(Clone, Copy)]
struct Metrics {
    nodes: usize,
    text: usize,
    depth: usize,
}
fn measure(
    root: &MathExpression,
    max_depth: usize,
    max_nodes: usize,
    max_text: usize,
    input: bool,
) -> Result<Metrics, SubstitutionError> {
    let mut stack = vec![(root, 0usize)];
    let mut result = Metrics {
        nodes: 0,
        text: 0,
        depth: 0,
    };
    while let Some((v, depth)) = stack.pop() {
        if depth > max_depth {
            return Err(if input {
                SubstitutionError::InputDepthLimitExceeded { limit: max_depth }
            } else {
                SubstitutionError::OutputDepthLimitExceeded { limit: max_depth }
            });
        }
        result.depth = result.depth.max(depth);
        result.nodes = add(result.nodes, 1)?;
        if let MathExpressionKind::UnitedValue(v) = &v.kind {
            result.nodes = add(result.nodes, v.units.factors.len())?;
        }
        if result.nodes > max_nodes {
            return Err(if input {
                SubstitutionError::InputNodeLimitExceeded { limit: max_nodes }
            } else {
                SubstitutionError::OutputNodeLimitExceeded { limit: max_nodes }
            });
        }
        result.text = add(result.text, expression_text(v)?)?;
        if result.text > max_text {
            return Err(if input {
                SubstitutionError::InputTextLimitExceeded { limit: max_text }
            } else {
                SubstitutionError::OutputTextLimitExceeded { limit: max_text }
            });
        }
        push_children(v, add(depth, 1)?, &mut stack, max_nodes, input)?;
    }
    Ok(result)
}

fn push_children<'a>(
    v: &'a MathExpression,
    depth: usize,
    stack: &mut Vec<(&'a MathExpression, usize)>,
    max_nodes: usize,
    input: bool,
) -> Result<(), SubstitutionError> {
    let mut push = |child: &'a MathExpression| {
        if stack.len() >= max_nodes {
            return Err(if input {
                SubstitutionError::InputNodeLimitExceeded { limit: max_nodes }
            } else {
                SubstitutionError::OutputNodeLimitExceeded { limit: max_nodes }
            });
        }
        stack.push((child, depth));
        Ok(())
    };
    match &v.kind {
        MathExpressionKind::Real(_)
        | MathExpressionKind::Identifier(_)
        | MathExpressionKind::Unsupported(_) => {}
        MathExpressionKind::Binary(v) => {
            push(&v.right)?;
            push(&v.left)?;
        }
        MathExpressionKind::Definition(v) => {
            push(&v.value)?;
            push(&v.target)?;
        }
        MathExpressionKind::Evaluation(v) => {
            if let Some(x) = v.saved_result.as_deref() {
                push(x)?
            }
            if let Some(x) = v.unit_override.as_deref() {
                push(x)?
            }
            push(&v.expression)?;
        }
        MathExpressionKind::FunctionCall(v) => {
            for x in v.arguments.iter().rev() {
                push(x)?
            }
            push(&v.callee)?;
        }
        MathExpressionKind::FunctionDefinition(v) => {
            push(&v.body)?;
            for x in v.parameters.iter().rev() {
                push(x)?
            }
            push(&v.name)?;
        }
        MathExpressionKind::Unary(v) => push(&v.operand)?,
        MathExpressionKind::Grouping(v) => push(&v.expression)?,
        MathExpressionKind::ArrayIndex(v) => {
            for x in v.indices.iter().rev() {
                push(x)?
            }
            push(&v.target)?;
        }
        MathExpressionKind::Matrix(v) => {
            for x in v.elements.iter().rev() {
                push(x)?
            }
        }
        MathExpressionKind::Vector(v) => {
            for x in v.elements.iter().rev() {
                push(x)?
            }
        }
        MathExpressionKind::Range(v) => {
            push(&v.end)?;
            if let Some(x) = v.next.as_deref() {
                push(x)?
            }
            push(&v.start)?;
        }
        MathExpressionKind::Integral(v) => {
            if let Some(x) = &v.bounds {
                push(&x.upper)?;
                push(&x.lower)?
            }
            push(&v.integrand)?;
            push(&v.bound_variable)?;
        }
        MathExpressionKind::Derivative(v) => {
            if let Some(x) = v.degree.as_deref() {
                push(x)?
            }
            push(&v.expression)?;
            push(&v.bound_variable)?;
        }
        MathExpressionKind::Aggregate(v) => {
            if let Some(x) = &v.bounds {
                push(&x.upper)?;
                push(&x.lower)?
            }
            push(&v.body)?;
            push(&v.bound_variable)?;
        }
        MathExpressionKind::Comparison(v) => {
            push(&v.right)?;
            push(&v.left)?;
        }
        MathExpressionKind::Boolean(v) => {
            push(&v.right)?;
            push(&v.left)?;
        }
        MathExpressionKind::LogicalNot(v) => push(&v.operand)?,
        MathExpressionKind::UnitedValue(v) => push(&v.value)?,
    }
    Ok(())
}

fn expression_text(v: &MathExpression) -> Result<usize, SubstitutionError> {
    match &v.kind {
        MathExpressionKind::Real(v) => Ok(v.lexeme.len()),
        MathExpressionKind::Identifier(v) => identifier_text(v),
        MathExpressionKind::UnitedValue(v) => units_text(&v.units),
        MathExpressionKind::Unsupported(v) => {
            let mut n = v.name.local_name.len();
            n = add(n, v.name.namespace_uri.as_ref().map_or(0, |x| x.len()))?;
            if let Some(x) = &v.feature {
                n = add(n, x.local_name.len())?;
                n = add(n, x.namespace_uri.as_ref().map_or(0, |x| x.len()))?;
            }
            Ok(n)
        }
        _ => Ok(0),
    }
}
fn identifier_text(v: &Identifier) -> Result<usize, SubstitutionError> {
    add(v.name.len(), v.subscript.as_ref().map_or(0, String::len))
}
fn units_text(v: &UnitMonomial) -> Result<usize, SubstitutionError> {
    let mut n = v.system.as_ref().map_or(0, String::len);
    for x in &v.factors {
        n = add(n, x.unit.len())?;
    }
    Ok(n)
}
fn add(a: usize, b: usize) -> Result<usize, SubstitutionError> {
    a.checked_add(b)
        .ok_or(SubstitutionError::ArithmeticOverflow)
}
