//! Minimal, fail-closed command-line adapter for the conversion core.

use conversion_core::{
    ConversionOptions, ConversionPipeline, ConversionRequest, FailureCode, PartialPolicy,
    TargetFormat,
};
use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const TEMP_ATTEMPTS: u32 = 16;

#[derive(Clone, Eq, PartialEq)]
pub struct CliArguments {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
}

impl fmt::Debug for CliArguments {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CliArguments")
            .field("input_present", &true)
            .field("output_present", &self.output.is_some())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitCode {
    Success = 0,
    Usage = 2,
    InvalidInput = 3,
    Conversion = 4,
    Filesystem = 5,
}

impl ExitCode {
    pub const fn value(self) -> i32 {
        self as i32
    }
}

#[derive(Debug)]
pub enum CliError {
    Usage(&'static str),
    Invalid(&'static str),
    Conversion(&'static str),
    Filesystem(&'static str),
}

impl CliError {
    pub const fn exit_code(&self) -> ExitCode {
        match self {
            Self::Usage(_) => ExitCode::Usage,
            Self::Invalid(_) => ExitCode::InvalidInput,
            Self::Conversion(_) => ExitCode::Conversion,
            Self::Filesystem(_) => ExitCode::Filesystem,
        }
    }

    pub const fn code(&self) -> &'static str {
        match self {
            Self::Usage(code)
            | Self::Invalid(code)
            | Self::Conversion(code)
            | Self::Filesystem(code) => code,
        }
    }
}

/// Parses the deliberately small stage-148 command contract without exposing
/// argument payloads in errors or debug output.
pub fn parse_args<I>(args: I) -> Result<CliArguments, CliError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    if args.next().as_deref() != Some(std::ffi::OsStr::new("convert")) {
        return Err(CliError::Usage("USAGE_ERROR"));
    }

    let mut input = None;
    let mut output = None;
    let mut target = None;
    while let Some(argument) = args.next() {
        if argument == "--to" {
            if target.is_some() {
                return Err(CliError::Usage("USAGE_ERROR"));
            }
            let value = args.next().ok_or(CliError::Usage("USAGE_ERROR"))?;
            if value != "docx" {
                return Err(CliError::Usage("UNSUPPORTED_TARGET"));
            }
            target = Some(TargetFormat::Docx);
        } else if argument == "--output" {
            if output.is_some() {
                return Err(CliError::Usage("USAGE_ERROR"));
            }
            let value = args.next().ok_or(CliError::Usage("USAGE_ERROR"))?;
            if value.to_string_lossy().starts_with('-') || value.is_empty() {
                return Err(CliError::Usage("USAGE_ERROR"));
            }
            output = Some(PathBuf::from(value));
        } else if argument.to_string_lossy().starts_with('-') || input.is_some() {
            return Err(CliError::Usage("USAGE_ERROR"));
        } else {
            input = Some(PathBuf::from(argument));
        }
    }

    if target.is_none() || input.is_none() {
        return Err(CliError::Usage("USAGE_ERROR"));
    }
    Ok(CliArguments {
        input: input.expect("checked above"),
        output,
    })
}

/// Converts one input using bounded reads and publishes through a same-directory
/// hard link. Standard Rust cannot close every TOCTOU window or provide a stable
/// Windows file-id on this toolchain; uncertain metadata therefore fails closed,
/// and this adapter never falls back to replacement-style rename.
pub fn execute(arguments: CliArguments) -> Result<String, CliError> {
    validate_path_components(&arguments.input, "INPUT_SYMLINK")?;
    let input_metadata = fs::symlink_metadata(&arguments.input)
        .map_err(|error| map_input_metadata_error(error.kind()))?;
    if !input_metadata.is_file() {
        return Err(CliError::Filesystem("INPUT_NOT_FILE"));
    }

    let output = arguments
        .output
        .unwrap_or_else(|| arguments.input.with_extension("docx"));
    reject_same_identity(&arguments.input, &output)?;
    ensure_output_available(&output)?;

    let bytes = read_bounded(&arguments.input)?;
    let outcome = ConversionPipeline::new()
        .convert(ConversionRequest::new(
            bytes,
            arguments
                .input
                .file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            TargetFormat::Docx,
            ConversionOptions {
                partial_policy: PartialPolicy::AllowSafePartial,
                ..ConversionOptions::default()
            },
        ))
        .map_err(map_conversion_error)?;

    let temp_cleanup_warning = write_atomic(&output, &outcome.artifact)?;
    let status = match outcome.report.status {
        conversion_core::ReportStatus::Completed => "completed",
        conversion_core::ReportStatus::CompletedWithWarnings => "completed with warnings",
    };
    let mut warning_codes = outcome
        .report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity != conversion_core::DiagnosticSeverity::FatalError)
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();
    if temp_cleanup_warning {
        warning_codes.push("OUTPUT_TEMP_CLEANUP_WARNING");
    }
    if warning_codes.is_empty() {
        Ok(status.to_owned())
    } else {
        Ok(format!("{status}: {}", warning_codes.join(",")))
    }
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, CliError> {
    let maximum = ConversionOptions::default()
        .limits
        .worksheet
        .max_input_bytes;
    let mut file = open_input(path)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(u64::try_from(maximum).unwrap_or(u64::MAX).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| CliError::Filesystem("INPUT_READ_ERROR"))?;
    if bytes.len() > maximum {
        return Err(CliError::Invalid("INPUT_TOO_LARGE"));
    }
    Ok(bytes)
}

fn open_input(path: &Path) -> Result<File, CliError> {
    validate_path_components(path, "INPUT_SYMLINK")?;
    let path_metadata =
        fs::symlink_metadata(path).map_err(|error| map_input_metadata_error(error.kind()))?;
    if is_reparse_or_symlink(&path_metadata) || !path_metadata.is_file() {
        return Err(CliError::Filesystem("INPUT_NOT_FILE"));
    }

    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options
            .share_mode(0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options
        .open(path)
        .map_err(|_| CliError::Filesystem("INPUT_READ_ERROR"))?;
    let handle_metadata = file
        .metadata()
        .map_err(|_| CliError::Filesystem("INPUT_READ_ERROR"))?;
    if is_reparse_or_symlink(&handle_metadata) || !handle_metadata.is_file() {
        return Err(CliError::Filesystem("INPUT_NOT_FILE"));
    }

    // Read from this same handle only after revalidating the pathname. Unix
    // std cannot provide O_NOFOLLOW portably, but a raced replacement is
    // rejected when its device/inode no longer matches the opened handle.
    validate_path_components(path, "INPUT_SYMLINK")?;
    let current_metadata =
        fs::symlink_metadata(path).map_err(|error| map_input_metadata_error(error.kind()))?;
    if is_reparse_or_symlink(&current_metadata)
        || file_identity(&handle_metadata).is_none()
        || file_identity(&handle_metadata) != file_identity(&current_metadata)
    {
        return Err(CliError::Filesystem("INPUT_IDENTITY_CHANGED"));
    }
    Ok(file)
}

fn reject_same_identity(input: &Path, output: &Path) -> Result<(), CliError> {
    validate_path_components(input, "INPUT_SYMLINK")?;
    validate_path_components(parent_dir(output), "OUTPUT_DIRECTORY_ERROR")?;
    let input_absolute =
        fs::canonicalize(input).map_err(|_| CliError::Filesystem("INPUT_READ_ERROR"))?;
    let output_absolute = match fs::symlink_metadata(output) {
        Ok(_) => fs::canonicalize(output).map_err(|_| CliError::Filesystem("OUTPUT_READ_ERROR"))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::canonicalize(parent_dir(output))
                .map_err(|_| CliError::Filesystem("OUTPUT_DIRECTORY_ERROR"))?
                .join(
                    output
                        .file_name()
                        .ok_or(CliError::Filesystem("OUTPUT_PATH_ERROR"))?,
                )
        }
        Err(_) => return Err(CliError::Filesystem("OUTPUT_READ_ERROR")),
    };
    if input_absolute == output_absolute {
        return Err(CliError::Filesystem("INPUT_OUTPUT_SAME"));
    }
    Ok(())
}

fn ensure_output_available(output: &Path) -> Result<(), CliError> {
    validate_path_components(parent_dir(output), "OUTPUT_DIRECTORY_ERROR")?;
    match fs::symlink_metadata(output) {
        Ok(metadata) => {
            if is_reparse_or_symlink(&metadata) {
                return Err(CliError::Filesystem("OUTPUT_SYMLINK"));
            }
            return Err(CliError::Filesystem("OUTPUT_EXISTS"));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(_) => return Err(CliError::Filesystem("OUTPUT_READ_ERROR")),
    }
    let parent = parent_dir(output);
    let metadata =
        fs::symlink_metadata(parent).map_err(|_| CliError::Filesystem("OUTPUT_DIRECTORY_ERROR"))?;
    if is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
        return Err(CliError::Filesystem("OUTPUT_DIRECTORY_ERROR"));
    }
    Ok(())
}

fn map_input_metadata_error(kind: io::ErrorKind) -> CliError {
    if kind == io::ErrorKind::NotFound {
        CliError::Filesystem("INPUT_NOT_FOUND")
    } else {
        CliError::Filesystem("INPUT_READ_ERROR")
    }
}

fn write_atomic(output: &Path, artifact: &[u8]) -> Result<bool, CliError> {
    let parent = parent_dir(output);
    ensure_output_available(output)?;
    let pid = std::process::id();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let mut temporary = None;
    for attempt in 0..TEMP_ATTEMPTS {
        let candidate = parent.join(format!(".mathmorph-{pid}-{nonce}-{attempt}.tmp"));
        match create_temp_file(&candidate) {
            Ok(file) => {
                let identity = file
                    .metadata()
                    .ok()
                    .and_then(|metadata| file_identity(&metadata));
                temporary = Some(TempFile {
                    path: candidate,
                    file,
                    identity,
                });
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(CliError::Filesystem("OUTPUT_TEMP_CREATE_ERROR")),
        }
    }
    let mut temporary = temporary.ok_or(CliError::Filesystem("OUTPUT_TEMP_COLLISION"))?;
    let result = publish_no_replace(output, &mut temporary, artifact);
    temporary.identity = temporary
        .file
        .metadata()
        .ok()
        .and_then(|metadata| file_identity(&metadata));
    let cleanup_failed = cleanup_temp(temporary).is_err();
    finalize_publication(result, cleanup_failed)
}

struct TempFile {
    path: PathBuf,
    file: File,
    identity: Option<FileIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(windows)]
    size: u64,
    #[cfg(windows)]
    created: u64,
    #[cfg(windows)]
    modified: u64,
}

#[cfg(unix)]
fn file_identity(metadata: &Metadata) -> Option<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    Some(FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(windows)]
fn file_identity(metadata: &Metadata) -> Option<FileIdentity> {
    use std::os::windows::fs::MetadataExt;
    Some(FileIdentity {
        // Stable std metadata does not expose a Windows file-id on this
        // toolchain. These immutable handle/path values are the strongest
        // portable ownership check available without platform FFI.
        size: metadata.file_size(),
        created: metadata.creation_time(),
        modified: metadata.last_write_time(),
    })
}

#[cfg(not(any(unix, windows)))]
fn file_identity(_metadata: &Metadata) -> Option<FileIdentity> {
    None
}

fn create_temp_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Keep the handle exclusive until publication and ownership cleanup.
        options.share_mode(0);
    }
    options.open(path)
}

fn publish_no_replace(
    output: &Path,
    temporary: &mut TempFile,
    artifact: &[u8],
) -> Result<(), CliError> {
    temporary
        .file
        .write_all(artifact)
        .map_err(|_| CliError::Filesystem("OUTPUT_WRITE_ERROR"))?;
    temporary
        .file
        .flush()
        .map_err(|_| CliError::Filesystem("OUTPUT_FLUSH_ERROR"))?;
    temporary
        .file
        .sync_all()
        .map_err(|_| CliError::Filesystem("OUTPUT_SYNC_ERROR"))?;

    temporary.identity = temporary
        .file
        .metadata()
        .ok()
        .and_then(|metadata| file_identity(&metadata));

    // The parent is checked again immediately before publication. The standard
    // library cannot close every TOCTOU window or identify Windows reparse
    // points behind an untrusted race, so publication fails closed on any
    // metadata uncertainty and never falls back to replacing rename.
    ensure_output_available(output)?;
    let path_identity = fs::symlink_metadata(&temporary.path)
        .ok()
        .and_then(|metadata| file_identity(&metadata));
    if temporary.identity.is_none() || temporary.identity != path_identity {
        return Err(CliError::Filesystem("OUTPUT_TEMP_OWNERSHIP_ERROR"));
    }
    link_no_replace(&temporary.path, output)
}

fn link_no_replace(temporary: &Path, output: &Path) -> Result<(), CliError> {
    match fs::hard_link(temporary, output) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            Err(CliError::Filesystem("OUTPUT_EXISTS"))
        }
        Err(_) => Err(CliError::Filesystem("OUTPUT_PUBLISH_ERROR")),
    }
}

