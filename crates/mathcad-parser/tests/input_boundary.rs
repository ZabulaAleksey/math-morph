use std::io::{Cursor, Write};

use mathcad_parser::{
    ContainerError, ContainerLimit, ContainerLimits, ContainerPartKind, DiagnosticCode,
    FormatDetector, FormatError, InputFormat, SafeMcdxReader, XmlMetadataError, XmlMetadataLimits,
    inspect_xml_metadata,
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

const XMCD: &[u8] = include_bytes!("../../../tests/fixtures/xmcd/minimal-worksheet30.xmcd");
const DTD_XMCD: &[u8] =
    include_bytes!("../../../tests/fixtures/security/doctype-external-entity.xmcd");

fn archive(entries: &[(&str, &[u8], CompressionMethod)]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, contents, method) in entries {
        let options = SimpleFileOptions::default().compression_method(*method);
        writer.start_file(*name, options).expect("test ZIP entry");
        writer.write_all(contents).expect("test ZIP contents");
    }
    writer.finish().expect("finish test ZIP").into_inner()
}

fn mcdx() -> Vec<u8> {
    archive(&[(
        "mathcad/worksheet.xml",
        br#"<worksheet xmlns="http://schemas.mathsoft.com/worksheet30"/>"#,
        CompressionMethod::Stored,
    )])
}

#[test]
fn detects_empty_and_extension_only_input_as_unknown() {
    let empty = FormatDetector::default()
        .detect(&[], None)
        .expect("empty input is controlled");
    assert_eq!(empty.detected, InputFormat::Unknown);

    let extension_only = FormatDetector::default()
        .detect(b"not XML", Some("worksheet.xmcd"))
        .expect("arbitrary input is controlled");
    assert_eq!(extension_only.declared, InputFormat::Xmcd);
    assert_eq!(extension_only.detected, InputFormat::Unknown);
    assert!(extension_only.diagnostics.is_empty());
}

#[test]
fn detects_xmcd_by_root_and_namespace() {
    let detection = FormatDetector::default()
        .detect(XMCD, Some("worksheet.bin"))
        .expect("valid XMCD");
    assert_eq!(detection.declared, InputFormat::Unknown);
    assert_eq!(detection.detected, InputFormat::Xmcd);
}

#[test]
fn rejects_doctype_instead_of_resolving_external_entity() {
    let error = FormatDetector::default()
        .detect(DTD_XMCD, Some("worksheet.xmcd"))
        .expect_err("DOCTYPE must be rejected");
    assert!(matches!(
        error,
        FormatError::Xml(XmlMetadataError::DoctypeForbidden)
    ));
}

#[test]
fn detects_only_mcdx_with_canonical_worksheet_part() {
    let valid = FormatDetector::default()
        .detect(&mcdx(), None)
        .expect("valid MCDX");
    assert_eq!(valid.detected, InputFormat::Mcdx);

    let generic = archive(&[("readme.txt", b"hello", CompressionMethod::Stored)]);
    let generic = FormatDetector::default()
        .detect(&generic, Some("fake.mcdx"))
        .expect("generic ZIP is safely inspected");
    assert_eq!(generic.detected, InputFormat::Unknown);

    let wrong_case = archive(&[(
        "Mathcad/Worksheet.xml",
        b"<worksheet/>",
        CompressionMethod::Stored,
    )]);
    let wrong_case = FormatDetector::default()
        .detect(&wrong_case, None)
        .expect("case variant is a safe unknown ZIP");
    assert_eq!(wrong_case.detected, InputFormat::Unknown);
}

#[test]
fn emits_exactly_one_extension_mismatch_for_known_formats() {
    for (bytes, name, declared, detected) in [
        (
            XMCD.to_vec(),
            "wrong.mcdx",
            InputFormat::Mcdx,
            InputFormat::Xmcd,
        ),
        (mcdx(), "wrong.xmcd", InputFormat::Xmcd, InputFormat::Mcdx),
    ] {
        let detection = FormatDetector::default()
            .detect(&bytes, Some(name))
            .expect("content remains authoritative");
        assert_eq!(detection.declared, declared);
        assert_eq!(detection.detected, detected);
        assert_eq!(detection.diagnostics.len(), 1);
        assert_eq!(
            detection.diagnostics[0].code,
            DiagnosticCode::FileExtensionMismatch
        );
        assert_eq!(
            detection.diagnostics[0].code.as_str(),
            "FILE_EXTENSION_MISMATCH"
        );
    }
}

