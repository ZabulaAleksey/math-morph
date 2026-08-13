use std::collections::{HashMap, HashSet};
use std::io::{self, Cursor, Read};
use std::path::{Component, Path};
use std::str;

use thiserror::Error;
use zip::{CompressionMethod, ZipArchive};

use crate::{Diagnostic, DiagnosticCode};

const MIB: u64 = 1024 * 1024;

/// Fail-closed лимиты недоверенного ZIP-контейнера.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContainerLimits {
    pub max_archive_bytes: u64,
    pub max_entries: usize,
    pub max_entry_uncompressed_bytes: u64,
    pub max_total_uncompressed_bytes: u64,
    pub max_compression_ratio: u64,
    pub max_name_bytes: usize,
}

impl Default for ContainerLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: 64 * MIB,
            max_entries: 4096,
            max_entry_uncompressed_bytes: 64 * MIB,
            max_total_uncompressed_bytes: 256 * MIB,
            max_compression_ratio: 100,
            max_name_bytes: 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainerLimit {
    ArchiveBytes,
    Entries,
    EntryBytes,
    TotalBytes,
    CompressionRatio,
    NameBytes,
}

#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("MCDX container is not a valid ZIP archive")]
    InvalidZip,
    #[error("MCDX entry {index} has an unsafe path")]
    UnsafePath { index: usize },
    #[error("MCDX entry {index} duplicates or conflicts with another path")]
    DuplicatePath { index: usize },
    #[error("MCDX entry {index} is a symbolic link")]
    Symlink { index: usize },
    #[error("MCDX entry {index} is encrypted")]
    EncryptedEntry { index: usize },
    #[error("MCDX entry {index} uses unsupported compression")]
    UnsupportedCompression { index: usize },
    #[error("MCDX entries have overlapping archive ranges")]
    OverlappingEntries,
    #[error("MCDX container exceeds the {limit:?} limit")]
    LimitExceeded {
        limit: ContainerLimit,
        index: Option<usize>,
    },
    #[error("MCDX entry {index} could not be read safely")]
    EntryRead { index: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContainerPartKind {
    Worksheet,
    EmbeddedResource {
        media_type_hint: Option<&'static str>,
    },
    Directory,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerPart {
    pub index: usize,
    pub name: String,
    pub is_directory: bool,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub crc32: u32,
    pub kind: ContainerPartKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerManifest {
    pub archive_size: u64,
    pub total_uncompressed_size: u64,
    pub parts: Vec<ContainerPart>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ContainerManifest {
    pub fn worksheet_part(&self) -> Option<&ContainerPart> {
        self.parts
            .iter()
            .find(|part| part.kind == ContainerPartKind::Worksheet)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SafeMcdxReader {
    limits: ContainerLimits,
}

impl SafeMcdxReader {
    pub const fn new(limits: ContainerLimits) -> Self {
        Self { limits }
    }

    pub fn inspect(&self, bytes: &[u8]) -> Result<ContainerManifest, ContainerError> {
        let archive_size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if archive_size > self.limits.max_archive_bytes {
            return Err(ContainerError::LimitExceeded {
                limit: ContainerLimit::ArchiveBytes,
                index: None,
            });
        }

        let preflight_entries = preflight_zip(bytes, &self.limits)?;
        let mut archive =
            ZipArchive::new(Cursor::new(bytes)).map_err(|_| ContainerError::InvalidZip)?;
        if archive.len() != preflight_entries {
            return Err(ContainerError::InvalidZip);
        }
        if archive.len() > self.limits.max_entries {
            return Err(ContainerError::LimitExceeded {
                limit: ContainerLimit::Entries,
                index: None,
            });
        }

        let mut exact_names = HashSet::new();
        let mut folded_names = HashMap::new();
        let mut total_uncompressed_size = 0_u64;
        let mut parts = Vec::with_capacity(archive.len());
        let mut diagnostics = Vec::new();

        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|_| ContainerError::EntryRead { index })?;
            let is_directory = entry.is_dir();
            let enclosed_name = entry.enclosed_name();
            let canonical_name =
                validate_name(index, entry.name(), enclosed_name.as_deref(), &self.limits)?;

            if !exact_names.insert(canonical_name.clone()) {
                return Err(ContainerError::DuplicatePath { index });
            }
            let folded = canonical_name.to_ascii_lowercase();
            if folded_names.insert(folded, index).is_some() {
                return Err(ContainerError::DuplicatePath { index });
            }
            if entry.is_symlink() {
                return Err(ContainerError::Symlink { index });
            }
            if entry.encrypted() {
                return Err(ContainerError::EncryptedEntry { index });
            }
            if !matches!(
                entry.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            ) {
                return Err(ContainerError::UnsupportedCompression { index });
            }

            let compressed_size = entry.compressed_size();
            let uncompressed_size = entry.size();
            if uncompressed_size > self.limits.max_entry_uncompressed_bytes {
                return Err(ContainerError::LimitExceeded {
                    limit: ContainerLimit::EntryBytes,
                    index: Some(index),
                });
            }
            if exceeds_ratio(
                uncompressed_size,
                compressed_size,
                self.limits.max_compression_ratio,
            ) {
                return Err(ContainerError::LimitExceeded {
                    limit: ContainerLimit::CompressionRatio,
                    index: Some(index),
                });
            }
            total_uncompressed_size = total_uncompressed_size
                .checked_add(uncompressed_size)
                .ok_or(ContainerError::LimitExceeded {
                    limit: ContainerLimit::TotalBytes,
                    index: Some(index),
                })?;
            if total_uncompressed_size > self.limits.max_total_uncompressed_bytes {
                return Err(ContainerError::LimitExceeded {
                    limit: ContainerLimit::TotalBytes,
                    index: Some(index),
                });
            }

            if !is_directory {
                let read_limit = self
                    .limits
                    .max_entry_uncompressed_bytes
                    .checked_add(1)
                    .unwrap_or(u64::MAX);
                let actual = io::copy(&mut entry.by_ref().take(read_limit), &mut io::sink())
                    .map_err(|_| ContainerError::EntryRead { index })?;
                if actual > self.limits.max_entry_uncompressed_bytes {
                    return Err(ContainerError::LimitExceeded {
                        limit: ContainerLimit::EntryBytes,
                        index: Some(index),
                    });
                }
                if actual != uncompressed_size {
                    return Err(ContainerError::EntryRead { index });
                }
            }

            let kind = classify_part(&canonical_name, is_directory);
            if kind == ContainerPartKind::Unknown {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticCode::UnknownContainerPart,
                    Some(index),
                ));
            }
            parts.push(ContainerPart {
                index,
                name: canonical_name,
                is_directory,
                compressed_size,
                uncompressed_size,
                crc32: entry.crc32(),
                kind,
            });
        }

        Ok(ContainerManifest {
            archive_size,
            total_uncompressed_size,
            parts,
            diagnostics,
        })
    }
}

impl Default for SafeMcdxReader {
    fn default() -> Self {
        Self::new(ContainerLimits::default())
    }
}

fn validate_name(
    index: usize,
    raw_name: &str,
    enclosed_name: Option<&Path>,
    limits: &ContainerLimits,
) -> Result<String, ContainerError> {
    let trimmed_name = validate_raw_name(index, raw_name, limits)?;
    let path = enclosed_name.ok_or(ContainerError::UnsafePath { index })?;
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::CurDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(ContainerError::UnsafePath { index });
    }
    Ok(trimmed_name)
}

fn validate_raw_name(
    index: usize,
    raw_name: &str,
    limits: &ContainerLimits,
) -> Result<String, ContainerError> {
    let trimmed_name = raw_name.trim_end_matches('/');
    let has_drive_prefix = raw_name.split('/').next().is_some_and(|component| {
        let bytes = component.as_bytes();
        bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
    });
    if raw_name.is_empty()
        || trimmed_name.is_empty()
        || raw_name.as_bytes().contains(&0)
        || raw_name.contains('\\')
        || raw_name.starts_with('/')
        || raw_name.ends_with("//")
        || has_drive_prefix
        || raw_name.len() > limits.max_name_bytes
    {
        return Err(if raw_name.len() > limits.max_name_bytes {
            ContainerError::LimitExceeded {
                limit: ContainerLimit::NameBytes,
                index: Some(index),
            }
        } else {
            ContainerError::UnsafePath { index }
        });
    }
    if trimmed_name.split('/').any(str::is_empty)
        || trimmed_name
            .split('/')
            .any(|part| part == "." || part == "..")
    {
        return Err(ContainerError::UnsafePath { index });
    }

    Ok(trimmed_name.to_owned())
}

fn preflight_zip(bytes: &[u8], limits: &ContainerLimits) -> Result<usize, ContainerError> {
    let eocd = find_eocd(bytes).ok_or(ContainerError::InvalidZip)?;
    if read_u16(bytes, eocd + 4)? != 0
        || read_u16(bytes, eocd + 6)? != 0
        || read_u16(bytes, eocd + 8)? != read_u16(bytes, eocd + 10)?
    {
        return Err(ContainerError::InvalidZip);
    }
    let entry_count = usize::from(read_u16(bytes, eocd + 10)?);
    if entry_count == usize::from(u16::MAX) {
        return Err(ContainerError::InvalidZip);
    }
    if entry_count > limits.max_entries {
        return Err(ContainerError::LimitExceeded {
            limit: ContainerLimit::Entries,
            index: None,
        });
    }
    let central_size =
        usize::try_from(read_u32(bytes, eocd + 12)?).map_err(|_| ContainerError::InvalidZip)?;
    let central_start =
        usize::try_from(read_u32(bytes, eocd + 16)?).map_err(|_| ContainerError::InvalidZip)?;
    let central_end = central_start
        .checked_add(central_size)
        .ok_or(ContainerError::InvalidZip)?;
    if central_end != eocd || central_end > bytes.len() {
        return Err(ContainerError::InvalidZip);
    }

    let mut cursor = central_start;
    let mut names = HashSet::new();
    let mut folded_names = HashSet::new();
    let mut ranges = Vec::with_capacity(entry_count);
    for index in 0..entry_count {
        if read_slice(bytes, cursor, 4)? != b"PK\x01\x02" {
            return Err(ContainerError::InvalidZip);
        }
        let flags = read_u16(bytes, cursor + 8)?;
        let method = read_u16(bytes, cursor + 10)?;
        let compressed_size = usize::try_from(read_u32(bytes, cursor + 20)?)
            .map_err(|_| ContainerError::InvalidZip)?;
        let name_len = usize::from(read_u16(bytes, cursor + 28)?);
        let extra_len = usize::from(read_u16(bytes, cursor + 30)?);
        let comment_len = usize::from(read_u16(bytes, cursor + 32)?);
        let external_attributes = read_u32(bytes, cursor + 38)?;
        let local_start = usize::try_from(read_u32(bytes, cursor + 42)?)
            .map_err(|_| ContainerError::InvalidZip)?;
        let header_end = cursor.checked_add(46).ok_or(ContainerError::InvalidZip)?;
        let name_end = header_end
            .checked_add(name_len)
            .ok_or(ContainerError::InvalidZip)?;
        let record_end = name_end
            .checked_add(extra_len)
            .and_then(|end| end.checked_add(comment_len))
            .ok_or(ContainerError::InvalidZip)?;
        if record_end > central_end {
            return Err(ContainerError::InvalidZip);
        }
        let raw_name = str::from_utf8(
            bytes
                .get(header_end..name_end)
                .ok_or(ContainerError::InvalidZip)?,
        )
        .map_err(|_| ContainerError::UnsafePath { index })?;
        let canonical_name = validate_raw_name(index, raw_name, limits)?;
        if !names.insert(canonical_name.clone())
            || !folded_names.insert(canonical_name.to_ascii_lowercase())
        {
            return Err(ContainerError::DuplicatePath { index });
        }
        if flags & 1 != 0 {
            return Err(ContainerError::EncryptedEntry { index });
        }
        if !matches!(method, 0 | 8) {
            return Err(ContainerError::UnsupportedCompression { index });
        }
        if ((external_attributes >> 16) & 0o170000) == 0o120000 {
            return Err(ContainerError::Symlink { index });
        }

        if read_slice(bytes, local_start, 4)? != b"PK\x03\x04" {
            return Err(ContainerError::InvalidZip);
        }
        let local_flags_offset = local_start
            .checked_add(6)
            .ok_or(ContainerError::InvalidZip)?;
        let local_method_offset = local_start
            .checked_add(8)
            .ok_or(ContainerError::InvalidZip)?;
        if read_u16(bytes, local_flags_offset)? != flags
            || read_u16(bytes, local_method_offset)? != method
        {
            return Err(ContainerError::InvalidZip);
        }
        let local_name_length_offset = local_start
            .checked_add(26)
            .ok_or(ContainerError::InvalidZip)?;
        let local_extra_length_offset = local_start
            .checked_add(28)
            .ok_or(ContainerError::InvalidZip)?;
        let local_name_len = usize::from(read_u16(bytes, local_name_length_offset)?);
        let local_extra_len = usize::from(read_u16(bytes, local_extra_length_offset)?);
        let local_name_start = local_start
            .checked_add(30)
            .ok_or(ContainerError::InvalidZip)?;
        let local_name_end = local_name_start
            .checked_add(local_name_len)
            .ok_or(ContainerError::InvalidZip)?;
        if bytes.get(local_name_start..local_name_end) != bytes.get(header_end..name_end) {
            return Err(ContainerError::InvalidZip);
        }
        let data_start = local_name_end
            .checked_add(local_extra_len)
            .ok_or(ContainerError::InvalidZip)?;
        let data_end = data_start
            .checked_add(compressed_size)
            .ok_or(ContainerError::InvalidZip)?;
        if data_end > central_start {
            return Err(ContainerError::InvalidZip);
        }
        ranges.push((local_start, data_end));
        cursor = record_end;
    }
    if cursor != central_end {
        return Err(ContainerError::InvalidZip);
    }
    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[1].0 < pair[0].1) {
        return Err(ContainerError::OverlappingEntries);
    }
    Ok(entry_count)
}