fn finalize_publication(
    publication: Result<(), CliError>,
    cleanup_failed: bool,
) -> Result<bool, CliError> {
    match publication {
        // Publication is the commit point. A later cleanup problem is a
        // warning, never a false failure that could trigger a destructive retry.
        Ok(()) => Ok(cleanup_failed),
        Err(error) => Err(error),
    }
}

fn cleanup_temp(temporary: TempFile) -> Result<(), ()> {
    let path = temporary.path;
    let identity = temporary.identity;
    #[cfg(unix)]
    {
        // Unix permits unlinking while the original handle remains open, so
        // inode ownership is checked and removed before the handle is dropped.
        let result = cleanup_owned_temp(&path, identity);
        drop(temporary.file);
        result
    }
    #[cfg(not(unix))]
    {
        // Windows share_mode(0) prevents replacement while the handle is open;
        // std requires closing it before removing the pathname.
        drop(temporary.file);
        cleanup_owned_temp(&path, identity)
    }
}

fn cleanup_owned_temp(path: &Path, identity: Option<FileIdentity>) -> Result<(), ()> {
    let Some(expected) = identity else {
        return Err(());
    };
    let Some(actual) = fs::symlink_metadata(path)
        .ok()
        .and_then(|metadata| file_identity(&metadata))
    else {
        return Ok(());
    };
    if actual != expected {
        return Err(());
    }
    fs::remove_file(path).map_err(|_| ())
}

