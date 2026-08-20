//! Backend-neutral indexing of worksheet definitions.
//!
//! This module deliberately stops at indexing definitions. It does not walk
//! expression semantics, collect references, evaluate values, or mutate the
//! source AST. Later dependency-analysis stages can build on the immutable
//! records produced here.

use math_model::{
    AggregateExpression, ArrayIndex, BinaryExpression, BooleanExpression, ComparisonExpression,
    Definition, DefinitionKind, Derivative, Evaluation, FunctionCall, FunctionDefinition, Grouping,
    Identifier, Integral, MathExpression, MathExpressionKind, Matrix, RangeExpression,
    UnaryExpression, UnitedValue, Vector,
};
use std::{collections::BTreeMap, fmt, sync::Arc};

/// Caller-supplied limits for [`SymbolTable::build`].
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SymbolTableLimits {
    pub max_input_expressions: usize,
    pub max_definitions: usize,
    pub max_ast_depth: usize,
    pub max_ast_nodes: usize,
    pub max_text_bytes: usize,
    pub max_identifier_bytes: usize,
    pub max_collection_elements: usize,
}

impl SymbolTableLimits {
    pub const HARD_MAX_INPUT_EXPRESSIONS: usize = 1_000_000;
    pub const HARD_MAX_DEFINITIONS: usize = 1_000_000;
    pub const HARD_MAX_AST_DEPTH: usize = 256;
    pub const HARD_MAX_AST_NODES: usize = 100_000;
    pub const HARD_MAX_TEXT_BYTES: usize = 64 * 1024 * 1024;
    pub const HARD_MAX_IDENTIFIER_BYTES: usize = 1024 * 1024;
    pub const HARD_MAX_COLLECTION_ELEMENTS: usize = 1_000_000;

    pub const fn new(
        max_input_expressions: usize,
        max_definitions: usize,
        max_ast_depth: usize,
        max_ast_nodes: usize,
        max_text_bytes: usize,
        max_identifier_bytes: usize,
        max_collection_elements: usize,
    ) -> Self {
        Self {
            max_input_expressions,
            max_definitions,
            max_ast_depth,
            max_ast_nodes,
            max_text_bytes,
            max_identifier_bytes,
            max_collection_elements,
        }
    }

    fn validate(self) -> Result<(), SymbolTableError> {
        let valid = self.max_input_expressions > 0
            && self.max_input_expressions <= Self::HARD_MAX_INPUT_EXPRESSIONS
            && self.max_definitions > 0
            && self.max_definitions <= Self::HARD_MAX_DEFINITIONS
            && self.max_ast_depth > 0
            && self.max_ast_depth <= Self::HARD_MAX_AST_DEPTH
            && self.max_ast_nodes > 0
            && self.max_ast_nodes <= Self::HARD_MAX_AST_NODES
            && self.max_text_bytes > 0
            && self.max_text_bytes <= Self::HARD_MAX_TEXT_BYTES
            && self.max_identifier_bytes > 0
            && self.max_identifier_bytes <= Self::HARD_MAX_IDENTIFIER_BYTES
            && self.max_collection_elements > 0
            && self.max_collection_elements <= Self::HARD_MAX_COLLECTION_ELEMENTS;
        if valid {
            Ok(())
        } else {
            Err(SymbolTableError::InvalidLimits)
        }
    }
}

impl Default for SymbolTableLimits {
    fn default() -> Self {
        Self::new(
            100_000,
            100_000,
            Self::HARD_MAX_AST_DEPTH,
            100_000,
            16 * 1024 * 1024,
            64 * 1024,
            1_000_000,
        )
    }
}

impl fmt::Debug for SymbolTableLimits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SymbolTableLimits")
            .field("max_input_expressions", &self.max_input_expressions)
            .field("max_definitions", &self.max_definitions)
            .field("max_ast_depth", &self.max_ast_depth)
            .field("max_ast_nodes", &self.max_ast_nodes)
            .field("max_text_bytes", &self.max_text_bytes)
            .field("max_identifier_bytes", &self.max_identifier_bytes)
            .field("max_collection_elements", &self.max_collection_elements)
            .finish()
    }
}

/// An expression together with its stable worksheet provenance.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SymbolInput<'a> {
    pub source_ordinal: usize,
    pub expression: &'a MathExpression,
}

