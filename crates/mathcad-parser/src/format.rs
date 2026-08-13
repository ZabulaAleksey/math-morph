use std::path::Path;

use thiserror::Error;

use crate::{
    ContainerError, ContainerLimits, Diagnostic, DiagnosticCode, SafeMcdxReader, XmlMetadataError,
    XmlMetadataLimits, inspect_xml_metadata,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputFormat {
    Xmcd,
    Mcdx,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormatDetection {
    pub declared: InputFormat,
    pub detected: InputFormat,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormatDetector {
    container_limits: ContainerLimits,
    xml_limits: XmlMetadataLimits,
}

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("XML-like input could not be inspected safely")]
    Xml(#[from] XmlMetadataError),
    #[error("ZIP-like input could not be inspected safely")]
    Container(#[from] ContainerError),
}

impl FormatDetector {
    pub const fn new(container_limits: ContainerLimits, xml_limits: XmlMetadataLimits) -> Self {
        Self {
            container_limits,
            xml_limits,
        }
    }

    pub fn detect(
        &self,
        bytes: &[u8],
        file_name: Option<&str>,
    ) -> Result<FormatDetection, FormatError> {
        let declared = declared_format(file_name);
        let detected = if looks_like_zip(bytes) {
            let manifest = SafeMcdxReader::new(self.container_limits).inspect(bytes)?;
            if manifest.worksheet_part().is_some() {
                InputFormat::Mcdx
            } else {
                InputFormat::Unknown
            }
        } else if looks_like_xml(bytes) {
            let metadata = inspect_xml_metadata(bytes, self.xml_limits)?;
            if metadata.root_local_name == "worksheet"
                && metadata
                    .root_namespace_uri
                    .as_deref()
                    .is_some_and(is_mathsoft_worksheet_namespace)
            {
                InputFormat::Xmcd
            } else {
                InputFormat::Unknown
            }
        } else {
            InputFormat::Unknown
        };

        let diagnostics = if declared != InputFormat::Unknown
            && detected != InputFormat::Unknown
            && declared != detected
        {
            vec![Diagnostic::warning(
                DiagnosticCode::FileExtensionMismatch,
                None,
            )]
        } else {
            Vec::new()
        };
        Ok(FormatDetection {
            declared,
            detected,
            diagnostics,
        })
    }
}

impl Default for FormatDetector {
    fn default() -> Self {
        Self::new(ContainerLimits::default(), XmlMetadataLimits::default())
    }
}

fn declared_format(file_name: Option<&str>) -> InputFormat {
    match file_name
        .and_then(|name| Path::new(name).extension())
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("xmcd") => InputFormat::Xmcd,
        Some("mcdx") => InputFormat::Mcdx,
        _ => InputFormat::Unknown,
    }
}

fn looks_like_xml(bytes: &[u8]) -> bool {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    bytes
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
        == Some(b'<')
}

fn looks_like_zip(bytes: &[u8]) -> bool {
    matches!(
        bytes.get(..4),
        Some([b'P', b'K', 3, 4] | [b'P', b'K', 5, 6] | [b'P', b'K', 7, 8])
    )
}

fn is_mathsoft_worksheet_namespace(namespace: &str) -> bool {
    namespace
        .strip_prefix("http://schemas.mathsoft.com/worksheet")
        .is_some_and(|version| {
            !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())
        })
}
