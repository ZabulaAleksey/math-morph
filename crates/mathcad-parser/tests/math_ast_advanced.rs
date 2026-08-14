use mathcad_parser::{
    AggregateOperator, ComparisonOperator, DerivativeStyle, DiagnosticCode, IntegralAlgorithm,
    MathAstError, MathExpression, MathExpressionKind, MathParseOutcome, RegionContent,
    VectorOrientation, WorksheetLimits, WorksheetParser,
};

const WS: &str = "http://schemas.mathsoft.com/worksheet30";
const ML: &str = "http://schemas.mathsoft.com/math30";

fn outcome_with_limits(expression: &str, limits: WorksheetLimits) -> MathParseOutcome {
    let bytes = format!(
        r#"<w:worksheet xmlns:w="{WS}" xmlns:m="{ML}" version="3.0.3"><w:regions><w:region region-id="1" top="0" left="0" height="1" width="1"><w:math>{expression}</w:math></w:region></w:regions></w:worksheet>"#
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

fn parsed(expression: &str) -> MathExpression {
    match outcome(expression) {
        MathParseOutcome::Parsed(expression) => expression,
        other => panic!("parsed expression expected, got {other:?}"),
    }
}

fn atom(expression: &MathExpression) -> String {
    match &expression.kind {
        MathExpressionKind::Real(real) => format!("real:{}", real.lexeme),
        MathExpressionKind::Identifier(id) => format!("id:{}", id.name),
        _ => format!("{:?}", expression.kind_name_for_test()),
    }
}

/// Canonical test-only S-expression renderer for the newly supported forms.
fn sexpr(expression: &MathExpression) -> String {
    match &expression.kind {
        MathExpressionKind::Real(_) | MathExpressionKind::Identifier(_) => atom(expression),
        MathExpressionKind::Matrix(matrix) => format!(
            "(matrix {}x{} {})",
            matrix.rows,
            matrix.columns,
            matrix
                .elements
                .iter()
                .map(sexpr)
                .collect::<Vec<_>>()
                .join(" ")
        ),
        MathExpressionKind::Range(range) => format!(
            "(range {} next={} {})",
            sexpr(&range.start),
            range
                .next
                .as_deref()
                .map(sexpr)
                .unwrap_or_else(|| "none".into()),
            sexpr(&range.end)
        ),
        MathExpressionKind::Integral(integral) => format!(
            "(integral algorithm={} {} {} bounds={})",
            integral
                .algorithm
                .map(|value| format!("{value:?}"))
                .unwrap_or_else(|| "none".to_owned()),
            sexpr(&integral.bound_variable),
            sexpr(&integral.integrand),
            integral
                .bounds
                .as_ref()
                .map(bounds_sexpr)
                .unwrap_or_else(|| "none".into())
        ),
        _ => format!("(other {})", expression.kind_name_for_test()),
    }
}

fn bounds_sexpr(bounds: &mathcad_parser::Bounds) -> String {
    format!("({} {})", sexpr(&bounds.lower), sexpr(&bounds.upper))
}

trait TestKindName {
    fn kind_name_for_test(&self) -> &'static str;
}

impl TestKindName for MathExpression {
    fn kind_name_for_test(&self) -> &'static str {
        match self.kind {
            MathExpressionKind::Real(_) => "real",
            MathExpressionKind::Identifier(_) => "id",
            MathExpressionKind::Binary(_) => "binary",
            MathExpressionKind::Definition(_) => "definition",
            MathExpressionKind::Evaluation(_) => "evaluation",
            MathExpressionKind::FunctionCall(_) => "call",
            MathExpressionKind::FunctionDefinition(_) => "function",
            MathExpressionKind::Unary(_) => "unary",
            MathExpressionKind::Grouping(_) => "grouping",
            MathExpressionKind::ArrayIndex(_) => "index",
            MathExpressionKind::Matrix(_) => "matrix",
            MathExpressionKind::Vector(_) => "vector",
            MathExpressionKind::Range(_) => "range",
            MathExpressionKind::Integral(_) => "integral",
            MathExpressionKind::Derivative(_) => "derivative",
            MathExpressionKind::Aggregate(_) => "aggregate",
            MathExpressionKind::Comparison(_) => "comparison",
        }
    }
}

#[test]
fn ac_045_and_046_parse_matrix_shape_and_vector_specialization() {
    let matrix = parsed(
        r#"<m:matrix rows="2" cols="2"><m:real>1</m:real><m:real>2</m:real><m:real>3</m:real><m:real>4</m:real></m:matrix>"#,
    );
    let MathExpressionKind::Matrix(matrix) = matrix.kind else {
        panic!("matrix")
    };
    assert_eq!(
        (matrix.rows, matrix.columns, matrix.elements.len()),
        (2, 2, 4)
    );
    assert!(
        matrix
            .elements
            .iter()
            .all(|item| item.span.start < item.span.end)
    );

    for (xml, orientation) in [
        (
            r#"<m:matrix rows="1" cols="2"><m:real>1</m:real><m:real>2</m:real></m:matrix>"#,
            VectorOrientation::Row,
        ),
        (
            r#"<m:matrix rows="2" cols="1"><m:real>1</m:real><m:real>2</m:real></m:matrix>"#,
            VectorOrientation::Column,
        ),
    ] {
        let MathExpressionKind::Vector(vector) = parsed(xml).kind else {
            panic!("vector")
        };
        assert_eq!(vector.orientation, orientation);
    }
    assert!(matches!(
        parsed(r#"<m:matrix rows="1" cols="1"><m:real>1</m:real></m:matrix>"#).kind,
        MathExpressionKind::Matrix(_)
    ));
    assert_eq!(
        sexpr(&parsed(
            r#"<m:matrix rows="1" cols="1"><m:real>9</m:real></m:matrix>"#
        )),
        "(matrix 1x1 real:9)"
    );
}

#[test]
fn ac_047_distinguishes_simple_and_explicit_next_ranges() {
    let simple = parsed(r#"<m:range><m:real>1</m:real><m:real>5</m:real></m:range>"#);
    let MathExpressionKind::Range(simple) = simple.kind else {
        panic!("range")
    };
    assert_eq!(
        (
            atom(&simple.start),
            simple.next.is_none(),
            atom(&simple.end)
        ),
        ("real:1".into(), true, "real:5".into())
    );
    let stepped = parsed(
        r#"<m:range><m:sequence><m:real>1</m:real><m:real>2</m:real></m:sequence><m:real>5</m:real></m:range>"#,
    );
    let MathExpressionKind::Range(stepped) = stepped.kind else {
        panic!("range")
    };
    assert_eq!(atom(stepped.next.as_deref().unwrap()), "real:2");
    assert_eq!(
        sexpr(&parsed(
            r#"<m:range><m:sequence><m:real>1</m:real><m:real>2</m:real></m:sequence><m:real>5</m:real></m:range>"#
        )),
        "(range real:1 next=real:2 real:5)"
    );
}

#[test]
fn ac_048_to_050_parse_calculus_lambda_metadata_and_bounds() {
    let integral = parsed(
        r#"<m:apply><m:integral algorithm="romberg"/><m:lambda><m:boundVars><m:id>x</m:id></m:boundVars><m:id>f</m:id></m:lambda><m:bounds><m:real>0</m:real><m:real>1</m:real></m:bounds></m:apply>"#,
    );
    let MathExpressionKind::Integral(integral) = integral.kind else {
        panic!("integral")
    };
    assert_eq!(integral.algorithm, Some(IntegralAlgorithm::Romberg));
    assert!(integral.bounds.is_some());
    assert_eq!(
        sexpr(&parsed(
            r#"<m:apply><m:integral algorithm="romberg"/><m:lambda><m:boundVars><m:id>x</m:id></m:boundVars><m:id>f</m:id></m:lambda><m:bounds><m:real>0</m:real><m:real>1</m:real></m:bounds></m:apply>"#
        )),
        "(integral algorithm=Romberg id:x id:f bounds=(real:0 real:1))"
    );

    let derivative = parsed(
        r#"<m:apply><m:derivative style="partial"/><m:lambda><m:boundVars><m:id>x</m:id></m:boundVars><m:id>f</m:id></m:lambda><m:degree><m:real>2</m:real></m:degree></m:apply>"#,
    );
    let MathExpressionKind::Derivative(derivative) = derivative.kind else {
        panic!("derivative")
    };
    assert_eq!(derivative.style, DerivativeStyle::Partial);
    assert_eq!(atom(derivative.degree.as_deref().unwrap()), "real:2");

    for (name, operator) in [
        ("summation", AggregateOperator::Summation),
        ("product", AggregateOperator::Product),
    ] {
        let xml = format!(
            r#"<m:apply><m:{name}/><m:lambda><m:boundVars><m:id>k</m:id></m:boundVars><m:id>a</m:id></m:lambda><m:bounds><m:real>1</m:real><m:real>3</m:real></m:bounds></m:apply>"#
        );
        let MathExpressionKind::Aggregate(aggregate) = parsed(&xml).kind else {
            panic!("aggregate")
        };
        assert_eq!(aggregate.operator, operator);
        assert!(aggregate.bounds.is_some());
    }
}

#[test]
fn ac_051_parses_six_strict_binary_comparisons() {
    for (name, operator) in [
        ("equal", ComparisonOperator::Equal),
        ("notEqual", ComparisonOperator::NotEqual),
        ("greaterOrEqual", ComparisonOperator::GreaterOrEqual),
        ("greaterThan", ComparisonOperator::GreaterThan),
        ("lessOrEqual", ComparisonOperator::LessOrEqual),
        ("lessThan", ComparisonOperator::LessThan),
    ] {
        let xml = format!(r#"<m:apply><m:{name}/><m:id>x</m:id><m:real>1</m:real></m:apply>"#);
        let MathExpressionKind::Comparison(comparison) = parsed(&xml).kind else {
            panic!("comparison")
        };
        assert_eq!(comparison.operator, operator);
        assert!(comparison.left.span.start < comparison.right.span.end);
    }
}

#[test]
fn rejects_matrix_shape_limit_and_foreign_qname() {
    for (xml, expected) in [
        (
            r#"<m:matrix rows="0" cols="1"/>"#,
            MathAstError::InvalidMatrixDimensions,
        ),
        (
            r#"<m:matrix rows="2" cols="2"><m:real>1</m:real></m:matrix>"#,
            MathAstError::MatrixElementCountMismatch {
                expected: 4,
                actual: 1,
            },
        ),
    ] {
        assert_eq!(outcome(xml), MathParseOutcome::Invalid(expected));
    }
    let limits = WorksheetLimits {
        max_matrix_elements: 3,
        ..WorksheetLimits::default()
    };
    assert_eq!(
        outcome_with_limits(
            r#"<m:matrix rows="2" cols="2"><m:real>1</m:real><m:real>2</m:real><m:real>3</m:real><m:real>4</m:real></m:matrix>"#,
            limits
        ),
        MathParseOutcome::Invalid(MathAstError::MatrixElementLimitExceeded)
    );
    let MathParseOutcome::Unsupported(diagnostic) = outcome(
        r#"<m:matrix xmlns:x="urn:foreign" rows="1" cols="1"><x:real>1</x:real></m:matrix>"#,
    ) else {
        panic!("unsupported")
    };
    assert_eq!(diagnostic.code, DiagnosticCode::UnsupportedMathNode);
}

#[test]
fn rejects_malformed_range_calculus_comparison_and_preserves_limits_and_redaction() {
    for (xml, expected) in [
        (
            r#"<m:range><m:sequence><m:real>1</m:real></m:sequence><m:real>3</m:real></m:range>"#,
            MathAstError::MalformedRange,
        ),
        (
            r#"<m:apply><m:integral/><m:lambda><m:boundVars><m:id>x</m:id><m:id>y</m:id></m:boundVars><m:id>f</m:id></m:lambda></m:apply>"#,
            MathAstError::InvalidBoundVariable,
        ),
        (
            r#"<m:apply><m:lessThan/><m:id>x</m:id></m:apply>"#,
            MathAstError::WrongComparisonArity {
                operator: ComparisonOperator::LessThan,
                actual: 1,
            },
        ),
        (
            r#"<m:apply><m:integral algorithm="invented"/><m:lambda><m:boundVars><m:id>x</m:id></m:boundVars><m:id>f</m:id></m:lambda></m:apply>"#,
            MathAstError::InvalidIntegralAlgorithm,
        ),
        (
            r#"<m:apply><m:derivative style="prime"/><m:lambda><m:boundVars><m:id>x</m:id></m:boundVars><m:id>f</m:id></m:lambda></m:apply>"#,
            MathAstError::InvalidDerivativeStyle,
        ),
    ] {
        assert_eq!(outcome(xml), MathParseOutcome::Invalid(expected));
    }
    let limits = WorksheetLimits {
        max_ast_nodes: 2,
        ..WorksheetLimits::default()
    };
    let limited = outcome_with_limits(
        r#"<m:matrix rows="1" cols="2"><m:id>secret</m:id><m:real>2</m:real></m:matrix>"#,
        limits,
    );
    assert_eq!(
        limited,
        MathParseOutcome::Invalid(MathAstError::NodeLimitExceeded)
    );
    assert!(!format!("{limited:?}").contains("secret"));
}
