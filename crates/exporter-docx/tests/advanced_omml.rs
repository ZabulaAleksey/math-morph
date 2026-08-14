use document_ir::ports::{AssetResolveError, AssetResolver, ResolvedAsset};
use document_ir::{
    AssetRefIr, BlockContentIr, BlockId, BlockIr, DocumentIrV1, FidelityIr, FormulaDisplayModeIr,
    FormulaIr, MetadataIr, PageIr, PageMarginsIr, PageOrientationIr, PhysicalSizeIr, ProvenanceIr,
    SourceKindIr,
};
use exporter_docx::{
    DocxError, DocxExportConfig, DocxExporter, DocxLimits, DocxValidator, EquationBackend,
    OmmlError, OmmlLimit, OmmlLimits, WordEquationExporter,
};
use math_model::{
    AggregateExpression, AggregateOperator, BinaryExpression, BinaryOperator, Bounds, Derivative,
    DerivativeStyle, ExpressionOrigin, FunctionCall, Grouping, Identifier, Integral,
    MathExpression, MathExpressionKind, Matrix, NumericBase, RealLiteral, UnaryExpression,
    UnaryOperator, Vector, VectorOrientation,
};

fn expression(kind: MathExpressionKind) -> MathExpression {
    MathExpression {
        kind,
        origin: ExpressionOrigin::Derived,
    }
}
fn real(value: &str) -> MathExpression {
    expression(MathExpressionKind::Real(RealLiteral {
        lexeme: value.into(),
        base: NumericBase::Decimal,
    }))
}
fn identifier(value: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: value.into(),
        subscript: None,
    }))
}
fn subscripted(value: &str, subscript: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: value.into(),
        subscript: Some(subscript.into()),
    }))
}
fn power(base: MathExpression, exponent: MathExpression) -> MathExpression {
    expression(MathExpressionKind::Binary(BinaryExpression {
        operator: BinaryOperator::Power,
        multiplication_style: None,
        left: Box::new(base),
        right: Box::new(exponent),
    }))
}
fn render(value: &MathExpression) -> String {
    WordEquationExporter::default()
        .export_expression(value)
        .unwrap()
        .as_str()
        .to_owned()
}

#[test]
fn scripts_roots_calls_grouping_and_rectangles_have_canonical_shapes() {
    assert!(render(&power(identifier("x"), real("2"))).contains("<m:sSup><m:e>"));
    assert!(!render(&power(identifier("x"), real("2"))).contains("<m:t>^</m:t>"));
    assert!(
        render(&expression(MathExpressionKind::Unary(UnaryExpression {
            operator: UnaryOperator::SquareRoot,
            operand: Box::new(identifier("x")),
        })))
        .contains("<m:radPr><m:degHide m:val=\"1\"/></m:radPr><m:deg></m:deg><m:e>")
    );
    assert!(render(&subscripted("x", "i")).contains("<m:sSub><m:e>"));
    let sub_sup = render(&power(subscripted("x", "i"), real("2")));
    assert!(sub_sup.contains("<m:sSubSup><m:e>") && !sub_sup.contains("<m:sSup><m:e><m:sSub>"));

    let call = expression(MathExpressionKind::FunctionCall(FunctionCall {
        callee: Box::new(identifier("f")),
        arguments: vec![identifier("x"), real("2")],
    }));
    assert!(render(&call).contains("<m:func><m:fName>"));
    assert!(
        render(&call)
            .contains("<m:begChr m:val=\"(\"/><m:sepChr m:val=\",\"/><m:endChr m:val=\")\"/>")
    );
    let invalid_call = expression(MathExpressionKind::FunctionCall(FunctionCall {
        callee: Box::new(real("1")),
        arguments: Vec::new(),
    }));
    assert_eq!(
        WordEquationExporter::default().export_expression(&invalid_call),
        Err(OmmlError::InvalidExpression)
    );
    let grouped = expression(MathExpressionKind::Grouping(Grouping {
        expression: Box::new(identifier("x")),
        unpaired: false,
    }));
    assert!(
        render(&grouped)
            .contains("<m:d><m:dPr><m:begChr m:val=\"(\"/><m:endChr m:val=\")\"/></m:dPr>")
    );
    assert_eq!(
        WordEquationExporter::default().export_expression(&expression(
            MathExpressionKind::Grouping(Grouping {
                expression: Box::new(identifier("x")),
                unpaired: true,
            })
        )),
        Err(OmmlError::InvalidExpression)
    );

    let row = expression(MathExpressionKind::Vector(Vector {
        orientation: VectorOrientation::Row,
        elements: vec![real("1"), real("2")],
    }));
    let column = expression(MathExpressionKind::Vector(Vector {
        orientation: VectorOrientation::Column,
        elements: vec![real("1"), real("2")],
    }));
    assert_eq!(render(&row).matches("<m:mr>").count(), 1);
    assert_eq!(render(&column).matches("<m:mr>").count(), 2);
    let matrix = expression(MathExpressionKind::Matrix(Matrix {
        rows: 2,
        columns: 2,
        elements: vec![real("1"), real("2"), real("3"), real("4")],
    }));
    assert_eq!(render(&matrix).matches("<m:mr>").count(), 2);
    assert_eq!(
        WordEquationExporter::default().export_expression(&expression(MathExpressionKind::Matrix(
            Matrix {
                rows: usize::MAX,
                columns: 2,
                elements: Vec::new(),
            }
        ))),
        Err(OmmlError::InvalidExpression)
    );
}

