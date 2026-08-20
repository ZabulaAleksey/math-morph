use conversion_core::{
    ConversionOptions, ConversionPipeline, ConversionRequest, DiagnosticCode, FailureCode,
    Fidelity, PartialPolicy, ReportStatus, TargetFormat,
};
use exporter_docx::DocxValidator;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const WS: &str = "http://schemas.mathsoft.com/worksheet30";
const ML: &str = "http://schemas.mathsoft.com/math30";

fn worksheet(regions: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><x:worksheet xmlns:x="{WS}" xmlns:m="{ML}" version="3.0.3"><x:regions>{regions}</x:regions></x:worksheet>"#
    )
    .into_bytes()
}

fn region(id: u64, top: &str, content: &str) -> String {
    format!(
        r#"<x:region region-id="{id}" top="{top}" left="0" height="10" width="20">{content}</x:region>"#
    )
}

fn convert(
    bytes: Vec<u8>,
    policy: PartialPolicy,
) -> Result<conversion_core::ConversionOutcome, conversion_core::ConversionFailure> {
    ConversionPipeline::new().convert(ConversionRequest {
        bytes,
        file_name: Some("example.xmcd".to_owned()),
        target: TargetFormat::Docx,
        options: ConversionOptions {
            partial_policy: policy,
            ..ConversionOptions::default()
        },
    })
}

#[test]
fn supported_xmcd_exports_valid_docx_and_editable_omml() {
    let input = worksheet(&region(
        1,
        "0",
        r#"<x:text><x:p style="Normal">Hello</x:p></x:text>"#,
    ));
    let output = convert(input, PartialPolicy::Strict).expect("conversion");
    DocxValidator::default()
        .validate(&output.artifact)
        .expect("valid docx");
    assert_eq!(output.report.status, ReportStatus::Completed);
    assert_eq!(output.report.items[0].fidelity, Fidelity::Approximate);
}

#[test]
fn supported_math_exports_real_omml_in_document_xml() {
    let input = worksheet(&region(
        1,
        "0",
        r#"<x:math><m:apply><m:plus/><m:real>1</m:real><m:real>2</m:real></m:apply></x:math>"#,
    ));
    let output = convert(input, PartialPolicy::Strict).expect("math conversion");
    let mut archive = ZipArchive::new(Cursor::new(output.artifact)).expect("docx zip");
    let mut document = String::new();
    archive
        .by_name("word/document.xml")
        .expect("document xml")
        .read_to_string(&mut document)
        .expect("utf8 document xml");
    assert!(document.contains("<m:oMath"));
    assert!(document.contains("<m:r"));
}

#[test]
fn unsupported_omml_equation_is_recoverable_only_under_partial_policy() {
    let input = worksheet(&region(
        1,
        "0",
        r#"<x:math><m:apply><m:power/><m:id>x</m:id><m:real>2</m:real></m:apply></x:math>"#,
    ));
    let strict = convert(input.clone(), PartialPolicy::Strict).expect_err("strict equation");
    assert_eq!(strict.code, FailureCode::StrictUnsupportedContent);
    let partial =
        convert(input, PartialPolicy::AllowSafePartial).expect_err("empty partial equation");
    assert_eq!(partial.code, FailureCode::NoExportableContent);
    assert!(
        partial
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::UnsupportedRegion)
    );
}

#[test]
fn mixed_content_is_safe_partial_and_deterministic() {
    let input = worksheet(&format!(
        "{}{}",
        region(
            1,
            "0",
            r#"<x:text><x:p style="Normal">Hello</x:p></x:text>"#
        ),
        region(2, "1", r#"<x:plot item-idref="plot-1"/>"#)
    ));
    let first = convert(input.clone(), PartialPolicy::AllowSafePartial).expect("partial");
    let second = convert(input, PartialPolicy::AllowSafePartial).expect("partial");
    assert_eq!(first.artifact, second.artifact);
    assert_eq!(first.report, second.report);
    assert_eq!(first.report.status, ReportStatus::CompletedWithWarnings);
    assert!(
        first
            .report
            .diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnsupportedRegion)
    );
    assert_eq!(first.report.items[1].fidelity, Fidelity::Unsupported);
}

