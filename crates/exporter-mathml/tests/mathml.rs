use document_ir::ports::EquationExporter;
use exporter_mathml::{MathMlError, MathMlFragment, MathMlLimit, MathMlLimits, MathMlRenderer};
use math_model::{
    BinaryExpression, BinaryOperator, ExpressionOrigin, FunctionCall, Grouping, Identifier,
    MathExpression, MathExpressionKind, MultiplicationStyle, NumericBase, RealLiteral,
    UnaryExpression, UnaryOperator,
};

const ROOT: &str = "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">";

fn expression(kind: MathExpressionKind) -> MathExpression {
    MathExpression {
        kind,
        origin: ExpressionOrigin::Derived,
    }
}

fn real(lexeme: &str) -> MathExpression {
    expression(MathExpressionKind::Real(RealLiteral {
        lexeme: lexeme.into(),
        base: NumericBase::Decimal,
    }))
}

fn identifier(name: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: name.into(),
        subscript: None,
    }))
}

fn binary(
    operator: BinaryOperator,
    multiplication_style: Option<MultiplicationStyle>,
    left: MathExpression,
    right: MathExpression,
) -> MathExpression {
    expression(MathExpressionKind::Binary(BinaryExpression {
        operator,
        multiplication_style,
        left: Box::new(left),
        right: Box::new(right),
    }))
}

fn drop_left_linear_tree(expression: MathExpression) {
    let mut current = Some(expression);
    while let Some(MathExpression { kind, origin: _ }) = current.take() {
        match kind {
            MathExpressionKind::Binary(BinaryExpression { left, right, .. }) => {
                drop(right);
                current = Some(*left);
            }
            kind => drop(kind),
        }
    }
}

#[test]
fn scalar_roots_and_structural_shapes_are_exact_and_deterministic() {
    let renderer = MathMlRenderer::default();
    let cases = [
        (real("-1.25e+2"), "<mn>-1.25e+2</mn>"),
        (identifier("x"), "<mi>x</mi>"),
        (
            expression(MathExpressionKind::Identifier(Identifier {
                name: "x".into(),
                subscript: Some("i".into()),
            })),
            "<msub><mi>x</mi><mi>i</mi></msub>",
        ),
        (
            binary(BinaryOperator::Add, None, identifier("x"), real("1")),
            "<mrow><mi>x</mi><mo>+</mo><mn>1</mn></mrow>",
        ),
        (
            binary(BinaryOperator::Subtract, None, identifier("x"), real("1")),
            "<mrow><mi>x</mi><mo>&#x2212;</mo><mn>1</mn></mrow>",
        ),
        (
            binary(BinaryOperator::Divide, None, identifier("x"), real("2")),
            "<mfrac><mi>x</mi><mn>2</mn></mfrac>",
        ),
        (
            binary(BinaryOperator::Power, None, identifier("x"), real("2")),
            "<msup><mi>x</mi><mn>2</mn></msup>",
        ),
        (
            expression(MathExpressionKind::Unary(UnaryExpression {
                operator: UnaryOperator::SquareRoot,
                operand: Box::new(identifier("x")),
            })),
            "<msqrt><mi>x</mi></msqrt>",
        ),
        (
            expression(MathExpressionKind::Grouping(Grouping {
                expression: Box::new(identifier("x")),
                unpaired: false,
            })),
            "<mrow><mo fence=\"true\">(</mo><mi>x</mi><mo fence=\"true\">)</mo></mrow>",
        ),
    ];

    for (input, body) in cases {
        let output = renderer
            .export_expression(&input)
            .expect("supported expression");
        assert_eq!(output.as_str(), format!("{ROOT}{body}</math>"));
        assert_eq!(output.byte_len(), output.as_str().len());
        assert_eq!(renderer.export_expression(&input).unwrap(), output);
    }
}

#[test]
fn multiplication_policy_is_exact() {
    let renderer = MathMlRenderer::default();
    let cases = [
        (MultiplicationStyle::Default, "<mo>&#x00B7;</mo>"),
        (MultiplicationStyle::AutoSelect, "<mo>&#x00B7;</mo>"),
        (MultiplicationStyle::Dot, "<mo>&#x00B7;</mo>"),
        (MultiplicationStyle::NarrowDot, "<mo>&#x00B7;</mo>"),
        (MultiplicationStyle::LargeDot, "<mo>&#x00B7;</mo>"),
        (MultiplicationStyle::X, "<mo>&#x00D7;</mo>"),
        (MultiplicationStyle::ThinSpace, "<mo>&#x2009;</mo>"),
        (MultiplicationStyle::NoSpace, ""),
    ];

    for (style, token) in cases {
        let input = binary(
            BinaryOperator::Multiply,
            Some(style),
            identifier("x"),
            identifier("y"),
        );
        assert_eq!(
            renderer.export_expression(&input).unwrap().as_str(),
            format!("{ROOT}<mrow><mi>x</mi>{token}<mi>y</mi></mrow></math>")
        );
    }
}

