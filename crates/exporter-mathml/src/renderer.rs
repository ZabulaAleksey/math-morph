use std::fmt;

use document_ir::ports::EquationExporter;
use math_model::{
    BinaryExpression, BinaryOperator, Grouping, Identifier, MathExpression, MathExpressionKind,
    MultiplicationStyle, NumericBase, UnaryExpression, UnaryOperator,
};

use crate::{MathMlError, MathMlLimit};

const ROOT_START: &str = "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">";
const ROOT_END: &str = "</math>";

/// Caller-configurable bounds for one MathML fragment.
///
/// Defaults are depth `256`, nodes `100_000`, and output `4 MiB`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MathMlLimits {
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_output_bytes: u64,
}

impl Default for MathMlLimits {
    fn default() -> Self {
        Self {
            max_depth: 256,
            max_nodes: 100_000,
            max_output_bytes: 4 * 1024 * 1024,
        }
    }
}

/// Opaque MathML produced by [`MathMlRenderer`].
#[derive(Clone, Eq, PartialEq)]
pub struct MathMlFragment {
    xml: String,
}

impl MathMlFragment {
    pub fn as_str(&self) -> &str {
        &self.xml
    }

    pub fn byte_len(&self) -> usize {
        self.xml.len()
    }
}

impl fmt::Debug for MathMlFragment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MathMlFragment")
            .field("byte_len", &self.xml.len())
            .finish_non_exhaustive()
    }
}

/// Backend-neutral Presentation MathML renderer for the stage 090 scalar subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MathMlRenderer {
    limits: MathMlLimits,
}

impl MathMlRenderer {
    pub const fn new(limits: MathMlLimits) -> Self {
        Self { limits }
    }

    pub const fn limits(&self) -> &MathMlLimits {
        &self.limits
    }

    /// Renders an expression that was already obtained through a bounded construction or
    /// deserialization boundary.
    ///
    /// This method bounds its own traversal and output work. Because it borrows the expression,
    /// ownership and stack-safe teardown of a caller-constructed recursive AST remain the
    /// caller's responsibility.
    pub fn export_expression(
        &self,
        expression: &MathExpression,
    ) -> Result<MathMlFragment, MathMlError> {
        Accountant::new(self.limits).validate(expression)?;

        let mut renderer = Renderer::new(self.limits);
        renderer.push(ROOT_START)?;
        renderer.render(expression)?;
        renderer.push(ROOT_END)?;
        Ok(MathMlFragment {
            xml: renderer.output,
        })
    }
}

impl Default for MathMlRenderer {
    fn default() -> Self {
        Self::new(MathMlLimits::default())
    }
}

impl EquationExporter for MathMlRenderer {
    type Output = MathMlFragment;
    type Error = MathMlError;

    fn export(&self, expression: &MathExpression) -> Result<Self::Output, Self::Error> {
        self.export_expression(expression)
    }
}

struct Accountant {
    limits: MathMlLimits,
    nodes: usize,
    input_bytes: u64,
}

impl Accountant {
    fn new(limits: MathMlLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            input_bytes: 0,
        }
    }

    fn validate(mut self, expression: &MathExpression) -> Result<(), MathMlError> {
        let mut pending = vec![(expression, 0_usize)];
        while let Some((current, depth)) = pending.pop() {
            self.node(depth)?;
            match &current.kind {
                MathExpressionKind::Real(value) => {
                    self.text(&value.lexeme)?;
                    if !valid_real(&value.lexeme, value.base) {
                        return Err(MathMlError::InvalidLiteral);
                    }
                }
                MathExpressionKind::Identifier(value) => {
                    self.text(&value.name)?;
                    if let Some(subscript) = &value.subscript {
                        self.text(subscript)?;
                    }
                    validate_identifier(value)?;
                }
                MathExpressionKind::Binary(value) => {
                    validate_binary(value)?;
                    let child_depth = next_depth(depth)?;
                    pending.push((&value.right, child_depth));
                    pending.push((&value.left, child_depth));
                }
                MathExpressionKind::Unary(value) => {
                    if value.operator != UnaryOperator::SquareRoot {
                        return Err(MathMlError::UnsupportedExpression);
                    }
                    pending.push((&value.operand, next_depth(depth)?));
                }
                MathExpressionKind::Grouping(value) => {
                    validate_grouping(value)?;
                    pending.push((&value.expression, next_depth(depth)?));
                }
                _ => return Err(MathMlError::UnsupportedExpression),
            }
        }
        Ok(())
    }

    fn text(&mut self, value: &str) -> Result<(), MathMlError> {
        let bytes = u64::try_from(value.len())
            .map_err(|_| MathMlError::LimitExceeded(MathMlLimit::OutputBytes))?;
        self.input_bytes = self
            .input_bytes
            .checked_add(bytes)
            .ok_or(MathMlError::LimitExceeded(MathMlLimit::OutputBytes))?;
        if self.input_bytes > self.limits.max_output_bytes {
            return Err(MathMlError::LimitExceeded(MathMlLimit::OutputBytes));
        }
        Ok(())
    }

    fn node(&mut self, depth: usize) -> Result<(), MathMlError> {
        if depth > self.limits.max_depth {
            return Err(MathMlError::LimitExceeded(MathMlLimit::Depth));
        }
        self.nodes = self
            .nodes
            .checked_add(1)
            .ok_or(MathMlError::LimitExceeded(MathMlLimit::Nodes))?;
        if self.nodes > self.limits.max_nodes {
            return Err(MathMlError::LimitExceeded(MathMlLimit::Nodes));
        }
        Ok(())
    }
}

