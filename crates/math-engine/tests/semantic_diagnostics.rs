use math_engine::{
    DependencyGraph, DependencyGraphLimits, ReferenceAnalyzer, ReferenceLimits, SemanticDiagnostic,
    SemanticDiagnostics, SemanticDiagnosticsError, SemanticDiagnosticsLimits, SymbolTable,
    SymbolTableLimits, UndefinedReferenceCategory,
};
use math_model::{
    Definition, DefinitionKind, DefinitionStyle, ExpressionOrigin, FunctionCall,
    FunctionDefinition, Identifier, MathExpression, MathExpressionKind, NumericBase, RealLiteral,
    SourceSpan,
};

fn id(name: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Identifier(Identifier {
            name: name.into(),
            subscript: None,
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn source_id(name: &str, subscript: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Identifier(Identifier {
            name: name.into(),
            subscript: Some(subscript.into()),
        }),
        origin: ExpressionOrigin::Source(SourceSpan {
            start: 991_337,
            end: 991_338,
        }),
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
            target: Box::new(id(name)),
            value: Box::new(value),
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn call(name: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::FunctionCall(FunctionCall {
            callee: Box::new(id(name)),
            arguments: vec![],
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn function(name: &str, body: MathExpression) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
            style: DefinitionStyle::Equal,
            name: Box::new(id(name)),
            parameters: vec![],
            body: Box::new(body),
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn graph(inputs: &[(usize, MathExpression)]) -> DependencyGraph {
    let table = SymbolTable::build(
        inputs
            .iter()
            .map(|(ordinal, expression)| (*ordinal, expression)),
        SymbolTableLimits::default(),
    )
    .expect("symbol table");
    DependencyGraph::build(
        &table,
        &ReferenceAnalyzer::new(ReferenceLimits::default()),
        DependencyGraphLimits::default(),
    )
    .expect("dependency graph")
}

#[test]
fn undefined_references_are_typed_deterministic_and_redacted() {
    let graph = graph(&[
        (
            7,
            definition(
                "result",
                MathExpression {
                    kind: MathExpressionKind::Binary(math_model::BinaryExpression {
                        operator: math_model::BinaryOperator::Add,
                        multiplication_style: None,
                        left: Box::new(source_id("secret_variable", "secret_subscript")),
                        right: Box::new(real("771771.771771")),
                    }),
                    origin: ExpressionOrigin::Derived,
                },
            ),
        ),
        (9, definition("other", call("secret_function"))),
    ]);
    let before = graph.clone();

    let diagnostics = SemanticDiagnostics::from_graph(&graph, SemanticDiagnosticsLimits::default())
        .expect("diagnostics");
    assert_eq!(graph, before);
    assert_eq!(diagnostics.len(), 2);
    assert!(matches!(
        diagnostics.diagnostics()[0],
        SemanticDiagnostic::UndefinedReference(_)
    ));
    let collected: Vec<_> = diagnostics.undefined_references().collect();
    assert_eq!(
        collected[0].category(),
        UndefinedReferenceCategory::Variable
    );
    assert_eq!(collected[0].source_ordinal(), 7);
    assert_eq!(collected[0].occurrence_index(), 0);
    assert!(collected[0].has_source_provenance());
    assert_eq!(
        collected[1].category(),
        UndefinedReferenceCategory::Function { arity: 0 }
    );
    assert_eq!(collected[1].source_ordinal(), 9);
    let hidden = [
        "secret_variable",
        "secret_function",
        "secret_subscript",
        "771771.771771",
        "991337",
        "991338",
    ];
    let outputs = [
        format!("{diagnostics:?}"),
        format!("{:?}", diagnostics.diagnostics()[0]),
        format!(
            "{:?}",
            SemanticDiagnosticsError::DiagnosticLimitExceeded { limit: 1 }
        ),
        format!(
            "{}",
            SemanticDiagnosticsError::DiagnosticLimitExceeded { limit: 1 }
        ),
    ];
    for output in outputs {
        for sentinel in hidden {
            assert!(!output.contains(sentinel), "redaction leaked {sentinel}");
        }
    }
}

#[test]
fn forward_reference_becomes_an_undefined_diagnostic_at_use_ordinal() {
    let graph = graph(&[
        (3, definition("result", id("later"))),
        (8, definition("later", real("1"))),
    ]);
    let diagnostics = SemanticDiagnostics::from_graph(&graph, SemanticDiagnosticsLimits::default())
        .expect("diagnostics");

    let collected: Vec<_> = diagnostics.undefined_references().collect();
    assert_eq!(collected.len(), 1);
    assert_eq!(
        collected[0].category(),
        UndefinedReferenceCategory::Variable
    );
    assert_eq!(collected[0].source_ordinal(), 3);
    assert_eq!(collected[0].occurrence_index(), 0);
}

#[test]
fn diagnostic_limits_are_typed_and_fail_closed() {
    let graph = graph(&[
        (0, definition("first", id("missing_first"))),
        (1, definition("second", id("missing_second"))),
    ]);

    assert_eq!(
        SemanticDiagnostics::from_graph(&graph, SemanticDiagnosticsLimits::new(0))
            .expect_err("zero limit"),
        SemanticDiagnosticsError::InvalidLimits
    );
    assert_eq!(
        SemanticDiagnostics::from_graph(
            &graph,
            SemanticDiagnosticsLimits::new(SemanticDiagnosticsLimits::HARD_MAX_DIAGNOSTICS + 1,),
        )
        .expect_err("hard cap"),
        SemanticDiagnosticsError::InvalidLimits
    );
    assert_eq!(
        SemanticDiagnostics::from_graph(&graph, SemanticDiagnosticsLimits::new(1))
            .expect_err("bounded output"),
        SemanticDiagnosticsError::DiagnosticLimitExceeded { limit: 1 }
    );
}

#[test]
fn graph_work_limits_fail_before_scc_allocation() {
    let node_limited_graph = graph(&[
        (0, definition("first", real("1"))),
        (1, definition("second", real("2"))),
    ]);

    assert_eq!(
        SemanticDiagnostics::from_graph(
            &node_limited_graph,
            SemanticDiagnosticsLimits::with_graph_limits(10, 1, 10),
        )
        .expect_err("node work limit"),
        SemanticDiagnosticsError::NodeLimitExceeded { limit: 1 }
    );
    assert_eq!(
        SemanticDiagnostics::from_graph(
            &node_limited_graph,
            SemanticDiagnosticsLimits::with_graph_limits(10, 10, 0),
        )
        .expect_err("invalid edge work limit"),
        SemanticDiagnosticsError::InvalidLimits
    );

    let edge_limited_graph = graph(&[
        (0, function("first", call("first"))),
        (1, function("second", call("second"))),
    ]);
    assert_eq!(
        SemanticDiagnostics::from_graph(
            &edge_limited_graph,
            SemanticDiagnosticsLimits::with_graph_limits(10, 10, 1),
        )
        .expect_err("edge work limit"),
        SemanticDiagnosticsError::EdgeLimitExceeded { limit: 1 }
    );
}

#[test]
fn callable_self_cycle_is_circular_but_scalar_self_reference_stays_undefined() {
    let graph = graph(&[
        (2, definition("scalar", id("scalar"))),
        (
            7,
            function("secret_cycle_function", call("secret_cycle_function")),
        ),
    ]);
    let diagnostics = SemanticDiagnostics::from_graph(&graph, SemanticDiagnosticsLimits::default())
        .expect("diagnostics");

    let undefined: Vec<_> = diagnostics.undefined_references().collect();
    assert_eq!(undefined.len(), 1);
    assert_eq!(undefined[0].source_ordinal(), 2);
    let cycles: Vec<_> = diagnostics.circular_dependencies().collect();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].definition_id().source_ordinal(), 7);
    assert_eq!(cycles[0].cycle_leader(), cycles[0].definition_id());
    assert_eq!(cycles[0].cycle_size(), 1);
    assert!(!format!("{diagnostics:?}").contains("secret_cycle_function"));
}

#[test]
fn diagnostic_limit_covers_undefined_references_and_cycles_together() {
    let graph = graph(&[
        (2, definition("scalar", id("scalar"))),
        (7, function("loop", call("loop"))),
    ]);

    assert_eq!(
        SemanticDiagnostics::from_graph(&graph, SemanticDiagnosticsLimits::new(1))
            .expect_err("the shared diagnostic budget must fail closed"),
        SemanticDiagnosticsError::DiagnosticLimitExceeded { limit: 1 }
    );
}
