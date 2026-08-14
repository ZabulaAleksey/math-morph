use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

use document_ir::ports::{AssetResolveError, AssetResolver, ResolvedAsset};
use document_ir::{
    AssetId, AssetRefIr, BlockContentIr, BlockId, BlockIr, DocumentIrV1, FidelityIr, ImageIr,
    MediaTypeIr, MetadataIr, PageIr, PageMarginsIr, PageOrientationIr, ParagraphIr, PhysicalSizeIr,
    ProvenanceIr, RgbColorIr, SourceKindIr, TextBlockIr, TextRunIr, TextStyleIr, VerticalAlignIr,
};
use exporter_docx::{
    DocxError, DocxExporter, DocxLimit, DocxLimits, DocxValidationError, DocxValidator,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

struct NoAssets;

impl AssetResolver for NoAssets {
    fn resolve(&self, _: &AssetRefIr) -> Result<ResolvedAsset, AssetResolveError> {
        Err(AssetResolveError::Unavailable)
    }
}

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

fn text_block(id: &str, paragraphs: Vec<ParagraphIr>) -> BlockIr {
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
        content: BlockContentIr::Text(TextBlockIr { paragraphs }),
    }
}

fn image_block(
    id: &str,
    asset_id: &str,
    media_type: MediaTypeIr,
    size: Option<PhysicalSizeIr>,
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
        content: BlockContentIr::Image(ImageIr {
            asset: AssetRefIr {
                id: AssetId(asset_id.to_owned()),
                media_type,
            },
            alt_text: Some("safe & useful".to_owned()),
            size,
        }),
    }
}