#[test]
fn calculus_uses_only_standard_omml_and_requires_identifier_variables() {
    let integral = expression(MathExpressionKind::Integral(Integral {
        bound_variable: Box::new(subscripted("x", "i")),
        integrand: Box::new(identifier("f")),
        bounds: Some(Bounds {
            lower: Box::new(real("0")),
            upper: Box::new(real("1")),
        }),
        algorithm: None,
    }));
    assert!(render(&integral).contains("<m:naryPr><m:chr m:val=\"∫\"/></m:naryPr><m:sub>"));
    assert!(render(&integral).contains("<m:t>d</m:t>"));
    let derivative = expression(MathExpressionKind::Derivative(Derivative {
        bound_variable: Box::new(subscripted("x", "i")),
        expression: Box::new(identifier("f")),
        degree: Some(Box::new(real("2"))),
        style: DerivativeStyle::Partial,
    }));
    let derivative_xml = render(&derivative);
    assert!(derivative_xml.contains("<m:f>") && derivative_xml.contains("<m:t>∂</m:t>"));
    assert!(!derivative_xml.contains("derivative"));
    let aggregate = expression(MathExpressionKind::Aggregate(AggregateExpression {
        operator: AggregateOperator::Summation,
        bound_variable: Box::new(identifier("i")),
        body: Box::new(identifier("a")),
        bounds: Some(Bounds {
            lower: Box::new(real("1")),
            upper: Box::new(identifier("n")),
        }),
    }));
    assert!(render(&aggregate).contains("<m:chr m:val=\"∑\"/>"));
    let invalid = expression(MathExpressionKind::Integral(Integral {
        bound_variable: Box::new(real("1")),
        integrand: Box::new(identifier("f")),
        bounds: None,
        algorithm: None,
    }));
    assert_eq!(
        WordEquationExporter::default().export_expression(&invalid),
        Err(OmmlError::InvalidExpression)
    );
}

struct NoAssets;
impl AssetResolver for NoAssets {
    fn resolve(&self, _: &AssetRefIr) -> Result<ResolvedAsset, AssetResolveError> {
        Err(AssetResolveError::Unavailable)
    }
}
fn document(display: MathExpression) -> DocumentIrV1 {
    DocumentIrV1 {
        metadata: MetadataIr::default(),
        pages: vec![PageIr {
            size: PhysicalSizeIr::a4_portrait(),
            orientation: PageOrientationIr::Portrait,
            margins: PageMarginsIr {
                top_um: 25_400,
                right_um: 25_400,
                bottom_um: 25_400,
                left_um: 25_400,
            },
            blocks: vec![BlockIr {
                id: BlockId("equation".into()),
                provenance: ProvenanceIr {
                    source_kind: SourceKindIr::Derived,
                    region_id: None,
                    source_ordinal: None,
                    span: None,
                },
                fidelity: FidelityIr::Exact,
                placement: None,
                content: BlockContentIr::Equation(FormulaIr {
                    original: None,
                    display,
                    mode: FormulaDisplayModeIr::Display,
                }),
            }],
        }],
    }
}

