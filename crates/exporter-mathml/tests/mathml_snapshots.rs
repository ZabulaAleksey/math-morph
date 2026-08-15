use std::fs;
use std::path::{Path, PathBuf};

use exporter_mathml::MathMlRenderer;
use math_model::{
    BinaryExpression, BinaryOperator, ExpressionOrigin, Grouping, Identifier, MathExpression,
    MathExpressionKind, MultiplicationStyle, NumericBase, RealLiteral, SourceSpan, UnaryExpression,
    UnaryOperator,
};

const ROOT: &str = "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">";
const GOLDEN_NAMES: [&str; 17] = [
    "numeric-binary.mathml",
    "numeric-octal.mathml",
    "numeric-decimal.mathml",
    "numeric-hexadecimal.mathml",
    "identifier.mathml",
    "identifier-subscript.mathml",
    "identifier-escaped.mathml",
    "add.mathml",
    "subtract.mathml",
    "divide.mathml",
    "power.mathml",
    "multiply-dot.mathml",
    "multiply-x.mathml",
    "multiply-thin-space.mathml",
    "multiply-no-space.mathml",
    "square-root.mathml",
    "grouping.mathml",
];

fn expression(kind: MathExpressionKind) -> MathExpression {
    MathExpression {
        kind,
        origin: ExpressionOrigin::Derived,
    }
}

fn real(lexeme: &str, base: NumericBase) -> MathExpression {
    expression(MathExpressionKind::Real(RealLiteral {
        lexeme: lexeme.into(),
        base,
    }))
}

fn identifier(name: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: name.into(),
        subscript: None,
    }))
}

fn identifier_subscript(name: &str, subscript: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: name.into(),
        subscript: Some(subscript.into()),
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

fn cases() -> Vec<(&'static str, MathExpression)> {
    vec![
        ("numeric-binary.mathml", real("1010", NumericBase::Binary)),
        ("numeric-octal.mathml", real("755", NumericBase::Octal)),
        (
            "numeric-decimal.mathml",
            real("-12.5e+2", NumericBase::Decimal),
        ),
        (
            "numeric-hexadecimal.mathml",
            real("FF", NumericBase::Hexadecimal),
        ),
        ("identifier.mathml", identifier("x")),
        (
            "identifier-subscript.mathml",
            identifier_subscript("x", "i"),
        ),
        ("identifier-escaped.mathml", identifier("a<&>")),
        (
            "add.mathml",
            binary(
                BinaryOperator::Add,
                None,
                identifier("x"),
                real("1", NumericBase::Decimal),
            ),
        ),
        (
            "subtract.mathml",
            binary(
                BinaryOperator::Subtract,
                None,
                identifier("x"),
                real("1", NumericBase::Decimal),
            ),
        ),
        (
            "divide.mathml",
            binary(
                BinaryOperator::Divide,
                None,
                identifier("x"),
                real("2", NumericBase::Decimal),
            ),
        ),
        (
            "power.mathml",
            binary(
                BinaryOperator::Power,
                None,
                identifier("x"),
                real("2", NumericBase::Decimal),
            ),
        ),
        (
            "multiply-dot.mathml",
            binary(
                BinaryOperator::Multiply,
                Some(MultiplicationStyle::Dot),
                identifier("x"),
                identifier("y"),
            ),
        ),
        (
            "multiply-x.mathml",
            binary(
                BinaryOperator::Multiply,
                Some(MultiplicationStyle::X),
                identifier("x"),
                identifier("y"),
            ),
        ),
        (
            "multiply-thin-space.mathml",
            binary(
                BinaryOperator::Multiply,
                Some(MultiplicationStyle::ThinSpace),
                identifier("x"),
                identifier("y"),
            ),
        ),
        (
            "multiply-no-space.mathml",
            binary(
                BinaryOperator::Multiply,
                Some(MultiplicationStyle::NoSpace),
                identifier("x"),
                identifier("y"),
            ),
        ),
        (
            "square-root.mathml",
            expression(MathExpressionKind::Unary(UnaryExpression {
                operator: UnaryOperator::SquareRoot,
                operand: Box::new(identifier("x")),
            })),
        ),
        (
            "grouping.mathml",
            expression(MathExpressionKind::Grouping(Grouping {
                expression: Box::new(identifier("x")),
                unpaired: false,
            })),
        ),
    ]
}

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn mark_all_origins_as_source(root: &mut MathExpression) {
    let mut pending = vec![root];
    while let Some(current) = pending.pop() {
        current.origin = ExpressionOrigin::Source(SourceSpan { start: 11, end: 19 });
        match &mut current.kind {
            MathExpressionKind::Binary(binary) => {
                pending.push(binary.left.as_mut());
                pending.push(binary.right.as_mut());
            }
            MathExpressionKind::Unary(unary) => pending.push(unary.operand.as_mut()),
            MathExpressionKind::Grouping(grouping) => pending.push(grouping.expression.as_mut()),
            MathExpressionKind::Real(_) | MathExpressionKind::Identifier(_) => {}
            _ => unreachable!("snapshot cases contain only supported AST nodes"),
        }
    }
}

fn validate_golden_bytes(bytes: &[u8]) -> Result<&str, String> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err("BOM is not allowed".into());
    }
    if !bytes.ends_with(b"\n") {
        return Err("exactly one final LF is required".into());
    }
    let payload_bytes = &bytes[..bytes.len() - 1];
    if payload_bytes.ends_with(b"\n") {
        return Err("extra final LF is not allowed".into());
    }
    if bytes.contains(&b'\r') {
        return Err("CR is not allowed".into());
    }
    let payload = std::str::from_utf8(payload_bytes)
        .map_err(|_| "golden payload must be valid UTF-8".to_string())?;
    if payload.contains('\n') {
        return Err("payload must be single-line".into());
    }
    let body = payload
        .strip_prefix(ROOT)
        .and_then(|value| value.strip_suffix("</math>"))
        .ok_or_else(|| "invalid root envelope".to_string())?;
    if body.contains("<math") || body.contains("</math>") {
        return Err("nested or unbalanced math root is not allowed".into());
    }
    Ok(payload)
}

