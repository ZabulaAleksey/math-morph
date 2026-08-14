use mathcad_parser::{
    BinaryOperator, DiagnosticCode, MathAstError, MathExpression, MathExpressionKind,
    MathParseOutcome, NumericBase, RegionContent, WorksheetLimits, WorksheetParser,
};

const WS: &str = "http://schemas.mathsoft.com/worksheet30";
const ML: &str = "http://schemas.mathsoft.com/math30";

fn document(expression: &str) -> Vec<u8> {
    format!(
        r#"<w:worksheet xmlns:w="{WS}" xmlns:m="{ML}" version="3.0.3"><w:regions><w:region region-id="1" top="0" left="0" height="1" width="1"><w:math>{expression}</w:math></w:region></w:regions></w:worksheet>"#
    )
    .into_bytes()
}

fn outcome_with_limits(expression: &str, limits: WorksheetLimits) -> MathParseOutcome {
    let worksheet = WorksheetParser::new(limits)
        .parse(&document(expression))
        .expect("worksheet envelope");
    let RegionContent::Math(math) = worksheet
        .regions
        .into_iter()
        .next()
        .expect("region")
        .content
    else {
        panic!("math region expected")
    };
    math.outcome
}

fn outcome(expression: &str) -> MathParseOutcome {
    outcome_with_limits(expression, WorksheetLimits::default())
}

/// Canonical test-only renderer; production serialization is intentionally not introduced.
fn sexpr(expression: &MathExpression) -> String {
    match &expression.kind {
        MathExpressionKind::Real(real) => format!(
            "(real base={} \"{}\")",
            real.base.value(),
            real.lexeme.escape_default()
        ),
        MathExpressionKind::Identifier(identifier) => match &identifier.subscript {
            Some(subscript) => format!(
                "(id \"{}\" subscript=\"{}\")",
                identifier.name.escape_default(),
                subscript.escape_default()
            ),
            None => format!("(id \"{}\")", identifier.name.escape_default()),
        },
        MathExpressionKind::Binary(binary) => format!(
            "({} {} {})",
            match binary.operator {
                BinaryOperator::Add => "add",
                BinaryOperator::Subtract => "subtract",
                BinaryOperator::Multiply => "multiply",
                BinaryOperator::Divide => "divide",
                BinaryOperator::Power => "power",
            },
            sexpr(&binary.left),
            sexpr(&binary.right)
        ),
        other => format!("({other:?})"),
    }
}

fn parsed(expression: &str) -> MathExpression {
    match outcome(expression) {
        MathParseOutcome::Parsed(expression) => expression,
        other => panic!("parsed expression expected, got {other:?}"),
    }
}