#[test]
fn builds_deterministic_manifest_without_extracting_parts() {
    let bytes = archive(&[
        (
            "mathcad/worksheet.xml",
            b"<worksheet/>",
            CompressionMethod::Stored,
        ),
        ("resources/image.png", b"PNG", CompressionMethod::Stored),
        ("vendor/opaque.bin", b"opaque", CompressionMethod::Stored),
    ]);
    let manifest = SafeMcdxReader::default()
        .inspect(&bytes)
        .expect("safe inventory");

    assert_eq!(manifest.parts.len(), 3);
    assert_eq!(manifest.parts[0].kind, ContainerPartKind::Worksheet);
    assert_eq!(
        manifest.parts[1].kind,
        ContainerPartKind::EmbeddedResource {
            media_type_hint: Some("image/png")
        }
    );
    assert_eq!(manifest.parts[2].kind, ContainerPartKind::Unknown);
    assert_eq!(manifest.diagnostics.len(), 1);
    assert_eq!(
        manifest.diagnostics[0].code,
        DiagnosticCode::UnknownContainerPart
    );
    assert_eq!(manifest.worksheet_part().map(|part| part.index), Some(0));
}

#[test]
fn rejects_traversal_drive_backslash_and_ambiguous_paths() {
    for unsafe_name in [
        "../escape.xml",
        "/absolute.xml",
        "C:/drive.xml",
        "mathcad\\worksheet.xml",
        "mathcad//worksheet.xml",
    ] {
        let bytes = archive(&[(unsafe_name, b"x", CompressionMethod::Stored)]);
        let error = SafeMcdxReader::default()
            .inspect(&bytes)
            .expect_err("unsafe path");
        assert!(matches!(error, ContainerError::UnsafePath { index: 0 }));
    }
}

#[test]
fn rejects_case_insensitive_path_collisions() {
    let bytes = archive(&[
        ("Resources/Image.png", b"a", CompressionMethod::Stored),
        ("resources/image.png", b"b", CompressionMethod::Stored),
    ]);
    let error = SafeMcdxReader::default()
        .inspect(&bytes)
        .expect_err("duplicate path");
    assert!(
        matches!(error, ContainerError::DuplicatePath { index: 1 }),
        "{error:?}"
    );
}

#[test]
fn rejects_exact_duplicate_paths_even_if_archive_contains_them() {
    let mut bytes = archive(&[
        ("first.txt", b"a", CompressionMethod::Stored),
        ("other.txt", b"b", CompressionMethod::Stored),
    ]);
    replace_all(&mut bytes, b"other.txt", b"first.txt");
    let error = SafeMcdxReader::default()
        .inspect(&bytes)
        .expect_err("duplicate path");
    assert!(
        matches!(error, ContainerError::DuplicatePath { index: 1 }),
        "{error:?}"
    );
}

#[test]
fn rejects_symlink_entries() {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .add_symlink(
            "mathcad/worksheet.xml",
            "../outside",
            SimpleFileOptions::default(),
        )
        .expect("test symlink");
    let bytes = writer.finish().expect("finish test ZIP").into_inner();
    assert!(matches!(
        SafeMcdxReader::default().inspect(&bytes),
        Err(ContainerError::Symlink { index: 0 })
    ));
}

#[test]
fn rejects_encrypted_and_unsupported_compression_metadata() {
    let stored = archive(&[("entry.bin", b"value", CompressionMethod::Stored)]);

    let mut encrypted = stored.clone();
    patch_zip_u16(&mut encrypted, 6, 8, |flags| flags | 1);
    let encrypted_error = SafeMcdxReader::default()
        .inspect(&encrypted)
        .expect_err("encrypted entry");
    assert!(
        matches!(encrypted_error, ContainerError::EncryptedEntry { index: 0 }),
        "{encrypted_error:?}"
    );

    let mut unsupported = stored;
    patch_zip_u16(&mut unsupported, 8, 10, |_| 12);
    let compression_error = SafeMcdxReader::default()
        .inspect(&unsupported)
        .expect_err("unsupported compression");
    assert!(
        matches!(
            compression_error,
            ContainerError::UnsupportedCompression { index: 0 }
        ),
        "{compression_error:?}"
    );
}

#[test]
fn enforces_archive_entry_total_ratio_and_name_limits() {
    let one = archive(&[("one.txt", b"1234", CompressionMethod::Stored)]);
    let exact_limits = ContainerLimits {
        max_archive_bytes: u64::try_from(one.len()).expect("test size"),
        max_entries: 1,
        max_entry_uncompressed_bytes: 4,
        max_total_uncompressed_bytes: 4,
        max_compression_ratio: 1,
        max_name_bytes: "one.txt".len(),
    };
    SafeMcdxReader::new(exact_limits)
        .inspect(&one)
        .expect("exact limits are inclusive");

    let limits = ContainerLimits {
        max_archive_bytes: u64::try_from(one.len() - 1).expect("test size"),
        ..ContainerLimits::default()
    };
    assert_limit(one.as_slice(), limits, ContainerLimit::ArchiveBytes);

    let two = archive(&[
        ("one.txt", b"1", CompressionMethod::Stored),
        ("two.txt", b"2", CompressionMethod::Stored),
    ]);
    let limits = ContainerLimits {
        max_entries: 1,
        ..ContainerLimits::default()
    };
    assert_limit(two.as_slice(), limits, ContainerLimit::Entries);

    let limits = ContainerLimits {
        max_entry_uncompressed_bytes: 3,
        ..ContainerLimits::default()
    };
    assert_limit(one.as_slice(), limits, ContainerLimit::EntryBytes);

    let limits = ContainerLimits {
        max_total_uncompressed_bytes: 3,
        ..ContainerLimits::default()
    };
    assert_limit(one.as_slice(), limits, ContainerLimit::TotalBytes);

    let compressed = archive(&[("zeros.bin", &[0_u8; 2048], CompressionMethod::Deflated)]);
    let limits = ContainerLimits {
        max_compression_ratio: 2,
        ..ContainerLimits::default()
    };
    assert_limit(
        compressed.as_slice(),
        limits,
        ContainerLimit::CompressionRatio,
    );

    let long_name = archive(&[("long-name.txt", b"x", CompressionMethod::Stored)]);
    let limits = ContainerLimits {
        max_name_bytes: 4,
        ..ContainerLimits::default()
    };
    assert_limit(long_name.as_slice(), limits, ContainerLimit::NameBytes);
}

