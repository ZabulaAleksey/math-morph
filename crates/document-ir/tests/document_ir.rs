use document_ir::ports::{AssetResolveError, AssetResolver, EquationExporter, ResolvedAsset};
use document_ir::*;
use math_model::{
    ExpressionOrigin, Identifier, MathExpression, MathExpressionKind, NumericBase, RealLiteral,
    SourceSpan,
};

fn expression(kind: MathExpressionKind) -> MathExpression {
    MathExpression {
        kind,
        origin: ExpressionOrigin::Derived,
    }
}

fn identifier(name: &str) -> MathExpression {
    expression(MathExpressionKind::Identifier(Identifier {
        name: name.into(),
        subscript: None,
    }))
}

fn real(lexeme: &str) -> MathExpression {
    expression(MathExpressionKind::Real(RealLiteral {
        lexeme: lexeme.into(),
        base: NumericBase::Decimal,
    }))
}

fn provenance() -> ProvenanceIr {
    ProvenanceIr {
        source_kind: SourceKindIr::Xmcd,
        region_id: Some(1),
        source_ordinal: Some(0),
        span: Some(SourceSpan { start: 10, end: 20 }),
    }
}

fn block(id: &str, content: BlockContentIr) -> BlockIr {
    BlockIr {
        id: BlockId(id.into()),
        provenance: provenance(),
        fidelity: FidelityIr::Exact,
        placement: None,
        content,
    }
}

fn page(blocks: Vec<BlockIr>) -> PageIr {
    PageIr {
        size: PhysicalSizeIr::letter_portrait(),
        orientation: PageOrientationIr::Portrait,
        margins: PageMarginsIr {
            top_um: 25_400,
            right_um: 25_400,
            bottom_um: 25_400,
            left_um: 25_400,
        },
        blocks,
    }
}

fn document(blocks: Vec<BlockIr>) -> VersionedDocumentIr {
    VersionedDocumentIr::v1(DocumentIrV1 {
        metadata: MetadataIr::default(),
        pages: vec![page(blocks)],
    })
}

#[test]
fn ac_055_v1_golden_round_trip_is_deterministic() {
    let value = document(Vec::new());
    let expected = include_str!("golden/document-ir-v1.json").trim();
    let first = value.to_json().expect("serialize V1");
    let second = value.to_json().expect("serialize V1 twice");
    assert_eq!(first, second);
    assert_eq!(std::str::from_utf8(&first).unwrap(), expected);

    let decoded = VersionedDocumentIr::from_json(expected.as_bytes()).expect("read V1 golden");
    assert!(decoded == value);
    assert_eq!(decoded.schema_version(), DOCUMENT_IR_SCHEMA_VERSION);
}

#[test]
fn ac_055_rejects_unknown_version_fields_and_size_limits() {
    let unknown_version = br#"{"schema_version":2,"document":{"metadata":{},"pages":[]}}"#;
    assert_eq!(
        VersionedDocumentIr::from_json(unknown_version),
        Err(DocumentIrError::UnsupportedVersion)
    );

    let unknown_field = br#"{"schema_version":1,"document":{"metadata":{"title":null,"creator":null,"description":null,"language":null,"keywords":[],"secret":"payload"},"pages":[]}}"#;
    assert_eq!(
        VersionedDocumentIr::from_json(unknown_field),
        Err(DocumentIrError::Malformed)
    );
    assert_eq!(
        VersionedDocumentIr::from_json_with_limit(b"{}", 1),
        Err(DocumentIrError::InputLimitExceeded)
    );
    assert_eq!(
        document(Vec::new()).to_json_with_limit(1),
        Err(DocumentIrError::OutputLimitExceeded)
    );
}

