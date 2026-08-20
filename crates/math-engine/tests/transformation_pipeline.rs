use math_engine::{
    AppliedTransformation, NotationProfile, TransformError, TransformationLimits,
    TransformationPipeline,
};
use math_model::{Definition, DefinitionKind, DefinitionStyle, ExpressionOrigin, Identifier};
use math_model::{MathExpression, MathExpressionKind};

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
fn public_pipeline_preserves_original_and_records_explicit_style() {
    let original = MathExpression {
        kind: MathExpressionKind::Definition(Definition {
            kind: DefinitionKind::Define,
            style: DefinitionStyle::ColonEqual,
            target: Box::new(identifier("x")),
            value: Box::new(identifier("y")),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let before = original.clone();

    let result = TransformationPipeline::new(NotationProfile::with_definition_style(
        DefinitionStyle::Equal,
    ))
    .transform(&original)
    .expect("presentation transform");

    assert_eq!(original, before);
    assert!(matches!(
        result.applied_transformations.as_slice(),
        [AppliedTransformation::DefinitionStyle {
            from: DefinitionStyle::ColonEqual,
            to: DefinitionStyle::Equal,
        }]
    ));
    let MathExpressionKind::Definition(display) = result.display.kind else {
        panic!("expected typed definition display AST");
    };
    assert_eq!(display.style, DefinitionStyle::Equal);
    assert_eq!(display.kind, DefinitionKind::Define);
}

#[test]
fn public_pipeline_reports_bounded_depth_without_source_payload() {
    let nested = MathExpression {
        kind: MathExpressionKind::Unary(math_model::UnaryExpression {
            operator: math_model::UnaryOperator::Negate,
            operand: Box::new(identifier("private-symbol")),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = TransformationPipeline::with_limits(
        NotationProfile::faithful(),
        TransformationLimits::new(0, 100),
    )
    .transform(&nested)
    .expect_err("depth bound");

    assert_eq!(error, TransformError::DepthLimitExceeded { limit: 0 });
    assert!(!error.to_string().contains("private-symbol"));
    assert!(!format!("{error:?}").contains("private-symbol"));
}

#[test]
fn identical_function_definition_transforms_are_deterministic() {
    let original = MathExpression {
        kind: MathExpressionKind::FunctionDefinition(math_model::FunctionDefinition {
            style: DefinitionStyle::ColonEqual,
            name: Box::new(identifier("f")),
            parameters: vec![identifier("x")],
            body: Box::new(identifier("x")),
        }),
        origin: ExpressionOrigin::Derived,
    };
    let pipeline = TransformationPipeline::new(NotationProfile::with_definition_style(
        DefinitionStyle::Equal,
    ));

    let first = pipeline.transform(&original).expect("first transform");
    let second = pipeline.transform(&original).expect("second transform");

    assert_eq!(first, second);
    assert!(matches!(
        first.display.kind,
        MathExpressionKind::FunctionDefinition(ref value)
            if value.style == DefinitionStyle::Equal
    ));
}

#[test]
fn public_pipeline_enforces_node_limit_at_the_root() {
    let error = TransformationPipeline::with_limits(
        NotationProfile::faithful(),
        TransformationLimits::new(256, 0),
    )
    .transform(&identifier("private-symbol"))
    .expect_err("node bound");

    assert_eq!(error, TransformError::NodeLimitExceeded { limit: 0 });
    assert!(!error.to_string().contains("private-symbol"));
}

#[test]
fn public_pipeline_preflights_large_argument_lists_before_child_allocation() {
    let expression = MathExpression {
        kind: MathExpressionKind::FunctionCall(math_model::FunctionCall {
            callee: Box::new(identifier("private-callee")),
            arguments: vec![identifier("private-a"), identifier("private-b")],
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = TransformationPipeline::with_limits(
        NotationProfile::faithful(),
        TransformationLimits::new(256, 2),
    )
    .transform(&expression)
    .expect_err("argument list must be rejected by remaining-node preflight");

    assert_eq!(error, TransformError::NodeLimitExceeded { limit: 2 });
    assert!(!format!("{error:?}").contains("private-"));
}

#[test]
fn public_pipeline_rejects_limits_above_hard_ceiling() {
    let error = TransformationPipeline::with_limits(
        NotationProfile::faithful(),
        TransformationLimits::new(257, 100_000),
    )
    .transform(&identifier("private-symbol"))
    .expect_err("caller limits above the hard ceiling");

    assert_eq!(error, TransformError::InvalidLimits);
    assert!(!error.to_string().contains("private-symbol"));
    assert!(!format!("{error:?}").contains("private-symbol"));
}

#[test]
fn public_pipeline_preflights_vector_child_depth() {
    let expression = MathExpression {
        kind: MathExpressionKind::Vector(math_model::Vector {
            orientation: math_model::VectorOrientation::Row,
            elements: vec![identifier("private-a"), identifier("private-b")],
        }),
        origin: ExpressionOrigin::Derived,
    };
    let error = TransformationPipeline::with_limits(
        NotationProfile::faithful(),
        TransformationLimits::new(0, 100_000),
    )
    .transform(&expression)
    .expect_err("child depth must be rejected before output allocation");

    assert_eq!(error, TransformError::DepthLimitExceeded { limit: 0 });
    assert!(!format!("{error:?}").contains("private-"));
}
