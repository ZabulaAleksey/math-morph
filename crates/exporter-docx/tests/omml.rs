use document_ir::ports::EquationExporter;
use exporter_docx::{OmmlError, OmmlLimit, OmmlLimits, WordEquationExporter};
use math_model::{
    BinaryExpression, BinaryOperator, ExpressionOrigin, Grouping, Identifier, MathExpression,
    MathExpressionKind, MultiplicationStyle, NumericBase, RealLiteral,
};

const ROOT: &str =
    "<m:oMath xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\">";

fn expression(kind: MathExpressionKind) -> MathExpression {
    MathExpression {
        kind,
        origin: ExpressionOrigin::Derived,
    }
}

fn real(value: &str) -> MathExpression {
    expression(MathExpressionKind::Real(RealLiteral {
        lexeme: value.to_owned(),
        base: NumericBase::Decimal,
    }))
}

fn identifier(value: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: value.to_owned(),
        subscript: None,
    }))
}

fn binary(operator: BinaryOperator, left: MathExpression, right: MathExpression) -> MathExpression {
    expression(MathExpressionKind::Binary(BinaryExpression {
        operator,
        multiplication_style: (operator == BinaryOperator::Multiply)
            .then_some(MultiplicationStyle::Default),
        left: Box::new(left),
        right: Box::new(right),
    }))
}