#[test]
fn inventory_and_canonical_file_format_are_guarded() {
    let dir = golden_dir();
    let mut actual: Vec<String> = fs::read_dir(&dir)
        .expect("golden directory")
        .map(|entry| {
            let path = entry.expect("golden directory entry").path();
            assert_eq!(
                path.extension().and_then(|value| value.to_str()),
                Some("mathml")
            );
            path.file_name()
                .expect("golden filename")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    actual.sort_unstable();
    let mut expected = GOLDEN_NAMES.map(String::from).to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected, "golden inventory changed");

    for name in GOLDEN_NAMES {
        let bytes = fs::read(dir.join(name)).expect("golden bytes");
        validate_golden_bytes(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

#[test]
fn production_renderer_matches_every_golden_byte_exactly() {
    let renderer = MathMlRenderer::default();
    let dir = golden_dir();
    let mut case_names: Vec<_> = cases().iter().map(|(name, _)| *name).collect();
    case_names.sort_unstable();
    let mut expected_names = GOLDEN_NAMES;
    expected_names.sort_unstable();
    assert_eq!(
        case_names, expected_names,
        "snapshot cases/golden inventory diverged"
    );
    for (name, input) in cases() {
        let expected = fs::read_to_string(dir.join(name)).expect("golden text");
        let expected = validate_golden_bytes(expected.as_bytes()).expect("valid golden");
        let actual = renderer
            .export_expression(&input)
            .expect("supported snapshot case");
        assert_eq!(
            actual.as_str().as_bytes(),
            expected.as_bytes(),
            "snapshot {name}"
        );
        assert_eq!(
            renderer.export_expression(&input).unwrap(),
            actual,
            "determinism {name}"
        );
    }
}

#[test]
fn malformed_golden_payloads_are_rejected() {
    let valid =
        b"<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\"><mi>x</mi></math>\n";
    assert!(validate_golden_bytes(valid).is_ok());
    assert!(validate_golden_bytes(
        b"<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\"><mi>x</mi></math></math>\n"
    )
    .is_err());
    assert!(
        validate_golden_bytes(
            b"<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\"><math></math>\n"
        )
        .is_err()
    );
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(valid);
    assert!(validate_golden_bytes(&bom).is_err());
    assert!(validate_golden_bytes(
        b"<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\"><mi>x</mi></math>\r\n"
    )
    .is_err());
    assert!(validate_golden_bytes(
        b"<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">\n<mi>x</mi></math>\n"
    )
    .is_err());
    assert!(validate_golden_bytes(&[0xff, b'\n']).is_err());
}

#[test]
fn expression_origin_does_not_change_snapshot_output() {
    let renderer = MathMlRenderer::default();
    for (_, mut input) in cases() {
        let derived = renderer
            .export_expression(&input)
            .expect("derived expression");
        mark_all_origins_as_source(&mut input);
        let source = renderer
            .export_expression(&input)
            .expect("source expression");
        assert_eq!(source, derived);
    }
}
