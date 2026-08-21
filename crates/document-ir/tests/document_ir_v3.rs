use document_ir::{
    BlockContentIr, BlockId, BlockIr, DocumentIrV1, DocumentIrV3, DocumentIrValidationError,
    FidelityIr, MetadataIr, PageIr, PageMarginsIr, PageOrientationIr, PhysicalSizeIr, PlotIr,
    PlotMetadataIrV3, ProvenanceIr, SourceKindIr, VersionedDocumentIr,
};

fn plot_block(id: &str) -> BlockIr {
    BlockIr {
        id: BlockId(id.to_owned()),
        provenance: ProvenanceIr {
            source_kind: SourceKindIr::Xmcd,
            region_id: Some(7),
            source_ordinal: Some(2),
            span: Some(math_model::SourceSpan { start: 10, end: 20 }),
        },
        fidelity: FidelityIr::Unsupported,
        placement: None,
        content: BlockContentIr::Plot(PlotIr { preview: None }),
    }
}

fn v3(metadata: Vec<PlotMetadataIrV3>) -> VersionedDocumentIr {
    VersionedDocumentIr::v3(DocumentIrV3 {
        document: DocumentIrV1 {
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
                blocks: vec![plot_block("plot-7")],
            }],
        },
        plot_metadata: metadata,
    })
}

#[test]
fn v3_round_trip_preserves_confirmed_plot_metadata_and_v1_projection() {
    let value = v3(vec![PlotMetadataIrV3 {
        block_id: BlockId("plot-7".into()),
        item_idref: Some("plot-item-7".into()),
        disable_calc: true,
    }]);
    let first = value.to_json().expect("serialize V2");
    let second = value.to_json().expect("serialize V2 again");
    assert_eq!(first, second);
    let json: serde_json::Value = serde_json::from_slice(&first).unwrap();
    assert_eq!(json["schema_version"], 3);
    let decoded = VersionedDocumentIr::from_json(&first).expect("read V3");
    assert_eq!(decoded, value);
    assert_eq!(decoded.schema_version(), 3);
    assert!(matches!(
        decoded.as_v1().pages[0].blocks[0].content,
        BlockContentIr::Plot(_)
    ));
}

#[test]
fn v3_rejects_missing_wrong_order_and_oversized_plot_metadata() {
    assert_eq!(
        v3(Vec::new()).validate(),
        Err(DocumentIrValidationError::InvalidPlotMetadata)
    );
    assert_eq!(
        v3(vec![PlotMetadataIrV3 {
            block_id: BlockId("wrong".into()),
            item_idref: None,
            disable_calc: false,
        }])
        .validate(),
        Err(DocumentIrValidationError::InvalidPlotMetadata)
    );
    assert_eq!(
        v3(vec![PlotMetadataIrV3 {
            block_id: BlockId("plot-7".into()),
            item_idref: Some("x".repeat(129)),
            disable_calc: false,
        }])
        .validate(),
        Err(DocumentIrValidationError::InvalidPlotMetadata)
    );
}

#[test]
fn v3_debug_redacts_source_item_reference() {
    let value = PlotMetadataIrV3 {
        block_id: BlockId("plot-7".into()),
        item_idref: Some("SECRET_ITEM_REFERENCE".into()),
        disable_calc: false,
    };
    let debug = format!("{value:?}");
    assert!(debug.contains("has_item_idref: true"));
    assert!(!debug.contains("SECRET_ITEM_REFERENCE"));
}
