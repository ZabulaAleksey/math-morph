use std::io::{Cursor, Read, Write};

use document_ir::ports::{AssetResolveError, AssetResolver, ResolvedAsset};
use document_ir::{
    AssetRefIr, BlockContentIr, BlockId, BlockIr, DocumentIrV1, FidelityIr, FormulaDisplayModeIr,
    FormulaIr, MetadataIr, PageIr, PageMarginsIr, PageOrientationIr, PhysicalSizeIr, ProvenanceIr,
    SourceKindIr,
};
use exporter_docx::{
    DocxError, DocxExporter, DocxLimit, DocxLimits, DocxValidationError, DocxValidator, OmmlError,
    OmmlLimit,
};
use math_model::{
    BinaryExpression, BinaryOperator, ExpressionOrigin, Identifier, MathExpression,
    MathExpressionKind, MultiplicationStyle, NumericBase, RealLiteral,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

struct NoAssets;

impl AssetResolver for NoAssets {
    fn resolve(&self, _: &AssetRefIr) -> Result<ResolvedAsset, AssetResolveError> {
        Err(AssetResolveError::Unavailable)
    }
}

fn expression(kind: MathExpressionKind) -> MathExpression {
    MathExpression {
        kind,
        origin: ExpressionOrigin::Derived,
    }
}

fn real(value: &str) -> MathExpression {
    expression(MathExpressionKind::Real(RealLiteral {
        lexeme: value.to_owned(),
        base: NumericBase::Decimal,
    }))
}

fn identifier(value: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: value.to_owned(),
        subscript: None,
    }))
}

fn binary(operator: BinaryOperator, left: MathExpression, right: MathExpression) -> MathExpression {
    expression(MathExpressionKind::Binary(BinaryExpression {
        operator,
        multiplication_style: (operator == BinaryOperator::Multiply)
            .then_some(MultiplicationStyle::Default),
        left: Box::new(left),
        right: Box::new(right),
    }))
}

fn formula_block(
    id: &str,
    original: Option<MathExpression>,
    display: MathExpression,
    mode: FormulaDisplayModeIr,
) -> BlockIr {
    BlockIr {
        id: BlockId(id.to_owned()),
        provenance: ProvenanceIr {
            source_kind: SourceKindIr::Derived,
            region_id: None,
            source_ordinal: None,
            span: None,
        },
        fidelity: FidelityIr::Exact,
        placement: None,
        content: BlockContentIr::Equation(FormulaIr {
            original,
            display,
            mode,
        }),
    }
}

fn document(blocks: Vec<BlockIr>) -> DocumentIrV1 {
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
            blocks,
        }],
    }
}

fn parts(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
    (0..archive.len())
        .map(|index| {
            let mut entry = archive.by_index(index).unwrap();
            let name = entry.name().to_owned();
            let mut value = Vec::new();
            entry.read_to_end(&mut value).unwrap();
            (name, value)
        })
        .collect()
}

