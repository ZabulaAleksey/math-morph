use math_engine::{SubstitutionEngine, SubstitutionError, SymbolTable, SymbolTableLimits};
use math_model::{
    AggregateExpression, AggregateOperator, ArrayIndex, BinaryExpression, BinaryOperator,
    BooleanExpression, BooleanOperator, Bounds, ComparisonExpression, ComparisonOperator,
    Definition, DefinitionKind, DefinitionStyle, Derivative, DerivativeStyle, Evaluation,
    ExpressionOrigin, FunctionCall, FunctionDefinition, Identifier, Integral, LogicalNot,
    MathExpression, MathExpressionKind, Matrix, NumericBase, RangeExpression, RealLiteral,
    UnitMonomial, UnitReference, UnitedValue, Vector, VectorOrientation,
};
use std::num::NonZeroI64;

fn identifier(name: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Identifier(Identifier {
            name: name.into(),
            subscript: None,
        }),
        origin: ExpressionOrigin::Derived,
    }
}
fn real(value: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Real(RealLiteral {
            lexeme: value.into(),
            base: NumericBase::Decimal,
        }),
        origin: ExpressionOrigin::Derived,
    }
}
fn definition(name: &str, value: MathExpression) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Definition(Definition {
            kind: DefinitionKind::Define,
            style: DefinitionStyle::Equal,
            target: Box::new(identifier(name)),
            value: Box::new(value),
        }),
        origin: ExpressionOrigin::Derived,
    }
}
fn add(left: MathExpression, right: MathExpression) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Binary(BinaryExpression {
            operator: BinaryOperator::Add,
            multiplication_style: None,
            left: Box::new(left),
            right: Box::new(right),
        }),
        origin: ExpressionOrigin::Derived,
    }
}
fn table(values: &[(usize, MathExpression)]) -> SymbolTable {
    SymbolTable::build(
        values.iter().map(|(ordinal, value)| (*ordinal, value)),
        SymbolTableLimits::default(),
    )
    .expect("symbol table")
}

#[test]
fn once_uses_the_latest_strictly_prior_scalar_revision_without_mutating_inputs() {
    let symbols = table(&[
        (1, definition("a", real("1"))),
        (5, definition("a", real("2"))),
        (9, definition("later", real("3"))),
    ]);
    let original = identifier("a");
    let before_expression = original.clone();
    let before_symbols = symbols.clone();
    let result = SubstitutionEngine::default()
        .once(&original, 7, &symbols)
        .expect("substitution");

    assert_eq!(result.substitution_count(), 1);
    assert_eq!(original, before_expression);
    assert_eq!(symbols, before_symbols);
    assert_eq!(*result.expression(), real("2"));
    let selected = result
        .trace()
        .steps()
        .iter()
        .find(|step| step.kind() == math_engine::EvaluationTraceKind::BindingSelected)
        .unwrap();
    assert_eq!(selected.binding_source_ordinal(), Some(5));
}

