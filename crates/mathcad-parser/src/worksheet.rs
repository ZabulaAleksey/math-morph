use std::fmt;

use thiserror::Error;

use crate::{Diagnostic, OpaqueFragment, Region, SourceDocument};

const MIB: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorksheetLimits {
    pub max_input_bytes: usize,
    pub max_xml_depth: usize,
    pub max_xml_nodes: usize,
    pub max_regions: usize,
    pub max_namespace_declarations: usize,
    pub max_attributes_per_element: usize,
    pub max_token_bytes: usize,
    pub max_attribute_value_bytes: usize,
    pub max_retained_text_bytes: usize,
}

impl Default for WorksheetLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 32 * MIB,
            max_xml_depth: 128,
            max_xml_nodes: 250_000,
            max_regions: 50_000,
            max_namespace_declarations: 64,
            max_attributes_per_element: 256,
            max_token_bytes: 16 * 1024,
            max_attribute_value_bytes: 16 * 1024,
            max_retained_text_bytes: 8 * MIB,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorksheetLimit {
    InputBytes,
    XmlDepth,
    XmlNodes,
    Regions,
    NamespaceDeclarations,
    Attributes,
    TokenBytes,
    AttributeValueBytes,
    RetainedTextBytes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoordinateError {
    Missing,
    Malformed,
    NonFinite,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum WorksheetError {
    #[error("worksheet input exceeds a configured limit")]
    LimitExceeded(WorksheetLimit),
    #[error("worksheet XML DOCTYPE is forbidden")]
    DoctypeForbidden,
    #[error("worksheet XML uses an unsupported encoding")]
    UnsupportedEncoding,
    #[error("worksheet XML is malformed")]
    MalformedXml,
    #[error("worksheet XML uses an undeclared namespace prefix")]
    UnknownNamespacePrefix,
    #[error("worksheet root QName is unsupported")]
    UnsupportedRoot,
    #[error("worksheet version is unsupported")]
    UnsupportedVersion,
    #[error("worksheet region is missing a required identifier")]
    MissingRegionId,
    #[error("worksheet contains a duplicate region identifier")]
    DuplicateRegionId,
    #[error("worksheet region identifier is malformed")]
    MalformedRegionId,
    #[error("worksheet region coordinate is invalid")]
    InvalidCoordinate {
        field: &'static str,
        reason: CoordinateError,
    },
    #[error("worksheet region z-order is malformed")]
    MalformedZOrder,
    #[error("worksheet math region must contain exactly one math30 expression")]
    InvalidMathExpressionCount,
    #[error("worksheet math expression namespace is unsupported")]
    UnsupportedMathNamespace,
    #[error("worksheet picture metadata is malformed")]
    MalformedPicture,
    #[error("worksheet result format is malformed")]
    MalformedResultFormat,
    #[error("worksheet custom metadata value is malformed")]
    MalformedCustomValue,
    #[error("worksheet text paragraph is missing its style")]
    MissingTextStyle,
    #[error("worksheet boolean attribute is malformed")]
    MalformedBoolean,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct WorksheetIdentityInfo {
    pub document_id: Option<String>,
    pub branch_id: Option<String>,
    pub version_id: Option<String>,
    pub parent_version_id: Option<String>,
    pub revision: Option<String>,
    pub saved_on: Option<String>,
    pub comment: Option<OpaqueFragment>,
    pub opaque_fragments: Vec<OpaqueFragment>,
}

impl fmt::Debug for WorksheetIdentityInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorksheetIdentityInfo")
            .field("present_fields", &self.present_fields())
            .field("has_comment", &self.comment.is_some())
            .field("opaque_fragment_count", &self.opaque_fragments.len())
            .finish()
    }
}

impl WorksheetIdentityInfo {
    fn present_fields(&self) -> usize {
        [
            &self.document_id,
            &self.branch_id,
            &self.version_id,
            &self.parent_version_id,
            &self.revision,
            &self.saved_on,
        ]
        .into_iter()
        .filter(|value| value.is_some())
        .count()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CustomValueKind {
    Date,
    Number,
    Text,
    YesNo,
}

#[derive(Clone, Eq, PartialEq)]
pub struct WorksheetCustomValue {
    pub name: String,
    pub kind: CustomValueKind,
    pub value: String,
    pub span: crate::SourceSpan,
}

impl fmt::Debug for WorksheetCustomValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorksheetCustomValue")
            .field("kind", &self.kind)
            .field("span", &self.span)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct WorksheetUserData {
    pub author: Option<String>,
    pub company: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub revised_by: Option<String>,
    pub title: Option<String>,
    pub custom_values: Vec<WorksheetCustomValue>,
    pub opaque_fragments: Vec<OpaqueFragment>,
}

impl fmt::Debug for WorksheetUserData {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorksheetUserData")
            .field("text_field_count", &self.text_field_count())
            .field("custom_value_count", &self.custom_values.len())
            .field("opaque_fragment_count", &self.opaque_fragments.len())
            .finish()
    }
}

impl WorksheetUserData {
    fn text_field_count(&self) -> usize {
        [
            &self.author,
            &self.company,
            &self.description,
            &self.keywords,
            &self.revised_by,
            &self.title,
        ]
        .into_iter()
        .filter(|value| value.is_some())
        .count()
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct WorksheetMetadata {
    pub generator: Option<String>,
    pub identity_info: WorksheetIdentityInfo,
    pub user_data: WorksheetUserData,
    pub opaque_fragments: Vec<OpaqueFragment>,
}

impl fmt::Debug for WorksheetMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorksheetMetadata")
            .field("has_generator", &self.generator.is_some())
            .field("identity_info", &self.identity_info)
            .field("user_data", &self.user_data)
            .field("opaque_fragment_count", &self.opaque_fragments.len())
            .finish()
    }
}

#[derive(Clone, PartialEq)]
pub struct Worksheet {
    pub version: String,
    pub source: SourceDocument,
    pub metadata: Option<WorksheetMetadata>,
    /// Canonical document order. This vector is never visually sorted.
    pub regions: Vec<Region>,
    pub diagnostics: Vec<Diagnostic>,
}

impl fmt::Debug for Worksheet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Worksheet")
            .field("version", &self.version)
            .field("source", &self.source)
            .field("has_metadata", &self.metadata.is_some())
            .field("region_count", &self.regions.len())
            .field("diagnostic_count", &self.diagnostics.len())
            .finish()
    }
}

impl Worksheet {
    pub fn visual_order(&self) -> Vec<&Region> {
        let mut regions: Vec<_> = self.regions.iter().collect();
        regions.sort_by(|left, right| left.visual_cmp(right));
        regions
    }

    pub fn z_order(&self) -> Vec<&Region> {
        let mut regions: Vec<_> = self.regions.iter().collect();
        regions.sort_by(|left, right| left.z_cmp(right));
        regions
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorksheetParser {
    limits: WorksheetLimits,
}

impl WorksheetParser {
    pub const fn new(limits: WorksheetLimits) -> Self {
        Self { limits }
    }

    pub fn parse(&self, bytes: &[u8]) -> Result<Worksheet, WorksheetError> {
        crate::xml_worksheet::parse_worksheet(bytes, self.limits)
    }
}

impl Default for WorksheetParser {
    fn default() -> Self {
        Self::new(WorksheetLimits::default())
    }
}
