use crate::ast::{
    AggregateExpression, AggregateOperator, ArrayIndex, BinaryExpression, BinaryOperator, Bounds,
    ComparisonExpression, ComparisonOperator, Definition, DefinitionKind, DefinitionStyle,
    Derivative, DerivativeStyle, Evaluation, FunctionCall, FunctionDefinition, Grouping,
    Identifier, Integral, IntegralAlgorithm, MathAstError, MathExpression, MathExpressionKind,
    Matrix, MultiplicationStyle, NumericBase, RangeExpression, RealLiteral, UnaryExpression,
    UnaryOperator, Vector, VectorOrientation,
};
use crate::xml_worksheet::{Child, Node};
use crate::{
    BooleanExpression, BooleanOperator, Diagnostic, DiagnosticCode, ExpressionOrigin, LogicalNot,
    UnitMonomial, UnitReference, UnitedValue, UnsupportedNode, UnsupportedReason,
};
use std::num::NonZeroI64;

const MATH_NS: &str = "http://schemas.mathsoft.com/math30";
const UNITS_NS: &str = "http://schemas.mathsoft.com/units10";

pub(crate) enum MathXmlOutcome {
    Parsed {
        expression: MathExpression,
        diagnostics: Vec<Diagnostic>,
    },
    Invalid(MathAstError),
}