#[test]
fn backend_selection_and_validator_boundaries_are_fail_closed() {
    let value = power(subscripted("x", "i"), real("2"));
    let generated = DocxExporter::new(DocxLimits::default())
        .export(&document(value.clone()), &NoAssets)
        .unwrap();
    DocxValidator::default().validate(&generated).unwrap();
    assert_eq!(
        DocxExporter::default().config().equation_backend,
        EquationBackend::WordOmml
    );
    let unavailable = DocxExporter::with_config(
        DocxLimits::default(),
        DocxExportConfig {
            equation_backend: EquationBackend::MathType,
        },
    );
    assert_eq!(
        unavailable.export(&document(value), &NoAssets),
        Err(DocxError::EquationBackendUnavailable)
    );
    assert_eq!(
        WordEquationExporter::new(OmmlLimits {
            max_depth: 0,
            ..OmmlLimits::default()
        })
        .export_expression(&power(identifier("x"), real("2"))),
        Err(OmmlError::LimitExceeded(OmmlLimit::Depth))
    );
}

#[test]
fn validator_accepts_every_new_renderer_shape() {
    let call = expression(MathExpressionKind::FunctionCall(FunctionCall {
        callee: Box::new(identifier("f")),
        arguments: vec![expression(MathExpressionKind::Matrix(Matrix {
            rows: 1,
            columns: 2,
            elements: vec![identifier("x"), real("2")],
        }))],
    }));
    let calculus = expression(MathExpressionKind::Integral(Integral {
        bound_variable: Box::new(identifier("x")),
        integrand: Box::new(expression(MathExpressionKind::Derivative(Derivative {
            bound_variable: Box::new(identifier("x")),
            expression: Box::new(call),
            degree: Some(Box::new(real("2"))),
            style: DerivativeStyle::Derivative,
        }))),
        bounds: None,
        algorithm: None,
    }));
    let bytes = DocxExporter::default()
        .export(&document(calculus), &NoAssets)
        .unwrap();
    DocxValidator::default().validate(&bytes).unwrap();
}

#[test]
fn renderer_and_validator_share_exact_advanced_node_and_depth_budgets() {
    let subscript = subscripted("x", "i");
    assert_eq!(
        WordEquationExporter::new(OmmlLimits {
            max_nodes: 2,
            ..OmmlLimits::default()
        })
        .export_expression(&subscript),
        Err(OmmlError::LimitExceeded(OmmlLimit::Nodes))
    );
    assert!(
        WordEquationExporter::new(OmmlLimits {
            max_nodes: 3,
            max_depth: 1,
            ..OmmlLimits::default()
        })
        .export_expression(&subscript)
        .is_ok()
    );

    let function = expression(MathExpressionKind::FunctionCall(FunctionCall {
        callee: Box::new(identifier("f")),
        arguments: vec![identifier("x")],
    }));
    assert_eq!(
        WordEquationExporter::new(OmmlLimits {
            max_depth: 1,
            ..OmmlLimits::default()
        })
        .export_expression(&function),
        Err(OmmlError::LimitExceeded(OmmlLimit::Depth))
    );
    assert!(
        WordEquationExporter::new(OmmlLimits {
            max_nodes: 4,
            max_depth: 2,
            ..OmmlLimits::default()
        })
        .export_expression(&function)
        .is_ok()
    );

    let integral = expression(MathExpressionKind::Integral(Integral {
        bound_variable: Box::new(identifier("x")),
        integrand: Box::new(identifier("f")),
        bounds: None,
        algorithm: None,
    }));
    assert_eq!(
        WordEquationExporter::new(OmmlLimits {
            max_nodes: 3,
            ..OmmlLimits::default()
        })
        .export_expression(&integral),
        Err(OmmlError::LimitExceeded(OmmlLimit::Nodes))
    );
    let enough = DocxLimits {
        max_equation_nodes: 4,
        max_equation_depth: 1,
        ..DocxLimits::default()
    };
    let exported = DocxExporter::new(enough)
        .export(&document(integral), &NoAssets)
        .unwrap();
    DocxValidator::new(enough).validate(&exported).unwrap();

    let matrix = expression(MathExpressionKind::Matrix(Matrix {
        rows: 1,
        columns: 1,
        elements: vec![identifier("x")],
    }));
    assert_eq!(
        WordEquationExporter::new(OmmlLimits {
            max_depth: 2,
            ..OmmlLimits::default()
        })
        .export_expression(&matrix),
        Err(OmmlError::LimitExceeded(OmmlLimit::Depth))
    );
    assert!(
        WordEquationExporter::new(OmmlLimits {
            max_nodes: 4,
            max_depth: 3,
            ..OmmlLimits::default()
        })
        .export_expression(&matrix)
        .is_ok()
    );
}