fn multiply(
    style: MultiplicationStyle,
    left: MathExpression,
    right: MathExpression,
) -> MathExpression {
    expression(MathExpressionKind::Binary(BinaryExpression {
        operator: BinaryOperator::Multiply,
        multiplication_style: Some(style),
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

fn render(expression: &MathExpression) -> String {
    WordEquationExporter::default()
        .export(expression)
        .unwrap()
        .as_str()
        .to_owned()
}

#[test]
fn ac_071_to_073_render_number_and_italic_identifier_snapshots() {
    assert_eq!(
        render(&real("-12.5e+2")),
        format!("{ROOT}<m:r><m:t>-12.5e+2</m:t></m:r></m:oMath>")
    );
    assert_eq!(
        render(&identifier("x<&")),
        format!(
            "{ROOT}<m:r><m:rPr><m:sty m:val=\"i\"/></m:rPr><m:t>x&lt;&amp;</m:t></m:r></m:oMath>"
        )
    );
    assert!(render(&identifier(" x ")).contains("<m:t xml:space=\"preserve\"> x </m:t>"));
}

#[test]
fn ac_074_preserves_add_subtract_order_and_minus_glyph() {
    let expression = binary(
        BinaryOperator::Subtract,
        binary(BinaryOperator::Add, identifier("a"), real("2")),
        identifier("b"),
    );
    assert_eq!(
        render(&expression),
        format!(
            "{ROOT}<m:r><m:rPr><m:sty m:val=\"i\"/></m:rPr><m:t>a</m:t></m:r><m:r><m:t>+</m:t></m:r><m:r><m:t>2</m:t></m:r><m:r><m:t>−</m:t></m:r><m:r><m:rPr><m:sty m:val=\"i\"/></m:rPr><m:t>b</m:t></m:r></m:oMath>"
        )
    );
}

#[test]
fn ac_075_maps_every_multiplication_style_deterministically() {
    for style in [
        MultiplicationStyle::Default,
        MultiplicationStyle::AutoSelect,
        MultiplicationStyle::Dot,
        MultiplicationStyle::NarrowDot,
        MultiplicationStyle::LargeDot,
    ] {
        assert!(
            render(&multiply(style, real("2"), identifier("x")))
                .contains("<m:r><m:t>·</m:t></m:r>")
        );
    }
    assert!(
        render(&multiply(
            MultiplicationStyle::X,
            real("2"),
            identifier("x")
        ))
        .contains("<m:r><m:t>×</m:t></m:r>")
    );
    assert!(
        render(&multiply(
            MultiplicationStyle::ThinSpace,
            real("2"),
            identifier("x")
        ))
        .contains("<m:t xml:space=\"preserve\"> </m:t>")
    );
    let no_space = render(&multiply(
        MultiplicationStyle::NoSpace,
        real("2"),
        identifier("x"),
    ));
    assert!(!no_space.contains('·') && !no_space.contains('×') && !no_space.contains(' '));
}

#[test]
fn ac_076_renders_nested_fraction_with_structural_operand_grouping() {
    let numerator = binary(BinaryOperator::Add, identifier("x"), real("1"));
    let denominator = multiply(MultiplicationStyle::Dot, real("2"), identifier("y"));
    let expression = binary(BinaryOperator::Divide, numerator, denominator);
    assert_eq!(
        render(&expression),
        format!(
            "{ROOT}<m:f><m:fPr><m:type m:val=\"bar\"/></m:fPr><m:num><m:r><m:rPr><m:sty m:val=\"i\"/></m:rPr><m:t>x</m:t></m:r><m:r><m:t>+</m:t></m:r><m:r><m:t>1</m:t></m:r></m:num><m:den><m:r><m:t>2</m:t></m:r><m:r><m:t>·</m:t></m:r><m:r><m:rPr><m:sty m:val=\"i\"/></m:rPr><m:t>y</m:t></m:r></m:den></m:f></m:oMath>"
        )
    );
}

#[test]
fn trait_output_is_bounded_and_fragment_debug_is_redacted() {
    fn through_port<E>(exporter: &E, expression: &MathExpression) -> Result<E::Output, E::Error>
    where
        E: EquationExporter,
    {
        exporter.export(expression)
    }

    let fragment =
        through_port(&WordEquationExporter::default(), &identifier("secret-name")).unwrap();
    assert!(fragment.byte_len() > 0);
    assert!(!format!("{fragment:?}").contains("secret-name"));

    let error = WordEquationExporter::new(OmmlLimits {
        max_output_bytes: 16,
        ..OmmlLimits::default()
    })
    .export(&identifier("secret-name"))
    .unwrap_err();
    assert_eq!(error, OmmlError::LimitExceeded(OmmlLimit::OutputBytes));
    assert!(!format!("{error:?} {error}").contains("secret-name"));
}

#[test]
fn invalid_and_semantically_ambiguous_forms_fail_closed() {
    let subscript = expression(MathExpressionKind::Identifier(Identifier {
        name: "x".to_owned(),
        subscript: Some("secret".to_owned()),
    }));
    assert!(WordEquationExporter::default().export(&subscript).is_ok());
    assert_eq!(
        WordEquationExporter::default().export(&real("not-a-number")),
        Err(OmmlError::InvalidLiteral)
    );
    let power = binary(BinaryOperator::Power, real("2"), real("3"));
    assert!(WordEquationExporter::default().export(&power).is_ok());
    let grouped = expression(MathExpressionKind::Grouping(Grouping {
        expression: Box::new(identifier("x")),
        unpaired: false,
    }));
    assert!(WordEquationExporter::default().export(&grouped).is_ok());
    let needs_parentheses = multiply(
        MultiplicationStyle::Dot,
        binary(BinaryOperator::Add, identifier("a"), identifier("b")),
        identifier("c"),
    );
    assert_eq!(
        WordEquationExporter::default().export(&needs_parentheses),
        Err(OmmlError::SemanticGroupingRequired)
    );
    let right_nested_subtraction = binary(
        BinaryOperator::Subtract,
        identifier("a"),
        binary(BinaryOperator::Subtract, identifier("b"), identifier("c")),
    );
    assert_eq!(
        WordEquationExporter::default().export(&right_nested_subtraction),
        Err(OmmlError::SemanticGroupingRequired)
    );
}

#[test]
fn depth_node_and_structure_limits_are_total() {
    let sum = binary(BinaryOperator::Add, real("1"), real("2"));
    let fraction = binary(BinaryOperator::Divide, real("1"), real("2"));
    assert_eq!(
        WordEquationExporter::new(OmmlLimits {
            max_depth: 0,
            ..OmmlLimits::default()
        })
        .export(&fraction),
        Err(OmmlError::LimitExceeded(OmmlLimit::Depth))
    );
    assert_eq!(
        WordEquationExporter::new(OmmlLimits {
            max_nodes: 0,
            ..OmmlLimits::default()
        })
        .export(&sum),
        Err(OmmlError::LimitExceeded(OmmlLimit::Nodes))
    );
    let inconsistent_style = expression(MathExpressionKind::Binary(BinaryExpression {
        operator: BinaryOperator::Add,
        multiplication_style: Some(MultiplicationStyle::Dot),
        left: Box::new(real("1")),
        right: Box::new(real("2")),
    }));
    assert_eq!(
        WordEquationExporter::default().export(&inconsistent_style),
        Err(OmmlError::InvalidExpression)
    );
}

#[test]
fn deeply_left_associated_linear_forms_are_iterative_and_limit_checked() {
    let mut stress = real("0");
    for _ in 0..50_000 {
        stress = binary(BinaryOperator::Add, stress, real("1"));
    }
    assert_eq!(
        WordEquationExporter::new(OmmlLimits {
            max_nodes: 1,
            ..OmmlLimits::default()
        })
        .export(&stress),
        Err(OmmlError::LimitExceeded(OmmlLimit::Nodes))
    );
    drop_left_linear_tree(stress);

    let mut no_space_stress = real("1");
    for _ in 0..50_000 {
        no_space_stress = multiply(
            MultiplicationStyle::NoSpace,
            no_space_stress,
            identifier("x"),
        );
    }
    assert_eq!(
        WordEquationExporter::new(OmmlLimits {
            max_nodes: 0,
            ..OmmlLimits::default()
        })
        .export(&no_space_stress),
        Err(OmmlError::LimitExceeded(OmmlLimit::Nodes))
    );
    drop_left_linear_tree(no_space_stress);

    let mut allowed = real("1");
    for _ in 0..2_000 {
        allowed = multiply(MultiplicationStyle::NoSpace, allowed, identifier("x"));
    }
    let fragment = WordEquationExporter::default().export(&allowed).unwrap();
    assert!(fragment.byte_len() > 2_000);
    drop_left_linear_tree(allowed);
}