fn assert_limit(bytes: &[u8], limits: ContainerLimits, expected: ContainerLimit) {
    let error = SafeMcdxReader::new(limits)
        .inspect(bytes)
        .expect_err("limit must fail closed");
    assert!(matches!(
        error,
        ContainerError::LimitExceeded { limit, .. } if limit == expected
    ));
}

fn replace_all(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    assert_eq!(from.len(), to.len());
    let mut start = 0;
    while let Some(offset) = bytes[start..]
        .windows(from.len())
        .position(|window| window == from)
    {
        let offset = start + offset;
        bytes[offset..offset + to.len()].copy_from_slice(to);
        start = offset + to.len();
    }
}

fn patch_zip_u16(
    bytes: &mut [u8],
    local_offset: usize,
    central_offset: usize,
    patch: impl Fn(u16) -> u16,
) {
    for (signature, offset) in [
        (b"PK\x03\x04".as_slice(), local_offset),
        (b"PK\x01\x02".as_slice(), central_offset),
    ] {
        let position = bytes
            .windows(signature.len())
            .position(|window| window == signature)
            .expect("ZIP signature");
        let field = position + offset;
        let current = u16::from_le_bytes([bytes[field], bytes[field + 1]]);
        bytes[field..field + 2].copy_from_slice(&patch(current).to_le_bytes());
    }
}

#[test]
fn extracts_namespace_bindings_and_schema_locations_only() {
    let metadata = inspect_xml_metadata(XMCD, XmlMetadataLimits::default()).expect("valid XML");
    assert_eq!(metadata.root_local_name, "worksheet");
    assert_eq!(
        metadata.root_namespace_uri.as_deref(),
        Some("http://schemas.mathsoft.com/worksheet30")
    );
    assert!(metadata.namespace_bindings.iter().any(|binding| {
        binding.prefix.is_none()
            && binding.namespace_uri == "http://schemas.mathsoft.com/worksheet30"
    }));
    assert_eq!(metadata.schema_locations.len(), 1);
    assert_eq!(
        metadata.schema_locations[0].namespace_uri.as_deref(),
        Some("http://schemas.mathsoft.com/worksheet30")
    );
    assert_eq!(
        metadata.schema_locations[0].location,
        "https://example.invalid/worksheet30.xsd"
    );
}

#[test]
fn rejects_unsupported_encoding_and_malformed_schema_location() {
    let encoded = br#"<?xml version="1.0" encoding="windows-1251"?><worksheet/>"#;
    assert!(matches!(
        inspect_xml_metadata(encoded, XmlMetadataLimits::default()),
        Err(XmlMetadataError::UnsupportedEncoding)
    ));

    let malformed = br#"<worksheet xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="odd"/>"#;
    assert!(matches!(
        inspect_xml_metadata(malformed, XmlMetadataLimits::default()),
        Err(XmlMetadataError::MalformedSchemaLocation)
    ));
}

#[test]
fn enforces_xml_input_attribute_and_namespace_limits() {
    let limits = XmlMetadataLimits {
        max_input_bytes: 4,
        ..XmlMetadataLimits::default()
    };
    assert!(matches!(
        inspect_xml_metadata(b"<worksheet/>", limits),
        Err(XmlMetadataError::InputLimitExceeded)
    ));

    let limits = XmlMetadataLimits {
        max_root_attributes: 0,
        ..XmlMetadataLimits::default()
    };
    assert!(matches!(
        inspect_xml_metadata(b"<worksheet version=\"1\"/>", limits),
        Err(XmlMetadataError::AttributeLimitExceeded)
    ));

    let limits = XmlMetadataLimits {
        max_namespace_declarations: 1,
        ..XmlMetadataLimits::default()
    };
    assert!(matches!(
        inspect_xml_metadata(b"<worksheet xmlns:a=\"a\" xmlns:b=\"b\"/>", limits),
        Err(XmlMetadataError::NamespaceLimitExceeded) | Err(XmlMetadataError::MalformedXml)
    ));
}
