use crate::ast::{
    BinaryExpression, BinaryOperator, Identifier, MathAstError, MathExpression, MathExpressionKind,
    NumericBase, RealLiteral,
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
            "apply" => MathExpressionKind::Binary(self.binary(node)?),
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

    fn binary(&mut self, node: &Node) -> Result<BinaryExpression, Failure> {
        if node
            .children
            .iter()
            .any(|child| matches!(child, Child::Text { value, .. } if !value.trim().is_empty()))
        {
            return Err(MathAstError::MalformedLiteral.into());
        }
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
