use math_engine::{
    FunctionKey, ReferenceAnalyzer, ReferenceError, ReferenceIdentity, ReferenceInput,
    ReferenceLimits, SymbolKey,
};
use math_model::{
    AggregateExpression, AggregateOperator, ArrayIndex, Definition, DefinitionKind,
    DefinitionStyle, Derivative, DerivativeStyle, ExpressionOrigin, FunctionCall,
    FunctionDefinition, Grouping, Identifier, Integral, MathExpression, MathExpressionKind, Matrix,
    UnitMonomial, UnitReference, UnitedValue,
};
use std::num::NonZeroI64;

fn id(name: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Identifier(Identifier {
            name: name.into(),
            subscript: None,
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn raw_id(name: &str, subscript: Option<&str>) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Identifier(Identifier {
            name: name.into(),
            subscript: subscript.map(Into::into),
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn limits() -> ReferenceLimits {
    ReferenceLimits::new(100, 256, 100_000, 100_000, 100_000, 100_000, 100)
}

fn call(name: &str, args: Vec<MathExpression>) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::FunctionCall(FunctionCall {
            callee: Box::new(id(name)),
            arguments: args,
        }),
        origin: ExpressionOrigin::Derived,
    }
}

#[test]
fn free_references_are_first_occurrence_deduplicated_and_typed_by_namespace() {
    let expression = MathExpression {
        kind: MathExpressionKind::Definition(Definition {
            kind: DefinitionKind::Define,
            style: DefinitionStyle::Equal,
            target: Box::new(id("defined")),
            value: Box::new(MathExpression {
                kind: MathExpressionKind::Binary(math_model::BinaryExpression {
                    operator: math_model::BinaryOperator::Add,
                    multiplication_style: None,
                    left: Box::new(id("x")),
                    right: Box::new(call("f", vec![id("x")])),
                }),
                origin: ExpressionOrigin::Derived,
            }),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let analyzer = ReferenceAnalyzer::new(limits());
    let result = analyzer
        .analyze(&[ReferenceInput::new(7, &expression)])
        .expect("references");
    assert_eq!(result.len(), 2);
    assert!(
        matches!(result.references()[0].identity, ReferenceIdentity::Variable(ref key) if key == &SymbolKey::new("x", None))
    );
    assert!(
        matches!(result.references()[1].identity, ReferenceIdentity::Function(ref key) if key == &FunctionKey::new("f", None, 1))
    );
    assert_eq!(result.references()[0].source_ordinal, 7);
}

#[test]
fn dedup_is_scoped_to_each_input_source_site() {
    let first = MathExpression {
        kind: MathExpressionKind::Binary(math_model::BinaryExpression {
            operator: math_model::BinaryOperator::Add,
            multiplication_style: None,
            left: Box::new(id("x")),
            right: Box::new(id("x")),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let second = id("x");
    let result = ReferenceAnalyzer::new(limits())
        .analyze(&[
            ReferenceInput::new(4, &first),
            ReferenceInput::new(9, &second),
        ])
        .expect("references");
    assert_eq!(result.len(), 2);
    assert_eq!(result.references()[0].source_ordinal, 4);
    assert_eq!(result.references()[1].source_ordinal, 9);
}

#[test]
fn lexical_binders_exclude_parameters_and_calculus_bound_variables() {
    let function_expression = MathExpression {
        kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
            style: DefinitionStyle::Equal,
            name: Box::new(id("g")),
            parameters: vec![id("x")],
            body: Box::new(MathExpression {
                kind: MathExpressionKind::Integral(Integral {
                    bound_variable: Box::new(id("i")),
                    integrand: Box::new(MathExpression {
                        kind: MathExpressionKind::Aggregate(AggregateExpression {
                            operator: AggregateOperator::Summation,
                            bound_variable: Box::new(id("j")),
                            body: Box::new(MathExpression {
                                kind: MathExpressionKind::Derivative(Derivative {
                                    bound_variable: Box::new(id("k")),
                                    expression: Box::new(MathExpression {
                                        kind: MathExpressionKind::Binary(
                                            math_model::BinaryExpression {
                                                operator: math_model::BinaryOperator::Add,
                                                multiplication_style: None,
                                                left: Box::new(id("x")),
                                                right: Box::new(id("free")),
                                            },
                                        ),
                                        origin: ExpressionOrigin::Derived,
                                    }),
                                    degree: None,
                                    style: DerivativeStyle::Default,
                                }),
                                origin: ExpressionOrigin::Derived,
                            }),
                            bounds: None,
                        }),
                        origin: ExpressionOrigin::Derived,
                    }),
                    bounds: None,
                    algorithm: None,
                }),
                origin: ExpressionOrigin::Derived,
            }),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let result = ReferenceAnalyzer::new(limits())
        .analyze(&[ReferenceInput::new(0, &function_expression)])
        .expect("references");
    assert_eq!(result.len(), 1);
    assert!(
        matches!(result.references()[0].identity, ReferenceIdentity::Variable(ref key) if key == &SymbolKey::new("free", None))
    );
}

#[test]
fn function_callee_is_function_reference_and_array_target_is_variable_reference() {
    let expression = MathExpression {
        kind: MathExpressionKind::ArrayIndex(ArrayIndex {
            target: Box::new(call("f", vec![id("x")])),
            indices: vec![id("i")],
        }),
        origin: ExpressionOrigin::Derived,
    };
    let result = ReferenceAnalyzer::new(limits())
        .analyze(&[ReferenceInput::new(0, &expression)])
        .expect("references");
    assert_eq!(result.len(), 3);
    assert!(
        matches!(result.references()[0].identity, ReferenceIdentity::Function(ref key) if key == &FunctionKey::new("f", None, 1))
    );
    assert!(
        matches!(result.references()[1].identity, ReferenceIdentity::Variable(ref key) if key == &SymbolKey::new("x", None))
    );
    assert!(
        matches!(result.references()[2].identity, ReferenceIdentity::Variable(ref key) if key == &SymbolKey::new("i", None))
    );
}

#[test]
fn function_callee_has_its_own_node_and_depth_budget_accounting() {
    let expression = call("f", vec![]);
    let error = ReferenceAnalyzer::new(ReferenceLimits::new(10, 10, 1, 100, 100, 100, 100))
        .analyze(&[ReferenceInput::new(0, &expression)])
        .expect_err("callee node must count");
    assert_eq!(
        error,
        ReferenceError::NodeLimitExceeded {
            source_ordinal: 0,
            limit: 1,
        }
    );

    let nested = MathExpression {
        kind: MathExpressionKind::Unary(math_model::UnaryExpression {
            operator: math_model::UnaryOperator::Negate,
            operand: Box::new(expression),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = ReferenceAnalyzer::new(ReferenceLimits::new(10, 1, 100, 100, 100, 100, 100))
        .analyze(&[ReferenceInput::new(0, &nested)])
        .expect_err("callee depth must count");
    assert_eq!(
        error,
        ReferenceError::DepthLimitExceeded {
            source_ordinal: 0,
            limit: 1,
        }
    );
}

#[test]
fn malformed_binders_unsupported_nodes_and_ambiguous_callees_fail_closed_redacted() {
    let bad_callee = MathExpression {
        kind: MathExpressionKind::FunctionCall(FunctionCall {
            callee: Box::new(MathExpression {
                kind: MathExpressionKind::ArrayIndex(ArrayIndex {
                    target: Box::new(id("f")),
                    indices: vec![],
                }),
                origin: ExpressionOrigin::Derived,
            }),
            arguments: vec![],
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = ReferenceAnalyzer::new(limits())
        .analyze(&[ReferenceInput::new(3, &bad_callee)])
        .expect_err("ambiguous callee");
    assert_eq!(
        error,
        ReferenceError::AmbiguousFunctionCallee { source_ordinal: 3 }
    );
    assert!(!format!("{error:?}").contains("f"));

    let unsupported = MathExpression {
        kind: MathExpressionKind::Unsupported(math_model::UnsupportedNode {
            name: math_model::ExpandedName {
                namespace_uri: None,
                local_name: "secret-node".into(),
            },
            feature: None,
            span: math_model::SourceSpan { start: 0, end: 1 },
            reason: math_model::UnsupportedReason::UnknownExpression,
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = ReferenceAnalyzer::new(limits())
        .analyze(&[ReferenceInput::new(4, &unsupported)])
        .expect_err("unsupported");
    assert_eq!(
        error,
        ReferenceError::UnsupportedExpression { source_ordinal: 4 }
    );
    assert!(!format!("{error:?}").contains("secret"));
}

#[test]
fn borrowed_preflight_is_cumulative_and_original_ast_remains_unchanged() {
    let first = id("first");
    let second = id("second");
    let before = vec![first.clone(), second.clone()];
    let error = ReferenceAnalyzer::new(ReferenceLimits::new(10, 10, 1, 100, 100, 100, 100))
        .analyze(&[
            ReferenceInput::new(0, &first),
            ReferenceInput::new(1, &second),
        ])
        .expect_err("cumulative node limit");
    assert_eq!(
        error,
        ReferenceError::NodeLimitExceeded {
            source_ordinal: 1,
            limit: 1
        }
    );
    assert_eq!(vec![first, second], before);

    let error = ReferenceAnalyzer::new(ReferenceLimits::new(10, 0, 100, 100, 100, 100, 100))
        .analyze(&[])
        .expect_err("zero depth limit");
    assert_eq!(error, ReferenceError::InvalidLimits);
}

#[test]
fn text_collection_and_reference_budgets_fail_before_result_allocation() {
    let first = id("aa");
    let second = id("bb");
    let error = ReferenceAnalyzer::new(ReferenceLimits::new(10, 10, 100, 3, 100, 100, 100))
        .analyze(&[
            ReferenceInput::new(0, &first),
            ReferenceInput::new(1, &second),
        ])
        .expect_err("cumulative text bound");
    assert_eq!(
        error,
        ReferenceError::TextLimitExceeded {
            source_ordinal: 1,
            limit: 3
        }
    );

    let unit_expression = MathExpression {
        kind: MathExpressionKind::UnitedValue(UnitedValue {
            value: Box::new(id("value")),
            units: UnitMonomial {
                system: Some("secret-system".into()),
                factors: vec![
                    UnitReference {
                        unit: "m".into(),
                        power_numerator: 1,
                        power_denominator: NonZeroI64::new(1).unwrap(),
                    },
                    UnitReference {
                        unit: "s".into(),
                        power_numerator: -1,
                        power_denominator: NonZeroI64::new(1).unwrap(),
                    },
                ],
            },
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = ReferenceAnalyzer::new(ReferenceLimits::new(10, 10, 100, 100, 100, 1, 100))
        .analyze(&[ReferenceInput::new(2, &unit_expression)])
        .expect_err("unit collection bound");
    assert_eq!(
        error,
        ReferenceError::CollectionLimitExceeded {
            source_ordinal: 2,
            limit: 1,
        }
    );

    let repeated = call("f", vec![id("x"), id("x")]);
    let error = ReferenceAnalyzer::new(ReferenceLimits::new(10, 10, 100, 100, 100, 100, 1))
        .analyze(&[ReferenceInput::new(3, &repeated)])
        .expect_err("raw reference occurrence bound");
    assert_eq!(
        error,
        ReferenceError::ReferenceLimitExceeded {
            source_ordinal: 3,
            limit: 1,
        }
    );
}

#[test]
fn parameter_scope_lookup_scales_and_preserves_duplicate_shadow_counts() {
    let mut parameter_names: Vec<String> = (0..128).map(|i| format!("p{i}")).collect();
    parameter_names.extend(["shadow".into(), "shadow".into()]);
    let parameters = parameter_names.iter().map(|name| id(name)).collect();
    let body = MathExpression {
        kind: MathExpressionKind::Matrix(Matrix {
            elements: parameter_names.iter().map(|name| id(name)).collect(),
            rows: 1,
            columns: parameter_names.len(),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let function = MathExpression {
        kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
            style: DefinitionStyle::Equal,
            name: Box::new(id("many")),
            parameters,
            body: Box::new(body),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let expression = MathExpression {
        kind: MathExpressionKind::Matrix(Matrix {
            elements: vec![function, id("shadow")],
            rows: 1,
            columns: 2,
        }),
        origin: ExpressionOrigin::Derived,
    };

    let result = ReferenceAnalyzer::new(ReferenceLimits::new(
        10, 256, 1_000, 10_000, 10_000, 10_000, 10,
    ))
    .analyze(&[ReferenceInput::new(0, &expression)])
    .expect("bounded parameter lookup");
    assert_eq!(result.len(), 1);
    assert!(matches!(
        result.references()[0].identity,
        ReferenceIdentity::Variable(ref key) if key == &SymbolKey::new("shadow", None)
    ));
}

#[test]
fn united_value_preflights_and_collects_its_nested_value() {
    let unit_value = MathExpression {
        kind: MathExpressionKind::UnitedValue(UnitedValue {
            value: Box::new(id("x")),
            units: UnitMonomial {
                system: None,
                factors: vec![],
            },
        }),
        origin: ExpressionOrigin::Derived,
    };

    let result = ReferenceAnalyzer::new(limits())
        .analyze(&[ReferenceInput::new(11, &unit_value)])
        .expect("nested unit value reference");
    assert!(matches!(
        result.references()[0].identity,
        ReferenceIdentity::Variable(ref key) if key == &SymbolKey::new("x", None)
    ));

    let error = ReferenceAnalyzer::new(ReferenceLimits::new(10, 10, 1, 100, 100, 100, 100))
        .analyze(&[ReferenceInput::new(12, &unit_value)])
        .expect_err("nested unit value node must count");
    assert_eq!(
        error,
        ReferenceError::NodeLimitExceeded {
            source_ordinal: 12,
            limit: 1,
        }
    );

    let nested = MathExpression {
        kind: MathExpressionKind::Grouping(Grouping {
            expression: Box::new(unit_value),
            unpaired: false,
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = ReferenceAnalyzer::new(ReferenceLimits::new(10, 1, 100, 100, 100, 100, 100))
        .analyze(&[ReferenceInput::new(13, &nested)])
        .expect_err("nested unit value depth must count");
    assert_eq!(
        error,
        ReferenceError::DepthLimitExceeded {
            source_ordinal: 13,
            limit: 1,
        }
    );
}

#[test]
fn malformed_identifier_forms_are_rejected_with_contextual_redacted_errors() {
    let analyzer = ReferenceAnalyzer::new(limits());

    let error = analyzer
        .analyze(&[ReferenceInput::new(20, &raw_id("", None))])
        .expect_err("empty free name");
    assert_eq!(
        error,
        ReferenceError::InvalidReferenceIdentifier { source_ordinal: 20 }
    );

    let error = analyzer
        .analyze(&[ReferenceInput::new(21, &raw_id("x", Some("")))])
        .expect_err("empty free subscript");
    assert_eq!(
        error,
        ReferenceError::InvalidReferenceIdentifier { source_ordinal: 21 }
    );

    let definition = MathExpression {
        kind: MathExpressionKind::Definition(Definition {
            kind: DefinitionKind::Define,
            style: DefinitionStyle::Equal,
            target: Box::new(raw_id("", None)),
            value: Box::new(id("value")),
        }),
        origin: ExpressionOrigin::Derived,
    };
    assert_eq!(
        analyzer
            .analyze(&[ReferenceInput::new(22, &definition)])
            .expect_err("empty definition target"),
        ReferenceError::InvalidDefinitionTarget { source_ordinal: 22 }
    );

    let function_name = MathExpression {
        kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
            style: DefinitionStyle::Equal,
            name: Box::new(raw_id("f", Some(""))),
            parameters: vec![],
            body: Box::new(id("value")),
        }),
        origin: ExpressionOrigin::Derived,
    };
    assert_eq!(
        analyzer
            .analyze(&[ReferenceInput::new(23, &function_name)])
            .expect_err("empty function name subscript"),
        ReferenceError::InvalidFunctionName { source_ordinal: 23 }
    );

    let function_parameter = MathExpression {
        kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
            style: DefinitionStyle::Equal,
            name: Box::new(id("f")),
            parameters: vec![raw_id("p", Some(""))],
            body: Box::new(id("value")),
        }),
        origin: ExpressionOrigin::Derived,
    };
    assert_eq!(
        analyzer
            .analyze(&[ReferenceInput::new(24, &function_parameter)])
            .expect_err("empty parameter subscript"),
        ReferenceError::InvalidFunctionParameter {
            source_ordinal: 24,
            parameter_index: 0,
        }
    );

    let binder = MathExpression {
        kind: MathExpressionKind::Integral(Integral {
            bound_variable: Box::new(raw_id("", None)),
            integrand: Box::new(id("value")),
            bounds: None,
            algorithm: None,
        }),
        origin: ExpressionOrigin::Derived,
    };
    assert_eq!(
        analyzer
            .analyze(&[ReferenceInput::new(25, &binder)])
            .expect_err("empty calculus binder"),
        ReferenceError::InvalidBinder { source_ordinal: 25 }
    );

    let callee = MathExpression {
        kind: MathExpressionKind::FunctionCall(FunctionCall {
            callee: Box::new(raw_id("f", Some(""))),
            arguments: vec![],
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = analyzer
        .analyze(&[ReferenceInput::new(26, &callee)])
        .expect_err("empty callee subscript");
    assert_eq!(
        error,
        ReferenceError::InvalidFunctionCallee { source_ordinal: 26 }
    );
    assert!(!format!("{error:?}").contains('f'));
}