struct Renderer {
    limits: MathMlLimits,
    output: String,
}

impl Renderer {
    fn new(limits: MathMlLimits) -> Self {
        Self {
            limits,
            output: String::new(),
        }
    }

    fn render(&mut self, expression: &MathExpression) -> Result<(), MathMlError> {
        let mut pending = vec![RenderItem::Expression(expression)];
        while let Some(item) = pending.pop() {
            match item {
                RenderItem::Static(value) => self.push(value)?,
                RenderItem::Text(value) => self.push_escaped(value)?,
                RenderItem::Expression(current) => self.expression(current, &mut pending)?,
            }
        }
        Ok(())
    }

    fn expression<'a>(
        &mut self,
        expression: &'a MathExpression,
        pending: &mut Vec<RenderItem<'a>>,
    ) -> Result<(), MathMlError> {
        match &expression.kind {
            MathExpressionKind::Real(value) => pending.extend([
                RenderItem::Static("</mn>"),
                RenderItem::Text(&value.lexeme),
                RenderItem::Static("<mn>"),
            ]),
            MathExpressionKind::Identifier(value) => push_identifier(value, pending),
            MathExpressionKind::Binary(value) => push_binary(value, pending)?,
            MathExpressionKind::Unary(value) => push_square_root(value, pending)?,
            MathExpressionKind::Grouping(value) => push_grouping(value, pending)?,
            _ => return Err(MathMlError::UnsupportedExpression),
        }
        Ok(())
    }

    fn push_escaped(&mut self, value: &str) -> Result<(), MathMlError> {
        for character in value.chars() {
            match character {
                '&' => self.push("&amp;")?,
                '<' => self.push("&lt;")?,
                '>' => self.push("&gt;")?,
                _ => {
                    let mut buffer = [0_u8; 4];
                    self.push(character.encode_utf8(&mut buffer))?;
                }
            }
        }
        Ok(())
    }

    fn push(&mut self, value: &str) -> Result<(), MathMlError> {
        let next = self
            .output
            .len()
            .checked_add(value.len())
            .ok_or(MathMlError::LimitExceeded(MathMlLimit::OutputBytes))?;
        if u64::try_from(next).unwrap_or(u64::MAX) > self.limits.max_output_bytes {
            return Err(MathMlError::LimitExceeded(MathMlLimit::OutputBytes));
        }
        self.output.push_str(value);
        Ok(())
    }
}

enum RenderItem<'a> {
    Static(&'static str),
    Text(&'a str),
    Expression(&'a MathExpression),
}

fn push_identifier<'a>(identifier: &'a Identifier, pending: &mut Vec<RenderItem<'a>>) {
    match &identifier.subscript {
        None => pending.extend([
            RenderItem::Static("</mi>"),
            RenderItem::Text(&identifier.name),
            RenderItem::Static("<mi>"),
        ]),
        Some(subscript) => pending.extend([
            RenderItem::Static("</msub>"),
            RenderItem::Static("</mi>"),
            RenderItem::Text(subscript),
            RenderItem::Static("<mi>"),
            RenderItem::Static("</mi>"),
            RenderItem::Text(&identifier.name),
            RenderItem::Static("<mi>"),
            RenderItem::Static("<msub>"),
        ]),
    }
}

fn push_binary<'a>(
    binary: &'a BinaryExpression,
    pending: &mut Vec<RenderItem<'a>>,
) -> Result<(), MathMlError> {
    let (start, middle, end) = match binary.operator {
        BinaryOperator::Add => ("<mrow>", "<mo>+</mo>", "</mrow>"),
        BinaryOperator::Subtract => ("<mrow>", "<mo>&#x2212;</mo>", "</mrow>"),
        BinaryOperator::Multiply => ("<mrow>", multiplication_token(binary)?, "</mrow>"),
        BinaryOperator::Divide => ("<mfrac>", "", "</mfrac>"),
        BinaryOperator::Power => ("<msup>", "", "</msup>"),
    };
    pending.push(RenderItem::Static(end));
    pending.push(RenderItem::Expression(&binary.right));
    if !middle.is_empty() {
        pending.push(RenderItem::Static(middle));
    }
    pending.push(RenderItem::Expression(&binary.left));
    pending.push(RenderItem::Static(start));
    Ok(())
}

