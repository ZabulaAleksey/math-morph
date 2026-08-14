//! Generates a deterministic synthetic advanced-OMML reference artifact.
//! Run with: cargo run -p exporter-docx --example advanced_omml_reference

use document_ir::ports::{AssetResolveError, AssetResolver, ResolvedAsset};
use document_ir::{
    AssetRefIr, BlockContentIr, BlockId, BlockIr, DocumentIrV1, FidelityIr, FormulaDisplayModeIr,
    FormulaIr, MetadataIr, PageIr, PageMarginsIr, PageOrientationIr, PhysicalSizeIr, ProvenanceIr,
    SourceKindIr,
};
use exporter_docx::{DocxExporter, DocxValidator};
use math_model::{
    BinaryExpression, BinaryOperator, ExpressionOrigin, FunctionCall, Identifier, MathExpression,
    MathExpressionKind, NumericBase, RealLiteral, UnaryExpression, UnaryOperator,
};

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
        lexeme: value.into(),
        base: NumericBase::Decimal,
    }))
}
fn identifier(name: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: name.into(),
        subscript: None,
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = expression(MathExpressionKind::Binary(BinaryExpression {
        operator: BinaryOperator::Power,
        multiplication_style: None,
        left: Box::new(expression(MathExpressionKind::Unary(UnaryExpression {
            operator: UnaryOperator::SquareRoot,
            operand: Box::new(expression(MathExpressionKind::FunctionCall(FunctionCall {
                callee: Box::new(identifier("f")),
                arguments: vec![identifier("x"), real("2")],
            }))),
        }))),
        right: Box::new(real("3")),
    }));
    let document = DocumentIrV1 {
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
                id: BlockId("advanced-omml-reference".into()),
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
                    display: root,
                    mode: FormulaDisplayModeIr::Display,
                }),
            }],
        }],
    };
    let bytes = DocxExporter::default().export(&document, &NoAssets)?;
    DocxValidator::default().validate(&bytes)?;
    let directory = std::path::Path::new("target/word-reference");
    std::fs::create_dir_all(directory)?;
    std::fs::write(directory.join("advanced-omml-reference.docx"), bytes)?;
    Ok(())
}
