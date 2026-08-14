use std::fmt;

use document_ir::ports::EquationExporter;
use math_model::{
    AggregateExpression, AggregateOperator, BinaryExpression, BinaryOperator, Derivative,
    DerivativeStyle, FunctionCall, Grouping, Identifier, Integral, MathExpression,
    MathExpressionKind, Matrix, MultiplicationStyle, NumericBase, UnaryExpression, UnaryOperator,
    Vector, VectorOrientation,
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OmmlFragment")
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
        Accountant::new(self.limits).expression(expression, 0)?;
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

/// Applies the OMML limits to the same content-bearing nodes as the validator.
/// Property and container nodes are excluded; runs and primary mathematical
/// constructs are counted at their emitted depth.
struct Accountant {
    limits: OmmlLimits,
    nodes: usize,
    linear_work_items: usize,
}

impl Accountant {
    fn new(limits: OmmlLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            linear_work_items: 0,
        }
    }

    fn expression(&mut self, expression: &MathExpression, depth: usize) -> Result<(), OmmlError> {
        match &expression.kind {
            MathExpressionKind::Real(value) => {
                if !valid_real(&value.lexeme, value.base) {
                    return Err(OmmlError::InvalidLiteral);
                }
                self.node(depth)
            }
            MathExpressionKind::Identifier(value) => self.identifier(value, depth),
            MathExpressionKind::Binary(value) if value.operator != BinaryOperator::Power => {
                self.binary(value, depth)
            }
            MathExpressionKind::Binary(value) => self.power(value, depth),
            MathExpressionKind::Unary(value) => self.unary(value, depth),
            MathExpressionKind::FunctionCall(value) => self.function(value, depth),
            MathExpressionKind::Grouping(value) => self.grouping(value, depth),
            MathExpressionKind::Matrix(value) => self.matrix(value, depth),
            MathExpressionKind::Vector(value) => self.vector(value, depth),
            MathExpressionKind::Integral(value) => self.integral(value, depth),
            MathExpressionKind::Derivative(value) => self.derivative(value, depth),
            MathExpressionKind::Aggregate(value) => self.aggregate(value, depth),
            _ => Err(OmmlError::UnsupportedExpression),
        }
    }

    fn node(&mut self, depth: usize) -> Result<(), OmmlError> {
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
        Ok(())
    }

    fn child_depth(&self, depth: usize) -> Result<usize, OmmlError> {
        depth
            .checked_add(1)
            .ok_or(OmmlError::LimitExceeded(OmmlLimit::Depth))
    }

    fn charge_linear_work(&mut self) -> Result<(), OmmlError> {
        self.linear_work_items = self
            .linear_work_items
            .checked_add(1)
            .ok_or(OmmlError::LimitExceeded(OmmlLimit::Nodes))?;
        if self.linear_work_items > self.limits.max_nodes {
            Err(OmmlError::LimitExceeded(OmmlLimit::Nodes))
        } else {
            Ok(())
        }
    }

    fn identifier(&mut self, identifier: &Identifier, depth: usize) -> Result<(), OmmlError> {
        if identifier.name.is_empty() {
            return Err(OmmlError::InvalidLiteral);
        }
        match &identifier.subscript {
            None => self.node(depth),
            Some(value) if !value.is_empty() => {
                self.node(depth)?;
                let child = self.child_depth(depth)?;
                self.node(child)?;
                self.node(child)
            }
            Some(_) => Err(OmmlError::InvalidExpression),
        }
    }

    fn binary(&mut self, binary: &BinaryExpression, depth: usize) -> Result<(), OmmlError> {
        match binary.operator {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply => {
                self.linear_binary(binary, depth)
            }
            BinaryOperator::Divide => {
                if binary.multiplication_style.is_some() {
                    return Err(OmmlError::InvalidExpression);
                }
                self.node(depth)?;
                let child = self.child_depth(depth)?;
                self.expression(&binary.left, child)?;
                self.expression(&binary.right, child)
            }
            BinaryOperator::Power => Err(OmmlError::UnsupportedExpression),
        }
    }

    fn linear_binary(&mut self, binary: &BinaryExpression, depth: usize) -> Result<(), OmmlError> {
        let mut stack = vec![LinearItem::Binary(binary)];
        while let Some(item) = stack.pop() {
            match item {
                LinearItem::Binary(current) => {
                    self.charge_linear_work()?;
                    let operator = linear_operator(current)?;
                    if needs_grouping(&current.left, current.operator, Side::Left)
                        || needs_grouping(&current.right, current.operator, Side::Right)
                    {
                        return Err(OmmlError::SemanticGroupingRequired);
                    }
                    stack.push(LinearItem::Expression(&current.right));
                    stack.push(LinearItem::Operator(operator));
                    stack.push(LinearItem::Expression(&current.left));
                }
                LinearItem::Expression(current) => {
                    if let MathExpressionKind::Binary(child) = &current.kind
                        && is_linear_binary(child)
                    {
                        stack.push(LinearItem::Binary(child));
                    } else {
                        self.expression(current, depth)?;
                    }
                }
                LinearItem::Operator(operator) => {
                    if operator.emits_run() {
                        self.node(depth)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn power(&mut self, binary: &BinaryExpression, depth: usize) -> Result<(), OmmlError> {
        if binary.multiplication_style.is_some() {
            return Err(OmmlError::InvalidExpression);
        }
        self.node(depth)?;
        let child = self.child_depth(depth)?;
        if let MathExpressionKind::Identifier(identifier) = &binary.left.kind
            && let Some(subscript) = &identifier.subscript
        {
            if identifier.name.is_empty() || subscript.is_empty() {
                return Err(OmmlError::InvalidExpression);
            }
            self.node(child)?;
            self.node(child)?;
            return self.expression(&binary.right, child);
        }
        self.expression(&binary.left, child)?;
        self.expression(&binary.right, child)
    }

    fn unary(&mut self, unary: &UnaryExpression, depth: usize) -> Result<(), OmmlError> {
        if unary.operator != UnaryOperator::SquareRoot {
            return Err(OmmlError::UnsupportedExpression);
        }
        self.node(depth)?;
        self.expression(&unary.operand, self.child_depth(depth)?)
    }

    fn function(&mut self, call: &FunctionCall, depth: usize) -> Result<(), OmmlError> {
        let MathExpressionKind::Identifier(callee) = &call.callee.kind else {
            return Err(OmmlError::InvalidExpression);
        };
        if call.arguments.is_empty() {
            return Err(OmmlError::InvalidExpression);
        }
        self.node(depth)?;
        let child = self.child_depth(depth)?;
        self.identifier(callee, child)?;
        self.node(child)?;
        let argument_depth = self.child_depth(child)?;
        for argument in &call.arguments {
            self.expression(argument, argument_depth)?;
        }
        Ok(())
    }

    fn grouping(&mut self, grouping: &Grouping, depth: usize) -> Result<(), OmmlError> {
        if grouping.unpaired {
            return Err(OmmlError::InvalidExpression);
        }
        self.node(depth)?;
        self.expression(&grouping.expression, self.child_depth(depth)?)
    }

    fn matrix(&mut self, matrix: &Matrix, depth: usize) -> Result<(), OmmlError> {
        if matrix.rows == 0
            || matrix.columns == 0
            || matrix.rows.checked_mul(matrix.columns) != Some(matrix.elements.len())
        {
            return Err(OmmlError::InvalidExpression);
        }
        self.matrix_shape(matrix.rows, matrix.columns, &matrix.elements, depth)
    }

    fn vector(&mut self, vector: &Vector, depth: usize) -> Result<(), OmmlError> {
        if vector.elements.is_empty() {
            return Err(OmmlError::InvalidExpression);
        }
        let (rows, columns) = match vector.orientation {
            VectorOrientation::Row => (1, vector.elements.len()),
            VectorOrientation::Column => (vector.elements.len(), 1),
        };
        self.matrix_shape(rows, columns, &vector.elements, depth)
    }

    fn matrix_shape(
        &mut self,
        rows: usize,
        columns: usize,
        elements: &[MathExpression],
        depth: usize,
    ) -> Result<(), OmmlError> {
        self.node(depth)?;
        let matrix_depth = self.child_depth(depth)?;
        self.node(matrix_depth)?;
        let row_depth = self.child_depth(matrix_depth)?;
        let cell_depth = self.child_depth(row_depth)?;
        for row in elements.chunks_exact(columns).take(rows) {
            self.node(row_depth)?;
            for cell in row {
                self.expression(cell, cell_depth)?;
            }
        }
        Ok(())
    }

    fn integral(&mut self, integral: &Integral, depth: usize) -> Result<(), OmmlError> {
        let variable = identifier_ref(&integral.bound_variable)?;
        self.node(depth)?;
        let child = self.child_depth(depth)?;
        if let Some(bounds) = &integral.bounds {
            self.expression(&bounds.lower, child)?;
            self.expression(&bounds.upper, child)?;
        }
        self.expression(&integral.integrand, child)?;
        self.node(child)?;
        self.identifier(variable, child)
    }

    fn derivative(&mut self, derivative: &Derivative, depth: usize) -> Result<(), OmmlError> {
        let variable = identifier_ref(&derivative.bound_variable)?;
        self.node(depth)?;
        let child = self.child_depth(depth)?;
        if let Some(degree) = derivative.degree.as_deref() {
            self.node(child)?;
            let script_child = self.child_depth(child)?;
            self.node(script_child)?;
            self.expression(degree, script_child)?;
        } else {
            self.node(child)?;
        }
        self.expression(&derivative.expression, child)?;
        match (&variable.subscript, derivative.degree.as_deref()) {
            (_, None) => self.identifier(variable, child),
            (None, Some(degree)) => {
                self.node(child)?;
                let script_child = self.child_depth(child)?;
                self.node(script_child)?;
                self.expression(degree, script_child)
            }
            (Some(subscript), Some(degree)) if !subscript.is_empty() => {
                self.node(child)?;
                let script_child = self.child_depth(child)?;
                self.node(script_child)?;
                self.node(script_child)?;
                self.expression(degree, script_child)
            }
            (Some(_), Some(_)) => Err(OmmlError::InvalidExpression),
        }
    }

    fn aggregate(
        &mut self,
        aggregate: &AggregateExpression,
        depth: usize,
    ) -> Result<(), OmmlError> {
        let variable = identifier_ref(&aggregate.bound_variable)?;
        self.node(depth)?;
        let child = self.child_depth(depth)?;
        self.identifier(variable, child)?;
        if let Some(bounds) = &aggregate.bounds {
            self.node(child)?;
            self.expression(&bounds.lower, child)?;
            self.expression(&bounds.upper, child)?;
        }
        self.expression(&aggregate.body, child)
    }
}

struct Renderer {
    limits: OmmlLimits,
    linear_work_items: usize,
    output: String,
}
impl Renderer {
    fn new(limits: OmmlLimits) -> Self {
        Self {
            limits,
            linear_work_items: 0,
            output: String::new(),
        }
    }
    fn expression(&mut self, expression: &MathExpression, depth: usize) -> Result<(), OmmlError> {
        match &expression.kind {
            MathExpressionKind::Real(value) => {
                if !valid_real(&value.lexeme, value.base) {
                    return Err(OmmlError::InvalidLiteral);
                }
                self.run(&value.lexeme, false)
            }
            MathExpressionKind::Identifier(value) => self.identifier(value),
            MathExpressionKind::Binary(value) if value.operator != BinaryOperator::Power => {
                self.binary(value, depth)
            }
            MathExpressionKind::Binary(value) => self.power(value, depth),
            MathExpressionKind::Unary(value) => self.unary(value, depth),
            MathExpressionKind::FunctionCall(value) => self.function(value, depth),
            MathExpressionKind::Grouping(value) => self.grouping(value, depth),
            MathExpressionKind::Matrix(value) => self.matrix(value, depth),
            MathExpressionKind::Vector(value) => self.vector(value, depth),
            MathExpressionKind::Integral(value) => self.integral(value, depth),
            MathExpressionKind::Derivative(value) => self.derivative(value, depth),
            MathExpressionKind::Aggregate(value) => self.aggregate(value, depth),
            _ => Err(OmmlError::UnsupportedExpression),
        }
    }
    fn child_depth(&self, depth: usize) -> Result<usize, OmmlError> {
        depth
            .checked_add(1)
            .ok_or(OmmlError::LimitExceeded(OmmlLimit::Depth))
    }

    fn charge_linear_work(&mut self) -> Result<(), OmmlError> {
        self.linear_work_items = self
            .linear_work_items
            .checked_add(1)
            .ok_or(OmmlError::LimitExceeded(OmmlLimit::Nodes))?;
        if self.linear_work_items > self.limits.max_nodes {
            Err(OmmlError::LimitExceeded(OmmlLimit::Nodes))
        } else {
            Ok(())
        }
    }
    fn identifier(&mut self, identifier: &Identifier) -> Result<(), OmmlError> {
        if identifier.name.is_empty() {
            return Err(OmmlError::InvalidLiteral);
        }
        match &identifier.subscript {
            None => self.run(&identifier.name, true),
            Some(value) if !value.is_empty() => {
                self.push("<m:sSub><m:e>")?;
                self.bare_identifier(identifier)?;
                self.push("</m:e><m:sub>")?;
                self.run(value, false)?;
                self.push("</m:sub></m:sSub>")
            }
            Some(_) => Err(OmmlError::InvalidExpression),
        }
    }
    fn bare_identifier(&mut self, identifier: &Identifier) -> Result<(), OmmlError> {
        if identifier.name.is_empty() {
            Err(OmmlError::InvalidLiteral)
        } else {
            self.run(&identifier.name, true)
        }
    }
    fn binary(&mut self, binary: &BinaryExpression, depth: usize) -> Result<(), OmmlError> {
        match binary.operator {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply => {
                self.linear_binary(binary, depth)
            }
            BinaryOperator::Divide => {
                if binary.multiplication_style.is_some() {
                    return Err(OmmlError::InvalidExpression);
                }
                let next = self.child_depth(depth)?;
                self.push("<m:f><m:fPr><m:type m:val=\"bar\"/></m:fPr><m:num>")?;
                self.expression(&binary.left, next)?;
                self.push("</m:num><m:den>")?;
                self.expression(&binary.right, next)?;
                self.push("</m:den></m:f>")
            }
            BinaryOperator::Power => Err(OmmlError::UnsupportedExpression),
        }
    }

    fn linear_binary(&mut self, binary: &BinaryExpression, depth: usize) -> Result<(), OmmlError> {
        let mut stack = vec![LinearItem::Binary(binary)];
        while let Some(item) = stack.pop() {
            match item {
                LinearItem::Binary(current) => {
                    self.charge_linear_work()?;
                    let operator = linear_operator(current)?;
                    if needs_grouping(&current.left, current.operator, Side::Left)
                        || needs_grouping(&current.right, current.operator, Side::Right)
                    {
                        return Err(OmmlError::SemanticGroupingRequired);
                    }
                    stack.push(LinearItem::Expression(&current.right));
                    stack.push(LinearItem::Operator(operator));
                    stack.push(LinearItem::Expression(&current.left));
                }
                LinearItem::Expression(current) => {
                    if let MathExpressionKind::Binary(child) = &current.kind
                        && is_linear_binary(child)
                    {
                        stack.push(LinearItem::Binary(child));
                    } else {
                        self.expression(current, depth)?;
                    }
                }
                LinearItem::Operator(operator) => self.render_linear_operator(operator)?,
            }
        }
        Ok(())
    }
    fn power(&mut self, binary: &BinaryExpression, depth: usize) -> Result<(), OmmlError> {
        if binary.multiplication_style.is_some() {
            return Err(OmmlError::InvalidExpression);
        }
        let next = self.child_depth(depth)?;
        if let MathExpressionKind::Identifier(identifier) = &binary.left.kind
            && let Some(subscript) = &identifier.subscript
        {
            if identifier.name.is_empty() || subscript.is_empty() {
                return Err(OmmlError::InvalidExpression);
            }
            self.push("<m:sSubSup><m:e>")?;
            self.bare_identifier(identifier)?;
            self.push("</m:e><m:sub>")?;
            self.run(subscript, false)?;
            self.push("</m:sub><m:sup>")?;
            self.expression(&binary.right, next)?;
            return self.push("</m:sup></m:sSubSup>");
        }
        self.push("<m:sSup><m:e>")?;
        self.expression(&binary.left, next)?;
        self.push("</m:e><m:sup>")?;
        self.expression(&binary.right, next)?;
        self.push("</m:sup></m:sSup>")
    }
    fn unary(&mut self, unary: &UnaryExpression, depth: usize) -> Result<(), OmmlError> {
        if unary.operator != UnaryOperator::SquareRoot {
            return Err(OmmlError::UnsupportedExpression);
        }
        self.push("<m:rad><m:radPr><m:degHide m:val=\"1\"/></m:radPr><m:deg></m:deg><m:e>")?;
        self.expression(&unary.operand, self.child_depth(depth)?)?;
        self.push("</m:e></m:rad>")
    }
    fn function(&mut self, call: &FunctionCall, depth: usize) -> Result<(), OmmlError> {
        let MathExpressionKind::Identifier(callee) = &call.callee.kind else {
            return Err(OmmlError::InvalidExpression);
        };
        if call.arguments.is_empty() {
            return Err(OmmlError::InvalidExpression);
        }
        let next = self.child_depth(depth)?;
        self.push("<m:func><m:fName>")?;
        self.identifier(callee)?;
        self.push("</m:fName><m:e><m:d><m:dPr><m:begChr m:val=\"(\"/><m:sepChr m:val=\",\"/><m:endChr m:val=\")\"/></m:dPr>")?;
        for arg in &call.arguments {
            self.push("<m:e>")?;
            self.expression(arg, next)?;
            self.push("</m:e>")?;
        }
        self.push("</m:d></m:e></m:func>")
    }
    fn grouping(&mut self, grouping: &Grouping, depth: usize) -> Result<(), OmmlError> {
        if grouping.unpaired {
            return Err(OmmlError::InvalidExpression);
        }
        self.push("<m:d><m:dPr><m:begChr m:val=\"(\"/><m:endChr m:val=\")\"/></m:dPr><m:e>")?;
        self.expression(&grouping.expression, self.child_depth(depth)?)?;
        self.push("</m:e></m:d>")
    }
    fn matrix(&mut self, matrix: &Matrix, depth: usize) -> Result<(), OmmlError> {
        if matrix.rows == 0
            || matrix.columns == 0
            || matrix.rows.checked_mul(matrix.columns) != Some(matrix.elements.len())
        {
            return Err(OmmlError::InvalidExpression);
        }
        self.matrix_shape(matrix.rows, matrix.columns, &matrix.elements, depth)
    }
    fn vector(&mut self, vector: &Vector, depth: usize) -> Result<(), OmmlError> {
        if vector.elements.is_empty() {
            return Err(OmmlError::InvalidExpression);
        }
        let (rows, columns) = match vector.orientation {
            VectorOrientation::Row => (1, vector.elements.len()),
            VectorOrientation::Column => (vector.elements.len(), 1),
        };
        self.matrix_shape(rows, columns, &vector.elements, depth)
    }
    fn matrix_shape(
        &mut self,
        rows: usize,
        columns: usize,
        elements: &[MathExpression],
        depth: usize,
    ) -> Result<(), OmmlError> {
        let next = self.child_depth(depth)?;
        self.push("<m:d><m:dPr><m:begChr m:val=\"[\"/><m:endChr m:val=\"]\"/></m:dPr><m:e><m:m>")?;
        for row in elements.chunks_exact(columns).take(rows) {
            self.push("<m:mr>")?;
            for cell in row {
                self.push("<m:e>")?;
                self.expression(cell, next)?;
                self.push("</m:e>")?;
            }
            self.push("</m:mr>")?;
        }
        self.push("</m:m></m:e></m:d>")
    }
    fn integral(&mut self, integral: &Integral, depth: usize) -> Result<(), OmmlError> {
        let variable = identifier_ref(&integral.bound_variable)?;
        let next = self.child_depth(depth)?;
        self.push("<m:nary><m:naryPr><m:chr m:val=\"∫\"/></m:naryPr>")?;
        if let Some(bounds) = &integral.bounds {
            self.push("<m:sub>")?;
            self.expression(&bounds.lower, next)?;
            self.push("</m:sub><m:sup>")?;
            self.expression(&bounds.upper, next)?;
            self.push("</m:sup>")?;
        }
        self.push("<m:e>")?;
        self.expression(&integral.integrand, next)?;
        self.run("d", false)?;
        self.identifier(variable)?;
        self.push("</m:e></m:nary>")
    }
    fn derivative(&mut self, derivative: &Derivative, depth: usize) -> Result<(), OmmlError> {
        let variable = identifier_ref(&derivative.bound_variable)?;
        let glyph = match derivative.style {
            DerivativeStyle::Default | DerivativeStyle::Derivative => "d",
            DerivativeStyle::Partial => "∂",
        };
        let next = self.child_depth(depth)?;
        self.push("<m:f><m:fPr><m:type m:val=\"bar\"/></m:fPr><m:num>")?;
        self.derivative_glyph(glyph, derivative.degree.as_deref(), next)?;
        self.expression(&derivative.expression, next)?;
        self.push("</m:num><m:den>")?;
        self.derivative_variable(variable, derivative.degree.as_deref(), next)?;
        self.push("</m:den></m:f>")
    }
    fn derivative_glyph(
        &mut self,
        glyph: &str,
        degree: Option<&MathExpression>,
        depth: usize,
    ) -> Result<(), OmmlError> {
        if let Some(degree) = degree {
            self.push("<m:sSup><m:e>")?;
            self.run(glyph, false)?;
            self.push("</m:e><m:sup>")?;
            self.expression(degree, depth)?;
            self.push("</m:sup></m:sSup>")
        } else {
            self.run(glyph, false)
        }
    }
    fn derivative_variable(
        &mut self,
        value: &Identifier,
        degree: Option<&MathExpression>,
        depth: usize,
    ) -> Result<(), OmmlError> {
        let Some(degree) = degree else {
            return self.identifier(value);
        };
        match &value.subscript {
            None => {
                self.push("<m:sSup><m:e>")?;
                self.bare_identifier(value)?;
                self.push("</m:e><m:sup>")?;
                self.expression(degree, depth)?;
                self.push("</m:sup></m:sSup>")
            }
            Some(subscript) if !subscript.is_empty() => {
                self.push("<m:sSubSup><m:e>")?;
                self.bare_identifier(value)?;
                self.push("</m:e><m:sub>")?;
                self.run(subscript, false)?;
                self.push("</m:sub><m:sup>")?;
                self.expression(degree, depth)?;
                self.push("</m:sup></m:sSubSup>")
            }
            Some(_) => Err(OmmlError::InvalidExpression),
        }
    }
    fn aggregate(
        &mut self,
        aggregate: &AggregateExpression,
        depth: usize,
    ) -> Result<(), OmmlError> {
        let variable = identifier_ref(&aggregate.bound_variable)?;
        let glyph = match aggregate.operator {
            AggregateOperator::Summation => "∑",
            AggregateOperator::Product => "∏",
        };
        let next = self.child_depth(depth)?;
        self.push("<m:nary><m:naryPr><m:chr m:val=\"")?;
        self.push(glyph)?;
        self.push("\"/></m:naryPr><m:sub>")?;
        self.identifier(variable)?;
        if let Some(bounds) = &aggregate.bounds {
            self.run("=", false)?;
            self.expression(&bounds.lower, next)?;
            self.push("</m:sub><m:sup>")?;
            self.expression(&bounds.upper, next)?;
            self.push("</m:sup>")?;
        } else {
            self.push("</m:sub>")?;
        }
        self.push("<m:e>")?;
        self.expression(&aggregate.body, next)?;
        self.push("</m:e></m:nary>")
    }
    fn render_linear_operator(&mut self, operator: LinearOperator) -> Result<(), OmmlError> {
        match operator {
            LinearOperator::Add => self.run("+", false),
            LinearOperator::Subtract => self.run("−", false),
            LinearOperator::Multiply(style) => self.multiply(style),
        }
    }
    fn multiply(&mut self, style: MultiplicationStyle) -> Result<(), OmmlError> {
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
    fn run(&mut self, value: &str, italic: bool) -> Result<(), OmmlError> {
        self.push("<m:r>")?;
        if italic {
            self.push("<m:rPr><m:sty m:val=\"i\"/></m:rPr>")?;
        }
        self.push("<m:t")?;
        if needs_preserved_space(value) {
            self.push(" xml:space=\"preserve\"")?;
        }
        self.push(">")?;
        self.escaped(value)?;
        self.push("</m:t></m:r>")
    }
    fn escaped(&mut self, value: &str) -> Result<(), OmmlError> {
        if !value.chars().all(is_xml_10_char) {
            return Err(OmmlError::InvalidXmlText);
        }
        for c in value.chars() {
            match c {
                '&' => self.push("&amp;")?,
                '<' => self.push("&lt;")?,
                '>' => self.push("&gt;")?,
                _ => self.character(c)?,
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
    fn character(&mut self, value: char) -> Result<(), OmmlError> {
        let mut buffer = [0_u8; 4];
        self.push(value.encode_utf8(&mut buffer))
    }
}

fn identifier_ref(expression: &MathExpression) -> Result<&Identifier, OmmlError> {
    match &expression.kind {
        MathExpressionKind::Identifier(value) if !value.name.is_empty() => Ok(value),
        _ => Err(OmmlError::InvalidExpression),
    }
}
#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy)]
enum LinearOperator {
    Add,
    Subtract,
    Multiply(MultiplicationStyle),
}

impl LinearOperator {
    const fn emits_run(self) -> bool {
        !matches!(self, Self::Multiply(MultiplicationStyle::NoSpace))
    }
}

enum LinearItem<'a> {
    Expression(&'a MathExpression),
    Binary(&'a BinaryExpression),
    Operator(LinearOperator),
}

fn is_linear_binary(binary: &BinaryExpression) -> bool {
    matches!(
        binary.operator,
        BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply
    )
}

fn linear_operator(binary: &BinaryExpression) -> Result<LinearOperator, OmmlError> {
    match binary.operator {
        BinaryOperator::Add if binary.multiplication_style.is_none() => Ok(LinearOperator::Add),
        BinaryOperator::Subtract if binary.multiplication_style.is_none() => {
            Ok(LinearOperator::Subtract)
        }
        BinaryOperator::Multiply => binary
            .multiplication_style
            .map(LinearOperator::Multiply)
            .ok_or(OmmlError::InvalidExpression),
        _ => Err(OmmlError::InvalidExpression),
    }
}

fn needs_grouping(expression: &MathExpression, parent: BinaryOperator, side: Side) -> bool {
    if matches!(expression.kind, MathExpressionKind::Grouping(_)) {
        return false;
    }
    let MathExpressionKind::Binary(child) = &expression.kind else {
        return false;
    };
    match parent {
        BinaryOperator::Multiply => matches!(
            child.operator,
            BinaryOperator::Add | BinaryOperator::Subtract
        ),
        BinaryOperator::Subtract if matches!(side, Side::Right) => matches!(
            child.operator,
            BinaryOperator::Add | BinaryOperator::Subtract
        ),
        BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Divide => false,
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
    let mut digits = 0;
    let mut dot = false;
    for c in unsigned.chars() {
        if c == '.' && !dot {
            dot = true;
        } else if c.is_digit(u32::from(base.value())) {
            digits += 1;
        } else {
            return false;
        }
    }
    digits > 0
}
fn valid_decimal(value: &str) -> bool {
    let mut parts = value.split(['e', 'E']);
    let mantissa = parts.next().unwrap_or_default();
    let exponent = parts.next();
    if parts.next().is_some() {
        return false;
    }
    if let Some(exp) = exponent {
        let exp = exp.strip_prefix(['+', '-']).unwrap_or(exp);
        if exp.is_empty() || !exp.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }
    let mut digits = 0;
    let mut dot = false;
    for c in mantissa.chars() {
        if c == '.' && !dot {
            dot = true;
        } else if c.is_ascii_digit() {
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