#[test]
fn definition_target_is_not_substituted_and_callables_fail_closed_without_payload() {
    let symbols = table(&[(1, definition("a", real("1")))]);
    let input = definition("a", identifier("a"));
    let result = SubstitutionEngine::default()
        .once(&input, 2, &symbols)
        .expect("substitution");
    let MathExpressionKind::Definition(value) = &result.expression().kind else {
        panic!("definition")
    };
    assert_eq!(*value.target, identifier("a"));
    assert_eq!(*value.value, real("1"));
    let callable = MathExpression {
        kind: MathExpressionKind::FunctionCall(FunctionCall {
            callee: Box::new(identifier("secret_function")),
            arguments: vec![],
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = SubstitutionEngine::default()
        .once(&callable, 2, &symbols)
        .expect_err("callable must fail closed");
    assert_eq!(
        error,
        SubstitutionError::UnsupportedCallable { source_ordinal: 2 }
    );
    assert!(!format!("{error:?}").contains("secret_function"));
}

#[test]
fn recursive_substitution_keeps_the_original_visibility_point() {
    let symbols = table(&[
        (1, definition("c", real("3"))),
        (2, definition("b", identifier("c"))),
        (3, definition("a", identifier("b"))),
        (9, definition("b", real("99"))),
    ]);
    let result = SubstitutionEngine::default()
        .recursive(&identifier("a"), 5, &symbols)
        .expect("recursive substitution");
    assert_eq!(*result.expression(), real("3"));
    assert_eq!(result.substitution_count(), 3);
}

#[test]
fn recursive_substitution_detects_a_cycle_without_revealing_payload() {
    let symbols = table(&[
        (1, definition("a", identifier("b"))),
        (2, definition("b", identifier("a"))),
    ]);
    let error = SubstitutionEngine::default()
        .recursive(&identifier("a"), 3, &symbols)
        .expect_err("cycle");
    assert_eq!(
        error,
        SubstitutionError::CycleDetected { source_ordinal: 3 }
    );
    assert!(!format!("{error:?}").contains("a"));
}

#[test]
fn recursive_step_limit_fails_closed_before_unbounded_history_growth() {
    let symbols = table(&[
        (1, definition("c", real("3"))),
        (2, definition("b", identifier("c"))),
        (3, definition("a", identifier("b"))),
    ]);
    let mut limits = math_engine::SubstitutionLimits::new(10);
    limits.max_recursive_steps = 2;
    assert_eq!(
        SubstitutionEngine::new(limits).recursive(&identifier("a"), 4, &symbols),
        Err(SubstitutionError::RecursiveStepLimitExceeded { limit: 2 })
    );
}

#[test]
fn lexical_binders_are_preserved_while_free_body_and_bound_references_expand() {
    let symbols = table(&[(1, definition("a", real("7")))]);
    let function = MathExpression {
        kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
            style: DefinitionStyle::Equal,
            name: Box::new(identifier("f")),
            parameters: vec![identifier("x")],
            body: Box::new(add(identifier("x"), identifier("a"))),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let result = SubstitutionEngine::default()
        .once(&function, 2, &symbols)
        .expect("lexical substitution");
    let MathExpressionKind::FunctionDefinition(value) = &result.expression().kind else {
        panic!("function definition")
    };
    assert_eq!(value.name.as_ref(), &identifier("f"));
    assert_eq!(value.parameters, vec![identifier("x")]);
    assert_eq!(value.body.as_ref(), &add(identifier("x"), real("7")));

    let integral = MathExpression {
        kind: MathExpressionKind::Integral(Integral {
            bound_variable: Box::new(identifier("x")),
            integrand: Box::new(add(identifier("x"), identifier("a"))),
            bounds: Some(Bounds {
                lower: Box::new(identifier("a")),
                upper: Box::new(real("10")),
            }),
            algorithm: None,
        }),
        origin: ExpressionOrigin::Derived,
    };
    let result = SubstitutionEngine::default()
        .once(&integral, 2, &symbols)
        .expect("integral substitution");
    let MathExpressionKind::Integral(value) = &result.expression().kind else {
        panic!("integral")
    };
    assert_eq!(value.bound_variable.as_ref(), &identifier("x"));
    assert_eq!(value.integrand.as_ref(), &add(identifier("x"), real("7")));
    assert_eq!(value.bounds.as_ref().unwrap().lower.as_ref(), &real("7"));
}

#[test]
fn input_output_expansion_and_trace_budgets_fail_closed() {
    let symbols = table(&[
        (1, definition("b", real("2"))),
        (2, definition("a", identifier("b"))),
        (3, definition("wide", add(real("1"), real("2")))),
    ]);

    let mut limits = math_engine::SubstitutionLimits::new(10);
    limits.max_input_text_bytes = 1;
    assert_eq!(
        SubstitutionEngine::new(limits).once(&identifier("wide"), 4, &symbols),
        Err(SubstitutionError::InputTextLimitExceeded { limit: 1 })
    );

    let mut limits = math_engine::SubstitutionLimits::new(10);
    limits.max_output_nodes = 2;
    assert_eq!(
        SubstitutionEngine::new(limits).once(&identifier("wide"), 4, &symbols),
        Err(SubstitutionError::OutputNodeLimitExceeded { limit: 2 })
    );

    let mut limits = math_engine::SubstitutionLimits::new(10);
    limits.max_trace_steps = 2;
    assert_eq!(
        SubstitutionEngine::new(limits).once(&identifier("a"), 4, &symbols),
        Err(SubstitutionError::TraceLimitExceeded { limit: 2 })
    );

    let mut limits = math_engine::SubstitutionLimits::new(10);
    limits.max_substitution_depth = 1;
    assert_eq!(
        SubstitutionEngine::new(limits).recursive(&identifier("a"), 4, &symbols),
        Err(SubstitutionError::SubstitutionDepthLimitExceeded { limit: 1 })
    );

    let mut limits = math_engine::SubstitutionLimits::new(10);
    limits.max_expansion_steps = 1;
    assert_eq!(
        SubstitutionEngine::new(limits).recursive(&identifier("a"), 4, &symbols),
        Err(SubstitutionError::ExpansionStepLimitExceeded { limit: 1 })
    );
}

#[test]
fn recursive_trace_is_ordered_bounded_and_redacted() {
    let symbols = table(&[
        (1, definition("secret_b", real("2"))),
        (2, definition("secret_a", identifier("secret_b"))),
    ]);
    let result = SubstitutionEngine::default()
        .recursive(&identifier("secret_a"), 3, &symbols)
        .expect("recursive substitution");
    let kinds: Vec<_> = result
        .trace()
        .steps()
        .iter()
        .map(|step| step.kind())
        .collect();
    assert_eq!(
        kinds,
        vec![
            math_engine::EvaluationTraceKind::ReferenceObserved,
            math_engine::EvaluationTraceKind::BindingSelected,
            math_engine::EvaluationTraceKind::SubstitutionApplied,
            math_engine::EvaluationTraceKind::ReferenceObserved,
            math_engine::EvaluationTraceKind::BindingSelected,
            math_engine::EvaluationTraceKind::SubstitutionApplied,
            math_engine::EvaluationTraceKind::Completed,
        ]
    );
    assert!(!format!("{:?}", result.trace()).contains("secret"));
    assert_eq!(result.trace().steps().last().unwrap().count(), 2);
}

#[test]
fn traced_failures_emit_a_redacted_failed_status() {
    let failure = SubstitutionEngine::default()
        .once_with_failure_trace(&identifier("secret_missing"), 7, &table(&[]))
        .expect_err("unknown variable");
    assert_eq!(
        failure.error(),
        SubstitutionError::UnknownVariable { source_ordinal: 7 }
    );
    assert_eq!(failure.trace().steps().len(), 1);
    assert_eq!(
        failure.trace().steps()[0].kind(),
        math_engine::EvaluationTraceKind::Failed
    );
    assert!(!format!("{failure:?}").contains("secret_missing"));
}

#[test]
fn unit_collections_are_charged_to_the_cumulative_output_budget() {
    let units = UnitMonomial {
        system: None,
        factors: (0..8)
            .map(|_| UnitReference {
                unit: "m".into(),
                power_numerator: 1,
                power_denominator: NonZeroI64::new(1).unwrap(),
            })
            .collect(),
    };
    let united = MathExpression {
        kind: MathExpressionKind::UnitedValue(UnitedValue {
            value: Box::new(real("1")),
            units,
        }),
        origin: ExpressionOrigin::Derived,
    };
    let symbols = table(&[(1, definition("wide", united))]);
    let mut limits = math_engine::SubstitutionLimits::new(10);
    limits.max_output_nodes = 9;
    assert_eq!(
        SubstitutionEngine::new(limits).recursive(&identifier("wide"), 2, &symbols),
        Err(SubstitutionError::OutputNodeLimitExceeded { limit: 9 })
    );
}

#[test]
fn deeply_nested_large_binders_use_a_single_bounded_scope_stack() {
    let mut expression = real("1");
    for index in 0..64 {
        expression = MathExpression {
            kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
                style: DefinitionStyle::Equal,
                name: Box::new(identifier(&format!("f{index}"))),
                parameters: vec![identifier(&format!("{}_{}", "x".repeat(1024), index))],
                body: Box::new(expression),
            }),
            origin: ExpressionOrigin::Derived,
        };
    }
    let original = expression.clone();
    let result = SubstitutionEngine::default()
        .once(&expression, 1, &table(&[]))
        .expect("bounded binder traversal");
    assert_eq!(result.expression(), &original);
}

#[test]
fn every_structural_ast_branch_substitutes_free_references() {
    let symbols = table(&[(1, definition("a", real("7")))]);
    let derived = |kind| MathExpression {
        kind,
        origin: ExpressionOrigin::Derived,
    };
    let cases = vec![
        (
            derived(MathExpressionKind::Evaluation(Evaluation {
                expression: Box::new(identifier("a")),
                unit_override: Some(Box::new(identifier("a"))),
                saved_result: Some(Box::new(identifier("a"))),
            })),
            3,
        ),
        (
            derived(MathExpressionKind::ArrayIndex(ArrayIndex {
                target: Box::new(identifier("a")),
                indices: vec![identifier("a")],
            })),
            2,
        ),
        (
            derived(MathExpressionKind::Matrix(Matrix {
                rows: 1,
                columns: 1,
                elements: vec![identifier("a")],
            })),
            1,
        ),
        (
            derived(MathExpressionKind::Vector(Vector {
                orientation: VectorOrientation::Column,
                elements: vec![identifier("a")],
            })),
            1,
        ),
        (
            derived(MathExpressionKind::Range(RangeExpression {
                start: Box::new(identifier("a")),
                next: Some(Box::new(identifier("a"))),
                end: Box::new(identifier("a")),
            })),
            3,
        ),
        (
            derived(MathExpressionKind::Derivative(Derivative {
                bound_variable: Box::new(identifier("x")),
                expression: Box::new(add(identifier("x"), identifier("a"))),
                degree: Some(Box::new(identifier("a"))),
                style: DerivativeStyle::Derivative,
            })),
            2,
        ),
        (
            derived(MathExpressionKind::Aggregate(AggregateExpression {
                operator: AggregateOperator::Summation,
                bound_variable: Box::new(identifier("x")),
                body: Box::new(add(identifier("x"), identifier("a"))),
                bounds: Some(Bounds {
                    lower: Box::new(identifier("a")),
                    upper: Box::new(real("9")),
                }),
            })),
            2,
        ),
        (
            derived(MathExpressionKind::Comparison(ComparisonExpression {
                operator: ComparisonOperator::Equal,
                left: Box::new(identifier("a")),
                right: Box::new(identifier("a")),
            })),
            2,
        ),
        (
            derived(MathExpressionKind::Boolean(BooleanExpression {
                operator: BooleanOperator::And,
                left: Box::new(identifier("a")),
                right: Box::new(identifier("a")),
            })),
            2,
        ),
        (
            derived(MathExpressionKind::LogicalNot(LogicalNot {
                operand: Box::new(identifier("a")),
            })),
            1,
        ),
        (
            derived(MathExpressionKind::UnitedValue(UnitedValue {
                value: Box::new(identifier("a")),
                units: UnitMonomial {
                    system: None,
                    factors: Vec::new(),
                },
            })),
            1,
        ),
    ];
    for (expression, expected_count) in cases {
        let result = SubstitutionEngine::default()
            .once(&expression, 2, &symbols)
            .unwrap();
        assert_eq!(result.substitution_count(), expected_count);
    }
}
