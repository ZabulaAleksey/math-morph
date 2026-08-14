use crate::ast::{
    ArrayIndex, BinaryExpression, BinaryOperator, Definition, DefinitionKind, DefinitionStyle,
    Evaluation, FunctionCall, FunctionDefinition, Grouping, Identifier, MathAstError,
    MathExpression, MathExpressionKind, NumericBase, RealLiteral, UnaryExpression, UnaryOperator,
};
use crate::xml_worksheet::{Child, Node};

const MATH_NS: &str = "http://schemas.mathsoft.com/math30";

pub(crate) enum MathXmlOutcome {
    Parsed(MathExpression),
    Unsupported,
    Invalid(MathAstError),
}

pub(crate) fn parse_math_expression(node: &Node, max_nodes: usize) -> MathXmlOutcome {
    let mut parser = MathParser {
        max_nodes,
        nodes: 0,
    };
    match parser.expression(node) {
        Ok(expression) => MathXmlOutcome::Parsed(expression),
        Err(Failure::Unsupported) => MathXmlOutcome::Unsupported,
        Err(Failure::Invalid(error)) => MathXmlOutcome::Invalid(error),
    }
}

enum Failure {
    Unsupported,
    Invalid(MathAstError),
}

impl From<MathAstError> for Failure {
    fn from(error: MathAstError) -> Self {
        Self::Invalid(error)
    }
}

struct MathParser {
    max_nodes: usize,
    nodes: usize,
}

impl MathParser {
    fn expression(&mut self, node: &Node) -> Result<MathExpression, Failure> {
        self.nodes = self
            .nodes
            .checked_add(1)
            .ok_or(MathAstError::NodeLimitExceeded)?;
        if self.nodes > self.max_nodes {
            return Err(MathAstError::NodeLimitExceeded.into());
        }
        if node.name.namespace_uri.as_deref() != Some(MATH_NS) {
            return Err(Failure::Unsupported);
        }
        let kind = match node.name.local_name.as_str() {
            "real" => MathExpressionKind::Real(self.real(node)?),
            "id" => MathExpressionKind::Identifier(self.identifier(node)?),
            "apply" => self.application(node)?,
            "define" => self.definition(node, DefinitionKind::Define)?,
            "globalDefine" => self.definition(node, DefinitionKind::GlobalDefine)?,
            "localDefine" => self.definition(node, DefinitionKind::LocalDefine)?,
            "eval" => MathExpressionKind::Evaluation(self.evaluation(node)?),
            "parens" => MathExpressionKind::Grouping(self.grouping(node)?),
            _ => return Err(Failure::Unsupported),
        };
        Ok(MathExpression {
            kind,
            span: node.span,
        })
    }

    fn real(&self, node: &Node) -> Result<RealLiteral, Failure> {
        let lexeme = literal_text(node)?;
        let base = match node.attribute("base").unwrap_or("10") {
            "2" => NumericBase::Binary,
            "8" => NumericBase::Octal,
            "10" => NumericBase::Decimal,
            "16" => NumericBase::Hexadecimal,
            _ => return Err(MathAstError::InvalidRadix.into()),
        };
        if !valid_real(&lexeme, base) {
            return Err(MathAstError::MalformedReal.into());
        }
        Ok(RealLiteral { lexeme, base })
    }

    fn identifier(&self, node: &Node) -> Result<Identifier, Failure> {
        let name = literal_text(node)?;
        if name.is_empty() {
            return Err(MathAstError::MalformedLiteral.into());
        }
        Ok(Identifier {
            name,
            subscript: node.attribute("subscript").map(str::to_owned),
        })
    }

    fn application(&mut self, node: &Node) -> Result<MathExpressionKind, Failure> {
        if has_non_whitespace_text(node) {
            return Err(MathAstError::MalformedLiteral.into());
        }
        let children: Vec<_> = node.element_children().collect();
        let Some(head) = children.first() else {
            return Err(Failure::Unsupported);
        };
        if head.name.namespace_uri.as_deref() != Some(MATH_NS) {
            return Err(Failure::Unsupported);
        }
        let kind = match head.name.local_name.as_str() {
            "plus" | "minus" | "mult" | "div" | "pow" => {
                MathExpressionKind::Binary(self.binary(node)?)
            }
            "absval" | "conjugate" | "factorial" | "neg" | "sqrt" | "transpose" | "vectorize"
            | "vectorSum" | "determinant" => MathExpressionKind::Unary(self.unary(node)?),
            "indexer" => MathExpressionKind::ArrayIndex(self.array_index(node)?),
            "not" => return Err(Failure::Unsupported),
            local if is_supported_expression_form(local) => {
                MathExpressionKind::FunctionCall(self.function_call(node)?)
            }
            _ => return Err(Failure::Unsupported),
        };
        Ok(kind)
    }