fn push_square_root<'a>(
    unary: &'a UnaryExpression,
    pending: &mut Vec<RenderItem<'a>>,
) -> Result<(), MathMlError> {
    if unary.operator != UnaryOperator::SquareRoot {
        return Err(MathMlError::UnsupportedExpression);
    }
    pending.extend([
        RenderItem::Static("</msqrt>"),
        RenderItem::Expression(&unary.operand),
        RenderItem::Static("<msqrt>"),
    ]);
    Ok(())
}

fn push_grouping<'a>(
    grouping: &'a Grouping,
    pending: &mut Vec<RenderItem<'a>>,
) -> Result<(), MathMlError> {
    if grouping.unpaired {
        return Err(MathMlError::InvalidExpression);
    }
    pending.extend([
        RenderItem::Static("</mrow>"),
        RenderItem::Static("<mo fence=\"true\">)</mo>"),
        RenderItem::Expression(&grouping.expression),
        RenderItem::Static("<mo fence=\"true\">(</mo>"),
        RenderItem::Static("<mrow>"),
    ]);
    Ok(())
}

fn validate_identifier(identifier: &Identifier) -> Result<(), MathMlError> {
    if identifier.name.is_empty() {
        return Err(MathMlError::InvalidLiteral);
    }
    if !identifier.name.chars().all(is_xml_10_char) {
        return Err(MathMlError::InvalidXmlText);
    }
    if let Some(subscript) = &identifier.subscript {
        if subscript.is_empty() {
            return Err(MathMlError::InvalidExpression);
        }
        if !subscript.chars().all(is_xml_10_char) {
            return Err(MathMlError::InvalidXmlText);
        }
    }
    Ok(())
}

fn validate_binary(binary: &BinaryExpression) -> Result<(), MathMlError> {
    match binary.operator {
        BinaryOperator::Multiply if binary.multiplication_style.is_some() => Ok(()),
        BinaryOperator::Multiply => Err(MathMlError::InvalidExpression),
        _ if binary.multiplication_style.is_none() => Ok(()),
        _ => Err(MathMlError::InvalidExpression),
    }
}

fn validate_grouping(grouping: &Grouping) -> Result<(), MathMlError> {
    if grouping.unpaired {
        Err(MathMlError::InvalidExpression)
    } else {
        Ok(())
    }
}

fn multiplication_token(binary: &BinaryExpression) -> Result<&'static str, MathMlError> {
    match binary.multiplication_style {
        Some(
            MultiplicationStyle::Default
            | MultiplicationStyle::AutoSelect
            | MultiplicationStyle::Dot
            | MultiplicationStyle::NarrowDot
            | MultiplicationStyle::LargeDot,
        ) => Ok("<mo>&#x00B7;</mo>"),
        Some(MultiplicationStyle::X) => Ok("<mo>&#x00D7;</mo>"),
        Some(MultiplicationStyle::ThinSpace) => Ok("<mo>&#x2009;</mo>"),
        Some(MultiplicationStyle::NoSpace) => Ok(""),
        None => Err(MathMlError::InvalidExpression),
    }
}

fn next_depth(depth: usize) -> Result<usize, MathMlError> {
    depth
        .checked_add(1)
        .ok_or(MathMlError::LimitExceeded(MathMlLimit::Depth))
}

fn is_xml_10_char(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
        || ('\u{20}'..='\u{D7FF}').contains(&character)
        || ('\u{E000}'..='\u{FFFD}').contains(&character)
        || ('\u{10000}'..='\u{10FFFF}').contains(&character)
}

fn valid_real(lexeme: &str, base: NumericBase) -> bool {
    let unsigned = lexeme.strip_prefix(['+', '-']).unwrap_or(lexeme);
    if unsigned.is_empty() {
        return false;
    }
    if base == NumericBase::Decimal {
        return valid_decimal(unsigned);
    }
    let mut has_digit = false;
    let mut dot = false;
    for character in unsigned.chars() {
        if character == '.' && !dot {
            dot = true;
        } else if character.is_digit(u32::from(base.value())) {
            has_digit = true;
        } else {
            return false;
        }
    }
    has_digit
}

fn valid_decimal(value: &str) -> bool {
    let mut parts = value.split(['e', 'E']);
    let mantissa = parts.next().unwrap_or_default();
    let exponent = parts.next();
    if parts.next().is_some() {
        return false;
    }
    if let Some(exponent) = exponent {
        let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        if exponent.is_empty() || !exponent.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
    }
    let mut has_digit = false;
    let mut dot = false;
    for character in mantissa.chars() {
        if character == '.' && !dot {
            dot = true;
        } else if character.is_ascii_digit() {
            has_digit = true;
        } else {
            return false;
        }
    }
    has_digit
}