#[test]
fn ac_036_parses_real_identifier_and_all_binary_arithmetic_by_expanded_qname() {
    for (xml, expected) in [
        (
            r#"<m:real base="2">101.01</m:real>"#,
            r#"(real base=2 "101.01")"#,
        ),
        (r#"<m:real base="8">17</m:real>"#, r#"(real base=8 "17")"#),
        (
            r#"<m:real>-1.25e+3</m:real>"#,
            r#"(real base=10 "-1.25e+3")"#,
        ),
        (r#"<m:real base="16">aF</m:real>"#, r#"(real base=16 "aF")"#),
        (
            r#"<m:id subscript="n">x&amp;y</m:id>"#,
            r#"(id "x&y" subscript="n")"#,
        ),
    ] {
        assert_eq!(sexpr(&parsed(xml)), expected);
    }

    for (operator, name) in [
        ("plus", "add"),
        ("minus", "subtract"),
        ("mult", "multiply"),
        ("div", "divide"),
        ("pow", "power"),
    ] {
        let xml = format!(r#"<m:apply><m:{operator}/><m:real>1</m:real><m:id>x</m:id></m:apply>"#);
        assert_eq!(
            sexpr(&parsed(&xml)),
            format!(r#"({name} (real base=10 "1") (id "x"))"#)
        );
    }

    let alternate_prefix = format!(
        r#"<q:apply xmlns:q="{ML}"><q:plus/><q:real>1</q:real><q:real>2</q:real></q:apply>"#
    );
    assert_eq!(
        sexpr(&parsed(&alternate_prefix)),
        r#"(add (real base=10 "1") (real base=10 "2"))"#
    );
}

#[test]
fn ac_036_preserves_a_source_span_on_every_ast_expression() {
    let expression = parsed(
        r#"<m:apply><m:pow/><m:id>x</m:id><m:apply><m:plus/><m:real>2</m:real><m:real>3</m:real></m:apply></m:apply>"#,
    );
    fn assert_spans(expression: &MathExpression) {
        assert!(expression.span.start < expression.span.end);
        if let MathExpressionKind::Binary(binary) = &expression.kind {
            assert_spans(&binary.left);
            assert_spans(&binary.right);
            assert!(expression.span.start <= binary.left.span.start);
            assert!(binary.right.span.end <= expression.span.end);
        }
    }
    assert_spans(&expression);
}

#[test]
fn ac_037_canonical_nested_snapshot_is_deterministic() {
    let xml = r#"<m:apply><m:plus/><m:real>1</m:real><m:apply><m:mult/><m:id subscript="n">x</m:id><m:real base="16">FF</m:real></m:apply></m:apply>"#;
    let expected =
        r#"(add (real base=10 "1") (multiply (id "x" subscript="n") (real base=16 "FF")))"#;
    assert_eq!(sexpr(&parsed(xml)), expected);
    assert_eq!(sexpr(&parsed(xml)), expected);
}

#[test]
fn rejects_invalid_radix_real_lexemes_and_literal_structure_without_payload_in_errors() {
    for (xml, expected) in [
        (
            r#"<m:real base="3">secret</m:real>"#,
            MathAstError::InvalidRadix,
        ),
        (
            r#"<m:real base="2">102-secret</m:real>"#,
            MathAstError::MalformedReal,
        ),
        (r#"<m:real>1e+</m:real>"#, MathAstError::MalformedReal),
        (
            r#"<m:id><m:real>1</m:real></m:id>"#,
            MathAstError::MalformedLiteral,
        ),
    ] {
        let MathParseOutcome::Invalid(error) = outcome(xml) else {
            panic!("invalid outcome expected")
        };
        assert_eq!(error, expected);
        let rendered = format!("{error:?} {error}");
        assert!(!rendered.contains("secret"));
    }
}

#[test]
fn validates_binary_arity_and_expression_node_limit() {
    for (xml, actual) in [
        (r#"<m:apply><m:plus/><m:real>1</m:real></m:apply>"#, 1),
        (
            r#"<m:apply><m:plus/><m:real>1</m:real><m:real>2</m:real><m:real>3</m:real></m:apply>"#,
            3,
        ),
    ] {
        assert_eq!(
            outcome(xml),
            MathParseOutcome::Invalid(MathAstError::WrongBinaryArity {
                operator: BinaryOperator::Add,
                actual,
            })
        );
    }

    let limits = WorksheetLimits {
        max_ast_nodes: 2,
        ..WorksheetLimits::default()
    };
    assert_eq!(
        outcome_with_limits(
            r#"<m:apply><m:plus/><m:real>1</m:real><m:real>2</m:real></m:apply>"#,
            limits
        ),
        MathParseOutcome::Invalid(MathAstError::NodeLimitExceeded)
    );
}

#[test]
fn later_or_unknown_math_nodes_remain_diagnostic_fallbacks_not_ast_nodes() {
    for xml in [
        r#"<m:program><m:real>1</m:real></m:program>"#,
        r#"<m:future secret="payload"/>"#,
    ] {
        let MathParseOutcome::Unsupported(diagnostic) = outcome(xml) else {
            panic!("unsupported outcome expected")
        };
        assert_eq!(diagnostic.code, DiagnosticCode::UnsupportedMathNode);
        assert!(!format!("{diagnostic:?}").contains("payload"));
    }
}

#[test]
fn numeric_base_public_values_are_stable() {
    assert_eq!(NumericBase::Binary.value(), 2);
    assert_eq!(NumericBase::Octal.value(), 8);
    assert_eq!(NumericBase::Decimal.value(), 10);
    assert_eq!(NumericBase::Hexadecimal.value(), 16);
}
