//! Безопасная граница входных форматов Mathcad.

mod diagnostic;
mod format;
mod mcdx;
mod xml_metadata;

pub use diagnostic::{Diagnostic, DiagnosticCode, Severity};
pub use format::{FormatDetection, FormatDetector, FormatError, InputFormat};
pub use mcdx::{
    ContainerError, ContainerLimit, ContainerLimits, ContainerManifest, ContainerPart,
    ContainerPartKind, SafeMcdxReader,
};
pub use xml_metadata::{
    NamespaceBinding, SchemaLocation, XmlMetadata, XmlMetadataError, XmlMetadataLimits,
    inspect_xml_metadata,
};
