use math_engine::{
    FunctionKey, SymbolDefinition, SymbolInput, SymbolKey, SymbolTable, SymbolTableError,
    SymbolTableLimits,
};
use math_model::{
    BinaryExpression, BinaryOperator, Definition, DefinitionKind, DefinitionStyle, ExpandedName,
    ExpressionOrigin, FunctionDefinition, Identifier, MathExpression, MathExpressionKind,
    NumericBase, RealLiteral, SourceSpan, UnitMonomial, UnitReference, UnitedValue,
    UnsupportedNode, UnsupportedReason,
};
use std::{num::NonZeroI64, process::Command, sync::Arc};

fn identifier(name: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Identifier(Identifier {
            name: name.to_owned(),
            subscript: None,
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn scalar(name: &str, value: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Definition(Definition {
            kind: DefinitionKind::Define,
            style: DefinitionStyle::ColonEqual,
            target: Box::new(identifier(name)),
            value: Box::new(identifier(value)),
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn function(name: &str, parameters: &[&str], body: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
            style: DefinitionStyle::ColonEqual,
            name: Box::new(identifier(name)),
            parameters: parameters
                .iter()
                .map(|parameter| identifier(parameter))
                .collect(),
            body: Box::new(identifier(body)),
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn limits() -> SymbolTableLimits {
    SymbolTableLimits::new(100, 100, 256, 100_000, 100_000, 100_000, 100_000)
}

#[test]
fn scalar_and_function_namespaces_are_separate_and_function_arity_is_keyed() {
    let scalar_expression = scalar("f", "scalar-value");
    let one_argument = function("f", &["x"], "x");
    let two_arguments = function("f", &["x", "y"], "x");
    let inputs = vec![
        SymbolInput::new(4, &scalar_expression),
        SymbolInput::new(9, &one_argument),
        SymbolInput::new(20, &two_arguments),
    ];
    let table = SymbolTable::build(inputs, limits()).expect("table");

    assert!(table.variable(&SymbolKey::new("f", None)).is_some());
    assert!(table.function(&FunctionKey::new("f", None, 1)).is_some());
    assert!(table.function(&FunctionKey::new("f", None, 2)).is_some());
    assert!(table.function(&FunctionKey::new("f", None, 3)).is_none());
    assert_eq!(table.definition_count(), 3);
}

#[test]
fn revisions_are_retained_and_visibility_is_strictly_before_source_ordinal() {
    let first = scalar("x", "first");
    let unrelated = identifier("not_a_definition");
    let second = scalar("x", "second");
    let original = vec![first.clone(), unrelated, second.clone()];
    let table = SymbolTable::from_expressions(&original, limits()).expect("table");
    let key = SymbolKey::new("x", None);

    let history = table.variable_history(&key).expect("revision history");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].source_ordinal, 0);
    assert_eq!(history[1].source_ordinal, 2);
    assert_eq!(table.visible_variable_before(&key, 0), None);
    assert_eq!(
        table
            .visible_variable_before(&key, 1)
            .unwrap()
            .source_ordinal,
        0
    );
    assert_eq!(
        table
            .visible_variable_before(&key, 2)
            .unwrap()
            .source_ordinal,
        0
    );
    assert_eq!(table.variable(&key).unwrap().source_ordinal, 2);
    assert_eq!(table.definition_count(), 2);
    assert_eq!(original[0], first);
    assert_eq!(original[2], second);
}

#[test]
fn canonical_expression_is_shared_and_original_ast_is_unchanged() {
    let scalar_expression = scalar("x", "private-rhs");
    let function_expression = function("f", &["argument"], "private-body");
    let original = vec![scalar_expression.clone(), function_expression.clone()];
    let table = SymbolTable::from_expressions(&original, limits()).expect("table");

    let SymbolDefinition::Variable(variable) = &table.definitions()[0] else {
        panic!("variable")
    };
    let SymbolDefinition::Function(function) = &table.definitions()[1] else {
        panic!("function")
    };
    assert_eq!(variable.expression.as_ref(), &scalar_expression);
    assert_eq!(function.expression.as_ref(), &function_expression);
    assert!(std::sync::Arc::ptr_eq(
        &variable.expression,
        &table.variable_history(&variable.key).unwrap()[0].expression
    ));
    assert_eq!(original, vec![scalar_expression, function_expression]);
}

#[test]
fn malformed_targets_and_parameters_are_typed_and_redacted() {
    let malformed_target = MathExpression {
        kind: MathExpressionKind::Definition(Definition {
            kind: DefinitionKind::Define,
            style: DefinitionStyle::Equal,
            target: Box::new(MathExpression {
                kind: MathExpressionKind::Binary(BinaryExpression {
                    operator: BinaryOperator::Add,
                    multiplication_style: None,
                    left: Box::new(identifier("secret-left")),
                    right: Box::new(identifier("secret-right")),
                }),
                origin: ExpressionOrigin::Derived,
            }),
            value: Box::new(identifier("secret-value")),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = SymbolTable::build([SymbolInput::new(17, &malformed_target)], limits())
        .expect_err("malformed target");
    assert_eq!(
        error,
        SymbolTableError::InvalidDefinitionTarget { source_ordinal: 17 }
    );
    assert!(!format!("{error:?}").contains("secret"));

    let malformed_parameter = MathExpression {
        kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
            style: DefinitionStyle::Equal,
            name: Box::new(identifier("private-f")),
            parameters: vec![MathExpression {
                kind: MathExpressionKind::Binary(BinaryExpression {
                    operator: BinaryOperator::Add,
                    multiplication_style: None,
                    left: Box::new(identifier("a")),
                    right: Box::new(identifier("b")),
                }),
                origin: ExpressionOrigin::Derived,
            }],
            body: Box::new(identifier("private-body")),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = SymbolTable::build([SymbolInput::new(19, &malformed_parameter)], limits())
        .expect_err("malformed parameter");
    assert_eq!(
        error,
        SymbolTableError::InvalidFunctionParameter {
            source_ordinal: 19,
            parameter_index: 0,
        }
    );
    assert!(!format!("{error:?}").contains("private-"));
}

#[test]
fn limits_are_cumulative_and_zero_depth_or_order_violations_fail() {
    let first = identifier("first");
    let second = identifier("second");
    let error = SymbolTable::build(
        [SymbolInput::new(0, &first), SymbolInput::new(1, &second)],
        SymbolTableLimits::new(10, 10, 10, 1, 100, 100, 100),
    )
    .expect_err("cumulative node bound");
    assert_eq!(
        error,
        SymbolTableError::NodeLimitExceeded {
            source_ordinal: 1,
            limit: 1
        }
    );

    let error = SymbolTable::build(
        [SymbolInput::new(0, &first)],
        SymbolTableLimits::new(10, 10, 0, 100, 100, 100, 100),
    )
    .expect_err("zero depth budget");
    assert_eq!(error, SymbolTableError::InvalidLimits);

    let error = SymbolTable::build(
        [SymbolInput::new(2, &first), SymbolInput::new(1, &second)],
        limits(),
    )
    .expect_err("non-increasing ordinals");
    assert_eq!(
        error,
        SymbolTableError::NonIncreasingSourceOrdinal {
            previous: 2,
            current: 1,
        }
    );
}

#[test]
fn text_identifier_and_collection_budgets_are_checked_before_indexing() {
    let long_identifier = identifier("secret-name");
    let error = SymbolTable::build(
        [SymbolInput::new(3, &long_identifier)],
        SymbolTableLimits::new(10, 10, 10, 100, 100, 4, 100),
    )
    .expect_err("identifier budget");
    assert_eq!(
        error,
        SymbolTableError::IdentifierLimitExceeded {
            source_ordinal: 3,
            limit: 4,
        }
    );
    assert!(!format!("{error:?}").contains("secret"));

    let first = identifier("aa");
    let second = identifier("bb");
    let error = SymbolTable::build(
        [SymbolInput::new(0, &first), SymbolInput::new(1, &second)],
        SymbolTableLimits::new(10, 10, 10, 100, 3, 100, 100),
    )
    .expect_err("cumulative text budget");
    assert_eq!(
        error,
        SymbolTableError::TextLimitExceeded {
            source_ordinal: 1,
            limit: 3,
        }
    );

    let function_expression = function("f", &["x", "y"], "x");
    let error = SymbolTable::build(
        [SymbolInput::new(0, &function_expression)],
        SymbolTableLimits::new(10, 10, 10, 100, 100, 100, 1),
    )
    .expect_err("collection budget");
    assert_eq!(
        error,
        SymbolTableError::CollectionLimitExceeded {
            source_ordinal: 0,
            limit: 1,
        }
    );
}

#[test]
fn every_payload_branch_is_bounded_before_clone_and_lookup() {
    let real = MathExpression {
        kind: MathExpressionKind::Real(RealLiteral {
            lexeme: "123456".to_owned(),
            base: NumericBase::Decimal,
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = SymbolTable::build(
        [SymbolInput::new(0, &real)],
        SymbolTableLimits::new(10, 10, 10, 100, 4, 100, 100),
    )
    .expect_err("real text budget");
    assert!(matches!(error, SymbolTableError::TextLimitExceeded { .. }));

    let unsupported = MathExpression {
        kind: MathExpressionKind::Unsupported(UnsupportedNode {
            name: ExpandedName {
                namespace_uri: Some(Arc::from("private-namespace")),
                local_name: "private-local".to_owned(),
            },
            feature: Some(ExpandedName {
                namespace_uri: Some(Arc::from("private-feature-namespace")),
                local_name: "private-feature".to_owned(),
            }),
            span: SourceSpan { start: 0, end: 1 },
            reason: UnsupportedReason::UnknownExpression,
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = SymbolTable::build(
        [SymbolInput::new(0, &unsupported)],
        SymbolTableLimits::new(10, 10, 10, 100, 8, 100, 100),
    )
    .expect_err("unsupported text budget");
    assert!(matches!(error, SymbolTableError::TextLimitExceeded { .. }));
    assert!(!format!("{error:?}").contains("private"));

    let united = MathExpression {
        kind: MathExpressionKind::UnitedValue(UnitedValue {
            value: Box::new(identifier("x")),
            units: UnitMonomial {
                system: Some("private-system".to_owned()),
                factors: vec![
                    UnitReference {
                        unit: "private-unit-a".to_owned(),
                        power_numerator: 1,
                        power_denominator: NonZeroI64::new(1).unwrap(),
                    },
                    UnitReference {
                        unit: "private-unit-b".to_owned(),
                        power_numerator: 1,
                        power_denominator: NonZeroI64::new(1).unwrap(),
                    },
                ],
            },
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = SymbolTable::build(
        [SymbolInput::new(0, &united)],
        SymbolTableLimits::new(10, 10, 10, 100, 100, 100, 1),
    )
    .expect_err("unit factor collection budget");
    assert!(matches!(
        error,
        SymbolTableError::CollectionLimitExceeded { .. }
    ));

    let table = SymbolTable::build_with_defaults(std::iter::empty::<SymbolInput<'_>>())
        .expect("empty table");
    let oversized = Identifier {
        name: "x".repeat(SymbolTableLimits::default().max_identifier_bytes + 1),
        subscript: None,
    };
    let error = table
        .lookup_variable(&oversized)
        .expect_err("lookup identifier budget");
    assert_eq!(
        error,
        SymbolTableError::LookupIdentifierLimitExceeded {
            limit: SymbolTableLimits::default().max_identifier_bytes,
        }
    );
}

#[test]
fn unsupported_and_unit_nested_text_fields_have_direct_budget_regressions() {
    let cases = [
        UnsupportedNode {
            name: ExpandedName {
                namespace_uri: Some(Arc::from("private-namespace")),
                local_name: "a".to_owned(),
            },
            feature: None,
            span: SourceSpan { start: 0, end: 1 },
            reason: UnsupportedReason::UnknownExpression,
        },
        UnsupportedNode {
            name: ExpandedName {
                namespace_uri: None,
                local_name: "a".to_owned(),
            },
            feature: Some(ExpandedName {
                namespace_uri: None,
                local_name: "private-feature".to_owned(),
            }),
            span: SourceSpan { start: 0, end: 1 },
            reason: UnsupportedReason::UnknownExpression,
        },
        UnsupportedNode {
            name: ExpandedName {
                namespace_uri: None,
                local_name: "a".to_owned(),
            },
            feature: Some(ExpandedName {
                namespace_uri: Some(Arc::from("private-feature-namespace")),
                local_name: "b".to_owned(),
            }),
            span: SourceSpan { start: 0, end: 1 },
            reason: UnsupportedReason::UnknownExpression,
        },
    ];
    for node in cases {
        let expression = MathExpression {
            kind: MathExpressionKind::Unsupported(node),
            origin: ExpressionOrigin::Derived,
        };
        let error = SymbolTable::build(
            [SymbolInput::new(0, &expression)],
            SymbolTableLimits::new(10, 10, 10, 100, 3, 100, 100),
        )
        .expect_err("nested unsupported text budget");
        assert!(matches!(error, SymbolTableError::TextLimitExceeded { .. }));
        assert!(!format!("{error:?}").contains("private"));
    }

    for (system, unit) in [
        (Some("private-system".to_owned()), "u".to_owned()),
        (None, "private-unit".to_owned()),
    ] {
        let expression = MathExpression {
            kind: MathExpressionKind::UnitedValue(UnitedValue {
                value: Box::new(identifier("x")),
                units: UnitMonomial {
                    system,
                    factors: vec![UnitReference {
                        unit,
                        power_numerator: 1,
                        power_denominator: NonZeroI64::new(1).unwrap(),
                    }],
                },
            }),
            origin: ExpressionOrigin::Derived,
        };
        let error = SymbolTable::build(
            [SymbolInput::new(0, &expression)],
            SymbolTableLimits::new(10, 10, 10, 100, 4, 100, 10),
        )
        .expect_err("nested unit text budget");
        assert!(matches!(error, SymbolTableError::TextLimitExceeded { .. }));
        assert!(!format!("{error:?}").contains("private"));
    }
}

#[test]
fn depth_is_rejected_while_input_remains_borrowed_and_unchanged() {
    let nested = MathExpression {
        kind: MathExpressionKind::Unary(math_model::UnaryExpression {
            operator: math_model::UnaryOperator::Negate,
            operand: Box::new(MathExpression {
                kind: MathExpressionKind::Unary(math_model::UnaryExpression {
                    operator: math_model::UnaryOperator::Negate,
                    operand: Box::new(identifier("private-deep")),
                }),
                origin: ExpressionOrigin::Derived,
            }),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let before = nested.clone();
    let error = SymbolTable::build(
        [SymbolInput::new(0, &nested)],
        SymbolTableLimits::new(10, 10, 1, 100, 100, 100, 100),
    )
    .expect_err("depth preflight");
    assert!(matches!(error, SymbolTableError::DepthLimitExceeded { .. }));
    assert_eq!(nested, before);
}

#[test]
fn extreme_depth_is_rejected_before_any_recursive_clone() {
    const CHILD_ENV: &str = "MATHMORPH_STAGE100_DEEP_CHILD";
    if std::env::var_os(CHILD_ENV).is_some() {
        let mut expression = identifier("private-deep");
        for _ in 0..20_000 {
            expression = MathExpression {
                kind: MathExpressionKind::Unary(math_model::UnaryExpression {
                    operator: math_model::UnaryOperator::Negate,
                    operand: Box::new(expression),
                }),
                origin: ExpressionOrigin::Derived,
            };
        }
        let error = SymbolTable::build(
            [SymbolInput::new(0, &expression)],
            SymbolTableLimits::new(10, 10, 16, 100_000, 100_000, 100_000, 100_000),
        )
        .expect_err("extreme depth preflight");
        assert!(matches!(error, SymbolTableError::DepthLimitExceeded { .. }));
        std::mem::forget(expression);
        return;
    }

    let status = Command::new(std::env::current_exe().expect("current test executable"))
        .args([
            "--exact",
            "extreme_depth_is_rejected_before_any_recursive_clone",
            "--nocapture",
        ])
        .env(CHILD_ENV, "1")
        .status()
        .expect("run deep-AST subprocess");
    assert!(status.success(), "deep-AST subprocess must not abort");
}
