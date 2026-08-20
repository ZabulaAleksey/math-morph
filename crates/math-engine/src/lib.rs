//! Backend-neutral presentation transformations for the source math AST.

pub mod dependency_graph;
pub mod reference_analysis;
pub mod symbol_table;

pub use dependency_graph::{
    DefinitionId, DefinitionNamespace, DependencyEdge, DependencyGraph, DependencyGraphError,
    DependencyGraphLimits, DependencyNode, GraphLimits, UnresolvedReference,
};

pub use reference_analysis::{
    ARRAY_INDEX_TARGET_POLICY, ArrayIndexTargetPolicy, ReferenceAnalysis, ReferenceAnalyzer,
    ReferenceDedupPolicy, ReferenceError, ReferenceIdentity, ReferenceInput, ReferenceLimits,
    ReferenceOccurrence,
};
pub use symbol_table::{
    FunctionKey, FunctionSymbolDefinition, SymbolDefinition, SymbolInput, SymbolKey, SymbolTable,
    SymbolTableError, SymbolTableLimits, VariableDefinition,
};

use math_model::{DefinitionStyle, Identifier, MathExpression, MathExpressionKind};
use std::{collections::BTreeMap, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransformationLimits {
    pub max_depth: usize,
    pub max_nodes: usize,
}
impl Default for TransformationLimits {
    fn default() -> Self {
        Self {
            max_depth: 256,
            max_nodes: 100_000,
        }
    }
}
impl TransformationLimits {
    pub const HARD_MAX_DEPTH: usize = 256;
    pub const HARD_MAX_NODES: usize = 100_000;

    pub const fn new(max_depth: usize, max_nodes: usize) -> Self {
        Self {
            max_depth,
            max_nodes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SymbolMappingKey {
    Alpha,
    Beta,
    Gamma,
    Delta,
    Epsilon,
    Theta,
    Lambda,
    Mu,
    Pi,
    Sigma,
    Phi,
    Omega,
    Infinity,
}
impl SymbolMappingKey {
    pub const fn source_name(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Gamma => "gamma",
            Self::Delta => "delta",
            Self::Epsilon => "epsilon",
            Self::Theta => "theta",
            Self::Lambda => "lambda",
            Self::Mu => "mu",
            Self::Pi => "pi",
            Self::Sigma => "sigma",
            Self::Phi => "phi",
            Self::Omega => "omega",
            Self::Infinity => "infinity",
        }
    }
    pub fn from_source_name(name: &str) -> Option<Self> {
        match name {
            "alpha" => Some(Self::Alpha),
            "beta" => Some(Self::Beta),
            "gamma" => Some(Self::Gamma),
            "delta" => Some(Self::Delta),
            "epsilon" => Some(Self::Epsilon),
            "theta" => Some(Self::Theta),
            "lambda" => Some(Self::Lambda),
            "mu" => Some(Self::Mu),
            "pi" => Some(Self::Pi),
            "sigma" => Some(Self::Sigma),
            "phi" => Some(Self::Phi),
            "omega" => Some(Self::Omega),
            "infinity" => Some(Self::Infinity),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SymbolGlyph {
    GreekAlpha,
    GreekBeta,
    GreekGamma,
    GreekDelta,
    GreekEpsilon,
    GreekTheta,
    GreekLambda,
    GreekMu,
    GreekPi,
    GreekSigma,
    GreekPhi,
    GreekOmega,
    Infinity,
}
impl SymbolGlyph {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GreekAlpha => "α",
            Self::GreekBeta => "β",
            Self::GreekGamma => "γ",
            Self::GreekDelta => "δ",
            Self::GreekEpsilon => "ε",
            Self::GreekTheta => "θ",
            Self::GreekLambda => "λ",
            Self::GreekMu => "μ",
            Self::GreekPi => "π",
            Self::GreekSigma => "σ",
            Self::GreekPhi => "φ",
            Self::GreekOmega => "ω",
            Self::Infinity => "∞",
        }
    }
}
pub type SymbolMapping = SymbolGlyph;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolMappingRegistry {
    mappings: BTreeMap<SymbolMappingKey, SymbolGlyph>,
}
impl Default for SymbolMappingRegistry {
    fn default() -> Self {
        Self::new()
    }
}
impl SymbolMappingRegistry {
    pub const fn new() -> Self {
        Self {
            mappings: BTreeMap::new(),
        }
    }
    pub fn insert(&mut self, key: SymbolMappingKey, glyph: SymbolGlyph) -> Option<SymbolGlyph> {
        self.mappings.insert(key, glyph)
    }
    pub fn register(&mut self, key: SymbolMappingKey, glyph: SymbolGlyph) -> Option<SymbolGlyph> {
        self.insert(key, glyph)
    }
    pub fn with_mapping(mut self, key: SymbolMappingKey, glyph: SymbolGlyph) -> Self {
        self.insert(key, glyph);
        self
    }
    pub fn get(&self, key: SymbolMappingKey) -> Option<SymbolGlyph> {
        self.mappings.get(&key).copied()
    }
    pub fn contains(&self, key: SymbolMappingKey) -> bool {
        self.mappings.contains_key(&key)
    }
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }
    pub fn len(&self) -> usize {
        self.mappings.len()
    }
    fn lookup_identifier(
        &self,
        identifier: &Identifier,
    ) -> Option<(SymbolMappingKey, SymbolGlyph)> {
        let key = SymbolMappingKey::from_source_name(&identifier.name)?;
        self.get(key).map(|glyph| (key, glyph))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotationProfile {
    pub definition_style: Option<DefinitionStyle>,
    pub symbol_mappings: SymbolMappingRegistry,
}
impl NotationProfile {
    pub fn faithful() -> Self {
        Self {
            definition_style: None,
            symbol_mappings: SymbolMappingRegistry::new(),
        }
    }
    pub fn with_definition_style(style: DefinitionStyle) -> Self {
        Self {
            definition_style: Some(style),
            ..Self::faithful()
        }
    }
    pub fn definition_style(style: DefinitionStyle) -> Self {
        Self::with_definition_style(style)
    }
    pub fn with_symbol_mappings(mut self, mappings: SymbolMappingRegistry) -> Self {
        self.symbol_mappings = mappings;
        self
    }
    pub fn is_faithful(&self) -> bool {
        self.definition_style.is_none() && self.symbol_mappings.is_empty()
    }
}
impl Default for NotationProfile {
    fn default() -> Self {
        Self::faithful()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppliedTransformation {
    DefinitionStyle {
        from: DefinitionStyle,
        to: DefinitionStyle,
    },
    SymbolMapping {
        key: SymbolMappingKey,
    },
}
impl AppliedTransformation {
    pub const fn name(self) -> &'static str {
        match self {
            Self::DefinitionStyle { .. } => "definition_style",
            Self::SymbolMapping { .. } => "symbol_mapping",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformationResult {
    pub display: MathExpression,
    pub applied_transformations: Vec<AppliedTransformation>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TransformError {
    DepthLimitExceeded { limit: usize },
    NodeLimitExceeded { limit: usize },
    InvalidLimits,
    ArithmeticOverflow,
}
impl TransformError {
    const fn kind(self) -> &'static str {
        match self {
            Self::DepthLimitExceeded { .. } => "DepthLimitExceeded",
            Self::NodeLimitExceeded { .. } => "NodeLimitExceeded",
            Self::InvalidLimits => "InvalidLimits",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
        }
    }
}
impl fmt::Debug for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(self.kind()).finish()
    }
}
impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::DepthLimitExceeded { .. } => "transformation depth limit exceeded",
            Self::NodeLimitExceeded { .. } => "transformation node limit exceeded",
            Self::InvalidLimits => "transformation limits are invalid",
            Self::ArithmeticOverflow => "transformation accounting overflow",
        })
    }
}
impl std::error::Error for TransformError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformationPipeline {
    profile: NotationProfile,
    limits: TransformationLimits,
}
impl TransformationPipeline {
    pub fn new(profile: NotationProfile) -> Self {
        Self::with_limits(profile, TransformationLimits::default())
    }
    pub const fn with_limits(profile: NotationProfile, limits: TransformationLimits) -> Self {
        Self { profile, limits }
    }
    pub const fn new_with_limits(profile: NotationProfile, limits: TransformationLimits) -> Self {
        Self::with_limits(profile, limits)
    }
    pub fn profile(&self) -> &NotationProfile {
        &self.profile
    }
    pub const fn limits(&self) -> &TransformationLimits {
        &self.limits
    }
    pub fn transform(
        &self,
        expression: &MathExpression,
    ) -> Result<TransformationResult, TransformError> {
        if self.limits.max_depth > TransformationLimits::HARD_MAX_DEPTH
            || self.limits.max_nodes > TransformationLimits::HARD_MAX_NODES
        {
            return Err(TransformError::InvalidLimits);
        }
        let mut state = TransformState {
            limits: self.limits,
            nodes: 0,
            applied: Vec::new(),
        };
        let display = transform_expression(expression, 0, &self.profile, &mut state)?;
        Ok(TransformationResult {
            display,
            applied_transformations: state.applied,
        })
    }
    pub fn apply(
        &self,
        expression: &MathExpression,
    ) -> Result<TransformationResult, TransformError> {
        self.transform(expression)
    }
}
impl Default for TransformationPipeline {
    fn default() -> Self {
        Self::new(NotationProfile::faithful())
    }
}

struct TransformState {
    limits: TransformationLimits,
    nodes: usize,
    applied: Vec<AppliedTransformation>,
}

fn transform_expression(
    expression: &MathExpression,
    depth: usize,
    profile: &NotationProfile,
    state: &mut TransformState,
) -> Result<MathExpression, TransformError> {
    if depth > state.limits.max_depth {
        return Err(TransformError::DepthLimitExceeded {
            limit: state.limits.max_depth,
        });
    }
    state.nodes = state
        .nodes
        .checked_add(1)
        .ok_or(TransformError::ArithmeticOverflow)?;
    if state.nodes > state.limits.max_nodes {
        return Err(TransformError::NodeLimitExceeded {
            limit: state.limits.max_nodes,
        });
    }
    let child_depth = depth
        .checked_add(1)
        .ok_or(TransformError::ArithmeticOverflow)?;
    let origin = expression.origin;
    let kind = match &expression.kind {
        MathExpressionKind::Real(value) => MathExpressionKind::Real(value.clone()),
        MathExpressionKind::Identifier(identifier) => {
            let mut display = identifier.clone();
            if let Some((key, glyph)) = profile.symbol_mappings.lookup_identifier(identifier) {
                display.name = glyph.as_str().to_owned();
                state
                    .applied
                    .push(AppliedTransformation::SymbolMapping { key });
            }
            MathExpressionKind::Identifier(display)
        }
        MathExpressionKind::Binary(value) => {
            MathExpressionKind::Binary(math_model::BinaryExpression {
                operator: value.operator,
                multiplication_style: value.multiplication_style,
                left: Box::new(transform_expression(
                    &value.left,
                    child_depth,
                    profile,
                    state,
                )?),
                right: Box::new(transform_expression(
                    &value.right,
                    child_depth,
                    profile,
                    state,
                )?),
            })
        }
        MathExpressionKind::Definition(value) => {
            let style = definition_style(value.style, profile, state);
            MathExpressionKind::Definition(math_model::Definition {
                kind: value.kind,
                style,
                target: Box::new(transform_expression(
                    &value.target,
                    child_depth,
                    profile,
                    state,
                )?),
                value: Box::new(transform_expression(
                    &value.value,
                    child_depth,
                    profile,
                    state,
                )?),
            })
        }
        MathExpressionKind::Evaluation(value) => {
            MathExpressionKind::Evaluation(math_model::Evaluation {
                expression: Box::new(transform_expression(
                    &value.expression,
                    child_depth,
                    profile,
                    state,
                )?),
                unit_override: transform_optional(
                    &value.unit_override,
                    child_depth,
                    profile,
                    state,
                )?,
                saved_result: transform_optional(&value.saved_result, child_depth, profile, state)?,
            })
        }
        MathExpressionKind::FunctionCall(value) => {
            MathExpressionKind::FunctionCall(math_model::FunctionCall {
                callee: Box::new(transform_expression(
                    &value.callee,
                    child_depth,
                    profile,
                    state,
                )?),
                arguments: transform_vec(&value.arguments, child_depth, profile, state)?,
            })
        }
        MathExpressionKind::FunctionDefinition(value) => {
            MathExpressionKind::FunctionDefinition(math_model::FunctionDefinition {
                style: definition_style(value.style, profile, state),
                name: Box::new(transform_expression(
                    &value.name,
                    child_depth,
                    profile,
                    state,
                )?),
                parameters: transform_vec(&value.parameters, child_depth, profile, state)?,
                body: Box::new(transform_expression(
                    &value.body,
                    child_depth,
                    profile,
                    state,
                )?),
            })
        }
        MathExpressionKind::Unary(value) => {
            MathExpressionKind::Unary(math_model::UnaryExpression {
                operator: value.operator,
                operand: Box::new(transform_expression(
                    &value.operand,
                    child_depth,
                    profile,
                    state,
                )?),
            })
        }
        MathExpressionKind::Grouping(value) => MathExpressionKind::Grouping(math_model::Grouping {
            expression: Box::new(transform_expression(
                &value.expression,
                child_depth,
                profile,
                state,
            )?),
            unpaired: value.unpaired,
        }),
        MathExpressionKind::ArrayIndex(value) => {
            MathExpressionKind::ArrayIndex(math_model::ArrayIndex {
                target: Box::new(transform_expression(
                    &value.target,
                    child_depth,
                    profile,
                    state,
                )?),
                indices: transform_vec(&value.indices, child_depth, profile, state)?,
            })
        }
        MathExpressionKind::Matrix(value) => MathExpressionKind::Matrix(math_model::Matrix {
            rows: value.rows,
            columns: value.columns,
            elements: transform_vec(&value.elements, child_depth, profile, state)?,
        }),
        MathExpressionKind::Vector(value) => MathExpressionKind::Vector(math_model::Vector {
            orientation: value.orientation,
            elements: transform_vec(&value.elements, child_depth, profile, state)?,
        }),
        MathExpressionKind::Range(value) => {
            MathExpressionKind::Range(math_model::RangeExpression {
                start: Box::new(transform_expression(
                    &value.start,
                    child_depth,
                    profile,
                    state,
                )?),
                next: transform_optional(&value.next, child_depth, profile, state)?,
                end: Box::new(transform_expression(
                    &value.end,
                    child_depth,
                    profile,
                    state,
                )?),
            })
        }
        MathExpressionKind::Integral(value) => MathExpressionKind::Integral(math_model::Integral {
            bound_variable: Box::new(transform_expression(
                &value.bound_variable,
                child_depth,
                profile,
                state,
            )?),
            integrand: Box::new(transform_expression(
                &value.integrand,
                child_depth,
                profile,
                state,
            )?),
            bounds: transform_bounds(&value.bounds, child_depth, profile, state)?,
            algorithm: value.algorithm,
        }),
        MathExpressionKind::Derivative(value) => {
            MathExpressionKind::Derivative(math_model::Derivative {
                bound_variable: Box::new(transform_expression(
                    &value.bound_variable,
                    child_depth,
                    profile,
                    state,
                )?),
                expression: Box::new(transform_expression(
                    &value.expression,
                    child_depth,
                    profile,
                    state,
                )?),
                degree: transform_optional(&value.degree, child_depth, profile, state)?,
                style: value.style,
            })
        }
        MathExpressionKind::Aggregate(value) => {
            MathExpressionKind::Aggregate(math_model::AggregateExpression {
                operator: value.operator,
                bound_variable: Box::new(transform_expression(
                    &value.bound_variable,
                    child_depth,
                    profile,
                    state,
                )?),
                body: Box::new(transform_expression(
                    &value.body,
                    child_depth,
                    profile,
                    state,
                )?),
                bounds: transform_bounds(&value.bounds, child_depth, profile, state)?,
            })
        }
        MathExpressionKind::Comparison(value) => {
            MathExpressionKind::Comparison(math_model::ComparisonExpression {
                operator: value.operator,
                left: Box::new(transform_expression(
                    &value.left,
                    child_depth,
                    profile,
                    state,
                )?),
                right: Box::new(transform_expression(
                    &value.right,
                    child_depth,
                    profile,
                    state,
                )?),
            })
        }
        MathExpressionKind::Boolean(value) => {
            MathExpressionKind::Boolean(math_model::BooleanExpression {
                operator: value.operator,
                left: Box::new(transform_expression(
                    &value.left,
                    child_depth,
                    profile,
                    state,
                )?),
                right: Box::new(transform_expression(
                    &value.right,
                    child_depth,
                    profile,
                    state,
                )?),
            })
        }
        MathExpressionKind::LogicalNot(value) => {
            MathExpressionKind::LogicalNot(math_model::LogicalNot {
                operand: Box::new(transform_expression(
                    &value.operand,
                    child_depth,
                    profile,
                    state,
                )?),
            })
        }
        MathExpressionKind::UnitedValue(value) => {
            MathExpressionKind::UnitedValue(math_model::UnitedValue {
                value: Box::new(transform_expression(
                    &value.value,
                    child_depth,
                    profile,
                    state,
                )?),
                units: value.units.clone(),
            })
        }
        MathExpressionKind::Unsupported(value) => MathExpressionKind::Unsupported(value.clone()),
    };
    Ok(MathExpression { kind, origin })
}

fn definition_style(
    from: DefinitionStyle,
    profile: &NotationProfile,
    state: &mut TransformState,
) -> DefinitionStyle {
    let Some(to) = profile.definition_style else {
        return from;
    };
    if from != to {
        state
            .applied
            .push(AppliedTransformation::DefinitionStyle { from, to });
        to
    } else {
        from
    }
}

fn transform_optional(
    value: &Option<Box<MathExpression>>,
    depth: usize,
    profile: &NotationProfile,
    state: &mut TransformState,
) -> Result<Option<Box<MathExpression>>, TransformError> {
    value
        .as_deref()
        .map(|value| transform_expression(value, depth, profile, state).map(Box::new))
        .transpose()
}
fn transform_bounds(
    value: &Option<math_model::Bounds>,
    depth: usize,
    profile: &NotationProfile,
    state: &mut TransformState,
) -> Result<Option<math_model::Bounds>, TransformError> {
    value
        .as_ref()
        .map(|bounds| {
            Ok(math_model::Bounds {
                lower: Box::new(transform_expression(&bounds.lower, depth, profile, state)?),
                upper: Box::new(transform_expression(&bounds.upper, depth, profile, state)?),
            })
        })
        .transpose()
}
fn transform_vec(
    values: &[MathExpression],
    depth: usize,
    profile: &NotationProfile,
    state: &mut TransformState,
) -> Result<Vec<MathExpression>, TransformError> {
    if depth > state.limits.max_depth {
        return Err(TransformError::DepthLimitExceeded {
            limit: state.limits.max_depth,
        });
    }
    let remaining = state.limits.max_nodes.checked_sub(state.nodes).ok_or(
        TransformError::NodeLimitExceeded {
            limit: state.limits.max_nodes,
        },
    )?;
    if values.len() > remaining {
        return Err(TransformError::NodeLimitExceeded {
            limit: state.limits.max_nodes,
        });
    }
    let mut transformed = Vec::with_capacity(values.len());
    for value in values {
        transformed.push(transform_expression(value, depth, profile, state)?);
    }
    Ok(transformed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use math_model::{
        BinaryExpression, BinaryOperator, ExpressionOrigin, NumericBase, RealLiteral,
    };
    fn identifier(name: &str) -> MathExpression {
        MathExpression {
            kind: MathExpressionKind::Identifier(Identifier {
                name: name.to_owned(),
                subscript: None,
            }),
            origin: ExpressionOrigin::Derived,
        }
    }
    #[test]
    fn faithful_transform_clones_without_mutating_original() {
        let original = MathExpression {
            kind: MathExpressionKind::Binary(BinaryExpression {
                operator: BinaryOperator::Add,
                multiplication_style: None,
                left: Box::new(identifier("x")),
                right: Box::new(MathExpression {
                    kind: MathExpressionKind::Real(RealLiteral {
                        lexeme: "2".into(),
                        base: NumericBase::Decimal,
                    }),
                    origin: ExpressionOrigin::Source(math_model::SourceSpan { start: 1, end: 2 }),
                }),
            }),
            origin: ExpressionOrigin::Source(math_model::SourceSpan { start: 0, end: 3 }),
        };
        let before = original.clone();
        let result = TransformationPipeline::default()
            .transform(&original)
            .unwrap();
        assert_eq!(original, before);
        assert_eq!(result.display, before);
        assert!(result.applied_transformations.is_empty());
    }
    #[test]
    fn explicit_profile_records_definition_style_and_symbol_mapping_in_order() {
        let registry = SymbolMappingRegistry::new()
            .with_mapping(SymbolMappingKey::Alpha, SymbolGlyph::GreekAlpha);
        let profile = NotationProfile::with_definition_style(DefinitionStyle::Equal)
            .with_symbol_mappings(registry);
        let expression = MathExpression {
            kind: MathExpressionKind::Definition(math_model::Definition {
                kind: math_model::DefinitionKind::Define,
                style: DefinitionStyle::ColonEqual,
                target: Box::new(identifier("alpha")),
                value: Box::new(identifier("x")),
            }),
            origin: ExpressionOrigin::Derived,
        };
        let result = TransformationPipeline::new(profile)
            .transform(&expression)
            .unwrap();
        assert_eq!(result.applied_transformations.len(), 2);
        assert!(matches!(
            result.applied_transformations[0],
            AppliedTransformation::DefinitionStyle { .. }
        ));
        assert!(matches!(
            result.applied_transformations[1],
            AppliedTransformation::SymbolMapping {
                key: SymbolMappingKey::Alpha
            }
        ));
        assert_eq!(
            expression.kind,
            MathExpressionKind::Definition(math_model::Definition {
                kind: math_model::DefinitionKind::Define,
                style: DefinitionStyle::ColonEqual,
                target: Box::new(identifier("alpha")),
                value: Box::new(identifier("x"))
            })
        );
    }
    #[test]
    fn unknown_symbol_does_not_heuristically_change_identifier() {
        let profile = NotationProfile::faithful().with_symbol_mappings(
            SymbolMappingRegistry::new()
                .with_mapping(SymbolMappingKey::Alpha, SymbolGlyph::GreekAlpha),
        );
        let result = TransformationPipeline::new(profile)
            .transform(&identifier("secret"))
            .unwrap();
        assert_eq!(result.display, identifier("secret"));
        assert!(result.applied_transformations.is_empty());
    }
    #[test]
    fn limits_are_typed_and_redacted() {
        let expression = MathExpression {
            kind: MathExpressionKind::Unary(math_model::UnaryExpression {
                operator: math_model::UnaryOperator::Negate,
                operand: Box::new(identifier("secret-symbol")),
            }),
            origin: ExpressionOrigin::Derived,
        };
        let error = TransformationPipeline::with_limits(
            NotationProfile::faithful(),
            TransformationLimits::new(0, 10),
        )
        .transform(&expression)
        .unwrap_err();
        assert_eq!(error, TransformError::DepthLimitExceeded { limit: 0 });
        assert!(!format!("{error:?}").contains("secret-symbol"));
    }
}