impl<'a> SymbolInput<'a> {
    pub fn new(source_ordinal: usize, expression: &'a MathExpression) -> Self {
        Self {
            source_ordinal,
            expression,
        }
    }
}

impl<'a> From<(usize, &'a MathExpression)> for SymbolInput<'a> {
    fn from((source_ordinal, expression): (usize, &'a MathExpression)) -> Self {
        Self::new(source_ordinal, expression)
    }
}

impl fmt::Debug for SymbolInput<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SymbolInput")
            .field("source_ordinal", &self.source_ordinal)
            .field("expression_present", &true)
            .finish()
    }
}

/// Identity used by scalar definitions. Functions live in a separate
/// namespace and use [`FunctionKey`].
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SymbolKey {
    pub name: String,
    pub subscript: Option<String>,
}

impl SymbolKey {
    pub fn new(name: impl Into<String>, subscript: Option<String>) -> Self {
        Self {
            name: name.into(),
            subscript,
        }
    }

    pub fn from_identifier(identifier: &Identifier) -> Self {
        Self::new(identifier.name.clone(), identifier.subscript.clone())
    }
}

impl fmt::Debug for SymbolKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SymbolKey")
            .field("name_bytes", &self.name.len())
            .field("has_subscript", &self.subscript.is_some())
            .finish()
    }
}

/// Identity used by function definitions. Arity is part of the key.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FunctionKey {
    pub name: String,
    pub subscript: Option<String>,
    pub arity: usize,
}

impl FunctionKey {
    pub fn new(name: impl Into<String>, subscript: Option<String>, arity: usize) -> Self {
        Self {
            name: name.into(),
            subscript,
            arity,
        }
    }

    pub fn from_identifier(identifier: &Identifier, arity: usize) -> Self {
        Self::new(identifier.name.clone(), identifier.subscript.clone(), arity)
    }

    pub fn symbol_key(&self) -> SymbolKey {
        SymbolKey::new(self.name.clone(), self.subscript.clone())
    }
}

impl fmt::Debug for FunctionKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FunctionKey")
            .field("name_bytes", &self.name.len())
            .field("has_subscript", &self.subscript.is_some())
            .field("arity", &self.arity)
            .finish()
    }
}

/// One indexed scalar revision.
#[derive(Clone, Eq, PartialEq)]
pub struct VariableDefinition {
    pub source_ordinal: usize,
    pub key: SymbolKey,
    pub definition_kind: DefinitionKind,
    pub expression: Arc<MathExpression>,
}

impl fmt::Debug for VariableDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VariableDefinition")
            .field("source_ordinal", &self.source_ordinal)
            .field("definition_kind", &self.definition_kind)
            .field("expression_present", &true)
            .finish()
    }
}

/// One indexed function revision.
#[derive(Clone, Eq, PartialEq)]
pub struct FunctionSymbolDefinition {
    pub source_ordinal: usize,
    pub key: FunctionKey,
    pub expression: Arc<MathExpression>,
}

impl fmt::Debug for FunctionSymbolDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FunctionSymbolDefinition")
            .field("source_ordinal", &self.source_ordinal)
            .field("expression_present", &true)
            .finish()
    }
}

/// An ordered revision, retaining whether it belongs to the scalar or
/// function namespace.
#[derive(Clone, Eq, PartialEq)]
pub enum SymbolDefinition {
    Variable(VariableDefinition),
    Function(FunctionSymbolDefinition),
}

impl SymbolDefinition {
    pub const fn source_ordinal(&self) -> usize {
        match self {
            Self::Variable(definition) => definition.source_ordinal,
            Self::Function(definition) => definition.source_ordinal,
        }
    }
}

impl fmt::Debug for SymbolDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SymbolDefinition")
            .field(
                "namespace",
                &match self {
                    Self::Variable(_) => "variable",
                    Self::Function(_) => "function",
                },
            )
            .field("source_ordinal", &self.source_ordinal())
            .finish()
    }
}