fn run(text: &str, style: TextStyleIr) -> TextRunIr {
    TextRunIr {
        text: text.to_owned(),
        style,
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

fn package(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, value) in entries {
        writer.start_file(name, options).unwrap();
        writer.write_all(value).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn replace_part(bytes: &[u8], name: &str, replacement: &[u8]) -> Vec<u8> {
    let owned = parts(bytes);
    let entries = owned
        .iter()
        .map(|(part_name, value)| {
            (
                part_name.as_str(),
                if part_name == name {
                    replacement
                } else {
                    value
                },
            )
        })
        .collect::<Vec<_>>();
    package(&entries)
}

fn png(width: u32, height: u32, metadata: bool) -> Vec<u8> {
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
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut output, b"IHDR", &header);
    if metadata {
        chunk(&mut output, b"tEXt", b"secret=value");
    }
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

fn jpeg(width: u16, height: u16) -> Vec<u8> {
    let mut output = vec![
        0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xff, 0xc0, 0x00, 0x0b, 0x08,
    ];
    output.extend_from_slice(&height.to_be_bytes());
    output.extend_from_slice(&width.to_be_bytes());
    output.extend_from_slice(&[0x01, 0x01, 0x11, 0x00]);
    output.extend_from_slice(&[0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3f, 0x00]);
    output.extend_from_slice(&[0x00, 0xff, 0xd9]);
    output
}

#[test]
fn minimal_docx_is_deterministic_and_valid() {
    let document = document(Vec::new());
    let exporter = DocxExporter::default();
    let first = exporter.export(&document, &NoAssets).unwrap();
    let second = exporter.export(&document, &NoAssets).unwrap();

    assert_eq!(first, second);
    DocxValidator::default().validate(&first).unwrap();
    assert_eq!(
        parts(&first)
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["[Content_Types].xml", "_rels/.rels", "word/document.xml"]
    );
}

#[test]
fn text_order_escaping_whitespace_and_styles_are_preserved() {
    let style = TextStyleIr {
        bold: true,
        italic: true,
        underline: true,
        strike: true,
        vertical_align: VerticalAlignIr::Superscript,
        font_family: Some("A&B\"".to_owned()),
        font_size_half_points: Some(24),
        color: Some(RgbColorIr {
            red: 0x12,
            green: 0xAB,
            blue: 0xEF,
        }),
    };
    let document = document(vec![text_block(
        "text-1",
        vec![
            ParagraphIr {
                runs: vec![run(" <first> & ", style)],
            },
            ParagraphIr {
                runs: vec![
                    run("second", TextStyleIr::default()),
                    run("", TextStyleIr::default()),
                ],
            },
        ],
    )]);
    let bytes = DocxExporter::default()
        .export(&document, &NoAssets)
        .unwrap();
    let document_xml = parts(&bytes)
        .into_iter()
        .find(|(name, _)| name == "word/document.xml")
        .unwrap()
        .1;
    let xml = String::from_utf8(document_xml).unwrap();

    assert!(xml.contains("<w:t xml:space=\"preserve\"> &lt;first&gt; &amp; </w:t>"));
    assert!(xml.contains("<w:b/><w:i/><w:u w:val=\"single\"/><w:strike/>"));
    assert!(xml.contains("<w:vertAlign w:val=\"superscript\"/>"));
    assert!(xml.contains("<w:rFonts w:ascii=\"A&amp;B&quot;\" w:hAnsi=\"A&amp;B&quot;\"/>"));
    assert!(xml.contains("<w:sz w:val=\"24\"/><w:color w:val=\"12ABEF\"/>"));
    assert!(xml.contains("</w:p><w:p>"));
    assert!(xml.contains("<w:t>second</w:t></w:r><w:r><w:t/></w:r>"));
}

#[test]
fn invalid_xml_character_is_rejected_without_payload_in_error() {
    let secret = "secret\u{1}value";
    let document = document(vec![text_block(
        "text-1",
        vec![ParagraphIr {
            runs: vec![run(secret, TextStyleIr::default())],
        }],
    )]);
    let error = DocxExporter::default()
        .export(&document, &NoAssets)
        .unwrap_err();
    let rendered = format!("{error:?} {error}");
    assert!(!rendered.contains("secret"));
}

#[test]
fn png_and_jpeg_are_embedded_with_internal_relationships_and_exact_emu() {
    let png_bytes = png(2, 3, false);
    let jpeg_bytes = jpeg(4, 5);
    let assets = Assets(BTreeMap::from([
        (
            "asset-png".to_owned(),
            ResolvedAsset {
                media_type: MediaTypeIr::Png,
                bytes: png_bytes.clone(),
            },
        ),
        (
            "asset-jpeg".to_owned(),
            ResolvedAsset {
                media_type: MediaTypeIr::Jpeg,
                bytes: jpeg_bytes.clone(),
            },
        ),
    ]));
    let document = document(vec![
        image_block(
            "image-1",
            "asset-png",
            MediaTypeIr::Png,
            PhysicalSizeIr::new(1_000, 2_000),
        ),
        image_block(
            "image-2",
            "asset-jpeg",
            MediaTypeIr::Jpeg,
            PhysicalSizeIr::new(3_000, 4_000),
        ),
    ]);

    let bytes = DocxExporter::default().export(&document, &assets).unwrap();
    DocxValidator::default().validate(&bytes).unwrap();
    let parts = parts(&bytes).into_iter().collect::<BTreeMap<_, _>>();
    assert_eq!(parts["word/media/image1.png"], png_bytes);
    assert_eq!(parts["word/media/image2.jpg"], jpeg_bytes);
    let content_types = String::from_utf8(parts["[Content_Types].xml"].clone()).unwrap();
    assert!(content_types.contains("Extension=\"png\" ContentType=\"image/png\""));
    assert!(content_types.contains("Extension=\"jpg\" ContentType=\"image/jpeg\""));
    let relationships = String::from_utf8(parts["word/_rels/document.xml.rels"].clone()).unwrap();
    assert!(relationships.contains("Id=\"rId1\"") && relationships.contains("media/image1.png"));
    assert!(relationships.contains("Id=\"rId2\"") && relationships.contains("media/image2.jpg"));
    assert!(!relationships.contains("TargetMode"));
    let document_xml = String::from_utf8(parts["word/document.xml"].clone()).unwrap();
    assert!(document_xml.contains("wp:extent cx=\"36000\" cy=\"72000\""));
    assert!(document_xml.contains("a:ext cx=\"108000\" cy=\"144000\""));
    assert!(document_xml.contains("r:embed=\"rId1\""));
    assert!(document_xml.contains("descr=\"safe &amp; useful\""));
}

#[test]
fn image_failures_are_typed_and_redacted() {
    let size = PhysicalSizeIr::new(1_000, 1_000);
    let base = document(vec![image_block(
        "image-1",
        "secret-asset",
        MediaTypeIr::Png,
        size,
    )]);
    assert_eq!(
        DocxExporter::default().export(&base, &NoAssets),
        Err(DocxError::MissingAsset)
    );
    let mismatched = Assets(BTreeMap::from([(
        "secret-asset".to_owned(),
        ResolvedAsset {
            media_type: MediaTypeIr::Jpeg,
            bytes: jpeg(1, 1),
        },
    )]));
    assert_eq!(
        DocxExporter::default().export(&base, &mismatched),
        Err(DocxError::MediaTypeMismatch)
    );
    let metadata = Assets(BTreeMap::from([(
        "secret-asset".to_owned(),
        ResolvedAsset {
            media_type: MediaTypeIr::Png,
            bytes: png(1, 1, true),
        },
    )]));
    let error = DocxExporter::default()
        .export(&base, &metadata)
        .unwrap_err();
    assert_eq!(error, DocxError::ImageMetadataForbidden);
    assert!(!format!("{error:?} {error}").contains("secret"));

    let missing_size = document(vec![image_block(
        "image-1",
        "secret-asset",
        MediaTypeIr::Png,
        None,
    )]);
    assert_eq!(
        DocxExporter::default().export(&missing_size, &metadata),
        Err(DocxError::MissingImageSize)
    );
}

#[test]
fn duplicate_assets_and_image_limits_fail_closed() {
    let image = image_block(
        "image-1",
        "same",
        MediaTypeIr::Png,
        PhysicalSizeIr::new(1_000, 1_000),
    );
    let mut duplicate = image.clone();
    duplicate.id = BlockId("image-2".to_owned());
    let assets = Assets(BTreeMap::from([(
        "same".to_owned(),
        ResolvedAsset {
            media_type: MediaTypeIr::Png,
            bytes: png(2, 2, false),
        },
    )]));
    assert_eq!(
        DocxExporter::default().export(&document(vec![image, duplicate]), &assets),
        Err(DocxError::DuplicateAssetId)
    );

    let limits = DocxLimits {
        max_image_pixels: 3,
        ..DocxLimits::default()
    };
    assert_eq!(
        DocxExporter::new(limits).export(
            &document(vec![image_block(
                "image-1",
                "same",
                MediaTypeIr::Png,
                PhysicalSizeIr::new(1_000, 1_000),
            )]),
            &assets,
        ),
        Err(DocxError::LimitExceeded(DocxLimit::ImagePixels))
    );
}

#[test]
fn page_section_is_final_and_multiple_pages_or_overflow_are_rejected() {
    let mut page_document = document(Vec::new());
    page_document.pages[0].size = PhysicalSizeIr::new(279_400, 215_900).unwrap();
    page_document.pages[0].orientation = PageOrientationIr::Landscape;
    let bytes = DocxExporter::default()
        .export(&page_document, &NoAssets)
        .unwrap();
    let xml = parts(&bytes)
        .into_iter()
        .find(|(name, _)| name == "word/document.xml")
        .map(|(_, value)| String::from_utf8(value).unwrap())
        .unwrap();
    assert!(xml.ends_with("</w:sectPr></w:body></w:document>"));
    assert!(xml.contains("<w:pgSz w:w=\"15840\" w:h=\"12240\" w:orient=\"landscape\"/>"));
    assert!(
        xml.contains(
            "<w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/>"
        )
    );

    page_document.pages.push(page_document.pages[0].clone());
    assert_eq!(
        DocxExporter::default().export(&page_document, &NoAssets),
        Err(DocxError::MultiplePagesUnsupported)
    );

    let overflow_asset = Assets(BTreeMap::from([(
        "overflow".to_owned(),
        ResolvedAsset {
            media_type: MediaTypeIr::Png,
            bytes: png(1, 1, false),
        },
    )]));
    let overflow = document(vec![image_block(
        "image-1",
        "overflow",
        MediaTypeIr::Png,
        PhysicalSizeIr::new(u64::MAX / 36 + 1, 1),
    )]);
    assert_eq!(
        DocxExporter::default().export(&overflow, &overflow_asset),
        Err(DocxError::ArithmeticOverflow)
    );
}

#[test]
fn validator_rejects_package_and_xml_attacks() {
    let valid = DocxExporter::default()
        .export(&document(Vec::new()), &NoAssets)
        .unwrap();
    let valid_parts = parts(&valid);
    let content_types = valid_parts[0].1.as_slice();
    let root_rels = valid_parts[1].1.as_slice();
    let document_xml = valid_parts[2].1.as_slice();

    assert_eq!(
        DocxValidator::default().validate(&package(&[
            ("[Content_Types].xml", content_types),
            ("_rels/.rels", root_rels),
        ])),
        Err(DocxValidationError::MissingRequiredPart)
    );
    assert_eq!(
        DocxValidator::default().validate(&package(&[
            ("[Content_Types].xml", content_types),
            ("[content_types].xml", content_types),
            ("_rels/.rels", root_rels),
            ("word/document.xml", document_xml),
        ])),
        Err(DocxValidationError::DuplicatePart)
    );
    assert_eq!(
        DocxValidator::default().validate(&package(&[
            ("[Content_Types].xml", content_types),
            ("_rels/.rels", root_rels),
            ("word/document.xml", document_xml),
            ("../escape.xml", b"<x/>"),
        ])),
        Err(DocxValidationError::UnsafePartName)
    );
    let external = root_rels.windows(b" Target=\"".len()).next();
    assert!(external.is_some());
    let external_rels = String::from_utf8(root_rels.to_vec())
        .unwrap()
        .replace(" Target=\"", " TargetMode=\"External\" Target=\"");
    assert_eq!(
        DocxValidator::default().validate(&replace_part(
            &valid,
            "_rels/.rels",
            external_rels.as_bytes()
        )),
        Err(DocxValidationError::ExternalRelationship)
    );
    let dtd = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE w:document [<!ENTITY x SYSTEM \"file:///secret\">]><w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>&x;</w:body></w:document>";
    assert_eq!(
        DocxValidator::default().validate(&replace_part(&valid, "word/document.xml", dtd)),
        Err(DocxValidationError::DtdForbidden)
    );
    assert!(matches!(
        DocxValidator::default().validate(&replace_part(
            &valid,
            "word/document.xml",
            b"<w:document>"
        )),
        Err(DocxValidationError::InvalidDocumentXml)
    ));
    assert_eq!(
        DocxValidator::default().validate(&package(&[
            ("[Content_Types].xml", content_types),
            ("_rels/.rels", root_rels),
            ("word/document.xml", document_xml),
            ("word/vbaProject.bin", b"active"),
        ])),
        Err(DocxValidationError::ForbiddenContent)
    );
}

#[test]
fn validator_rejects_broken_image_relationship() {
    let assets = Assets(BTreeMap::from([(
        "image".to_owned(),
        ResolvedAsset {
            media_type: MediaTypeIr::Png,
            bytes: png(1, 1, false),
        },
    )]));
    let valid = DocxExporter::default()
        .export(
            &document(vec![image_block(
                "image-1",
                "image",
                MediaTypeIr::Png,
                PhysicalSizeIr::new(1_000, 1_000),
            )]),
            &assets,
        )
        .unwrap();
    let relationships = parts(&valid)
        .into_iter()
        .find(|(name, _)| name == "word/_rels/document.xml.rels")
        .map(|(_, value)| String::from_utf8(value).unwrap())
        .unwrap()
        .replace("media/image1.png", "media/image9.png");
    assert_eq!(
        DocxValidator::default().validate(&replace_part(
            &valid,
            "word/_rels/document.xml.rels",
            relationships.as_bytes(),
        )),
        Err(DocxValidationError::BrokenRelationship)
    );
}

#[test]
fn output_and_validator_limits_are_enforced_before_unbounded_writes() {
    let document = document(Vec::new());
    let valid = DocxExporter::default()
        .export(&document, &NoAssets)
        .unwrap();
    let uncompressed_bytes = parts(&valid)
        .iter()
        .map(|(_, value)| value.len() as u64)
        .sum();
    let export_limits = DocxLimits {
        max_output_bytes: uncompressed_bytes,
        ..DocxLimits::default()
    };
    assert_eq!(
        DocxExporter::new(export_limits).export(&document, &NoAssets),
        Err(DocxError::LimitExceeded(DocxLimit::OutputBytes))
    );

    let validation_limits = DocxLimits {
        max_output_bytes: valid.len() as u64 - 1,
        ..DocxLimits::default()
    };
    assert_eq!(
        DocxValidator::new(validation_limits).validate(&valid),
        Err(DocxValidationError::LimitExceeded(DocxLimit::OutputBytes))
    );
}

#[test]
fn validator_compares_expanded_qnames_and_drawing_ids() {
    let valid = DocxExporter::default()
        .export(&document(Vec::new()), &NoAssets)
        .unwrap();
    let content_types = parts(&valid)
        .into_iter()
        .find(|(name, _)| name == "[Content_Types].xml")
        .map(|(_, value)| String::from_utf8(value).unwrap())
        .unwrap()
        .replace(
            "http://schemas.openxmlformats.org/package/2006/content-types",
            "urn:wrong-content-types",
        );
    assert_eq!(
        DocxValidator::default().validate(&replace_part(
            &valid,
            "[Content_Types].xml",
            content_types.as_bytes(),
        )),
        Err(DocxValidationError::InvalidContentTypes)
    );

    let png_bytes = png(1, 1, false);
    let assets = Assets(BTreeMap::from([
        (
            "one".to_owned(),
            ResolvedAsset {
                media_type: MediaTypeIr::Png,
                bytes: png_bytes.clone(),
            },
        ),
        (
            "two".to_owned(),
            ResolvedAsset {
                media_type: MediaTypeIr::Png,
                bytes: png_bytes,
            },
        ),
    ]));
    let with_images = DocxExporter::default()
        .export(
            &document(vec![
                image_block(
                    "image-1",
                    "one",
                    MediaTypeIr::Png,
                    PhysicalSizeIr::new(1_000, 1_000),
                ),
                image_block(
                    "image-2",
                    "two",
                    MediaTypeIr::Png,
                    PhysicalSizeIr::new(1_000, 1_000),
                ),
            ]),
            &assets,
        )
        .unwrap();
    let document_xml = parts(&with_images)
        .into_iter()
        .find(|(name, _)| name == "word/document.xml")
        .map(|(_, value)| String::from_utf8(value).unwrap())
        .unwrap()
        .replace("id=\"2\" name=\"Image 2\"", "id=\"1\" name=\"Image 2\"");
    assert_eq!(
        DocxValidator::default().validate(&replace_part(
            &with_images,
            "word/document.xml",
            document_xml.as_bytes(),
        )),
        Err(DocxValidationError::DuplicateDrawingId)
    );
}
