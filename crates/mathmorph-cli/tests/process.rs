use exporter_docx::DocxValidator;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

const WS: &str = "http://schemas.mathsoft.com/worksheet30";
const ML: &str = "http://schemas.mathsoft.com/math30";

fn worksheet(regions: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0"?><x:worksheet xmlns:x="{WS}" xmlns:m="{ML}" version="3.0.3"><x:regions>{regions}</x:regions></x:worksheet>"#
    )
    .into_bytes()
}

fn region(id: u64, content: &str) -> String {
    format!(
        r#"<x:region region-id="{id}" top="{id}" left="0" height="10" width="20">{content}</x:region>"#
    )
}

fn supported() -> Vec<u8> {
    worksheet(&format!(
        "{}{}",
        region(1, r#"<x:text><x:p style="Normal">Hello</x:p></x:text>"#),
        region(
            2,
            r#"<x:math><m:apply><m:plus/><m:real>1</m:real><m:real>2</m:real></m:apply></x:math>"#
        )
    ))
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mathmorph"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run mathmorph")
}

fn temp_dir() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("mathmorph-cli-{}-{suffix}", std::process::id()));
    fs::create_dir(&path).expect("temp dir");
    path
}

fn mcdx() -> Vec<u8> {
    let mut output = Vec::new();
    let mut writer = ZipWriter::new(Cursor::new(&mut output));
    writer
        .start_file(
            "mathcad/worksheet.xml",
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )
        .expect("zip entry");
    writer
        .write_all(b"<worksheet xmlns=\"http://schemas.mathsoft.com/worksheet30\"/>")
        .expect("zip bytes");
    writer.finish().expect("zip finish");
    output
}

