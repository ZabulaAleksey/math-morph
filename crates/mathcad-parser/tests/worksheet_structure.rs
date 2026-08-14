use mathcad_parser::{
    CoordinateError, CustomValueKind, DiagnosticCode, InlineKind, MathParseOutcome, PictureKind,
    RegionContent, TextRun, WorksheetError, WorksheetLimit, WorksheetLimits, WorksheetParser,
};

const WS: &str = "http://schemas.mathsoft.com/worksheet30";
const ML: &str = "http://schemas.mathsoft.com/math30";

fn worksheet(body: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<x:worksheet xmlns:x="{WS}" xmlns:m="{ML}" version="3.0.3">{body}</x:worksheet>"#
    )
    .into_bytes()
}

fn region(id: u64, top: &str, left: &str, z: Option<i64>, content: &str) -> String {
    let z = z.map_or_else(String::new, |value| format!(r#" z-order="{value}""#));
    format!(
        r#"<x:region region-id="{id}" top="{top}" left="{left}" height="10" width="20"{z}>{content}</x:region>"#
    )
}

#[test]
fn ac_027_parses_reordered_metadata_and_rejects_version_or_qname() {
    let bytes = worksheet(
        r#"<x:metadata>
          <x:userData><x:title>classified title</x:title><x:author>Ada</x:author>
            <x:customValues><x:customValue name="department" type="text">R&amp;D</x:customValue></x:customValues>
          </x:userData>
          <x:futureMetadata/>
          <x:identityInfo><x:comment>opaque comment</x:comment><x:savedOn>2026-08-14</x:savedOn><x:documentID>doc-1</x:documentID></x:identityInfo>
          <x:generator>Mathcad</x:generator>
        </x:metadata><x:regions/>"#,
    );
    let parsed = WorksheetParser::default().parse(&bytes).expect("worksheet");
    let metadata = parsed.metadata.expect("metadata");
    assert_eq!(metadata.generator.as_deref(), Some("Mathcad"));
    assert_eq!(metadata.identity_info.document_id.as_deref(), Some("doc-1"));
    assert!(metadata.identity_info.comment.is_some());
    assert_eq!(metadata.user_data.author.as_deref(), Some("Ada"));
    assert_eq!(metadata.user_data.custom_values.len(), 1);
    assert_eq!(
        metadata.user_data.custom_values[0].kind,
        CustomValueKind::Text
    );
    assert_eq!(metadata.user_data.custom_values[0].name, "department");
    assert_eq!(metadata.user_data.custom_values[0].value, "R&D");
    assert_eq!(metadata.opaque_fragments.len(), 1);

    let wrong_version = String::from_utf8(bytes.clone())
        .expect("UTF-8")
        .replace("3.0.3", "3.0.2");
    assert_eq!(
        WorksheetParser::default().parse(wrong_version.as_bytes()),
        Err(WorksheetError::UnsupportedVersion)
    );
    let wrong_namespace = String::from_utf8(bytes)
        .expect("UTF-8")
        .replace(WS, "http://schemas.mathsoft.com/worksheet20");
    assert_eq!(
        WorksheetParser::default().parse(wrong_namespace.as_bytes()),
        Err(WorksheetError::UnsupportedRoot)
    );
}

#[test]
fn ac_028_to_030_discovers_nested_regions_and_keeps_three_orders_distinct() {
    let nested = region(
        2,
        "1",
        "5",
        Some(20),
        "<x:text><x:p style=\"Normal\">nested</x:p></x:text>",
    );
    let area = region(
        1,
        "5",
        "0",
        Some(10),
        &format!(r#"<x:area>{nested}</x:area>"#),
    );
    let tied = region(
        3,
        "1.0",
        "5.0",
        Some(-1),
        "<x:text><x:p style=\"Normal\">tie</x:p></x:text>",
    );
    let last = region(
        4,
        "0",
        "9",
        None,
        "<x:text><x:p style=\"Normal\">last</x:p></x:text>",
    );
    let bytes = worksheet(&format!(r#"<x:regions>{area}{tied}{last}</x:regions>"#));
    let parsed = WorksheetParser::default().parse(&bytes).expect("worksheet");

    assert_eq!(
        parsed
            .regions
            .iter()
            .map(|region| region.id)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert!(matches!(parsed.regions[0].content, RegionContent::Area(_)));
    assert_eq!(parsed.regions[3].layout.z_order, 0);
    assert_eq!(parsed.regions[1].layout.top.lexeme, "1");
    assert_eq!(
        parsed
            .visual_order()
            .iter()
            .map(|region| region.id)
            .collect::<Vec<_>>(),
        [4, 2, 3, 1]
    );
    assert_eq!(
        parsed
            .z_order()
            .iter()
            .map(|region| region.id)
            .collect::<Vec<_>>(),
        [3, 4, 1, 2]
    );
}

#[test]
fn ac_029_rejects_missing_malformed_and_non_finite_layout_and_duplicate_ids() {
    let missing = worksheet(
        r#"<x:regions><x:region region-id="1" left="0" height="1" width="1"><x:text/></x:region></x:regions>"#,
    );
    assert_eq!(
        WorksheetParser::default().parse(&missing),
        Err(WorksheetError::InvalidCoordinate {
            field: "top",
            reason: CoordinateError::Missing
        })
    );
    for (lexeme, reason) in [
        ("not-a-number", CoordinateError::Malformed),
        ("NaN", CoordinateError::NonFinite),
        ("inf", CoordinateError::NonFinite),
    ] {
        let bytes = worksheet(&format!(
            "<x:regions>{}</x:regions>",
            region(1, lexeme, "0", None, "<x:text/>")
        ));
        assert_eq!(
            WorksheetParser::default().parse(&bytes),
            Err(WorksheetError::InvalidCoordinate {
                field: "top",
                reason
            })
        );
    }

    let same = region(7, "0", "0", None, "<x:text/>");
    let duplicate = worksheet(&format!(r#"<x:regions>{same}{same}</x:regions>"#));
    assert_eq!(
        WorksheetParser::default().parse(&duplicate),
        Err(WorksheetError::DuplicateRegionId)
    );
}

#[test]
fn ac_031_preserves_mixed_text_and_inline_order_with_unknown_fallback() {
    let content = r#"<x:text><x:p style="Body">before<x:b font="bold-font">bold<x:i>italic</x:i></x:b><x:sp count="3"/><x:br/>after &amp; end<x:future>opaque</x:future></x:p></x:text>"#;
    let bytes = worksheet(&format!(
        "<x:regions>{}</x:regions>",
        region(1, "0", "0", None, content)
    ));
    let parsed = WorksheetParser::default().parse(&bytes).expect("worksheet");
    let RegionContent::Text(text) = &parsed.regions[0].content else {
        panic!("text region expected")
    };
    let runs = &text.paragraphs[0].runs;
    assert_eq!(text.paragraphs[0].style.0, "Body");
    assert!(matches!(runs[0], TextRun::Text { .. }));
    assert!(matches!(
        runs[1],
        TextRun::Inline {
            kind: InlineKind::Bold,
            ..
        }
    ));
    assert!(matches!(
        runs[2],
        TextRun::Inline {
            kind: InlineKind::Space,
            ..
        }
    ));
    let TextRun::Inline { attributes, .. } = &runs[1] else {
        unreachable!()
    };
    assert_eq!(attributes[0].name.local_name, "font");
    assert_eq!(attributes[0].value.0, "bold-font");
    assert!(matches!(runs.last(), Some(TextRun::Opaque(_))));
    assert_eq!(
        parsed
            .diagnostics
            .iter()
            .map(|item| item.code)
            .collect::<Vec<_>>(),
        [DiagnosticCode::UnknownInlineNode]
    );
}

#[test]
fn ac_032_to_035_classifies_math_plot_picture_table_program_and_unknown() {
    let math = region(
        1,
        "0",
        "0",
        None,
        r#"<x:math disable-calc="true" optimize="1"><m:real>1</m:real><x:resultFormat><x:table item-idref="table-1"/></x:resultFormat></x:math>"#,
    );
    let plot = region(
        2,
        "1",
        "0",
        None,
        r#"<x:plot item-idref="plot-1" disable-calc="1"/>"#,
    );
    let picture = region(
        3,
        "2",
        "0",
        None,
        r#"<x:picture><x:png item-idref="image-1" display-width="640.0" display-height="480"/></x:picture>"#,
    );
    let program = region(
        4,
        "3",
        "0",
        None,
        r#"<x:math><m:program><m:id>x</m:id></m:program></x:math>"#,
    );
    let unknown = region(5, "4", "0", None, r#"<x:future-region secret="payload"/>"#);
    let bytes = worksheet(&format!(
        r#"<x:regions>{math}{plot}{picture}{program}{unknown}</x:regions>"#
    ));
    let parsed = WorksheetParser::default().parse(&bytes).expect("worksheet");

    let RegionContent::Math(math) = &parsed.regions[0].content else {
        panic!("math expected")
    };
    assert!(math.disable_calc && math.optimize);
    assert!(matches!(math.outcome, MathParseOutcome::Pending));
    assert!(math.expression_span.start < math.expression_span.end);
    let table = math
        .result_format
        .as_ref()
        .and_then(|result| result.table.as_ref())
        .expect("opaque table result");
    assert_eq!(table.item_idref, "table-1");
    assert!(parsed.source.bytes(math.expression_span).is_some());

    let RegionContent::Plot(plot) = &parsed.regions[1].content else {
        panic!("plot expected")
    };
    assert_eq!(plot.item_idref.as_deref(), Some("plot-1"));
    assert!(plot.disable_calc);

    let RegionContent::Picture(picture) = &parsed.regions[2].content else {
        panic!("picture expected")
    };
    assert_eq!(picture.kind, PictureKind::Png);
    assert_eq!(
        picture.display_width.as_ref().map(|value| value.value),
        Some(640.0)
    );
    assert_eq!(
        picture
            .display_height
            .as_ref()
            .map(|value| value.lexeme.as_str()),
        Some("480")
    );

    let RegionContent::Math(program) = &parsed.regions[3].content else {
        panic!("program must remain math")
    };
    assert!(matches!(program.outcome, MathParseOutcome::Unsupported(_)));
    assert!(matches!(
        parsed.regions[4].content,
        RegionContent::Opaque(_)
    ));
    assert_eq!(
        parsed
            .diagnostics
            .iter()
            .map(|item| item.code)
            .collect::<Vec<_>>(),
        [
            DiagnosticCode::UnsupportedMathNode,
            DiagnosticCode::UnknownRegionContent
        ]
    );
    let rendered_regions = format!("{:?}", parsed.regions);
    for payload in ["table-1", "plot-1", "image-1", "payload"] {
        assert!(!rendered_regions.contains(payload));
    }
}

#[test]
fn ac_034_distinguishes_all_picture_kinds_and_validates_kind_specific_metadata() {
    let jpg = region(
        1,
        "0",
        "0",
        None,
        r#"<x:picture><x:jpg item-idref="jpg-ref" quality="90"/></x:picture>"#,
    );
    let metafile = region(
        2,
        "1",
        "0",
        None,
        r#"<x:picture><x:metafile item-idref="wmf-ref" mapping-mode="8" x-extent="12.5" y-extent="7"/></x:picture>"#,
    );
    let parsed = WorksheetParser::default()
        .parse(&worksheet(&format!(
            "<x:regions>{jpg}{metafile}</x:regions>"
        )))
        .expect("picture kinds");
    let RegionContent::Picture(jpg) = &parsed.regions[0].content else {
        panic!("JPG expected")
    };
    assert_eq!((jpg.kind, jpg.quality), (PictureKind::Jpg, Some(90)));
    let RegionContent::Picture(metafile) = &parsed.regions[1].content else {
        panic!("metafile expected")
    };
    assert_eq!(metafile.kind, PictureKind::Metafile);
    assert_eq!(metafile.mapping_mode, Some(8));
    assert_eq!(
        metafile
            .x_extent
            .as_ref()
            .map(|value| value.lexeme.as_str()),
        Some("12.5")
    );

    let invalid = worksheet(&format!(
        "<x:regions>{}</x:regions>",
        region(
            3,
            "0",
            "0",
            None,
            r#"<x:picture><x:metafile item-idref="missing-metadata"/></x:picture>"#
        )
    ));
    assert_eq!(
        WorksheetParser::default().parse(&invalid),
        Err(WorksheetError::MalformedPicture)
    );

    let invented_shape = worksheet(&format!(
        "<x:regions>{}</x:regions>",
        region(
            1,
            "0",
            "0",
            None,
            r#"<x:picture kind="png" item-idref="not-an-xsd-shape"/>"#
        )
    ));
    assert_eq!(
        WorksheetParser::default().parse(&invented_shape),
        Err(WorksheetError::MalformedPicture)
    );
}

#[test]
fn rejects_dtd_entities_mixed_math_namespace_and_enforces_limits_without_payload_leak() {
    let dtd = format!(
        r#"<!DOCTYPE x:worksheet [<!ENTITY leak SYSTEM "file:///secret">]><x:worksheet xmlns:x="{WS}" version="3.0.3">&leak;</x:worksheet>"#
    );
    assert_eq!(
        WorksheetParser::default().parse(dtd.as_bytes()),
        Err(WorksheetError::DoctypeForbidden)
    );

    let malformed_namespace = br#"<worksheet xmlns="http://schemas.mathsoft.com/worksheet30" xmlns:bad="&undefined;" version="3.0.3"><regions/></worksheet>"#;
    assert_eq!(
        WorksheetParser::default().parse(malformed_namespace),
        Err(WorksheetError::MalformedXml)
    );

    let mixed = worksheet(&format!(
        r#"<x:regions>{}</x:regions>"#,
        region(
            1,
            "0",
            "0",
            None,
            r#"<x:math><bad:real xmlns:bad="http://schemas.mathsoft.com/math20">secret</bad:real></x:math>"#
        )
    ));
    assert_eq!(
        WorksheetParser::default().parse(&mixed),
        Err(WorksheetError::UnsupportedMathNamespace)
    );

    let bytes = worksheet("<x:regions/>");
    let input_limited = WorksheetParser::new(WorksheetLimits {
        max_input_bytes: bytes.len() - 1,
        ..WorksheetLimits::default()
    });
    let error = input_limited.parse(&bytes).expect_err("input limit");
    assert_eq!(
        error,
        WorksheetError::LimitExceeded(WorksheetLimit::InputBytes)
    );

    let nested = worksheet(
        "<x:metadata><x:userData><x:title>secret sentinel</x:title></x:userData></x:metadata>",
    );
    let depth_limited = WorksheetParser::new(WorksheetLimits {
        max_xml_depth: 2,
        ..WorksheetLimits::default()
    });
    let error = depth_limited.parse(&nested).expect_err("depth limit");
    let rendered = format!("{error:?} {error}");
    assert!(!rendered.contains("secret sentinel"));

    let parsed = WorksheetParser::default()
        .parse(&nested)
        .expect("valid worksheet");
    assert!(!format!("{parsed:?}").contains("secret sentinel"));

    let empty_leaf = worksheet("<x:regions/>");
    let shallow = WorksheetParser::new(WorksheetLimits {
        max_xml_depth: 1,
        ..WorksheetLimits::default()
    });
    assert_eq!(
        shallow.parse(&empty_leaf),
        Err(WorksheetError::LimitExceeded(WorksheetLimit::XmlDepth))
    );

    let malformed_boolean = worksheet(&format!(
        "<x:regions>{}</x:regions>",
        region(1, "0", "0", None, r#"<x:plot disable-calc="sometimes"/>"#)
    ));
    assert_eq!(
        WorksheetParser::default().parse(&malformed_boolean),
        Err(WorksheetError::MalformedBoolean)
    );

    let missing_style = worksheet(&format!(
        "<x:regions>{}</x:regions>",
        region(1, "0", "0", None, "<x:text><x:p>text</x:p></x:text>")
    ));
    assert_eq!(
        WorksheetParser::default().parse(&missing_style),
        Err(WorksheetError::MissingTextStyle)
    );
}

#[test]
fn all_worksheet_resource_limits_fail_closed() {
    let one_region = worksheet(&format!(
        "<x:regions>{}</x:regions>",
        region(
            1,
            "0",
            "0",
            None,
            "<x:text><x:p style=\"Normal\">payload</x:p></x:text>"
        )
    ));
    for (limits, expected) in [
        (
            WorksheetLimits {
                max_xml_nodes: 1,
                ..WorksheetLimits::default()
            },
            WorksheetLimit::XmlNodes,
        ),
        (
            WorksheetLimits {
                max_regions: 0,
                ..WorksheetLimits::default()
            },
            WorksheetLimit::Regions,
        ),
        (
            WorksheetLimits {
                max_attributes_per_element: 0,
                ..WorksheetLimits::default()
            },
            WorksheetLimit::Attributes,
        ),
        (
            WorksheetLimits {
                max_attribute_value_bytes: 2,
                ..WorksheetLimits::default()
            },
            WorksheetLimit::AttributeValueBytes,
        ),
        (
            WorksheetLimits {
                max_token_bytes: 3,
                ..WorksheetLimits::default()
            },
            WorksheetLimit::TokenBytes,
        ),
        (
            WorksheetLimits {
                max_retained_text_bytes: 0,
                ..WorksheetLimits::default()
            },
            WorksheetLimit::RetainedTextBytes,
        ),
    ] {
        assert_eq!(
            WorksheetParser::new(limits).parse(&one_region),
            Err(WorksheetError::LimitExceeded(expected))
        );
    }

    let namespaces = br#"<worksheet xmlns="http://schemas.mathsoft.com/worksheet30" xmlns:a="a" xmlns:b="b" version="3.0.3"><regions/></worksheet>"#;
    let limits = WorksheetLimits {
        max_namespace_declarations: 2,
        ..WorksheetLimits::default()
    };
    assert_eq!(
        WorksheetParser::new(limits).parse(namespaces),
        Err(WorksheetError::LimitExceeded(
            WorksheetLimit::NamespaceDeclarations
        ))
    );
}
