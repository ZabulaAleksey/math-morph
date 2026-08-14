use mathcad_parser::{
    BinaryOperator, DefinitionKind, DefinitionStyle, MathAstError, MathExpression,
    MathExpressionKind, MathParseOutcome, RegionContent, UnaryOperator, WorksheetLimits,
    WorksheetParser,
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

fn parsed(expression: &str) -> MathExpression {
    match outcome(expression) {
        MathParseOutcome::Parsed { expression, .. } => expression,
        other => panic!("parsed expression expected, got {other:?}"),
    }
}

fn sexpr(expression: &MathExpression) -> String {
    match &expression.kind {
        MathExpressionKind::Real(real) => {
            format!(
                "(real {} \"{}\")",
                real.base.value(),
                real.lexeme.escape_default()
            )
        }
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
            binary_name(binary.operator),
            sexpr(&binary.left),
            sexpr(&binary.right)
        ),
        MathExpressionKind::Definition(definition) => format!(
            "(define {:?} {:?} {} {})",
            definition.kind,
            definition.style,
            sexpr(&definition.target),
            sexpr(&definition.value)
        ),
        MathExpressionKind::Evaluation(evaluation) => format!(
            "(eval {} unit={} result={})",
            sexpr(&evaluation.expression),
            evaluation
                .unit_override
                .as_deref()
                .map(sexpr)
                .unwrap_or_else(|| "none".to_owned()),
            evaluation
                .saved_result
                .as_deref()
                .map(sexpr)
                .unwrap_or_else(|| "none".to_owned())
        ),
        MathExpressionKind::FunctionCall(call) => format!(
            "(call {}{})",
            sexpr(&call.callee),
            call.arguments
                .iter()
                .map(|argument| format!(" {}", sexpr(argument)))
                .collect::<String>()
        ),
        MathExpressionKind::FunctionDefinition(function) => format!(
            "(function {:?} {} ({}) {})",
            function.style,
            sexpr(&function.name),
            function
                .parameters
                .iter()
                .map(sexpr)
                .collect::<Vec<_>>()
                .join(" "),
            sexpr(&function.body)
        ),
        MathExpressionKind::Unary(unary) => {
            format!("(unary {:?} {})", unary.operator, sexpr(&unary.operand))
        }
        MathExpressionKind::Grouping(grouping) => format!(
            "(parens unpaired={} {})",
            grouping.unpaired,
            sexpr(&grouping.expression)
        ),
        MathExpressionKind::ArrayIndex(index) => format!(
            "(index {} ({}))",
            sexpr(&index.target),
            index
                .indices
                .iter()
                .map(sexpr)
                .collect::<Vec<_>>()
                .join(" ")
        ),
        _ => "(later-stage-expression)".to_owned(),
    }
}

fn binary_name(operator: BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Add => "add",
        BinaryOperator::Subtract => "subtract",
        BinaryOperator::Multiply => "multiply",
        BinaryOperator::Divide => "divide",
        BinaryOperator::Power => "power",
    }
}

#[test]
fn ac_038_parses_three_definition_kinds_and_scoped_styles() {
    for (xml, kind, style) in [
        (
            r#"<m:define style="colon-equal"><m:id>x</m:id><m:real>1</m:real></m:define>"#,
            DefinitionKind::Define,
            DefinitionStyle::ColonEqual,
        ),
        (
            r#"<m:globalDefine style="triple-equal"><m:id>x</m:id><m:real>1</m:real></m:globalDefine>"#,
            DefinitionKind::GlobalDefine,
            DefinitionStyle::TripleEqual,
        ),
        (
            r#"<m:localDefine style="left-arrow"><m:id>x</m:id><m:real>1</m:real></m:localDefine>"#,
            DefinitionKind::LocalDefine,
            DefinitionStyle::LeftArrow,
        ),
    ] {
        let expression = parsed(xml);
        let MathExpressionKind::Definition(definition) = expression.kind else {
            panic!("definition expected")
        };
        assert_eq!((definition.kind, definition.style), (kind, style));
        let span = definition.target.source_span().expect("source origin");
        assert!(span.start < span.end);
    }
}

