use document_ir::ports::EquationExporter;
use exporter_mathml::{MathMlError, MathMlLimit, MathMlLimits, MathMlRenderer};
use exporter_mathtype::{
    MATHTYPE_MATHML_MEDIA_TYPE, MathTypeAdapter, MathTypeError, MathTypePayload,
    MathTypePayloadFormat,
};
use math_model::{
    BinaryExpression, BinaryOperator, ExpressionOrigin, FunctionCall, Identifier, MathExpression,
    MathExpressionKind, NumericBase, RealLiteral,
};

const ROOT: &str = r#"<math xmlns="http://www.w3.org/1998/Math/MathML" display="block">"#;

fn expression(kind: MathExpressionKind) -> MathExpression {
    MathExpression {
        kind,
        origin: ExpressionOrigin::Derived,
    }
}

fn identifier(name: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: name.into(),
        subscript: None,
    }))
}

fn real(lexeme: &str) -> MathExpression {
    expression(MathExpressionKind::Real(RealLiteral {
        lexeme: lexeme.into(),
        base: NumericBase::Decimal,
    }))
}

fn add(left: MathExpression, right: MathExpression) -> MathExpression {
    expression(MathExpressionKind::Binary(BinaryExpression {
        operator: BinaryOperator::Add,
        multiplication_style: None,
        left: Box::new(left),
        right: Box::new(right),
    }))
}

#[test]
fn supported_expression_becomes_an_opaque_presentation_mathml_payload() {
    let adapter = MathTypeAdapter::default();
    let input = add(identifier("x"), real("1"));

    let payload = adapter
        .adapt_expression(&input)
        .expect("supported scalar expression");
    let renderer_output = MathMlRenderer::default()
        .export_expression(&input)
        .expect("same supported scalar expression");

    assert_eq!(payload.as_mathml(), renderer_output.as_str());
    assert_eq!(
        payload.as_mathml(),
        format!("{ROOT}<mrow><mi>x</mi><mo>+</mo><mn>1</mn></mrow></math>")
    );
    assert_eq!(payload.format(), MathTypePayloadFormat::PresentationMathMl);
    assert_eq!(payload.media_type(), MATHTYPE_MATHML_MEDIA_TYPE);
    assert_eq!(payload.as_bytes(), payload.as_mathml().as_bytes());
    assert_eq!(payload.byte_len(), payload.as_mathml().len());
    assert_eq!(adapter.adapt_expression(&input).unwrap(), payload);
    assert_eq!(adapter.limits(), &MathMlLimits::default());
}

#[test]
fn adapter_implements_the_backend_neutral_equation_port() {
    fn export_through_port<E>(
        exporter: &E,
        input: &MathExpression,
    ) -> Result<MathTypePayload, MathTypeError>
    where
        E: EquationExporter<Output = MathTypePayload, Error = MathTypeError>,
    {
        exporter.export(input)
    }

    let payload = export_through_port(&MathTypeAdapter::default(), &identifier("x")).unwrap();
    assert_eq!(payload.as_mathml(), format!("{ROOT}<mi>x</mi></math>"));
}

#[test]
fn unsupported_input_and_resource_limits_fail_closed() {
    let unsupported = expression(MathExpressionKind::FunctionCall(FunctionCall {
        callee: Box::new(identifier("private_function")),
        arguments: vec![real("1")],
    }));
    let error = MathTypeAdapter::default()
        .adapt_expression(&unsupported)
        .unwrap_err();
    assert_eq!(error.mathml_error(), MathMlError::UnsupportedExpression);
    assert!(!format!("{error:?}").contains("private_function"));

    let nested = add(identifier("x"), identifier("y"));

    let depth_limited = MathTypeAdapter::new(MathMlLimits {
        max_depth: 0,
        ..MathMlLimits::default()
    });
    let error = depth_limited.adapt_expression(&nested).unwrap_err();
    assert_eq!(
        error.mathml_error(),
        MathMlError::LimitExceeded(MathMlLimit::Depth)
    );

    let node_limited = MathTypeAdapter::new(MathMlLimits {
        max_nodes: 1,
        ..MathMlLimits::default()
    });
    let error = node_limited.adapt_expression(&nested).unwrap_err();
    assert_eq!(
        error.mathml_error(),
        MathMlError::LimitExceeded(MathMlLimit::Nodes)
    );

    let output_limited = MathTypeAdapter::new(MathMlLimits {
        max_output_bytes: 1,
        ..MathMlLimits::default()
    });
    let error = output_limited
        .adapt_expression(&identifier("x"))
        .unwrap_err();
    assert_eq!(
        error.mathml_error(),
        MathMlError::LimitExceeded(MathMlLimit::OutputBytes)
    );
}

#[test]
fn payload_and_error_debug_output_do_not_reveal_formula_text() {
    let payload = MathTypeAdapter::default()
        .adapt_expression(&identifier("private_identifier"))
        .unwrap();
    let payload_debug = format!("{payload:?}");
    assert!(!payload_debug.contains("private_identifier"));
    assert!(payload_debug.contains("byte_len"));

    let invalid = identifier("private\0identifier");
    let error = MathTypeAdapter::default()
        .adapt_expression(&invalid)
        .unwrap_err();
    assert_eq!(error.mathml_error(), MathMlError::InvalidXmlText);
    assert!(!format!("{error}").contains("private"));
}