#[test]
fn converts_supported_xmcd_to_valid_docx_with_omml() {
    let dir = temp_dir();
    let input = dir.join("sample.xmcd");
    let output = dir.join("result.docx");
    fs::write(&input, supported()).expect("input");
    let result = run(
        &dir,
        &[
            "convert",
            "sample.xmcd",
            "--to",
            "docx",
            "--output",
            "result.docx",
        ],
    );
    assert!(result.status.success(), "stderr: {:?}", result.stderr);
    let bytes = fs::read(&output).expect("docx");
    DocxValidator::default()
        .validate(&bytes)
        .expect("valid docx");
    let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("docx zip");
    let mut document = String::new();
    archive
        .by_name("word/document.xml")
        .expect("document")
        .read_to_string(&mut document)
        .expect("xml");
    assert!(document.contains("<m:oMath"));
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn uses_neighboring_default_output_and_partial_warning() {
    let dir = temp_dir();
    fs::write(
        dir.join("sample.xmcd"),
        worksheet(&format!(
            "{}{}",
            region(1, r#"<x:text><x:p style="Normal">Hello</x:p></x:text>"#),
            region(2, r#"<x:plot item-idref="p"/>"#)
        )),
    )
    .expect("input");
    let result = run(&dir, &["convert", "sample.xmcd", "--to", "docx"]);
    assert!(result.status.success());
    assert!(dir.join("sample.docx").is_file());
    assert!(String::from_utf8_lossy(&result.stdout).contains("warnings"));
    assert!(String::from_utf8_lossy(&result.stdout).contains("UNSUPPORTED_REGION"));
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn invalid_and_mcdx_inputs_do_not_create_output() {
    let dir = temp_dir();
    fs::write(dir.join("bad.xmcd"), b"invalid").expect("bad input");
    let invalid = run(&dir, &["convert", "bad.xmcd", "--to", "docx"]);
    assert_eq!(invalid.status.code(), Some(3));
    assert!(!dir.join("bad.docx").exists());
    fs::write(dir.join("archive.mcdx"), mcdx()).expect("mcdx");
    let mcdx = run(&dir, &["convert", "archive.mcdx", "--to", "docx"]);
    assert_eq!(mcdx.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&mcdx.stderr).contains("MCDX_CONTENT_UNSUPPORTED"));
    assert!(!dir.join("archive.docx").exists());
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn inspect_mcdx_fails_closed() {
    let dir = temp_dir();
    fs::write(dir.join("archive.mcdx"), mcdx()).expect("mcdx");
    let result = run(&dir, &["inspect", "archive.mcdx"]);
    assert_eq!(result.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&result.stderr).contains("MCDX_CONTENT_UNSUPPORTED"));
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn existing_output_is_unchanged_and_usage_is_redacted() {
    let dir = temp_dir();
    let input = dir.join("secret.xmcd");
    let output = dir.join("existing.docx");
    fs::write(&input, supported()).expect("input");
    fs::write(&output, b"KEEP SECRET_FORMULA").expect("output");
    let existing = run(
        &dir,
        &[
            "convert",
            "secret.xmcd",
            "--to",
            "docx",
            "--output",
            "existing.docx",
        ],
    );
    assert_eq!(existing.status.code(), Some(5));
    assert_eq!(
        fs::read(&output).expect("output bytes"),
        b"KEEP SECRET_FORMULA"
    );
    let usage = run(&dir, &["convert", input.to_str().expect("path")]);
    assert_eq!(usage.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&usage.stderr);
    assert!(!stderr.contains(input.to_str().expect("path")));
    assert!(!stderr.contains("SECRET_FORMULA"));
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn oversized_input_fails_before_conversion() {
    let dir = temp_dir();
    let input = dir.join("large.xmcd");
    fs::write(&input, vec![b'x'; 32 * 1024 * 1024 + 1]).expect("large input");
    let result = run(&dir, &["convert", "large.xmcd", "--to", "docx"]);
    assert_eq!(result.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&result.stderr).contains("INPUT_TOO_LARGE"));
    assert!(!dir.join("large.docx").exists());
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn inspect_validate_and_export_ir_emit_versioned_json() {
    let dir = temp_dir();
    fs::write(dir.join("sample.xmcd"), supported()).expect("input");
    let inspect = run(&dir, &["inspect", "sample.xmcd"]);
    assert!(inspect.status.success(), "{:?}", inspect.stderr);
    let inspected: serde_json::Value =
        serde_json::from_slice(&inspect.stdout).expect("inspect json");
    assert_eq!(inspected["schema_version"], 1);
    assert_eq!(inspected["report"]["detected_format"], "xmcd");

    let validate = run(&dir, &["validate", "sample.xmcd"]);
    assert!(validate.status.success(), "{:?}", validate.stderr);
    let validated: serde_json::Value =
        serde_json::from_slice(&validate.stdout).expect("validate json");
    assert_eq!(validated["schema_version"], 1);
    assert_eq!(validated["kind"], "conversion_report");

    let export = run(
        &dir,
        &["export-ir", "sample.xmcd", "--output", "sample.ir.json"],
    );
    assert!(export.status.success(), "{:?}", export.stderr);
    let ir: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("sample.ir.json")).unwrap()).unwrap();
    assert_eq!(ir["schema_version"], 1);
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn exporter_registry_distinguishes_unknown_and_known_unavailable_targets() {
    let dir = temp_dir();
    fs::write(dir.join("sample.xmcd"), supported()).expect("input");
    let unavailable = run(&dir, &["convert", "sample.xmcd", "--to", "typst"]);
    assert_eq!(unavailable.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&unavailable.stderr).contains("EXPORTER_UNAVAILABLE"));
    let unknown = run(&dir, &["convert", "sample.xmcd", "--to", "secret-format"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("UNSUPPORTED_TARGET"));
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn validate_corrupted_input_emits_machine_readable_error() {
    let dir = temp_dir();
    fs::write(dir.join("bad.xmcd"), b"broken").unwrap();
    let result = run(&dir, &["validate", "bad.xmcd"]);
    assert_eq!(result.status.code(), Some(3));
    let error: serde_json::Value = serde_json::from_slice(&result.stderr).expect("error json");
    assert_eq!(error["schema_version"], 1);
    assert_eq!(error["kind"], "error");
    assert!(error["code"].as_str().is_some());
    fs::remove_dir_all(dir).unwrap();
}
