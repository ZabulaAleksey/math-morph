use math_engine::{
    DependencyGraph, DependencyGraphLimits, EvaluationPlan, EvaluationPlanError,
    EvaluationPlanLimits, ReferenceAnalyzer, ReferenceLimits, SymbolTable, SymbolTableLimits,
};
use math_model::{
    Definition, DefinitionKind, DefinitionStyle, ExpressionOrigin, FunctionCall,
    FunctionDefinition, Identifier, MathExpression, MathExpressionKind, NumericBase, RealLiteral,
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

fn plan(graph: &DependencyGraph) -> EvaluationPlan {
    EvaluationPlan::build(graph, EvaluationPlanLimits::default()).expect("evaluation plan")
}

fn ordinals(plan: &EvaluationPlan) -> Vec<usize> {
    plan.order().iter().map(|id| id.source_ordinal()).collect()
}

#[test]
fn dependency_chain_places_dependencies_before_dependents() {
    let graph = graph(&[
        (0, definition("a", real("1"))),
        (1, definition("b", id("a"))),
        (2, definition("c", id("b"))),
    ]);
    assert_eq!(ordinals(&plan(&graph)), [0, 1, 2]);
}

#[test]
fn independent_ready_nodes_use_source_ordinal_tie_breaking() {
    let graph = graph(&[
        (7, definition("a", real("1"))),
        (8, definition("b", real("2"))),
        (12, definition("c", real("3"))),
    ]);
    assert_eq!(ordinals(&plan(&graph)), [7, 8, 12]);
}

#[test]
fn diamond_graph_is_stable_and_dependency_safe() {
    let graph = graph(&[
        (0, definition("a", real("1"))),
        (1, definition("b", id("a"))),
        (2, definition("c", id("a"))),
        (
            3,
            definition(
                "d",
                MathExpression {
                    kind: MathExpressionKind::Binary(math_model::BinaryExpression {
                        operator: math_model::BinaryOperator::Add,
                        multiplication_style: None,
                        left: Box::new(id("b")),
                        right: Box::new(id("c")),
                    }),
                    origin: ExpressionOrigin::Derived,
                },
            ),
        ),
    ]);
    assert_eq!(ordinals(&plan(&graph)), [0, 1, 2, 3]);
}

#[test]
fn repeated_definitions_preserve_source_compatible_order() {
    let graph = graph(&[
        (0, definition("x", real("1"))),
        (1, definition("x", real("2"))),
        (2, definition("y", id("x"))),
    ]);
    let plan = plan(&graph);
    assert_eq!(ordinals(&plan), [0, 1, 2]);
    assert_eq!(plan.len(), graph.node_count());
}

#[test]
fn unresolved_dependencies_return_no_partial_plan() {
    let graph = graph(&[(0, definition("x", id("missing")))]);
    assert_eq!(
        EvaluationPlan::build(&graph, EvaluationPlanLimits::default())
            .expect_err("unresolved dependency"),
        EvaluationPlanError::UnresolvedDependencies { count: 1 }
    );
}

#[test]
fn self_and_multiple_self_cycles_fail_closed() {
    let graph = graph(&[(0, function("f", call("f"))), (1, function("g", call("g")))]);
    assert_eq!(
        EvaluationPlan::build(&graph, EvaluationPlanLimits::default()).expect_err("cycles"),
        EvaluationPlanError::CyclePresent { remaining: 2 }
    );
}

#[test]
fn node_edge_ready_output_limits_and_invalid_caps_are_typed() {
    let dependency_graph = graph(&[
        (0, definition("a", real("1"))),
        (1, definition("b", id("a"))),
        (2, definition("c", id("b"))),
    ]);

    assert_eq!(
        EvaluationPlan::build(&dependency_graph, EvaluationPlanLimits::new(0, 10, 10, 10))
            .expect_err("zero node limit"),
        EvaluationPlanError::InvalidLimits
    );
    assert_eq!(
        EvaluationPlan::build(
            &dependency_graph,
            EvaluationPlanLimits::new(EvaluationPlanLimits::HARD_MAX_NODES + 1, 10, 10, 10,),
        )
        .expect_err("over-hard node limit"),
        EvaluationPlanError::InvalidLimits
    );
    assert_eq!(
        EvaluationPlan::build(&dependency_graph, EvaluationPlanLimits::new(2, 10, 10, 10))
            .expect_err("node limit"),
        EvaluationPlanError::NodeLimitExceeded { limit: 2 }
    );
    assert_eq!(
        EvaluationPlan::build(&dependency_graph, EvaluationPlanLimits::new(10, 1, 10, 10))
            .expect_err("edge limit"),
        EvaluationPlanError::EdgeLimitExceeded { limit: 1 }
    );
    assert_eq!(
        EvaluationPlan::build(&dependency_graph, EvaluationPlanLimits::new(10, 10, 10, 2))
            .expect_err("output limit"),
        EvaluationPlanError::OutputLimitExceeded { limit: 2 }
    );

    let independent = graph(&[
        (0, definition("a", real("1"))),
        (1, definition("b", real("2"))),
        (2, definition("c", real("3"))),
    ]);
    assert_eq!(
        EvaluationPlan::build(&independent, EvaluationPlanLimits::new(10, 10, 1, 10))
            .expect_err("ready limit"),
        EvaluationPlanError::ReadyLimitExceeded { limit: 1 }
    );
}

#[test]
fn plans_are_deterministic_and_do_not_mutate_the_graph() {
    let graph = graph(&[
        (0, definition("a", real("1"))),
        (1, definition("b", id("a"))),
        (2, definition("c", id("a"))),
    ]);
    let before = graph.clone();
    let first = plan(&graph);
    let second = plan(&graph);
    assert_eq!(first, second);
    assert_eq!(graph, before);
}

#[test]
fn large_dag_uses_iterative_bounded_traversal() {
    let mut inputs = Vec::with_capacity(1000);
    for index in 0..1000 {
        let value = if index == 0 {
            real("1")
        } else {
            id(&format!("x{}", index - 1))
        };
        inputs.push((index, definition(&format!("x{index}"), value)));
    }
    let graph = graph(&inputs);
    let evaluation_plan = plan(&graph);
    assert_eq!(evaluation_plan.len(), 1000);
    assert_eq!(evaluation_plan.order()[0].source_ordinal(), 0);
    assert_eq!(evaluation_plan.order()[999].source_ordinal(), 999);
}