/// Typed, redacted failures while indexing definitions.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SymbolTableError {
    InvalidLimits,
    InputLimitExceeded {
        limit: usize,
    },
    DefinitionLimitExceeded {
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
    NonIncreasingSourceOrdinal {
        previous: usize,
        current: usize,
    },
    TextLimitExceeded {
        source_ordinal: usize,
        limit: usize,
    },
    IdentifierLimitExceeded {
        source_ordinal: usize,
        limit: usize,
    },
    LookupIdentifierLimitExceeded {
        limit: usize,
    },
    CollectionLimitExceeded {
        source_ordinal: usize,
        limit: usize,
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
    ArithmeticOverflow,
}

impl SymbolTableError {
    const fn kind(self) -> &'static str {
        match self {
            Self::InvalidLimits => "InvalidLimits",
            Self::InputLimitExceeded { .. } => "InputLimitExceeded",
            Self::DefinitionLimitExceeded { .. } => "DefinitionLimitExceeded",
            Self::DepthLimitExceeded { .. } => "DepthLimitExceeded",
            Self::NodeLimitExceeded { .. } => "NodeLimitExceeded",
            Self::NonIncreasingSourceOrdinal { .. } => "NonIncreasingSourceOrdinal",
            Self::TextLimitExceeded { .. } => "TextLimitExceeded",
            Self::IdentifierLimitExceeded { .. } => "IdentifierLimitExceeded",
            Self::LookupIdentifierLimitExceeded { .. } => "LookupIdentifierLimitExceeded",
            Self::CollectionLimitExceeded { .. } => "CollectionLimitExceeded",
            Self::InvalidDefinitionTarget { .. } => "InvalidDefinitionTarget",
            Self::InvalidFunctionName { .. } => "InvalidFunctionName",
            Self::InvalidFunctionParameter { .. } => "InvalidFunctionParameter",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
        }
    }
}

impl fmt::Debug for SymbolTableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple(self.kind()).finish()
    }
}

impl fmt::Display for SymbolTableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLimits => "symbol table limits are invalid",
            Self::InputLimitExceeded { .. } => "symbol table input limit exceeded",
            Self::DefinitionLimitExceeded { .. } => "symbol table definition limit exceeded",
            Self::DepthLimitExceeded { .. } => "symbol table AST depth limit exceeded",
            Self::NodeLimitExceeded { .. } => "symbol table AST node limit exceeded",
            Self::NonIncreasingSourceOrdinal { .. } => {
                "symbol table source ordinals are not increasing"
            }
            Self::TextLimitExceeded { .. } => "symbol table text budget exceeded",
            Self::IdentifierLimitExceeded { .. } => "symbol table identifier budget exceeded",
            Self::LookupIdentifierLimitExceeded { .. } => {
                "symbol table lookup identifier budget exceeded"
            }
            Self::CollectionLimitExceeded { .. } => "symbol table collection budget exceeded",
            Self::InvalidDefinitionTarget { .. } => "definition target is not an identifier",
            Self::InvalidFunctionName { .. } => "function name is not an identifier",
            Self::InvalidFunctionParameter { .. } => "function parameter is not an identifier",
            Self::ArithmeticOverflow => "symbol table accounting overflow",
        })
    }
}

impl std::error::Error for SymbolTableError {}

/// Immutable, deterministic definition index for a worksheet.
#[derive(Clone, Eq, PartialEq)]
pub struct SymbolTable {
    variable_definitions: BTreeMap<SymbolKey, Vec<VariableDefinition>>,
    function_definitions: BTreeMap<FunctionKey, Vec<FunctionSymbolDefinition>>,
    ordered_definitions: Vec<SymbolDefinition>,
    input_count: usize,
    limits: SymbolTableLimits,
}

impl fmt::Debug for SymbolTable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SymbolTable")
            .field("input_count", &self.input_count)
            .field("definition_count", &self.ordered_definitions.len())
            .field("variable_key_count", &self.variable_definitions.len())
            .field("function_key_count", &self.function_definitions.len())
            .field("limits", &self.limits)
            .finish()
    }
}

impl SymbolTable {
    pub fn new<'a, I, T>(inputs: I, limits: SymbolTableLimits) -> Result<Self, SymbolTableError>
    where
        I: IntoIterator<Item = T>,
        T: Into<SymbolInput<'a>>,
    {
        Self::build(inputs, limits)
    }

