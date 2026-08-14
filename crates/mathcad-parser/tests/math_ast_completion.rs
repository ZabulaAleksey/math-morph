use mathcad_parser::{
    BinaryOperator, BooleanOperator, DiagnosticCode, MathAstError, MathExpression,
    MathExpressionKind, MathParseOutcome, MultiplicationStyle, RegionContent, UnsupportedReason,
    WorksheetLimits, WorksheetParser,
};

const WS: &str = "http://schemas.mathsoft.com/worksheet30";
const ML: &str = "http://schemas.mathsoft.com/math30";
const U: &str = "http://schemas.mathsoft.com/units10";

fn outcome_with_limits(expression: &str, limits: WorksheetLimits) -> MathParseOutcome {
    let bytes = format!(
        r#"<w:worksheet xmlns:w="{WS}" xmlns:m="{ML}" xmlns:u="{U}" version="3.0.3"><w:regions><w:region region-id="1" top="0" left="0" height="1" width="1"><w:math>{expression}</w:math></w:region></w:regions></w:worksheet>"#
    );
    let worksheet = WorksheetParser::new(limits)
        .parse(bytes.as_bytes())
        .expect("worksheet envelope");
    let RegionContent::Math(math) = worksheet.regions.into_iter().next().unwrap().content else {
        panic!("math region expected")
    };
    math.outcome
}

fn outcome(expression: &str) -> MathParseOutcome {
    outcome_with_limits(expression, WorksheetLimits::default())
}

fn parsed(expression: &str) -> (MathExpression, Vec<mathcad_parser::Diagnostic>) {
    match outcome(expression) {
        MathParseOutcome::Parsed {
            expression,
            diagnostics,
        } => (expression, diagnostics),
        other => panic!("parsed expression expected, got {other:?}"),
    }
}