    fn binary(&mut self, node: &Node) -> Result<BinaryExpression, Failure> {
        let children: Vec<_> = node.element_children().collect();
        let Some(operator_node) = children.first() else {
            return Err(Failure::Unsupported);
        };
        if operator_node.name.namespace_uri.as_deref() != Some(MATH_NS) {
            return Err(Failure::Unsupported);
        }
        let operator = match operator_node.name.local_name.as_str() {
            "plus" => BinaryOperator::Add,
            "minus" => BinaryOperator::Subtract,
            "mult" => BinaryOperator::Multiply,
            "div" => BinaryOperator::Divide,
            "pow" => BinaryOperator::Power,
            _ => return Err(Failure::Unsupported),
        };
        let actual = children.len().saturating_sub(1);
        if actual != 2 {
            return Err(MathAstError::WrongBinaryArity { operator, actual }.into());
        }
        let left = self.expression(children[1])?;
        let right = self.expression(children[2])?;
        Ok(BinaryExpression {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn definition(
        &mut self,
        node: &Node,
        kind: DefinitionKind,
    ) -> Result<MathExpressionKind, Failure> {
        if has_non_whitespace_text(node) {
            return Err(MathAstError::InvalidDefinitionTarget.into());
        }
        let children: Vec<_> = node.element_children().collect();
        if children.len() != 2 {
            return Err(MathAstError::InvalidDefinitionTarget.into());
        }
        let style = definition_style(node.attribute("style"), kind)?;
        if children[0].name.namespace_uri.as_deref() == Some(MATH_NS)
            && children[0].name.local_name == "function"
        {
            if kind != DefinitionKind::Define {
                return Err(MathAstError::InvalidDefinitionTarget.into());
            }
            return Ok(MathExpressionKind::FunctionDefinition(
                self.function_definition(children[0], children[1], style)?,
            ));
        }
        let target = self.expression(children[0])?;
        if !matches!(target.kind, MathExpressionKind::Identifier(_)) {
            return Err(MathAstError::InvalidDefinitionTarget.into());
        }
        let value = self.expression(children[1])?;
        Ok(MathExpressionKind::Definition(Definition {
            kind,
            style,
            target: Box::new(target),
            value: Box::new(value),
        }))
    }

    fn evaluation(&mut self, node: &Node) -> Result<Evaluation, Failure> {
        if has_non_whitespace_text(node) {
            return Err(MathAstError::MalformedEvaluation.into());
        }
        let children: Vec<_> = node.element_children().collect();
        let Some(first) = children.first() else {
            return Err(MathAstError::MalformedEvaluation.into());
        };
        if first.name.namespace_uri.as_deref() == Some(MATH_NS)
            && matches!(first.name.local_name.as_str(), "unitOverride" | "result")
        {
            return Err(MathAstError::MalformedEvaluation.into());
        }
        let expression = self.expression(first)?;
        let mut unit_override = None;
        let mut saved_result = None;
        for wrapper in &children[1..] {
            if wrapper.name.namespace_uri.as_deref() != Some(MATH_NS) {
                return Err(MathAstError::MalformedEvaluation.into());
            }
            match wrapper.name.local_name.as_str() {
                "unitOverride" if unit_override.is_none() && saved_result.is_none() => {
                    unit_override = Some(Box::new(
                        self.single_wrapper_expression(wrapper, MathAstError::MalformedEvaluation)?,
                    ));
                }
                "result" if saved_result.is_none() => {
                    if has_non_whitespace_text(wrapper) {
                        return Err(MathAstError::MalformedEvaluation.into());
                    }
                    let results: Vec<_> = wrapper.element_children().collect();
                    if results.len() > 1 {
                        return Err(MathAstError::MalformedEvaluation.into());
                    }
                    saved_result = results
                        .first()
                        .map(|result| self.expression(result).map(Box::new))
                        .transpose()?;
                }
                _ => return Err(MathAstError::MalformedEvaluation.into()),
            }
        }
        Ok(Evaluation {
            expression: Box::new(expression),
            unit_override,
            saved_result,
        })
    }

    fn function_call(&mut self, node: &Node) -> Result<FunctionCall, Failure> {
        let children: Vec<_> = node.element_children().collect();
        let actual = children.len().saturating_sub(1);
        if actual < 1 {
            return Err(MathAstError::WrongFunctionArity { actual }.into());
        }
        let callee = self.expression(children[0])?;
        let arguments = children[1..]
            .iter()
            .map(|argument| self.expression(argument))
            .collect::<Result<_, _>>()?;
        Ok(FunctionCall {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn function_definition(
        &mut self,
        function: &Node,
        body: &Node,
        style: DefinitionStyle,
    ) -> Result<FunctionDefinition, Failure> {
        if has_non_whitespace_text(function) {
            return Err(MathAstError::MalformedFunctionDefinition.into());
        }
        let children: Vec<_> = function.element_children().collect();
        if children.len() != 2
            || !children[1].is(MATH_NS, "boundVars")
            || has_non_whitespace_text(children[1])
        {
            return Err(MathAstError::MalformedFunctionDefinition.into());
        }
        let name = self.expression(children[0])?;
        if !matches!(name.kind, MathExpressionKind::Identifier(_)) {
            return Err(MathAstError::InvalidFunctionName.into());
        }
        let parameter_nodes: Vec<_> = children[1].element_children().collect();
        if parameter_nodes.is_empty() {
            return Err(MathAstError::InvalidFunctionParameter.into());
        }
        let mut parameters = Vec::with_capacity(parameter_nodes.len());
        for parameter in parameter_nodes {
            let parameter = self.expression(parameter)?;
            if !matches!(parameter.kind, MathExpressionKind::Identifier(_)) {
                return Err(MathAstError::InvalidFunctionParameter.into());
            }
            parameters.push(parameter);
        }
        Ok(FunctionDefinition {
            style,
            name: Box::new(name),
            parameters,
            body: Box::new(self.expression(body)?),
        })
    }

    fn unary(&mut self, node: &Node) -> Result<UnaryExpression, Failure> {
        let children: Vec<_> = node.element_children().collect();
        let operator = match children[0].name.local_name.as_str() {
            "absval" => UnaryOperator::AbsoluteValue,
            "conjugate" => UnaryOperator::Conjugate,
            "factorial" => UnaryOperator::Factorial,
            "neg" => UnaryOperator::Negate,
            "sqrt" => UnaryOperator::SquareRoot,
            "transpose" => UnaryOperator::Transpose,
            "vectorize" => UnaryOperator::Vectorize,
            "vectorSum" => UnaryOperator::VectorSum,
            "determinant" => UnaryOperator::Determinant,
            _ => return Err(Failure::Unsupported),
        };
        let actual = children.len().saturating_sub(1);
        if actual != 1 {
            return Err(MathAstError::WrongUnaryArity { operator, actual }.into());
        }
        Ok(UnaryExpression {
            operator,
            operand: Box::new(self.expression(children[1])?),
        })
    }

    fn grouping(&mut self, node: &Node) -> Result<Grouping, Failure> {
        if has_non_whitespace_text(node) {
            return Err(MathAstError::MalformedGrouping.into());
        }
        let children: Vec<_> = node.element_children().collect();
        if children.len() != 1 {
            return Err(MathAstError::MalformedGrouping.into());
        }
        let unpaired = strict_boolean(node.attribute("unpaired").unwrap_or("false"))?;
        Ok(Grouping {
            expression: Box::new(self.expression(children[0])?),
            unpaired,
        })
    }

    fn array_index(&mut self, node: &Node) -> Result<ArrayIndex, Failure> {
        let children: Vec<_> = node.element_children().collect();
        let actual = children.len().saturating_sub(1);
        if actual != 2 {
            return Err(MathAstError::WrongArrayIndexArity { actual }.into());
        }
        let target = self.expression(children[1])?;
        let index = children[2];
        let indices = if index.is(MATH_NS, "sequence") {
            if has_non_whitespace_text(index) {
                return Err(MathAstError::MalformedArrayIndex.into());
            }
            let nodes: Vec<_> = index.element_children().collect();
            if nodes.is_empty() {
                return Err(MathAstError::MalformedArrayIndex.into());
            }
            nodes
                .iter()
                .map(|index| self.expression(index))
                .collect::<Result<_, _>>()?
        } else {
            vec![self.expression(index)?]
        };
        Ok(ArrayIndex {
            target: Box::new(target),
            indices,
        })
    }

    fn single_wrapper_expression(
        &mut self,
        wrapper: &Node,
        error: MathAstError,
    ) -> Result<MathExpression, Failure> {
        if has_non_whitespace_text(wrapper) {
            return Err(error.into());
        }
        let children: Vec<_> = wrapper.element_children().collect();
        if children.len() != 1 {
            return Err(error.into());
        }
        self.expression(children[0])
    }
}

fn definition_style(value: Option<&str>, kind: DefinitionKind) -> Result<DefinitionStyle, Failure> {
    let value = value.unwrap_or("default");
    let style = match (kind, value) {
        (_, "default") => DefinitionStyle::Default,
        (DefinitionKind::Define, "colon-equal") => DefinitionStyle::ColonEqual,
        (
            DefinitionKind::Define | DefinitionKind::GlobalDefine | DefinitionKind::LocalDefine,
            "equal",
        ) => DefinitionStyle::Equal,
        (DefinitionKind::GlobalDefine, "triple-equal") => DefinitionStyle::TripleEqual,
        (DefinitionKind::LocalDefine, "left-arrow") => DefinitionStyle::LeftArrow,
        _ => return Err(MathAstError::InvalidDefinitionStyle.into()),
    };
    Ok(style)
}

fn strict_boolean(value: &str) -> Result<bool, Failure> {
    match value {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(MathAstError::InvalidBooleanAttribute.into()),
    }
}

fn has_non_whitespace_text(node: &Node) -> bool {
    node.children
        .iter()
        .any(|child| matches!(child, Child::Text { value, .. } if !value.trim().is_empty()))
}

fn is_supported_expression_form(local: &str) -> bool {
    matches!(
        local,
        "real" | "id" | "apply" | "define" | "globalDefine" | "localDefine" | "eval" | "parens"
    )
}

fn literal_text(node: &Node) -> Result<String, Failure> {
    let mut output = String::new();
    for child in &node.children {
        match child {
            Child::Text { value, .. } => output.push_str(value),
            Child::Node(_) => return Err(MathAstError::MalformedLiteral.into()),
        }
    }
    if output.is_empty() {
        return Err(MathAstError::MalformedLiteral.into());
    }
    Ok(output)
}

fn valid_real(lexeme: &str, base: NumericBase) -> bool {
    let unsigned = lexeme.strip_prefix(['+', '-']).unwrap_or(lexeme);
    if unsigned.is_empty() {
        return false;
    }
    if base == NumericBase::Decimal {
        return valid_decimal(unsigned);
    }
    let radix = u32::from(base.value());
    let mut digits = 0_usize;
    let mut dot_seen = false;
    for character in unsigned.chars() {
        if character == '.' && !dot_seen {
            dot_seen = true;
        } else if character.is_digit(radix) {
            digits += 1;
        } else {
            return false;
        }
    }
    digits > 0
}

fn valid_decimal(unsigned: &str) -> bool {
    let mut exponent_parts = unsigned.split(['e', 'E']);
    let mantissa = exponent_parts.next().unwrap_or_default();
    let exponent = exponent_parts.next();
    if exponent_parts.next().is_some() {
        return false;
    }
    if let Some(exponent) = exponent {
        let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        if exponent.is_empty() || !exponent.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
    }
    let mut digits = 0_usize;
    let mut dot_seen = false;
    for character in mantissa.chars() {
        if character == '.' && !dot_seen {
            dot_seen = true;
        } else if character.is_ascii_digit() {
            digits += 1;
        } else {
            return false;
        }
    }
    digits > 0
}