#[test]
fn ac_056_to_060_blocks_round_trip_without_binary_assets_or_paths() {
    let style = TextStyleIr {
        bold: true,
        italic: true,
        underline: true,
        strike: true,
        vertical_align: VerticalAlignIr::Superscript,
        font_family: Some("Secret Font".into()),
        font_size_half_points: Some(24),
        color: Some(RgbColorIr {
            red: 1,
            green: 2,
            blue: 3,
        }),
    };
    let image = ImageIr {
        asset: AssetRefIr {
            id: AssetId("asset-1".into()),
            media_type: MediaTypeIr::Png,
        },
        alt_text: Some("private caption".into()),
        size: Some(PhysicalSizeIr::new(10_000, 20_000).unwrap()),
    };
    let blocks = vec![
        block(
            "text",
            BlockContentIr::Text(TextBlockIr {
                paragraphs: vec![ParagraphIr {
                    runs: vec![TextRunIr {
                        text: "private text".into(),
                        style,
                    }],
                }],
            }),
        ),
        block(
            "equation",
            BlockContentIr::Equation(FormulaIr {
                original: Some(identifier("source-secret")),
                display: real("42"),
                mode: FormulaDisplayModeIr::Display,
            }),
        ),
        block("image", BlockContentIr::Image(image.clone())),
        block(
            "plot",
            BlockContentIr::Plot(PlotIr {
                preview: Some(image.clone()),
            }),
        ),
        block(
            "diagram",
            BlockContentIr::Diagram(DiagramIr {
                preview: Some(image),
                primitives: vec![DiagramPrimitiveIr {
                    kind: DiagramPrimitiveKindIr::Connector,
                    bounds: Some(BlockPlacementIr {
                        x_um: -1,
                        y_um: 2,
                        width_um: 3,
                        height_um: 4,
                        z_index: 5,
                        visual_ordinal: Some(6),
                    }),
                }],
            }),
        ),
        block(
            "table",
            BlockContentIr::Table(TableIr {
                rows: vec![TableRowIr {
                    cells: vec![TableCellIr {
                        blocks: vec![block(
                            "nested-text",
                            BlockContentIr::Text(TextBlockIr::default()),
                        )],
                    }],
                }],
            }),
        ),
        block(
            "unsupported",
            BlockContentIr::Unsupported(UnsupportedBlockIr {
                kind: "private future kind".into(),
            }),
        ),
    ];
    let value = document(blocks);
    value.validate().expect("rich V1 is valid");
    let json = value.to_json().expect("serialize rich V1");
    let decoded = VersionedDocumentIr::from_json(&json).expect("deserialize rich V1");
    assert!(decoded == value);
    assert!(!json.windows(7).any(|window| window == b"bytes\":"));
    assert!(!json.windows(6).any(|window| window == b"path\":"));
    assert!(!json.windows(5).any(|window| window == b"url\":"));

    let debug = format!("{value:?}");
    assert!(!debug.contains("private"));
    assert!(!debug.contains("source-secret"));
    assert!(!debug.contains("asset-1"));

    let ids: Vec<_> = decoded.as_v1().pages[0]
        .blocks
        .iter()
        .map(|block| block.id.0.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "text",
            "equation",
            "image",
            "plot",
            "diagram",
            "table",
            "unsupported"
        ]
    );
}

#[test]
fn ac_057_keeps_original_and_display_expressions_distinct() {
    let value = document(vec![block(
        "equation",
        BlockContentIr::Equation(FormulaIr {
            original: Some(identifier("x")),
            display: real("7"),
            mode: FormulaDisplayModeIr::Inline,
        }),
    )]);
    let decoded = VersionedDocumentIr::from_json(&value.to_json().unwrap()).unwrap();
    let BlockContentIr::Equation(formula) = &decoded.as_v1().pages[0].blocks[0].content else {
        panic!("equation expected")
    };
    assert!(matches!(
        formula.original.as_ref().map(|value| &value.kind),
        Some(MathExpressionKind::Identifier(_))
    ));
    assert!(matches!(formula.display.kind, MathExpressionKind::Real(_)));
}

#[test]
fn ac_061_rejects_invalid_geometry_ids_and_provenance() {
    assert_eq!(
        VersionedDocumentIr::v1(DocumentIrV1 {
            metadata: MetadataIr::default(),
            pages: Vec::new(),
        })
        .validate(),
        Err(DocumentIrValidationError::MissingPage)
    );

    let duplicate = document(vec![
        block("same", BlockContentIr::Text(TextBlockIr::default())),
        block("same", BlockContentIr::Text(TextBlockIr::default())),
    ]);
    assert_eq!(
        duplicate.validate(),
        Err(DocumentIrValidationError::DuplicateBlockId)
    );

    let mut invalid_placement = block("placed", BlockContentIr::Text(TextBlockIr::default()));
    invalid_placement.placement = Some(BlockPlacementIr {
        x_um: 0,
        y_um: 0,
        width_um: 0,
        height_um: 1,
        z_index: 0,
        visual_ordinal: None,
    });
    assert_eq!(
        document(vec![invalid_placement]).validate(),
        Err(DocumentIrValidationError::InvalidPlacement)
    );

    let mut invalid_span = block("span", BlockContentIr::Text(TextBlockIr::default()));
    invalid_span.provenance.span = Some(SourceSpan { start: 9, end: 2 });
    assert_eq!(
        document(vec![invalid_span]).validate(),
        Err(DocumentIrValidationError::InvalidProvenance)
    );
}

struct FakeEquationExporter;

impl EquationExporter for FakeEquationExporter {
    type Output = usize;
    type Error = ();

    fn export(&self, expression: &MathExpression) -> Result<Self::Output, Self::Error> {
        Ok(usize::from(matches!(
            expression.kind,
            MathExpressionKind::Identifier(_)
        )))
    }
}

struct FakeAssetResolver;

impl AssetResolver for FakeAssetResolver {
    fn resolve(&self, reference: &AssetRefIr) -> Result<ResolvedAsset, AssetResolveError> {
        Ok(ResolvedAsset {
            media_type: reference.media_type,
            bytes: vec![1, 2, 3],
        })
    }
}

#[test]
fn ac_058_and_070_ports_are_backend_neutral() {
    assert_eq!(FakeEquationExporter.export(&identifier("x")), Ok(1));
    let reference = AssetRefIr {
        id: AssetId("asset".into()),
        media_type: MediaTypeIr::Png,
    };
    let resolved = FakeAssetResolver.resolve(&reference).unwrap();
    assert_eq!(resolved.media_type, MediaTypeIr::Png);
    assert_eq!(resolved.bytes.len(), 3);
    assert!(!format!("{resolved:?}").contains("[1, 2, 3]"));
}