fn validate_path_components(path: &Path, code: &'static str) -> Result<(), CliError> {
    validate_local_path_prefix(path, code)?;
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if is_reparse_or_symlink(&metadata) => {
                return Err(CliError::Filesystem(code));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(_) => return Err(CliError::Filesystem(code)),
        }
    }
    Ok(())
}

#[cfg(windows)]
fn validate_local_path_prefix(path: &Path, code: &'static str) -> Result<(), CliError> {
    use std::path::{Component, Prefix};
    if let Some(Component::Prefix(prefix)) = path.components().next() {
        if !matches!(prefix.kind(), Prefix::Disk(_)) {
            return Err(CliError::Filesystem(code));
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn validate_local_path_prefix(_path: &Path, _code: &'static str) -> Result<(), CliError> {
    Ok(())
}

#[cfg(windows)]
fn is_reparse_or_symlink(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_type().is_symlink()
        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_or_symlink(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn parent_dir(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn map_conversion_error(error: conversion_core::ConversionFailure) -> CliError {
    match error.code {
        FailureCode::InvalidInput
        | FailureCode::McdxContentUnsupported
        | FailureCode::ParserFailure
        | FailureCode::StrictUnsupportedContent
        | FailureCode::NoExportableContent
        | FailureCode::DiagnosticLimitExceeded
        | FailureCode::ItemLimitExceeded => CliError::Invalid(failure_code(error.code)),
        FailureCode::UnsupportedTarget => CliError::Usage("UNSUPPORTED_TARGET"),
        FailureCode::TransformationFailure
        | FailureCode::IrValidationFailure
        | FailureCode::ExportFailure
        | FailureCode::DocxValidationFailure => CliError::Conversion(failure_code(error.code)),
    }
}

const fn failure_code(code: FailureCode) -> &'static str {
    match code {
        FailureCode::UnsupportedTarget => "UNSUPPORTED_TARGET",
        FailureCode::InvalidInput => "INVALID_INPUT",
        FailureCode::McdxContentUnsupported => "MCDX_CONTENT_UNSUPPORTED",
        FailureCode::ParserFailure => "PARSER_FAILURE",
        FailureCode::StrictUnsupportedContent => "UNSUPPORTED_CONTENT",
        FailureCode::NoExportableContent => "NO_EXPORTABLE_CONTENT",
        FailureCode::DiagnosticLimitExceeded => "DIAGNOSTIC_LIMIT_EXCEEDED",
        FailureCode::ItemLimitExceeded => "ITEM_LIMIT_EXCEEDED",
        FailureCode::TransformationFailure => "TRANSFORMATION_FAILURE",
        FailureCode::IrValidationFailure => "IR_VALIDATION_FAILURE",
        FailureCode::ExportFailure => "EXPORT_FAILURE",
        FailureCode::DocxValidationFailure => "DOCX_VALIDATION_FAILURE",
    }
}

pub fn render_error(error: &CliError) -> String {
    format!("error[{}]", error.code())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_the_convert_docx_contract() {
        let parsed = parse_args([
            "convert".into(),
            "worksheet.xmcd".into(),
            "--to".into(),
            "docx".into(),
        ])
        .expect("arguments");
        assert_eq!(parsed.input, PathBuf::from("worksheet.xmcd"));
        assert!(parse_args(["inspect".into()]).is_err());
        assert!(parse_args(["convert".into(), "x".into(), "--to".into(), "pdf".into()]).is_err());
    }

    #[test]
    fn diagnostics_are_redacted() {
        let error = CliError::Invalid("INVALID_INPUT");
        assert_eq!(render_error(&error), "error[INVALID_INPUT]");
        let arguments = CliArguments {
            input: PathBuf::from("/absolute/private/input.xmcd"),
            output: Some(PathBuf::from("/absolute/private/output.docx")),
        };
        let rendered = format!("{arguments:?}");
        assert!(rendered.contains("input_present"));
        assert!(!rendered.contains("input.xmcd"));
        assert!(!rendered.contains("output.docx"));
    }

    fn test_directory() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("mathmorph-cli-unit-{}-{nonce}", std::process::id()));
        fs::create_dir(&directory).expect("test directory");
        directory
    }

    #[test]
    fn no_replace_publish_keeps_destination_created_after_temp() {
        let directory = test_directory();
        let output = directory.join("result.docx");
        let temporary_path = directory.join("owned.tmp");
        let mut file = create_temp_file(&temporary_path).expect("temp file");
        file.write_all(b"new").expect("temporary bytes");
        file.flush().expect("temporary flush");
        let identity = file
            .metadata()
            .ok()
            .and_then(|metadata| file_identity(&metadata));
        fs::write(&output, b"pre-existing").expect("destination");
        let result = link_no_replace(&temporary_path, &output);
        assert_eq!(
            result.err().map(|error| error.code()),
            Some("OUTPUT_EXISTS")
        );
        drop(file);
        let _ = cleanup_owned_temp(&temporary_path, identity);
        assert_eq!(
            fs::read(&output).expect("destination bytes"),
            b"pre-existing"
        );
        assert!(!temporary_path.exists());
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn publication_commit_point_stays_successful_when_cleanup_warns() {
        assert!(finalize_publication(Ok(()), true).expect("published cleanup warning"));
        assert!(!finalize_publication(Ok(()), false).expect("published cleanly"));
        let failure =
            finalize_publication(Err(CliError::Filesystem("OUTPUT_PUBLISH_ERROR")), false)
                .expect_err("publication failure");
        assert_eq!(failure.code(), "OUTPUT_PUBLISH_ERROR");
    }

    #[test]
    fn same_input_output_is_rejected_before_conversion() {
        let directory = test_directory();
        let input = directory.join("same.xmcd");
        fs::write(&input, b"not a worksheet").expect("input");
        let error = execute(CliArguments {
            input: input.clone(),
            output: Some(input),
        })
        .expect_err("same identity");
        assert_eq!(error.code(), "INPUT_OUTPUT_SAME");
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn output_and_ancestor_symlinks_fail_closed_when_supported() {
        let directory = test_directory();
        let input = directory.join("input.xmcd");
        fs::write(&input, b"not a worksheet").expect("input");
        let target = directory.join("target.docx");
        fs::write(&target, b"target").expect("target");
        let output_link = directory.join("output.docx");
        if create_file_symlink(&target, &output_link).is_ok() {
            let error = execute(CliArguments {
                input: input.clone(),
                output: Some(output_link),
            })
            .expect_err("output symlink");
            assert_eq!(error.code(), "OUTPUT_SYMLINK");
        }
        let input_link = directory.join("input-link.xmcd");
        if create_file_symlink(&input, &input_link).is_ok() {
            let error = execute(CliArguments {
                input: input_link,
                output: Some(directory.join("input-link.docx")),
            })
            .expect_err("input symlink");
            assert_eq!(error.code(), "INPUT_SYMLINK");
        }
        let real_parent = directory.join("real-parent");
        fs::create_dir(&real_parent).expect("real parent");
        let linked_parent = directory.join("linked-parent");
        if create_directory_symlink(&real_parent, &linked_parent).is_ok() {
            let error = execute(CliArguments {
                input,
                output: Some(linked_parent.join("result.docx")),
            })
            .expect_err("ancestor symlink");
            assert_eq!(error.code(), "OUTPUT_DIRECTORY_ERROR");
        }
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn cleanup_only_removes_matching_temp_identity() {
        let directory = test_directory();
        let path = directory.join("owned.tmp");
        let file = create_temp_file(&path).expect("temp file");
        let identity = file
            .metadata()
            .ok()
            .and_then(|metadata| file_identity(&metadata));
        drop(file);
        assert!(cleanup_owned_temp(&path, identity).is_ok());
        assert!(!path.exists());

        let file = create_temp_file(&path).expect("replacement");
        drop(file);
        fs::write(&path, b"replacement").expect("replacement bytes");
        assert!(cleanup_owned_temp(&path, identity).is_err());
        assert!(path.exists());
        let _ = fs::remove_file(path);
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn temp_files_are_private_on_unix() {
        use std::os::unix::fs::PermissionsExt;
        let directory = test_directory();
        let path = directory.join("private.tmp");
        let file = create_temp_file(&path).expect("temp file");
        let mode = file.metadata().expect("metadata").permissions().mode() & 0o777;
        drop(file);
        let _ = fs::remove_file(path);
        fs::remove_dir_all(directory).expect("cleanup");
        assert_eq!(mode, 0o600);
    }

    #[cfg(windows)]
    #[test]
    fn network_and_device_prefixes_are_rejected_without_io() {
        for path in [
            Path::new(r"\\server\share\worksheet.xmcd"),
            Path::new(r"\\?\UNC\server\share\worksheet.xmcd"),
            Path::new(r"\\.\pipe\mathmorph"),
        ] {
            let error =
                validate_path_components(path, "NON_LOCAL_PATH").expect_err("non-local path");
            assert_eq!(error.code(), "NON_LOCAL_PATH");
        }
    }

    #[cfg(unix)]
    fn create_file_symlink(target: &Path, link: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn create_file_symlink(target: &Path, link: &Path) -> io::Result<()> {
        std::os::windows::fs::symlink_file(target, link)
    }

    #[cfg(unix)]
    fn create_directory_symlink(target: &Path, link: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn create_directory_symlink(target: &Path, link: &Path) -> io::Result<()> {
        std::os::windows::fs::symlink_dir(target, link)
    }
}