#[test]
fn dynamic_text_is_escaped_and_invalid_inputs_are_redacted_errors() {
    let renderer = MathMlRenderer::default();
    let escaped = identifier("a<&>");
    assert_eq!(
        renderer.export_expression(&escaped).unwrap().as_str(),
        format!("{ROOT}<mi>a&lt;&amp;&gt;</mi></math>")
    );

    let invalid_xml = identifier("hidden\0payload");
    assert_eq!(
        renderer.export_expression(&invalid_xml),
        Err(MathMlError::InvalidXmlText)
    );
    let invalid_number = real("not-a-number");
    assert_eq!(
        renderer.export_expression(&invalid_number),
        Err(MathMlError::InvalidLiteral)
    );
    let debug = format!(
        "{:?}",
        renderer.export_expression(&invalid_xml).unwrap_err()
    );
    assert!(!debug.contains("hidden"));

    let fragment = renderer
        .export_expression(&identifier("private-name"))
        .unwrap();
    let fragment_debug = format!("{fragment:?}");
    assert!(!fragment_debug.contains("private-name"));

    let oversized = identifier(&format!("{}\0", "a".repeat(256)));
    let input_limited = MathMlRenderer::new(MathMlLimits {
        max_output_bytes: 128,
        ..MathMlLimits::default()
    });
    assert_eq!(
        input_limited.export_expression(&oversized),
        Err(MathMlError::LimitExceeded(MathMlLimit::OutputBytes))
    );
}

#[test]
fn unsupported_or_ambiguous_input_fails_closed() {
    let renderer = MathMlRenderer::default();
    let unsupported = expression(MathExpressionKind::FunctionCall(FunctionCall {
        callee: Box::new(identifier("secret_function")),
        arguments: vec![real("1")],
    }));
    assert_eq!(
        renderer.export_expression(&unsupported),
        Err(MathMlError::UnsupportedExpression)
    );
    let unpaired = expression(MathExpressionKind::Grouping(Grouping {
        expression: Box::new(identifier("x")),
        unpaired: true,
    }));
    assert_eq!(
        renderer.export_expression(&unpaired),
        Err(MathMlError::InvalidExpression)
    );
    let missing_style = binary(
        BinaryOperator::Multiply,
        None,
        identifier("x"),
        identifier("y"),
    );
    assert_eq!(
        renderer.export_expression(&missing_style),
        Err(MathMlError::InvalidExpression)
    );
}

#[test]
fn limits_and_deep_left_associated_input_are_bounded_without_recursion() {
    assert_eq!(
        MathMlLimits::default(),
        MathMlLimits {
            max_depth: 256,
            max_nodes: 100_000,
            max_output_bytes: 4 * 1024 * 1024,
        }
    );
    let child = binary(BinaryOperator::Add, None, identifier("x"), identifier("y"));
    let depth_limited = MathMlRenderer::new(MathMlLimits {
        max_depth: 0,
        ..MathMlLimits::default()
    });
    assert_eq!(
        depth_limited.export_expression(&child),
        Err(MathMlError::LimitExceeded(MathMlLimit::Depth))
    );
    let node_limited = MathMlRenderer::new(MathMlLimits {
        max_nodes: 1,
        ..MathMlLimits::default()
    });
    assert_eq!(
        node_limited.export_expression(&child),
        Err(MathMlError::LimitExceeded(MathMlLimit::Nodes))
    );
    let output_limited = MathMlRenderer::new(MathMlLimits {
        max_output_bytes: 1,
        ..MathMlLimits::default()
    });
    assert_eq!(
        output_limited.export_expression(&identifier("x")),
        Err(MathMlError::LimitExceeded(MathMlLimit::OutputBytes))
    );

    let mut deep = identifier("x");
    for _ in 0..50_000 {
        deep = binary(BinaryOperator::Add, None, deep, identifier("x"));
    }
    assert_eq!(
        MathMlRenderer::default().export_expression(&deep),
        Err(MathMlError::LimitExceeded(MathMlLimit::Depth))
    );
    drop_left_linear_tree(deep);
}

#[test]
fn renderer_works_through_the_backend_neutral_port() {
    fn export_through_port<E>(
        exporter: &E,
        input: &MathExpression,
    ) -> Result<MathMlFragment, MathMlError>
    where
        E: EquationExporter<Output = MathMlFragment, Error = MathMlError>,
    {
        exporter.export(input)
    }

    let fragment = export_through_port(&MathMlRenderer::default(), &identifier("x")).unwrap();
    assert_eq!(fragment.as_str(), format!("{ROOT}<mi>x</mi></math>"));
}