fn replace_document_xml(bytes: &[u8], replacement: &[u8]) -> Vec<u8> {
    let owned = parts(bytes);
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, value) in &owned {
        writer.start_file(name, options).unwrap();
        writer
            .write_all(if name == "word/document.xml" {
                replacement
            } else {
                value
            })
            .unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn document_xml(bytes: &[u8]) -> String {
    parts(bytes)
        .into_iter()
        .find(|(name, _)| name == "word/document.xml")
        .map(|(_, value)| String::from_utf8(value).unwrap())
        .unwrap()
}

#[test]
fn inline_and_display_equations_use_only_formula_display() {
    let unsupported_original = binary(BinaryOperator::Power, identifier("secret"), real("9"));
    let document = document(vec![
        formula_block(
            "inline",
            Some(unsupported_original),
            identifier("shown"),
            FormulaDisplayModeIr::Inline,
        ),
        formula_block(
            "display",
            None,
            binary(BinaryOperator::Add, real("1"), real("2")),
            FormulaDisplayModeIr::Display,
        ),
    ]);
    let bytes = DocxExporter::default()
        .export(&document, &NoAssets)
        .unwrap();
    DocxValidator::default().validate(&bytes).unwrap();
    let xml = document_xml(&bytes);

    assert!(xml.contains("<w:p><m:oMath xmlns:m="));
    assert!(xml.contains("<w:p><m:oMathPara xmlns:m="));
    assert!(xml.contains("<m:t>shown</m:t>"));
    assert!(xml.contains("<m:t>1</m:t></m:r><m:r><m:t>+</m:t></m:r><m:r><m:t>2</m:t>"));
    assert!(!xml.contains("secret"));
}

#[test]
fn docx_equation_limits_and_unsupported_nodes_remain_typed_and_redacted() {
    let sum = document(vec![formula_block(
        "equation",
        None,
        binary(BinaryOperator::Add, real("1"), real("2")),
        FormulaDisplayModeIr::Inline,
    )]);
    let limits = DocxLimits {
        max_equation_nodes: 1,
        ..DocxLimits::default()
    };
    assert_eq!(
        DocxExporter::new(limits).export(&sum, &NoAssets),
        Err(DocxError::Equation(OmmlError::LimitExceeded(
            OmmlLimit::Nodes
        )))
    );

    let power = document(vec![formula_block(
        "equation",
        None,
        binary(BinaryOperator::Power, identifier("secret"), real("2")),
        FormulaDisplayModeIr::Inline,
    )]);
    assert!(DocxExporter::default().export(&power, &NoAssets).is_ok());
}

#[test]
fn validator_accepts_only_the_generated_omml_subset() {
    let document = document(vec![formula_block(
        "fraction",
        None,
        binary(BinaryOperator::Divide, identifier("x"), real("2")),
        FormulaDisplayModeIr::Display,
    )]);
    let valid = DocxExporter::default()
        .export(&document, &NoAssets)
        .unwrap();
    let xml = document_xml(&valid);

    let wrong_namespace = xml.replace(
        "http://schemas.openxmlformats.org/officeDocument/2006/math",
        "urn:wrong-math",
    );
    assert_eq!(
        DocxValidator::default()
            .validate(&replace_document_xml(&valid, wrong_namespace.as_bytes(),)),
        Err(DocxValidationError::InvalidDocumentXml)
    );

    let missing_denominator = xml.replace("<m:den><m:r><m:t>2</m:t></m:r></m:den>", "");
    assert_eq!(
        DocxValidator::default().validate(&replace_document_xml(
            &valid,
            missing_denominator.as_bytes(),
        )),
        Err(DocxValidationError::InvalidEquation)
    );

    let unsupported = xml.replace("<m:num>", "<m:num><m:sSup><m:e/><m:sup/></m:sSup>");
    assert_eq!(
        DocxValidator::default().validate(&replace_document_xml(&valid, unsupported.as_bytes(),)),
        Err(DocxValidationError::InvalidEquation)
    );
}

#[test]
fn validator_enforces_equation_depth_node_and_output_limits() {
    let linear = document(vec![formula_block(
        "linear",
        None,
        binary(BinaryOperator::Add, real("1"), real("2")),
        FormulaDisplayModeIr::Inline,
    )]);
    let linear_docx = DocxExporter::default().export(&linear, &NoAssets).unwrap();
    let node_limits = DocxLimits {
        max_equation_nodes: 2,
        ..DocxLimits::default()
    };
    assert_eq!(
        DocxValidator::new(node_limits).validate(&linear_docx),
        Err(DocxValidationError::LimitExceeded(DocxLimit::EquationNodes))
    );

    let fraction = document(vec![formula_block(
        "fraction",
        None,
        binary(BinaryOperator::Divide, real("1"), real("2")),
        FormulaDisplayModeIr::Inline,
    )]);
    let fraction_docx = DocxExporter::default()
        .export(&fraction, &NoAssets)
        .unwrap();
    let depth_limits = DocxLimits {
        max_equation_depth: 0,
        ..DocxLimits::default()
    };
    assert_eq!(
        DocxValidator::new(depth_limits).validate(&fraction_docx),
        Err(DocxValidationError::LimitExceeded(DocxLimit::EquationDepth))
    );

    let output_limits = DocxLimits {
        max_equation_output_bytes: 32,
        ..DocxLimits::default()
    };
    assert_eq!(
        DocxValidator::new(output_limits).validate(&linear_docx),
        Err(DocxValidationError::LimitExceeded(
            DocxLimit::EquationOutputBytes
        ))
    );
}

#[test]
fn validator_rejects_new_equation_attributes_and_child_order() {
    let power = document(vec![formula_block(
        "power",
        None,
        binary(BinaryOperator::Power, identifier("x"), real("2")),
        FormulaDisplayModeIr::Inline,
    )]);
    let valid = DocxExporter::default().export(&power, &NoAssets).unwrap();
    let xml = document_xml(&valid);
    let invalid_attribute = xml.replace("<m:sSup>", "<m:sSup m:val=\"unexpected\">");
    assert_eq!(
        DocxValidator::default()
            .validate(&replace_document_xml(&valid, invalid_attribute.as_bytes())),
        Err(DocxValidationError::InvalidEquation)
    );
    let invalid_order = xml.replace(
        "<m:sSup><m:e><m:r><m:rPr><m:sty m:val=\"i\"/></m:rPr><m:t>x</m:t></m:r></m:e><m:sup><m:r><m:t>2</m:t></m:r></m:sup></m:sSup>",
        "<m:sSup><m:sup><m:r><m:t>2</m:t></m:r></m:sup><m:e><m:r><m:rPr><m:sty m:val=\"i\"/></m:rPr><m:t>x</m:t></m:r></m:e></m:sSup>",
    );
    assert_eq!(
        DocxValidator::default().validate(&replace_document_xml(&valid, invalid_order.as_bytes())),
        Err(DocxValidationError::InvalidEquation)
    );
}