    /// Builds from explicitly ordered input. Tuple `(ordinal, expression)` is
    /// accepted as a convenience through `Into<SymbolInput>`.
    pub fn build<'a, I, T>(inputs: I, limits: SymbolTableLimits) -> Result<Self, SymbolTableError>
    where
        I: IntoIterator<Item = T>,
        T: Into<SymbolInput<'a>>,
    {
        limits.validate()?;
        let mut table = Self {
            variable_definitions: BTreeMap::new(),
            function_definitions: BTreeMap::new(),
            ordered_definitions: Vec::new(),
            input_count: 0,
            limits,
        };
        let mut previous_ordinal = None;
        let mut total_nodes = 0;
        let mut total_text_bytes = 0;
        let mut total_collection_elements = 0;
        for input in inputs {
            table.input_count = table
                .input_count
                .checked_add(1)
                .ok_or(SymbolTableError::ArithmeticOverflow)?;
            if table.input_count > limits.max_input_expressions {
                return Err(SymbolTableError::InputLimitExceeded {
                    limit: limits.max_input_expressions,
                });
            }
            let input = input.into();
            if let Some(previous) = previous_ordinal {
                if input.source_ordinal <= previous {
                    return Err(SymbolTableError::NonIncreasingSourceOrdinal {
                        previous,
                        current: input.source_ordinal,
                    });
                }
            }
            previous_ordinal = Some(input.source_ordinal);
            validate_expression_limits(
                input.expression,
                input.source_ordinal,
                limits,
                0,
                &mut ValidationState {
                    nodes: &mut total_nodes,
                    text_bytes: &mut total_text_bytes,
                    collection_elements: &mut total_collection_elements,
                },
            )?;
            match &input.expression.kind {
                MathExpressionKind::Definition(definition) => {
                    let Some(identifier) = definition.target.as_identifier() else {
                        return Err(SymbolTableError::InvalidDefinitionTarget {
                            source_ordinal: input.source_ordinal,
                        });
                    };
                    if !identifier_is_well_formed(identifier) {
                        return Err(SymbolTableError::InvalidDefinitionTarget {
                            source_ordinal: input.source_ordinal,
                        });
                    }
                    let key = SymbolKey::from_identifier(identifier);
                    let count = table
                        .ordered_definitions
                        .len()
                        .checked_add(1)
                        .ok_or(SymbolTableError::ArithmeticOverflow)?;
                    if count > limits.max_definitions {
                        return Err(SymbolTableError::DefinitionLimitExceeded {
                            limit: limits.max_definitions,
                        });
                    }
                    let canonical = Arc::new(input.expression.clone());
                    let record = VariableDefinition {
                        source_ordinal: input.source_ordinal,
                        key: key.clone(),
                        definition_kind: definition.kind,
                        expression: canonical,
                    };
                    table
                        .variable_definitions
                        .entry(key)
                        .or_default()
                        .push(record.clone());
                    table
                        .ordered_definitions
                        .push(SymbolDefinition::Variable(record));
                }
                MathExpressionKind::FunctionDefinition(definition) => {
                    let Some(identifier) = definition.name.as_identifier() else {
                        return Err(SymbolTableError::InvalidFunctionName {
                            source_ordinal: input.source_ordinal,
                        });
                    };
                    if !identifier_is_well_formed(identifier) {
                        return Err(SymbolTableError::InvalidFunctionName {
                            source_ordinal: input.source_ordinal,
                        });
                    }
                    for (parameter_index, parameter) in definition.parameters.iter().enumerate() {
                        if !parameter
                            .as_identifier()
                            .is_some_and(identifier_is_well_formed)
                        {
                            return Err(SymbolTableError::InvalidFunctionParameter {
                                source_ordinal: input.source_ordinal,
                                parameter_index,
                            });
                        }
                    }
                    let key = FunctionKey::from_identifier(identifier, definition.parameters.len());
                    let count = table
                        .ordered_definitions
                        .len()
                        .checked_add(1)
                        .ok_or(SymbolTableError::ArithmeticOverflow)?;
                    if count > limits.max_definitions {
                        return Err(SymbolTableError::DefinitionLimitExceeded {
                            limit: limits.max_definitions,
                        });
                    }
                    let canonical = Arc::new(input.expression.clone());
                    let record = FunctionSymbolDefinition {
                        source_ordinal: input.source_ordinal,
                        key: key.clone(),
                        expression: canonical,
                    };
                    table
                        .function_definitions
                        .entry(key)
                        .or_default()
                        .push(record.clone());
                    table
                        .ordered_definitions
                        .push(SymbolDefinition::Function(record));
                }
                _ => {}
            }
        }
        Ok(table)
    }

