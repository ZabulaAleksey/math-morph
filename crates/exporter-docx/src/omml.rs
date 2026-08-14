use std::fmt;

use document_ir::ports::EquationExporter;
use math_model::{
    BinaryExpression, BinaryOperator, MathExpression, MathExpressionKind, MultiplicationStyle,
    NumericBase,
};

use crate::xml::is_xml_10_char;
use crate::{OmmlError, OmmlLimit};

const OFFICE_MATH_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OmmlLimits {
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_output_bytes: u64,
}

impl Default for OmmlLimits {
    fn default() -> Self {
        Self {
            max_depth: 256,
            max_nodes: 100_000,
            max_output_bytes: 4 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct OmmlFragment {
    xml: String,
}

impl OmmlFragment {
    pub fn as_str(&self) -> &str {
        &self.xml
    }

    pub fn byte_len(&self) -> usize {
        self.xml.len()
    }
}

impl fmt::Debug for OmmlFragment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OmmlFragment")
            .field("byte_len", &self.xml.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WordEquationExporter {
    limits: OmmlLimits,
}

impl WordEquationExporter {
    pub const fn new(limits: OmmlLimits) -> Self {
        Self { limits }
    }

    pub const fn limits(&self) -> &OmmlLimits {
        &self.limits
    }

    pub fn export_expression(
        &self,
        expression: &MathExpression,
    ) -> Result<OmmlFragment, OmmlError> {
        let mut renderer = Renderer::new(self.limits);
        renderer.push("<m:oMath xmlns:m=\"")?;
        renderer.push(OFFICE_MATH_NS)?;
        renderer.push("\">")?;
        renderer.expression(expression, 0)?;
        renderer.push("</m:oMath>")?;
        Ok(OmmlFragment {
            xml: renderer.output,
        })
    }
}

impl Default for WordEquationExporter {
    fn default() -> Self {
        Self::new(OmmlLimits::default())
    }
}

impl EquationExporter for WordEquationExporter {
    type Output = OmmlFragment;
    type Error = OmmlError;

    fn export(&self, expression: &MathExpression) -> Result<Self::Output, Self::Error> {
        self.export_expression(expression)
    }
}

struct Renderer {
    limits: OmmlLimits,
    nodes: usize,
    output: String,
}

impl Renderer {
    fn new(limits: OmmlLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            output: String::new(),
        }
    }

    fn expression(&mut self, expression: &MathExpression, depth: usize) -> Result<(), OmmlError> {
        if depth > self.limits.max_depth {
            return Err(OmmlError::LimitExceeded(OmmlLimit::Depth));
        }
        self.nodes = self
            .nodes
            .checked_add(1)
            .ok_or(OmmlError::LimitExceeded(OmmlLimit::Nodes))?;
        if self.nodes > self.limits.max_nodes {
            return Err(OmmlError::LimitExceeded(OmmlLimit::Nodes));
        }
        match &expression.kind {
            MathExpressionKind::Real(literal) => {
                if !valid_real(&literal.lexeme, literal.base) {
                    return Err(OmmlError::InvalidLiteral);
                }
                self.run(&literal.lexeme, false)
            }
            MathExpressionKind::Identifier(identifier) => {
                if identifier.subscript.is_some() {
                    return Err(OmmlError::IdentifierSubscriptUnsupported);
                }
                if identifier.name.is_empty() {
                    return Err(OmmlError::InvalidLiteral);
                }
                self.run(&identifier.name, true)
            }
            MathExpressionKind::Binary(binary) if binary.operator != BinaryOperator::Power => {
                self.binary(binary, depth)
            }
            _ => Err(OmmlError::UnsupportedExpression),
        }
    }

    fn binary(&mut self, binary: &BinaryExpression, depth: usize) -> Result<(), OmmlError> {
        match binary.operator {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Divide
                if binary.multiplication_style.is_some() =>
            {
                Err(OmmlError::InvalidExpression)
            }
            BinaryOperator::Add => {
                self.linear_operand(&binary.left, binary.operator, Side::Left, depth)?;
                self.run("+", false)?;
                self.linear_operand(&binary.right, binary.operator, Side::Right, depth)
            }
            BinaryOperator::Subtract => {
                self.linear_operand(&binary.left, binary.operator, Side::Left, depth)?;
                self.run("−", false)?;
                self.linear_operand(&binary.right, binary.operator, Side::Right, depth)
            }
            BinaryOperator::Multiply => {
                let style = binary
                    .multiplication_style
                    .ok_or(OmmlError::InvalidExpression)?;
                self.linear_operand(&binary.left, binary.operator, Side::Left, depth)?;
                self.multiplication_operator(style)?;
                self.linear_operand(&binary.right, binary.operator, Side::Right, depth)
            }
            BinaryOperator::Divide => {
                let next = depth
                    .checked_add(1)
                    .ok_or(OmmlError::LimitExceeded(OmmlLimit::Depth))?;
                self.push("<m:f><m:fPr><m:type m:val=\"bar\"/></m:fPr><m:num>")?;
                self.expression(&binary.left, next)?;
                self.push("</m:num><m:den>")?;
                self.expression(&binary.right, next)?;
                self.push("</m:den></m:f>")
            }
            BinaryOperator::Power => Err(OmmlError::UnsupportedExpression),
        }
    }

    fn linear_operand(
        &mut self,
        expression: &MathExpression,
        parent: BinaryOperator,
        side: Side,
        depth: usize,
    ) -> Result<(), OmmlError> {
        if needs_grouping(expression, parent, side) {
            return Err(OmmlError::SemanticGroupingRequired);
        }
        let next = depth
            .checked_add(1)
            .ok_or(OmmlError::LimitExceeded(OmmlLimit::Depth))?;
        self.expression(expression, next)
    }

    fn multiplication_operator(&mut self, style: MultiplicationStyle) -> Result<(), OmmlError> {
        match style {
            MultiplicationStyle::Default
            | MultiplicationStyle::AutoSelect
            | MultiplicationStyle::Dot
            | MultiplicationStyle::NarrowDot
            | MultiplicationStyle::LargeDot => self.run("·", false),
            MultiplicationStyle::X => self.run("×", false),
            MultiplicationStyle::ThinSpace => {
                self.push("<m:r><m:t xml:space=\"preserve\"> </m:t></m:r>")
            }
            MultiplicationStyle::NoSpace => Ok(()),
        }
    }

    fn run(&mut self, text: &str, italic: bool) -> Result<(), OmmlError> {
        self.push("<m:r>")?;
        if italic {
            self.push("<m:rPr><m:sty m:val=\"i\"/></m:rPr>")?;
        }
        self.push("<m:t")?;
        if needs_preserved_space(text) {
            self.push(" xml:space=\"preserve\"")?;
        }
        self.push(">")?;
        self.escaped_text(text)?;
        self.push("</m:t></m:r>")
    }

    fn escaped_text(&mut self, value: &str) -> Result<(), OmmlError> {
        if !value.chars().all(is_xml_10_char) {
            return Err(OmmlError::InvalidXmlText);
        }
        for character in value.chars() {
            match character {
                '&' => self.push("&amp;")?,
                '<' => self.push("&lt;")?,
                '>' => self.push("&gt;")?,
                _ => self.push_character(character)?,
            }
        }
        Ok(())
    }

    fn push(&mut self, value: &str) -> Result<(), OmmlError> {
        let next = self
            .output
            .len()
            .checked_add(value.len())
            .ok_or(OmmlError::LimitExceeded(OmmlLimit::OutputBytes))?;
        if u64::try_from(next).unwrap_or(u64::MAX) > self.limits.max_output_bytes {
            return Err(OmmlError::LimitExceeded(OmmlLimit::OutputBytes));
        }
        self.output.push_str(value);
        Ok(())
    }

    fn push_character(&mut self, character: char) -> Result<(), OmmlError> {
        let mut buffer = [0_u8; 4];
        self.push(character.encode_utf8(&mut buffer))
    }
}

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

fn needs_grouping(expression: &MathExpression, parent: BinaryOperator, side: Side) -> bool {
    let MathExpressionKind::Binary(child) = &expression.kind else {
        return matches!(expression.kind, MathExpressionKind::Grouping(_));
    };
    match parent {
        BinaryOperator::Multiply => {
            matches!(
                child.operator,
                BinaryOperator::Add | BinaryOperator::Subtract
            )
        }
        BinaryOperator::Subtract if matches!(side, Side::Right) => {
            matches!(
                child.operator,
                BinaryOperator::Add | BinaryOperator::Subtract
            )
        }
        BinaryOperator::Add | BinaryOperator::Subtract => false,
        BinaryOperator::Divide => false,
        BinaryOperator::Power => true,
    }
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

fn needs_preserved_space(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
        || value.contains("  ")
        || value.contains(['\t', '\n', '\r'])
}