#[test]
fn ac_039_to_041_parse_evaluation_calls_and_function_definitions() {
    let evaluation = parsed(
        r#"<m:eval><m:id>x</m:id><m:unitOverride><m:id>m</m:id></m:unitOverride><m:result><m:real>2</m:real></m:result></m:eval>"#,
    );
    assert_eq!(
        sexpr(&evaluation),
        r#"(eval (id "x") unit=(id "m") result=(real 10 "2"))"#
    );

    let call = parsed(r#"<m:apply><m:id>f</m:id><m:real>1</m:real><m:id>x</m:id></m:apply>"#);
    assert_eq!(sexpr(&call), r#"(call (id "f") (real 10 "1") (id "x"))"#);

    let function = parsed(
        r#"<m:define style="equal"><m:function><m:id>f</m:id><m:boundVars><m:id>x</m:id><m:id>y</m:id></m:boundVars></m:function><m:apply><m:plus/><m:id>x</m:id><m:id>y</m:id></m:apply></m:define>"#,
    );
    assert_eq!(
        sexpr(&function),
        r#"(function Equal (id "f") ((id "x") (id "y")) (add (id "x") (id "y")))"#
    );
}

#[test]
fn ac_042_parses_every_scoped_non_boolean_unary_operator() {
    for (xml_name, operator) in [
        ("absval", UnaryOperator::AbsoluteValue),
        ("conjugate", UnaryOperator::Conjugate),
        ("factorial", UnaryOperator::Factorial),
        ("neg", UnaryOperator::Negate),
        ("sqrt", UnaryOperator::SquareRoot),
        ("transpose", UnaryOperator::Transpose),
        ("vectorize", UnaryOperator::Vectorize),
        ("vectorSum", UnaryOperator::VectorSum),
        ("determinant", UnaryOperator::Determinant),
    ] {
        let expression = parsed(&format!(
            r#"<m:apply><m:{xml_name}/><m:id>x</m:id></m:apply>"#
        ));
        let MathExpressionKind::Unary(unary) = expression.kind else {
            panic!("unary expected")
        };
        assert_eq!(unary.operator, operator);
    }

    let MathParseOutcome::Parsed {
        expression,
        diagnostics,
    } = outcome(r#"<m:apply><m:not/><m:id>x</m:id></m:apply>"#)
    else {
        panic!("boolean not must parse")
    };
    assert!(matches!(expression.kind, MathExpressionKind::LogicalNot(_)));
    assert!(diagnostics.is_empty());
}

#[test]
fn ac_043_and_044_keep_grouping_literal_subscript_and_array_indices_distinct() {
    assert_eq!(
        sexpr(&parsed(
            r#"<m:parens unpaired="true"><m:id>x</m:id></m:parens>"#
        )),
        r#"(parens unpaired=true (id "x"))"#
    );
    assert_eq!(
        sexpr(&parsed(r#"<m:id subscript="literal">A</m:id>"#)),
        r#"(id "A" subscript="literal")"#
    );
    assert_eq!(
        sexpr(&parsed(
            r#"<m:apply><m:indexer/><m:id>A</m:id><m:sequence><m:real>0</m:real><m:real>1</m:real></m:sequence></m:apply>"#
        )),
        r#"(index (id "A") ((real 10 "0") (real 10 "1")))"#
    );
}

#[test]
fn rejects_invalid_definition_and_function_targets_or_styles() {
    for (xml, expected) in [
        (
            r#"<m:define><m:real>1</m:real><m:real>2</m:real></m:define>"#,
            MathAstError::InvalidDefinitionTarget,
        ),
        (
            r#"<m:globalDefine style="left-arrow"><m:id>x</m:id><m:real>2</m:real></m:globalDefine>"#,
            MathAstError::InvalidDefinitionStyle,
        ),
        (
            r#"<m:define><m:function><m:real>1</m:real><m:boundVars><m:id>x</m:id></m:boundVars></m:function><m:real>2</m:real></m:define>"#,
            MathAstError::InvalidFunctionName,
        ),
        (
            r#"<m:define><m:function><m:id>f</m:id><m:boundVars><m:real>1</m:real></m:boundVars></m:function><m:real>2</m:real></m:define>"#,
            MathAstError::InvalidFunctionParameter,
        ),
        (
            r#"<m:define><m:function><m:id>f</m:id><m:boundVars/></m:function><m:real>2</m:real></m:define>"#,
            MathAstError::InvalidFunctionParameter,
        ),
    ] {
        assert_eq!(outcome(xml), MathParseOutcome::Invalid(expected));
    }
}

#[test]
fn rejects_wrong_eval_call_unary_grouping_and_index_forms() {
    for (xml, expected) in [
        (
            r#"<m:eval><m:id>x</m:id><m:result><m:real>1</m:real><m:real>2</m:real></m:result></m:eval>"#,
            MathAstError::MalformedEvaluation,
        ),
        (
            r#"<m:apply><m:id>f</m:id></m:apply>"#,
            MathAstError::WrongFunctionArity { actual: 0 },
        ),
        (
            r#"<m:apply><m:sqrt/><m:id>x</m:id><m:id>y</m:id></m:apply>"#,
            MathAstError::WrongUnaryArity {
                operator: UnaryOperator::SquareRoot,
                actual: 2,
            },
        ),
        (
            r#"<m:parens unpaired="maybe"><m:id>x</m:id></m:parens>"#,
            MathAstError::InvalidBooleanAttribute,
        ),
        (
            r#"<m:parens><m:id>x</m:id><m:id>y</m:id></m:parens>"#,
            MathAstError::MalformedGrouping,
        ),
        (
            r#"<m:apply><m:indexer/><m:id>A</m:id></m:apply>"#,
            MathAstError::WrongArrayIndexArity { actual: 1 },
        ),
        (
            r#"<m:apply><m:indexer/><m:id>A</m:id><m:sequence/></m:apply>"#,
            MathAstError::MalformedArrayIndex,
        ),
        (
            r#"<m:apply><m:indexer/><m:id>A</m:id><m:sequence><m:real>0</m:real></m:sequence></m:apply>"#,
            MathAstError::MalformedArrayIndex,
        ),
    ] {
        assert_eq!(outcome(xml), MathParseOutcome::Invalid(expected));
    }
}

#[test]
fn stage_038_to_044_nodes_obey_the_shared_ast_limit_and_redact_debug_payload() {
    let limits = WorksheetLimits {
        max_ast_nodes: 2,
        ..WorksheetLimits::default()
    };
    let outcome = outcome_with_limits(
        r#"<m:eval><m:id>secret-name</m:id><m:unitOverride><m:id>m</m:id></m:unitOverride></m:eval>"#,
        limits,
    );
    assert_eq!(
        outcome,
        MathParseOutcome::Invalid(MathAstError::NodeLimitExceeded)
    );
    assert!(!format!("{outcome:?}").contains("secret-name"));
}