    pub fn from_expressions(
        expressions: &[MathExpression],
        limits: SymbolTableLimits,
    ) -> Result<Self, SymbolTableError> {
        Self::build(expressions.iter().enumerate(), limits)
    }

    pub fn from_ordered<'a, I, T>(
        inputs: I,
        limits: SymbolTableLimits,
    ) -> Result<Self, SymbolTableError>
    where
        I: IntoIterator<Item = T>,
        T: Into<SymbolInput<'a>>,
    {
        Self::build(inputs, limits)
    }

    pub fn build_with_defaults<'a, I, T>(inputs: I) -> Result<Self, SymbolTableError>
    where
        I: IntoIterator<Item = T>,
        T: Into<SymbolInput<'a>>,
    {
        Self::build(inputs, SymbolTableLimits::default())
    }

    pub fn limits(&self) -> SymbolTableLimits {
        self.limits
    }
    pub fn input_count(&self) -> usize {
        self.input_count
    }
    pub fn definition_count(&self) -> usize {
        self.ordered_definitions.len()
    }
    pub fn variable_key_count(&self) -> usize {
        self.variable_definitions.len()
    }
    pub fn function_key_count(&self) -> usize {
        self.function_definitions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ordered_definitions.is_empty()
    }
    pub fn definitions(&self) -> &[SymbolDefinition] {
        &self.ordered_definitions
    }
    pub fn variable_history(&self, key: &SymbolKey) -> Option<&[VariableDefinition]> {
        self.variable_definitions.get(key).map(Vec::as_slice)
    }
    pub fn function_history(&self, key: &FunctionKey) -> Option<&[FunctionSymbolDefinition]> {
        self.function_definitions.get(key).map(Vec::as_slice)
    }
    pub fn variable(&self, key: &SymbolKey) -> Option<&VariableDefinition> {
        self.variable_history(key)
            .and_then(|history| history.last())
    }
    pub fn function(&self, key: &FunctionKey) -> Option<&FunctionSymbolDefinition> {
        self.function_history(key)
            .and_then(|history| history.last())
    }
    pub fn visible_variable_before(
        &self,
        key: &SymbolKey,
        source_ordinal: usize,
    ) -> Option<&VariableDefinition> {
        self.variable_history(key).and_then(|history| {
            history
                .iter()
                .rev()
                .find(|revision| revision.source_ordinal < source_ordinal)
        })
    }
    pub fn visible_function_before(
        &self,
        key: &FunctionKey,
        source_ordinal: usize,
    ) -> Option<&FunctionSymbolDefinition> {
        self.function_history(key).and_then(|history| {
            history
                .iter()
                .rev()
                .find(|revision| revision.source_ordinal < source_ordinal)
        })
    }
    pub fn lookup_variable(
        &self,
        identifier: &Identifier,
    ) -> Result<Option<&VariableDefinition>, SymbolTableError> {
        validate_lookup_identifier(identifier, self.limits)?;
        Ok(self.variable(&SymbolKey::from_identifier(identifier)))
    }
    pub fn lookup_function(
        &self,
        identifier: &Identifier,
        arity: usize,
    ) -> Result<Option<&FunctionSymbolDefinition>, SymbolTableError> {
        validate_lookup_identifier(identifier, self.limits)?;
        Ok(self.function(&FunctionKey::from_identifier(identifier, arity)))
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

fn identifier_is_well_formed(identifier: &Identifier) -> bool {
    !identifier.name.is_empty()
        && identifier
            .subscript
            .as_ref()
            .is_none_or(|subscript| !subscript.is_empty())
}

fn validate_lookup_identifier(
    identifier: &Identifier,
    limits: SymbolTableLimits,
) -> Result<(), SymbolTableError> {
    let bytes = identifier
        .name
        .len()
        .checked_add(identifier.subscript.as_ref().map_or(0, String::len))
        .ok_or(SymbolTableError::ArithmeticOverflow)?;
    if bytes > limits.max_identifier_bytes {
        return Err(SymbolTableError::LookupIdentifierLimitExceeded {
            limit: limits.max_identifier_bytes,
        });
    }
    Ok(())
}

struct ValidationState<'a> {
    nodes: &'a mut usize,
    text_bytes: &'a mut usize,
    collection_elements: &'a mut usize,
}

