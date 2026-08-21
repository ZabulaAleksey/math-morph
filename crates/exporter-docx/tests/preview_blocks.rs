use std::collections::BTreeMap;

use document_ir::ports::{AssetResolveError, AssetResolver, ResolvedAsset};
use document_ir::{
    AssetId, AssetRefIr, BlockContentIr, BlockId, BlockIr, DiagramIr, DocumentIrV1, DocumentIrV3,
    FidelityIr, ImageIr, MediaTypeIr, MetadataIr, PageIr, PageMarginsIr, PageOrientationIr,
    PhysicalSizeIr, PlotIr, ProvenanceIr, SourceKindIr, VersionedDocumentIr,
};
use exporter_docx::{DocxError, DocxExporter, DocxValidator};

#[derive(Default)]
struct Assets(BTreeMap<String, ResolvedAsset>);

impl AssetResolver for Assets {
    fn resolve(&self, reference: &AssetRefIr) -> Result<ResolvedAsset, AssetResolveError> {
        self.0
            .get(&reference.id.0)
            .cloned()
            .ok_or(AssetResolveError::Unavailable)
    }
}

fn png() -> Vec<u8> {
    fn crc(bytes: &[u8]) -> u32 {
        let mut crc = 0xffff_ffff_u32;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                let mask = 0_u32.wrapping_sub(crc & 1);
                crc = (crc >> 1) ^ (0xedb8_8320 & mask);
            }
        }
        !crc
    }
    fn chunk(output: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        output.extend_from_slice(&(data.len() as u32).to_be_bytes());
        output.extend_from_slice(kind);
        output.extend_from_slice(data);
        let mut protected = Vec::from(*kind);
        protected.extend_from_slice(data);
        output.extend_from_slice(&crc(&protected).to_be_bytes());
    }
    let mut output = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend_from_slice(&1_u32.to_be_bytes());
    header.extend_from_slice(&1_u32.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut output, b"IHDR", &header);
    chunk(
        &mut output,
        b"IDAT",
        &[
            0x78, 0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0xf0, 0x1f, 0x00, 0x05, 0x00, 0x01, 0xff,
        ],
    );
    chunk(&mut output, b"IEND", &[]);
    output
}

fn image(id: &str) -> ImageIr {
    ImageIr {
        asset: AssetRefIr {
            id: AssetId(id.into()),
            media_type: MediaTypeIr::Png,
        },
        alt_text: Some("explicit raster preview".into()),
        size: Some(PhysicalSizeIr::new(20_000, 10_000).unwrap()),
    }
}

fn block(id: &str, fidelity: FidelityIr, content: BlockContentIr) -> BlockIr {
    BlockIr {
        id: BlockId(id.into()),
        provenance: ProvenanceIr {
            source_kind: SourceKindIr::Derived,
            region_id: None,
            source_ordinal: None,
            span: None,
        },
        fidelity,
        placement: None,
        content,
    }
}

fn document(blocks: Vec<BlockIr>) -> DocumentIrV1 {
    DocumentIrV1 {
        metadata: MetadataIr::default(),
        pages: vec![PageIr {
            size: PhysicalSizeIr::a4_portrait(),
            orientation: PageOrientationIr::Portrait,
            margins: PageMarginsIr {
                top_um: 10_000,
                right_um: 10_000,
                bottom_um: 10_000,
                left_um: 10_000,
            },
            blocks,
        }],
    }
}

#[test]
fn explicit_plot_and_diagram_previews_export_as_valid_fallback_images() {
    let document = document(vec![
        block(
            "plot",
            FidelityIr::FallbackRendered,
            BlockContentIr::Plot(PlotIr {
                preview: Some(image("plot-preview")),
            }),
        ),
        block(
            "diagram",
            FidelityIr::FallbackRendered,
            BlockContentIr::Diagram(DiagramIr {
                preview: Some(image("diagram-preview")),
                primitives: Vec::new(),
            }),
        ),
    ]);
    let bytes = png();
    let assets = Assets(BTreeMap::from([
        (
            "plot-preview".into(),
            ResolvedAsset {
                media_type: MediaTypeIr::Png,
                bytes: bytes.clone(),
            },
        ),
        (
            "diagram-preview".into(),
            ResolvedAsset {
                media_type: MediaTypeIr::Png,
                bytes,
            },
        ),
    ]));
    let output = DocxExporter::default().export(&document, &assets).unwrap();
    DocxValidator::default().validate(&output).unwrap();
}

#[test]
fn missing_preview_or_non_degraded_fidelity_fails_closed() {
    let missing = document(vec![block(
        "plot",
        FidelityIr::Unsupported,
        BlockContentIr::Plot(PlotIr { preview: None }),
    )]);
    assert_eq!(
        DocxExporter::default().export(&missing, &Assets::default()),
        Err(DocxError::UnsupportedContent)
    );
    let wrong_fidelity = document(vec![block(
        "diagram",
        FidelityIr::Exact,
        BlockContentIr::Diagram(DiagramIr {
            preview: Some(image("preview")),
            primitives: Vec::new(),
        }),
    )]);
    assert_eq!(
        DocxExporter::default().export(&wrong_fidelity, &Assets::default()),
        Err(DocxError::UnsupportedContent)
    );
}

#[test]
fn versioned_export_rejects_invalid_schema_3_before_using_v1_projection() {
    let invalid = VersionedDocumentIr::v3(DocumentIrV3 {
        document: document(vec![block(
            "plot",
            FidelityIr::FallbackRendered,
            BlockContentIr::Plot(PlotIr {
                preview: Some(image("preview")),
            }),
        )]),
        plot_metadata: Vec::new(),
    });
    assert!(invalid.validate().is_err());
    assert_eq!(
        DocxExporter::default().export_versioned(&invalid, &Assets::default()),
        Err(DocxError::InvalidDocument)
    );
}
