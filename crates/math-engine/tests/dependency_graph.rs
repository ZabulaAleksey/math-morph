use math_engine::{
    DefinitionNamespace, DependencyGraph, DependencyGraphError, DependencyGraphLimits, FunctionKey,
    ReferenceAnalyzer, ReferenceError, ReferenceIdentity, ReferenceLimits, SymbolKey, SymbolTable,
    SymbolTableLimits,
};
use math_model::{
    Definition, DefinitionKind, DefinitionStyle, ExpressionOrigin, FunctionCall,
    FunctionDefinition, Identifier, MathExpression, MathExpressionKind, Matrix, NumericBase,
    RealLiteral,
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

fn real(lexeme: &str) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::Real(RealLiteral {
            lexeme: lexeme.into(),
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

fn call(name: &str, arguments: Vec<MathExpression>) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::FunctionCall(FunctionCall {
            callee: Box::new(id(name)),
            arguments,
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn matrix(elements: Vec<MathExpression>) -> MathExpression {
    let columns = elements.len();
    MathExpression {
        kind: MathExpressionKind::Matrix(Matrix {
            rows: 1,
            columns,
            elements,
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn function(name: &str, parameters: Vec<&str>, body: MathExpression) -> MathExpression {
    MathExpression {
        kind: MathExpressionKind::FunctionDefinition(FunctionDefinition {
            style: DefinitionStyle::Equal,
            name: Box::new(id(name)),
            parameters: parameters.into_iter().map(id).collect(),
            body: Box::new(body),
        }),
        origin: ExpressionOrigin::Derived,
    }
}

fn table(inputs: &[(usize, MathExpression)]) -> SymbolTable {
    SymbolTable::build(
        inputs
            .iter()
            .map(|(ordinal, expression)| (*ordinal, expression)),
        SymbolTableLimits::default(),
    )
    .expect("symbol table")
}

fn graph(inputs: &[(usize, MathExpression)], analyzer: &ReferenceAnalyzer) -> DependencyGraph {
    let table = table(inputs);
    DependencyGraph::build(&table, analyzer, DependencyGraphLimits::default()).expect("graph")
}

#[test]
fn prior_scalar_and_function_references_resolve_to_visible_nodes() {
    let inputs = vec![
        (0, definition("x", real("1"))),
        (1, definition("y", id("x"))),
        (2, function("f", vec![], real("1"))),
        (3, function("g", vec![], call("f", vec![]))),
    ];
    let graph = graph(&inputs, &ReferenceAnalyzer::new(ReferenceLimits::default()));

    assert_eq!(
        graph
            .nodes()
            .iter()
            .map(|node| node.source_ordinal())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert_eq!(graph.edges().len(), 2);
    assert_eq!(graph.edges()[0].from().source_ordinal(), 1);
    assert_eq!(graph.edges()[0].to().source_ordinal(), 0);
    assert_eq!(graph.edges()[1].from().source_ordinal(), 3);
    assert_eq!(graph.edges()[1].to().source_ordinal(), 2);
    assert!(graph.unresolved().is_empty());
}

#[test]
fn redefinition_resolves_to_latest_prior_revision() {
    let inputs = vec![
        (0, definition("x", real("1"))),
        (1, definition("x", real("2"))),
        (2, definition("y", id("x"))),
    ];
    let graph = graph(&inputs, &ReferenceAnalyzer::new(ReferenceLimits::default()));

    assert_eq!(graph.edges().len(), 1);
    assert_eq!(graph.edges()[0].from().source_ordinal(), 2);
    assert_eq!(graph.edges()[0].to().source_ordinal(), 1);
}

#[test]
fn forward_reference_is_unresolved_without_a_guessed_edge() {
    let inputs = vec![
        (0, definition("y", id("x"))),
        (1, definition("x", real("1"))),
    ];
    let graph = graph(&inputs, &ReferenceAnalyzer::new(ReferenceLimits::default()));

    assert!(graph.edges().is_empty());
    assert_eq!(graph.unresolved().len(), 1);
    assert_eq!(graph.unresolved()[0].from().source_ordinal(), 0);
    assert!(matches!(
        graph.unresolved()[0].identity(),
        ReferenceIdentity::Variable(key) if key == &SymbolKey::new("x", None)
    ));
}

#[test]
fn unresolved_identity_is_retained_but_debug_is_redacted() {
    let inputs = vec![(7, definition("y", call("secret_function", vec![])))];
    let graph = graph(&inputs, &ReferenceAnalyzer::new(ReferenceLimits::default()));
    let debug = format!("{:?}", graph.unresolved()[0]);

    assert!(!debug.contains("secret_function"));
    assert!(matches!(
        graph.unresolved()[0].identity(),
        ReferenceIdentity::Function(key) if key == &FunctionKey::new("secret_function", None, 0)
    ));
}

#[test]
fn scalar_function_namespaces_and_function_arity_are_separate() {
    let inputs = vec![
        (0, definition("f", real("1"))),
        (1, function("f", vec!["x"], real("1"))),
        (2, function("f", vec!["x", "y"], real("1"))),
        (3, definition("scalar_user", id("f"))),
        (4, function("one_user", vec![], call("f", vec![real("1")]))),
        (
            5,
            function("two_user", vec![], call("f", vec![real("1"), real("2")])),
        ),
    ];
    let graph = graph(&inputs, &ReferenceAnalyzer::new(ReferenceLimits::default()));

    assert_eq!(graph.edges().len(), 3);
    assert_eq!(
        graph.edges()[0].to().namespace(),
        DefinitionNamespace::Variable
    );
    assert_eq!(
        graph.edges()[1].to().namespace(),
        DefinitionNamespace::Function { arity: 1 }
    );
    assert_eq!(
        graph.edges()[2].to().namespace(),
        DefinitionNamespace::Function { arity: 2 }
    );
}

#[test]
fn exact_function_self_reference_is_an_edge_but_scalar_self_reference_is_unresolved() {
    let inputs = vec![
        (0, definition("x", id("x"))),
        (1, function("f", vec![], call("f", vec![]))),
    ];
    let graph = graph(&inputs, &ReferenceAnalyzer::new(ReferenceLimits::default()));

    assert_eq!(graph.edges().len(), 1);
    assert_eq!(graph.edges()[0].from(), graph.edges()[0].to());
    assert_eq!(graph.edges()[0].from().source_ordinal(), 1);
    assert_eq!(graph.unresolved().len(), 1);
    assert_eq!(graph.unresolved()[0].from().source_ordinal(), 0);
}

#[test]
fn graph_order_and_equality_are_deterministic() {
    let inputs = vec![
        (0, definition("x", real("1"))),
        (1, definition("y", id("x"))),
        (2, definition("z", id("y"))),
    ];
    let analyzer = ReferenceAnalyzer::new(ReferenceLimits::default());
    let first = graph(&inputs, &analyzer);
    let second = graph(&inputs, &analyzer);

    assert_eq!(first, second);
    assert_eq!(
        first
            .edges()
            .iter()
            .map(|edge| (edge.from().source_ordinal(), edge.to().source_ordinal()))
            .collect::<Vec<_>>(),
        [(1, 0), (2, 1)]
    );
}

#[test]
fn graph_node_edge_and_unresolved_limits_are_typed_and_capped() {
    let inputs = vec![
        (0, definition("x", real("1"))),
        (1, definition("y", id("x"))),
        (2, definition("z", id("x"))),
    ];
    let symbol_table = table(&inputs);
    let analyzer = ReferenceAnalyzer::new(ReferenceLimits::default());

    assert_eq!(
        DependencyGraph::build(
            &symbol_table,
            &analyzer,
            DependencyGraphLimits::new(0, 10, 10)
        )
        .expect_err("zero node limit"),
        DependencyGraphError::InvalidLimits
    );
    assert_eq!(
        DependencyGraph::build(
            &symbol_table,
            &analyzer,
            DependencyGraphLimits::new(DependencyGraphLimits::HARD_MAX_NODES + 1, 10, 10),
        )
        .expect_err("over-hard node limit"),
        DependencyGraphError::InvalidLimits
    );
    assert_eq!(
        DependencyGraph::build(
            &symbol_table,
            &analyzer,
            DependencyGraphLimits::new(2, 10, 10)
        )
        .expect_err("node limit"),
        DependencyGraphError::NodeLimitExceeded { limit: 2 }
    );
    assert_eq!(
        DependencyGraph::build(
            &symbol_table,
            &analyzer,
            DependencyGraphLimits::new(10, 1, 10)
        )
        .expect_err("edge limit"),
        DependencyGraphError::EdgeLimitExceeded { limit: 1 }
    );

    let unresolved_table = table(&[
        (0, definition("a", id("missing_a"))),
        (1, definition("b", id("missing_b"))),
    ]);
    assert_eq!(
        DependencyGraph::build(
            &unresolved_table,
            &analyzer,
            DependencyGraphLimits::new(10, 10, 1),
        )
        .expect_err("unresolved limit"),
        DependencyGraphError::UnresolvedLimitExceeded { limit: 1 }
    );
    assert_eq!(
        DependencyGraph::build(
            &unresolved_table,
            &analyzer,
            DependencyGraphLimits::new(10, 10, DependencyGraphLimits::HARD_MAX_UNRESOLVED + 1),
        )
        .expect_err("over-hard unresolved limit"),
        DependencyGraphError::InvalidLimits
    );
}

#[test]
fn reference_budgets_are_cumulative_across_all_definitions() {
    let inputs = vec![
        (0, definition("x", real("1"))),
        (1, definition("y", real("2"))),
    ];
    let symbol_table = table(&inputs);
    let analyzer = ReferenceAnalyzer::new(ReferenceLimits::new(10, 256, 3, 100, 100, 100, 10));
    let error = DependencyGraph::build(&symbol_table, &analyzer, DependencyGraphLimits::default())
        .expect_err("reference node budget must be cumulative");

    assert_eq!(
        error,
        DependencyGraphError::ReferenceAnalysis(ReferenceError::NodeLimitExceeded {
            source_ordinal: 1,
            limit: 3,
        })
    );
}

#[test]
fn graph_output_cap_is_after_per_site_deduplication() {
    let duplicate = definition("y", matrix(vec![id("missing"), id("missing")]));
    let table = table(&[(0, duplicate)]);
    let analyzer = ReferenceAnalyzer::new(ReferenceLimits::default());
    let graph = DependencyGraph::build(&table, &analyzer, DependencyGraphLimits::new(10, 1, 1))
        .expect("duplicate references should deduplicate before output cap");
    assert_eq!(graph.unresolved_count(), 1);
}

#[test]
fn graph_output_cap_rejects_distinct_materialized_references_before_graph_build() {
    let distinct = definition(
        "y",
        matrix(vec![id("missing_a"), id("missing_b"), id("missing_c")]),
    );
    let table = table(&[(0, distinct)]);
    let analyzer = ReferenceAnalyzer::new(ReferenceLimits::default());
    let error = DependencyGraph::build(&table, &analyzer, DependencyGraphLimits::new(10, 1, 1))
        .expect_err("distinct references exceed graph materialized cap");
    assert_eq!(
        error,
        DependencyGraphError::ReferenceOutputLimitExceeded { limit: 2 }
    );
}

#[test]
fn graph_build_does_not_mutate_the_original_ast() {
    let inputs = vec![
        (0, definition("x", real("1"))),
        (1, definition("y", id("x"))),
    ];
    let before = inputs.clone();
    let table = table(&inputs);
    let _graph = DependencyGraph::build(
        &table,
        &ReferenceAnalyzer::new(ReferenceLimits::default()),
        DependencyGraphLimits::default(),
    )
    .expect("graph");

    assert_eq!(inputs, before);
}