fn add_text(
    state: &mut ValidationState<'_>,
    bytes: usize,
    source_ordinal: usize,
    limits: SymbolTableLimits,
) -> Result<(), SymbolTableError> {
    *state.text_bytes = state
        .text_bytes
        .checked_add(bytes)
        .ok_or(SymbolTableError::ArithmeticOverflow)?;
    if *state.text_bytes > limits.max_text_bytes {
        return Err(SymbolTableError::TextLimitExceeded {
            source_ordinal,
            limit: limits.max_text_bytes,
        });
    }
    Ok(())
}

fn add_collection(
    state: &mut ValidationState<'_>,
    count: usize,
    source_ordinal: usize,
    limits: SymbolTableLimits,
) -> Result<(), SymbolTableError> {
    *state.collection_elements = state
        .collection_elements
        .checked_add(count)
        .ok_or(SymbolTableError::ArithmeticOverflow)?;
    if *state.collection_elements > limits.max_collection_elements {
        return Err(SymbolTableError::CollectionLimitExceeded {
            source_ordinal,
            limit: limits.max_collection_elements,
        });
    }
    Ok(())
}

fn add_identifier(
    state: &mut ValidationState<'_>,
    identifier: &Identifier,
    source_ordinal: usize,
    limits: SymbolTableLimits,
) -> Result<(), SymbolTableError> {
    let bytes = identifier
        .name
        .len()
        .checked_add(identifier.subscript.as_ref().map_or(0, String::len))
        .ok_or(SymbolTableError::ArithmeticOverflow)?;
    if bytes > limits.max_identifier_bytes {
        return Err(SymbolTableError::IdentifierLimitExceeded {
            source_ordinal,
            limit: limits.max_identifier_bytes,
        });
    }
    add_text(state, identifier.name.len(), source_ordinal, limits)?;
    if let Some(subscript) = &identifier.subscript {
        add_text(state, subscript.len(), source_ordinal, limits)?;
    }
    Ok(())
}