fn find_eocd(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 22 {
        return None;
    }
    let start = bytes.len().saturating_sub(22 + usize::from(u16::MAX));
    (start..=bytes.len() - 22).rev().find(|position| {
        bytes.get(*position..*position + 4) == Some(b"PK\x05\x06")
            && read_u16(bytes, *position + 20)
                .ok()
                .is_some_and(|comment_len| *position + 22 + usize::from(comment_len) == bytes.len())
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, ContainerError> {
    let value = read_slice(bytes, offset, 2)?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, ContainerError> {
    let value = read_slice(bytes, offset, 4)?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn read_slice(bytes: &[u8], offset: usize, length: usize) -> Result<&[u8], ContainerError> {
    let end = offset
        .checked_add(length)
        .ok_or(ContainerError::InvalidZip)?;
    bytes.get(offset..end).ok_or(ContainerError::InvalidZip)
}

fn exceeds_ratio(uncompressed: u64, compressed: u64, maximum: u64) -> bool {
    if uncompressed == 0 {
        return false;
    }
    if compressed == 0 {
        return true;
    }
    uncompressed > compressed.saturating_mul(maximum)
}

fn classify_part(name: &str, is_directory: bool) -> ContainerPartKind {
    if is_directory {
        return ContainerPartKind::Directory;
    }
    if name == "mathcad/worksheet.xml" {
        return ContainerPartKind::Worksheet;
    }
    let media_type_hint = match Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => Some("image/png"),
        Some("jpg" | "jpeg") => Some("image/jpeg"),
        Some("gif") => Some("image/gif"),
        Some("bmp") => Some("image/bmp"),
        Some("svg") => Some("image/svg+xml"),
        _ => None,
    };
    if media_type_hint.is_some() {
        ContainerPartKind::EmbeddedResource { media_type_hint }
    } else {
        ContainerPartKind::Unknown
    }
}