#[test]
fn strict_and_all_unsupported_have_no_artifact() {
    let input = worksheet(&region(1, "0", r#"<x:plot item-idref="plot-1"/>"#));
    let strict = convert(input.clone(), PartialPolicy::Strict).expect_err("strict failure");
    assert_eq!(strict.code, FailureCode::StrictUnsupportedContent);
    let partial =
        convert(input, PartialPolicy::AllowSafePartial).expect_err("empty partial failure");
    assert_eq!(partial.code, FailureCode::NoExportableContent);
}

#[test]
fn mcdx_is_typed_unsupported_and_dtd_is_rejected() {
    let mut output = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut output));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer
            .start_file("mathcad/worksheet.xml", options)
            .expect("worksheet part");
        writer
            .write_all(b"<x:worksheet xmlns:x=\"http://schemas.mathsoft.com/worksheet30\" version=\"3.0.3\"/>")
            .expect("worksheet bytes");
        writer.finish().expect("mcdx zip");
    }
    let mcdx = output;
    let failure = ConversionPipeline::new()
        .convert(ConversionRequest {
            bytes: mcdx,
            file_name: Some("x.mcdx".to_owned()),
            target: TargetFormat::Docx,
            options: ConversionOptions::default(),
        })
        .expect_err("mcdx");
    assert_eq!(failure.code, FailureCode::McdxContentUnsupported);
    let dtd = format!(
        r#"<!DOCTYPE x:worksheet [<!ENTITY leak SYSTEM "file:///secret">]><x:worksheet xmlns:x="{WS}" version="3.0.3"><x:regions/></x:worksheet>"#
    );
    let failure = ConversionPipeline::new()
        .convert(ConversionRequest {
            bytes: dtd.into_bytes(),
            file_name: Some("x.xmcd".to_owned()),
            target: TargetFormat::Docx,
            options: ConversionOptions::default(),
        })
        .expect_err("dtd");
    assert_eq!(failure.code, FailureCode::InvalidInput);
    assert!(!format!("{failure:?}").contains("secret"));
}

#[test]
fn report_model_covers_all_fidelity_values_and_keeps_diagnostic_order() {
    assert_eq!(Fidelity::Exact, Fidelity::Exact);
    assert_eq!(Fidelity::Approximate, Fidelity::Approximate);
    assert_eq!(Fidelity::Unsupported, Fidelity::Unsupported);
    assert_eq!(Fidelity::FallbackRendered, Fidelity::FallbackRendered);

    let input = worksheet(&format!(
        "{}{}",
        region(2, "0", r#"<x:future-region/>"#),
        region(1, "1", r#"<x:plot item-idref="plot-1"/>"#)
    ));
    let output = convert(input, PartialPolicy::AllowSafePartial).expect_err("all unsupported");
    assert_eq!(output.code, FailureCode::NoExportableContent);
    assert!(output.diagnostics.len() <= ConversionOptions::default().limits.max_diagnostics);
}

#[test]
fn failure_diagnostics_never_exceed_zero_or_small_caps() {
    let input = worksheet(&region(1, "0", r#"<x:plot item-idref="plot-1"/>"#));
    for max_diagnostics in [0, 1, 2] {
        let failure = ConversionPipeline::new()
            .convert(ConversionRequest {
                bytes: input.clone(),
                file_name: Some("x.xmcd".to_owned()),
                target: TargetFormat::Docx,
                options: ConversionOptions {
                    limits: conversion_core::ConversionLimits {
                        max_diagnostics,
                        ..Default::default()
                    },
                    partial_policy: PartialPolicy::Strict,
                },
            })
            .expect_err("unsupported input");
        assert!(failure.diagnostics.len() <= max_diagnostics);
        if max_diagnostics == 0 {
            assert_eq!(failure.code, FailureCode::DiagnosticLimitExceeded);
        }
    }

    let failure = ConversionPipeline::new()
        .convert(ConversionRequest {
            bytes: input,
            file_name: Some("x.mcdx".to_owned()),
            target: TargetFormat::Docx,
            options: ConversionOptions {
                limits: conversion_core::ConversionLimits {
                    max_diagnostics: 1,
                    ..Default::default()
                },
                partial_policy: PartialPolicy::Strict,
            },
        })
        .expect_err("full diagnostic cap");
    assert_eq!(failure.code, FailureCode::DiagnosticLimitExceeded);
    assert_eq!(failure.diagnostics.len(), 1);
}

#[test]
fn malformed_xml_and_input_limit_fail_closed_with_bounded_diagnostics() {
    let malformed = b"<x:worksheet xmlns:x=\"http://schemas.mathsoft.com/worksheet30\" version=\"3.0.3\"><x:regions>".to_vec();
    let failure = convert(malformed, PartialPolicy::AllowSafePartial).expect_err("malformed xml");
    assert_eq!(failure.code, FailureCode::ParserFailure);
    assert!(failure.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == conversion_core::DiagnosticSeverity::FatalError
    }));

    let input = worksheet(&region(
        1,
        "0",
        r#"<x:text><x:p style="Normal">bounded</x:p></x:text>"#,
    ));
    let failure = ConversionPipeline::new()
        .convert(ConversionRequest {
            bytes: input,
            file_name: Some("x.xmcd".to_owned()),
            target: TargetFormat::Docx,
            options: ConversionOptions {
                limits: conversion_core::ConversionLimits {
                    worksheet: mathcad_parser::WorksheetLimits {
                        max_input_bytes: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                partial_policy: PartialPolicy::AllowSafePartial,
            },
        })
        .expect_err("input limit");
    assert_eq!(failure.code, FailureCode::ParserFailure);
}