pub(crate) fn parse_math_expression(
    node: &Node,
    max_nodes: usize,
    max_matrix_elements: usize,
    max_unit_factors: usize,
) -> MathXmlOutcome {
    let mut parser = MathParser {
        max_nodes,
        nodes: 0,
        max_matrix_elements,
        max_unit_factors,
        diagnostics: Vec::new(),
    };
    match parser.expression(node) {
        Ok(expression) => MathXmlOutcome::Parsed {
            expression,
            diagnostics: parser.diagnostics,
        },
        Err(Failure::Unsupported) => MathXmlOutcome::Parsed {
            expression: parser.unsupported(node, None, UnsupportedReason::UnknownExpression),
            diagnostics: parser.diagnostics,
        },
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
    max_matrix_elements: usize,
    max_unit_factors: usize,
    diagnostics: Vec<Diagnostic>,
}

impl MathParser {
    fn expression(&mut self, node: &Node) -> Result<MathExpression, Failure> {
        self.consume_node()?;
        if node.name.namespace_uri.as_deref() != Some(MATH_NS) {
            return Ok(self.unsupported(node, None, UnsupportedReason::UnknownExpression));
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
            "matrix" => self.matrix(node)?,
            "range" => MathExpressionKind::Range(self.range(node)?),
            "unitedValue" => MathExpressionKind::UnitedValue(self.united_value(node)?),
            _ => return Ok(self.unsupported(node, None, UnsupportedReason::UnknownExpression)),
        };
        Ok(MathExpression {
            kind,
            origin: ExpressionOrigin::Source(node.span),
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
            return Ok(self
                .unsupported(node, None, UnsupportedReason::UnknownOperator)
                .kind);
        };
        if head.name.namespace_uri.as_deref() != Some(MATH_NS) {
            if matches!(head.name.local_name.as_str(), "and" | "or" | "xor" | "not") {
                return Err(MathAstError::InvalidBooleanOperatorQName.into());
            }
            return Ok(self
                .unsupported(
                    node,
                    Some(head.name.clone()),
                    UnsupportedReason::UnknownOperator,
                )
                .kind);
        }
        let kind = match head.name.local_name.as_str() {
            "plus" | "minus" | "mult" | "div" | "pow" => {
                MathExpressionKind::Binary(self.binary(node)?)
            }
            "absval" | "conjugate" | "factorial" | "neg" | "sqrt" | "transpose" | "vectorize"
            | "vectorSum" | "determinant" => MathExpressionKind::Unary(self.unary(node)?),
            "indexer" => MathExpressionKind::ArrayIndex(self.array_index(node)?),
            "integral" => MathExpressionKind::Integral(self.integral(node, head)?),
            "derivative" => MathExpressionKind::Derivative(self.derivative(node, head)?),
            "summation" => {
                MathExpressionKind::Aggregate(self.aggregate(node, AggregateOperator::Summation)?)
            }
            "product" => {
                MathExpressionKind::Aggregate(self.aggregate(node, AggregateOperator::Product)?)
            }
            "equal" | "notEqual" | "greaterOrEqual" | "greaterThan" | "lessOrEqual"
            | "lessThan" => MathExpressionKind::Comparison(self.comparison(node)?),
            "and" | "or" | "xor" => MathExpressionKind::Boolean(self.boolean(node, head)?),
            "not" => MathExpressionKind::LogicalNot(self.logical_not(node, head)?),
            local if is_supported_expression_form(local) => {
                MathExpressionKind::FunctionCall(self.function_call(node)?)
            }
            _ => {
                return Ok(self
                    .unsupported(
                        node,
                        Some(head.name.clone()),
                        UnsupportedReason::UnknownOperator,
                    )
                    .kind);
            }
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
            multiplication_style: if operator == BinaryOperator::Multiply {
                Some(multiplication_style(operator_node.attribute("style"))?)
            } else {
                None
            },
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
            if nodes.len() < 2 {
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

    fn matrix(&mut self, node: &Node) -> Result<MathExpressionKind, Failure> {
        if has_non_whitespace_text(node) {
            return Err(MathAstError::InvalidMatrixDimensions.into());
        }
        let rows = positive_dimension(node.attribute("rows"))?;
        let columns = positive_dimension(node.attribute("cols"))?;
        let expected = rows
            .checked_mul(columns)
            .ok_or(MathAstError::InvalidMatrixDimensions)?;
        if expected > self.max_matrix_elements {
            return Err(MathAstError::MatrixElementLimitExceeded.into());
        }
        let children: Vec<_> = node.element_children().collect();
        if children.len() != expected {
            return Err(MathAstError::MatrixElementCountMismatch {
                expected,
                actual: children.len(),
            }
            .into());
        }
        let elements = children
            .iter()
            .map(|element| self.expression(element))
            .collect::<Result<Vec<_>, _>>()?;
        if rows == 1 && columns > 1 {
            Ok(MathExpressionKind::Vector(Vector {
                orientation: VectorOrientation::Row,
                elements,
            }))
        } else if columns == 1 && rows > 1 {
            Ok(MathExpressionKind::Vector(Vector {
                orientation: VectorOrientation::Column,
                elements,
            }))
        } else {
            Ok(MathExpressionKind::Matrix(Matrix {
                rows,
                columns,
                elements,
            }))
        }
    }

    fn range(&mut self, node: &Node) -> Result<RangeExpression, Failure> {
        if has_non_whitespace_text(node) {
            return Err(MathAstError::MalformedRange.into());
        }
        let children: Vec<_> = node.element_children().collect();
        if children.len() != 2 {
            return Err(MathAstError::MalformedRange.into());
        }
        let (start, next) = if children[0].is(MATH_NS, "sequence") {
            if has_non_whitespace_text(children[0]) {
                return Err(MathAstError::MalformedRange.into());
            }
            let sequence: Vec<_> = children[0].element_children().collect();
            if sequence.len() != 2 {
                return Err(MathAstError::MalformedRange.into());
            }
            (
                self.expression(sequence[0])?,
                Some(Box::new(self.expression(sequence[1])?)),
            )
        } else {
            (self.expression(children[0])?, None)
        };
        Ok(RangeExpression {
            start: Box::new(start),
            next,
            end: Box::new(self.expression(children[1])?),
        })
    }

    fn integral(&mut self, node: &Node, head: &Node) -> Result<Integral, Failure> {
        let (bound_variable, integrand, bounds) = self.lambda_with_optional_bounds(node)?;
        let algorithm = head
            .attribute("algorithm")
            .map(integral_algorithm)
            .transpose()?;
        Ok(Integral {
            bound_variable: Box::new(bound_variable),
            integrand: Box::new(integrand),
            bounds,
            algorithm,
        })
    }

    fn derivative(&mut self, node: &Node, head: &Node) -> Result<Derivative, Failure> {
        let children: Vec<_> = node.element_children().collect();
        if children.len() < 2 || children.len() > 3 || !children[1].is(MATH_NS, "lambda") {
            return Err(MathAstError::MalformedCalculus.into());
        }
        let (bound_variable, expression) = self.lambda(children[1])?;
        let degree = match children.get(2) {
            Some(wrapper) if wrapper.is(MATH_NS, "degree") => Some(Box::new(
                self.single_wrapper_expression(wrapper, MathAstError::MalformedCalculus)?,
            )),
            Some(_) => return Err(MathAstError::MalformedCalculus.into()),
            None => None,
        };
        Ok(Derivative {
            bound_variable: Box::new(bound_variable),
            expression: Box::new(expression),
            degree,
            style: derivative_style(head.attribute("style"))?,
        })
    }

    fn aggregate(
        &mut self,
        node: &Node,
        operator: AggregateOperator,
    ) -> Result<AggregateExpression, Failure> {
        let (bound_variable, body, bounds) = self.lambda_with_optional_bounds(node)?;
        Ok(AggregateExpression {
            operator,
            bound_variable: Box::new(bound_variable),
            body: Box::new(body),
            bounds,
        })
    }

    fn lambda_with_optional_bounds(
        &mut self,
        node: &Node,
    ) -> Result<(MathExpression, MathExpression, Option<Bounds>), Failure> {
        let children: Vec<_> = node.element_children().collect();
        if children.len() < 2 || children.len() > 3 || !children[1].is(MATH_NS, "lambda") {
            return Err(MathAstError::MalformedCalculus.into());
        }
        let (bound_variable, body) = self.lambda(children[1])?;
        let bounds = match children.get(2) {
            Some(wrapper) if wrapper.is(MATH_NS, "bounds") => Some(self.bounds(wrapper)?),
            Some(_) => return Err(MathAstError::MalformedCalculus.into()),
            None => None,
        };
        Ok((bound_variable, body, bounds))
    }

    fn lambda(&mut self, node: &Node) -> Result<(MathExpression, MathExpression), Failure> {
        if has_non_whitespace_text(node) {
            return Err(MathAstError::MalformedCalculus.into());
        }
        let children: Vec<_> = node.element_children().collect();
        if children.len() != 2 || !children[0].is(MATH_NS, "boundVars") {
            return Err(MathAstError::MalformedCalculus.into());
        }
        let variables: Vec<_> = children[0].element_children().collect();
        if has_non_whitespace_text(children[0]) || variables.len() != 1 {
            return Err(MathAstError::InvalidBoundVariable.into());
        }
        let variable = self.expression(variables[0])?;
        if !matches!(variable.kind, MathExpressionKind::Identifier(_)) {
            return Err(MathAstError::InvalidBoundVariable.into());
        }
        Ok((variable, self.expression(children[1])?))
    }

    fn bounds(&mut self, node: &Node) -> Result<Bounds, Failure> {
        if has_non_whitespace_text(node) {
            return Err(MathAstError::MalformedCalculus.into());
        }
        let children: Vec<_> = node.element_children().collect();
        if children.len() != 2 {
            return Err(MathAstError::MalformedCalculus.into());
        }
        Ok(Bounds {
            lower: Box::new(self.expression(children[0])?),
            upper: Box::new(self.expression(children[1])?),
        })
    }

    fn comparison(&mut self, node: &Node) -> Result<ComparisonExpression, Failure> {
        let children: Vec<_> = node.element_children().collect();
        let operator = match children[0].name.local_name.as_str() {
            "equal" => ComparisonOperator::Equal,
            "notEqual" => ComparisonOperator::NotEqual,
            "greaterOrEqual" => ComparisonOperator::GreaterOrEqual,
            "greaterThan" => ComparisonOperator::GreaterThan,
            "lessOrEqual" => ComparisonOperator::LessOrEqual,
            "lessThan" => ComparisonOperator::LessThan,
            _ => return Err(Failure::Unsupported),
        };
        let actual = children.len().saturating_sub(1);
        if actual != 2 {
            return Err(MathAstError::WrongComparisonArity { operator, actual }.into());
        }
        Ok(ComparisonExpression {
            operator,
            left: Box::new(self.expression(children[1])?),
            right: Box::new(self.expression(children[2])?),
        })
    }

    fn boolean(&mut self, node: &Node, marker: &Node) -> Result<BooleanExpression, Failure> {
        let operator = match marker.name.local_name.as_str() {
            "and" => BooleanOperator::And,
            "or" => BooleanOperator::Or,
            "xor" => BooleanOperator::Xor,
            _ => return Err(MathAstError::InvalidBooleanOperatorQName.into()),
        };
        if marker.has_attributes() || !marker.children.is_empty() {
            return Err(MathAstError::NonEmptyBooleanMarker.into());
        }
        let children: Vec<_> = node.element_children().collect();
        let actual = children.len().saturating_sub(1);
        if actual != 2 {
            return Err(MathAstError::WrongBooleanArity { operator, actual }.into());
        }
        Ok(BooleanExpression {
            operator,
            left: Box::new(self.expression(children[1])?),
            right: Box::new(self.expression(children[2])?),
        })
    }

    fn logical_not(&mut self, node: &Node, marker: &Node) -> Result<LogicalNot, Failure> {
        if marker.name.local_name != "not" || marker.has_attributes() || !marker.children.is_empty()
        {
            return Err(MathAstError::NonEmptyBooleanMarker.into());
        }
        let children: Vec<_> = node.element_children().collect();
        let actual = children.len().saturating_sub(1);
        if actual != 1 {
            return Err(MathAstError::WrongLogicalNotArity { actual }.into());
        }
        Ok(LogicalNot {
            operand: Box::new(self.expression(children[1])?),
        })
    }

    fn united_value(&mut self, node: &Node) -> Result<UnitedValue, Failure> {
        if has_non_whitespace_text(node) {
            return Err(MathAstError::MalformedUnitedValue.into());
        }
        let children: Vec<_> = node.element_children().collect();
        if children.len() != 2 {
            return Err(MathAstError::MalformedUnitedValue.into());
        }
        let base = children[0];
        if base.name.namespace_uri.as_deref() != Some(MATH_NS) {
            return Err(MathAstError::MalformedUnitedValue.into());
        }
        let value = match base.name.local_name.as_str() {
            "real" | "matrix" => self.expression(base)?,
            "imag" | "complex" | "str" | "placeholder" => {
                self.consume_node()?;
                self.unsupported(base, None, UnsupportedReason::UnsupportedBaseValue)
            }
            _ => return Err(MathAstError::MalformedUnitedValue.into()),
        };
        Ok(UnitedValue {
            value: Box::new(value),
            units: self.unit_monomial(children[1])?,
        })
    }

    fn unit_monomial(&mut self, node: &Node) -> Result<UnitMonomial, Failure> {
        self.consume_node()?;
        if !node.is(UNITS_NS, "unitMonomial") {
            return Err(MathAstError::InvalidUnitQName.into());
        }
        if has_non_whitespace_text(node) {
            return Err(MathAstError::MalformedUnitMonomial.into());
        }
        let factors: Vec<_> = node.element_children().collect();
        if factors.is_empty() {
            return Err(MathAstError::MalformedUnitMonomial.into());
        }
        if factors.len() > self.max_unit_factors {
            return Err(MathAstError::UnitFactorLimitExceeded.into());
        }
        let factors = factors
            .into_iter()
            .map(|factor| self.unit_reference(factor))
            .collect::<Result<_, _>>()?;
        Ok(UnitMonomial {
            system: node.attribute("system").map(str::to_owned),
            factors,
        })
    }

    fn unit_reference(&mut self, node: &Node) -> Result<UnitReference, Failure> {
        self.consume_node()?;
        if !node.is(UNITS_NS, "unitReference") {
            return Err(MathAstError::InvalidUnitQName.into());
        }
        if !node.children.is_empty() {
            return Err(MathAstError::MalformedUnitMonomial.into());
        }
        let unit = node
            .attribute("unit")
            .filter(|unit| !unit.is_empty())
            .ok_or(MathAstError::MissingUnitName)?
            .to_owned();
        let power_numerator = signed_power(node.attribute("power-numerator"))?;
        let power_denominator = NonZeroI64::new(signed_power(node.attribute("power-denominator"))?)
            .ok_or(MathAstError::ZeroUnitPowerDenominator)?;
        Ok(UnitReference {
            unit,
            power_numerator,
            power_denominator,
        })
    }

    fn consume_node(&mut self) -> Result<(), Failure> {
        self.nodes = self
            .nodes
            .checked_add(1)
            .ok_or(MathAstError::NodeLimitExceeded)?;
        if self.nodes > self.max_nodes {
            return Err(MathAstError::NodeLimitExceeded.into());
        }
        Ok(())
    }

    fn unsupported(
        &mut self,
        node: &Node,
        feature: Option<crate::ExpandedName>,
        reason: UnsupportedReason,
    ) -> MathExpression {
        self.diagnostics.push(Diagnostic::warning(
            DiagnosticCode::UnsupportedMathNode,
            None,
        ));
        MathExpression {
            kind: MathExpressionKind::Unsupported(UnsupportedNode {
                name: node.name.clone(),
                feature,
                span: node.span,
                reason,
            }),
            origin: ExpressionOrigin::Source(node.span),
        }
    }
}

fn signed_power(value: Option<&str>) -> Result<i64, Failure> {
    value
        .unwrap_or("1")
        .parse::<i64>()
        .map_err(|_| MathAstError::InvalidUnitPower.into())
}

fn multiplication_style(value: Option<&str>) -> Result<MultiplicationStyle, Failure> {
    match value.unwrap_or("default") {
        "default" => Ok(MultiplicationStyle::Default),
        "auto-select" => Ok(MultiplicationStyle::AutoSelect),
        "dot" => Ok(MultiplicationStyle::Dot),
        "narrow-dot" => Ok(MultiplicationStyle::NarrowDot),
        "large-dot" => Ok(MultiplicationStyle::LargeDot),
        "x" => Ok(MultiplicationStyle::X),
        "thin-space" => Ok(MultiplicationStyle::ThinSpace),
        "no-space" => Ok(MultiplicationStyle::NoSpace),
        _ => Err(MathAstError::InvalidMultiplicationStyle.into()),
    }
}

fn positive_dimension(value: Option<&str>) -> Result<usize, Failure> {
    let value = value.ok_or(MathAstError::InvalidMatrixDimensions)?;
    let value = value
        .parse::<usize>()
        .map_err(|_| MathAstError::InvalidMatrixDimensions)?;
    if value == 0 {
        return Err(MathAstError::InvalidMatrixDimensions.into());
    }
    Ok(value)
}

fn integral_algorithm(value: &str) -> Result<IntegralAlgorithm, Failure> {
    match value {
        "equal-interval" => Ok(IntegralAlgorithm::EqualInterval),
        "adaptive" => Ok(IntegralAlgorithm::Adaptive),
        "infinite" => Ok(IntegralAlgorithm::Infinite),
        "oscillating" => Ok(IntegralAlgorithm::Oscillating),
        "limit-end-points" => Ok(IntegralAlgorithm::LimitEndPoints),
        "romberg" => Ok(IntegralAlgorithm::Romberg),
        _ => Err(MathAstError::InvalidIntegralAlgorithm.into()),
    }
}

fn derivative_style(value: Option<&str>) -> Result<DerivativeStyle, Failure> {
    match value.unwrap_or("default") {
        "default" => Ok(DerivativeStyle::Default),
        "derivative" => Ok(DerivativeStyle::Derivative),
        "partial" => Ok(DerivativeStyle::Partial),
        _ => Err(MathAstError::InvalidDerivativeStyle.into()),
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
        "real"
            | "id"
            | "apply"
            | "define"
            | "globalDefine"
            | "localDefine"
            | "eval"
            | "parens"
            | "matrix"
            | "range"
            | "unitedValue"
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