fn validate_expression_limits(
    expression: &MathExpression,
    source_ordinal: usize,
    limits: SymbolTableLimits,
    depth: usize,
    state: &mut ValidationState<'_>,
) -> Result<(), SymbolTableError> {
    if depth > limits.max_ast_depth {
        return Err(SymbolTableError::DepthLimitExceeded {
            source_ordinal,
            limit: limits.max_ast_depth,
        });
    }
    *state.nodes = state
        .nodes
        .checked_add(1)
        .ok_or(SymbolTableError::ArithmeticOverflow)?;
    if *state.nodes > limits.max_ast_nodes {
        return Err(SymbolTableError::NodeLimitExceeded {
            source_ordinal,
            limit: limits.max_ast_nodes,
        });
    }
    match &expression.kind {
        MathExpressionKind::Real(real) => {
            add_text(state, real.lexeme.len(), source_ordinal, limits)?
        }
        MathExpressionKind::Identifier(identifier) => {
            add_identifier(state, identifier, source_ordinal, limits)?
        }
        MathExpressionKind::Unsupported(node) => {
            add_text(state, node.name.local_name.len(), source_ordinal, limits)?;
            if let Some(namespace) = &node.name.namespace_uri {
                add_text(state, namespace.len(), source_ordinal, limits)?;
            }
            if let Some(feature) = &node.feature {
                add_text(state, feature.local_name.len(), source_ordinal, limits)?;
                if let Some(namespace) = &feature.namespace_uri {
                    add_text(state, namespace.len(), source_ordinal, limits)?;
                }
            }
        }
        _ => {}
    }
    match &expression.kind {
        MathExpressionKind::FunctionCall(FunctionCall { arguments, .. }) => {
            add_collection(state, arguments.len(), source_ordinal, limits)?;
        }
        MathExpressionKind::FunctionDefinition(FunctionDefinition { parameters, .. }) => {
            add_collection(state, parameters.len(), source_ordinal, limits)?;
        }
        MathExpressionKind::ArrayIndex(ArrayIndex { indices, .. }) => {
            add_collection(state, indices.len(), source_ordinal, limits)?
        }
        MathExpressionKind::Matrix(Matrix { elements, .. })
        | MathExpressionKind::Vector(Vector { elements, .. }) => {
            add_collection(state, elements.len(), source_ordinal, limits)?
        }
        MathExpressionKind::UnitedValue(UnitedValue { units, .. }) => {
            add_collection(state, units.factors.len(), source_ordinal, limits)?;
            if let Some(system) = &units.system {
                add_text(state, system.len(), source_ordinal, limits)?;
            }
            for factor in &units.factors {
                add_text(state, factor.unit.len(), source_ordinal, limits)?;
            }
        }
        _ => {}
    }
    let mut visit = |child: &MathExpression| {
        validate_expression_limits(child, source_ordinal, limits, depth + 1, state)
    };
    match &expression.kind {
        MathExpressionKind::Real(_)
        | MathExpressionKind::Identifier(_)
        | MathExpressionKind::Unsupported(_) => {}
        MathExpressionKind::Binary(BinaryExpression { left, right, .. })
        | MathExpressionKind::Comparison(ComparisonExpression { left, right, .. })
        | MathExpressionKind::Boolean(BooleanExpression { left, right, .. }) => {
            visit(left)?;
            visit(right)?;
        }
        MathExpressionKind::Definition(Definition { target, value, .. }) => {
            visit(target)?;
            visit(value)?;
        }
        MathExpressionKind::Evaluation(Evaluation {
            expression,
            unit_override,
            saved_result,
        }) => {
            visit(expression)?;
            if let Some(value) = unit_override {
                visit(value)?;
            }
            if let Some(value) = saved_result {
                visit(value)?;
            }
        }
        MathExpressionKind::FunctionCall(FunctionCall { callee, arguments }) => {
            visit(callee)?;
            for argument in arguments {
                visit(argument)?;
            }
        }
        MathExpressionKind::FunctionDefinition(FunctionDefinition {
            name,
            parameters,
            body,
            ..
        }) => {
            visit(name)?;
            for parameter in parameters {
                visit(parameter)?;
            }
            visit(body)?;
        }
        MathExpressionKind::Unary(UnaryExpression { operand, .. })
        | MathExpressionKind::Grouping(Grouping {
            expression: operand,
            ..
        })
        | MathExpressionKind::LogicalNot(math_model::LogicalNot { operand }) => visit(operand)?,
        MathExpressionKind::ArrayIndex(ArrayIndex { target, indices }) => {
            visit(target)?;
            for index in indices {
                visit(index)?;
            }
        }
        MathExpressionKind::Matrix(Matrix { elements, .. })
        | MathExpressionKind::Vector(Vector { elements, .. }) => {
            for element in elements {
                visit(element)?;
            }
        }
        MathExpressionKind::Range(RangeExpression { start, next, end }) => {
            visit(start)?;
            if let Some(value) = next {
                visit(value)?;
            }
            visit(end)?;
        }
        MathExpressionKind::Integral(Integral {
            bound_variable,
            integrand,
            bounds,
            ..
        }) => {
            visit(bound_variable)?;
            visit(integrand)?;
            if let Some(bounds) = bounds {
                visit(&bounds.lower)?;
                visit(&bounds.upper)?;
            }
        }
        MathExpressionKind::Derivative(Derivative {
            bound_variable,
            expression,
            degree,
            ..
        }) => {
            visit(bound_variable)?;
            visit(expression)?;
            if let Some(value) = degree {
                visit(value)?;
            }
        }
        MathExpressionKind::Aggregate(AggregateExpression {
            bound_variable,
            body,
            bounds,
            ..
        }) => {
            visit(bound_variable)?;
            visit(body)?;
            if let Some(bounds) = bounds {
                visit(&bounds.lower)?;
                visit(&bounds.upper)?;
            }
        }
        MathExpressionKind::UnitedValue(UnitedValue { value, .. }) => visit(value)?,
    }
    Ok(())
}