#[test]
fn ac_052_parses_boolean_structure_and_nested_comparisons() {
    for (name, operator) in [
        ("and", BooleanOperator::And),
        ("or", BooleanOperator::Or),
        ("xor", BooleanOperator::Xor),
    ] {
        let operands =
            r#"<m:apply><m:lessThan/><m:id>x</m:id><m:real>1</m:real></m:apply><m:id>ready</m:id>"#;
        let (expression, diagnostics) =
            parsed(&format!(r#"<m:apply><m:{name}/>{operands}</m:apply>"#));
        let MathExpressionKind::Boolean(boolean) = expression.kind else {
            panic!("boolean expression expected")
        };
        assert_eq!(boolean.operator, operator);
        assert!(matches!(
            boolean.left.kind,
            MathExpressionKind::Comparison(_)
        ));
        assert!(matches!(
            boolean.right.kind,
            MathExpressionKind::Identifier(_)
        ));
        assert!(diagnostics.is_empty());
    }

    let (expression, diagnostics) = parsed(
        r#"<m:apply><m:not/><m:apply><m:lessThan/><m:id>x</m:id><m:real>1</m:real></m:apply></m:apply>"#,
    );
    let MathExpressionKind::LogicalNot(logical_not) = expression.kind else {
        panic!("logical not expected")
    };
    assert!(matches!(
        logical_not.operand.kind,
        MathExpressionKind::Comparison(_)
    ));
    assert!(diagnostics.is_empty());
}

#[test]
fn ac_052_rejects_wrong_arity_marker_content_and_foreign_qname() {
    for (xml, expected) in [
        (
            r#"<m:apply><m:and/><m:id>x</m:id></m:apply>"#,
            MathAstError::WrongBooleanArity {
                operator: BooleanOperator::And,
                actual: 1,
            },
        ),
        (
            r#"<m:apply><m:not flag="payload"/><m:id>x</m:id></m:apply>"#,
            MathAstError::NonEmptyBooleanMarker,
        ),
        (
            r#"<m:apply><m:not/><m:id>x</m:id><m:id>y</m:id></m:apply>"#,
            MathAstError::WrongLogicalNotArity { actual: 2 },
        ),
        (
            r#"<m:apply><m:or>payload</m:or><m:id>x</m:id><m:id>y</m:id></m:apply>"#,
            MathAstError::NonEmptyBooleanMarker,
        ),
        (
            r#"<m:apply><x:and xmlns:x="urn:foreign"/><m:id>x</m:id><m:id>y</m:id></m:apply>"#,
            MathAstError::InvalidBooleanOperatorQName,
        ),
    ] {
        let result = outcome(xml);
        assert_eq!(result, MathParseOutcome::Invalid(expected));
        assert!(!format!("{result:?}").contains("payload"));
    }
}

#[test]
fn ac_053_parses_simple_compound_and_rational_power_units() {
    let (simple, diagnostics) = parsed(
        r#"<m:unitedValue><m:real>2</m:real><u:unitMonomial><u:unitReference unit="m"/></u:unitMonomial></m:unitedValue>"#,
    );
    let MathExpressionKind::UnitedValue(simple) = simple.kind else {
        panic!("united value expected")
    };
    assert_eq!(simple.units.system, None);
    assert_eq!(simple.units.factors[0].power_numerator, 1);
    assert_eq!(simple.units.factors[0].power_denominator.get(), 1);
    assert!(diagnostics.is_empty());

    let (compound, _) = parsed(
        r#"<m:unitedValue><m:real>9.81</m:real><u:unitMonomial system="SI"><u:unitReference unit="m"/><u:unitReference unit="s" power-numerator="-2" power-denominator="3"/></u:unitMonomial></m:unitedValue>"#,
    );
    let MathExpressionKind::UnitedValue(compound) = compound.kind else {
        panic!("united value expected")
    };
    assert_eq!(compound.units.system.as_deref(), Some("SI"));
    assert_eq!(compound.units.factors.len(), 2);
    assert_eq!(
        (
            compound.units.factors[1].power_numerator,
            compound.units.factors[1].power_denominator.get(),
        ),
        (-2, 3)
    );

    let (placeholder, diagnostics) = parsed(
        r#"<m:unitedValue><m:placeholder/><u:unitMonomial><u:unitReference unit="m"/></u:unitMonomial></m:unitedValue>"#,
    );
    let MathExpressionKind::UnitedValue(placeholder) = placeholder.kind else {
        panic!("united value expected")
    };
    assert!(matches!(
        placeholder.value.kind,
        MathExpressionKind::Unsupported(_)
    ));
    assert_eq!(diagnostics[0].code, DiagnosticCode::UnsupportedMathNode);
}

#[test]
fn ac_053_rejects_invalid_units_and_enforces_factor_and_ast_limits() {
    for (xml, expected) in [
        (
            r#"<m:unitedValue><m:real>1</m:real><u:unitMonomial><u:unitReference/></u:unitMonomial></m:unitedValue>"#,
            MathAstError::MissingUnitName,
        ),
        (
            r#"<m:unitedValue><m:real>1</m:real><u:unitMonomial><u:unitReference unit="m" power-denominator="0"/></u:unitMonomial></m:unitedValue>"#,
            MathAstError::ZeroUnitPowerDenominator,
        ),
        (
            r#"<m:unitedValue><m:real>1</m:real><u:unitMonomial><u:unitReference unit="m" power-numerator="9223372036854775808"/></u:unitMonomial></m:unitedValue>"#,
            MathAstError::InvalidUnitPower,
        ),
        (
            r#"<m:unitedValue><m:real>1</m:real><m:unitMonomial><m:unitReference unit="m"/></m:unitMonomial></m:unitedValue>"#,
            MathAstError::InvalidUnitQName,
        ),
    ] {
        assert_eq!(outcome(xml), MathParseOutcome::Invalid(expected));
    }

    let factors = r#"<u:unitReference unit="m"/><u:unitReference unit="s"/>"#;
    let factor_limited = WorksheetLimits {
        max_unit_factors: 1,
        ..WorksheetLimits::default()
    };
    assert_eq!(
        outcome_with_limits(
            &format!(
                r#"<m:unitedValue><m:real>1</m:real><u:unitMonomial>{factors}</u:unitMonomial></m:unitedValue>"#
            ),
            factor_limited,
        ),
        MathParseOutcome::Invalid(MathAstError::UnitFactorLimitExceeded)
    );

    let ast_limited = WorksheetLimits {
        max_ast_nodes: 3,
        ..WorksheetLimits::default()
    };
    assert_eq!(
        outcome_with_limits(
            r#"<m:unitedValue><m:real>1</m:real><u:unitMonomial><u:unitReference unit="m"/></u:unitMonomial></m:unitedValue>"#,
            ast_limited,
        ),
        MathParseOutcome::Invalid(MathAstError::NodeLimitExceeded)
    );
}

#[test]
fn ac_054_preserves_unknown_nested_nodes_and_stable_warnings() {
    let (expression, diagnostics) = parsed(
        r#"<m:apply><m:plus/><m:real>1</m:real><m:future secret="payload"><m:id>hidden</m:id></m:future></m:apply>"#,
    );
    let MathExpressionKind::Binary(binary) = expression.kind else {
        panic!("known parent must survive")
    };
    let MathExpressionKind::Unsupported(unsupported) = &binary.right.kind else {
        panic!("unsupported nested child expected")
    };
    assert_eq!(unsupported.reason, UnsupportedReason::UnknownExpression);
    assert_eq!(unsupported.span, binary.right.source_span().unwrap());
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, DiagnosticCode::UnsupportedMathNode);
    let debug = format!("{binary:?} {diagnostics:?}");
    assert!(!debug.contains("payload"));
    assert!(!debug.contains("future"));

    let (unknown_apply, diagnostics) =
        parsed(r#"<m:apply><m:futureOp secret="payload"/><m:id>x</m:id></m:apply>"#);
    let MathExpressionKind::Unsupported(node) = unknown_apply.kind else {
        panic!("whole apply must be unsupported")
    };
    assert!(node.feature.is_some());
    assert_eq!(node.reason, UnsupportedReason::UnknownOperator);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn multiplication_style_is_preserved_and_validated() {
    for (style, expected) in [
        (None, MultiplicationStyle::Default),
        (Some("auto-select"), MultiplicationStyle::AutoSelect),
        (Some("dot"), MultiplicationStyle::Dot),
        (Some("narrow-dot"), MultiplicationStyle::NarrowDot),
        (Some("large-dot"), MultiplicationStyle::LargeDot),
        (Some("x"), MultiplicationStyle::X),
        (Some("thin-space"), MultiplicationStyle::ThinSpace),
        (Some("no-space"), MultiplicationStyle::NoSpace),
    ] {
        let attribute = style
            .map(|value| format!(r#" style="{value}""#))
            .unwrap_or_default();
        let (expression, _) = parsed(&format!(
            r#"<m:apply><m:mult{attribute}/><m:real>2</m:real><m:id>x</m:id></m:apply>"#
        ));
        let MathExpressionKind::Binary(binary) = expression.kind else {
            panic!("binary multiplication expected")
        };
        assert_eq!(binary.operator, BinaryOperator::Multiply);
        assert_eq!(binary.multiplication_style, Some(expected));
    }
    assert_eq!(
        outcome(r#"<m:apply><m:mult style="secret"/><m:real>2</m:real><m:id>x</m:id></m:apply>"#),
        MathParseOutcome::Invalid(MathAstError::InvalidMultiplicationStyle)
    );
}
