use thiserror::Error;

use document_ir::DocumentIrValidationError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocxLimit {
    OutputBytes,
    Entries,
    XmlBytes,
    Blocks,
    Paragraphs,
    Runs,
    Images,
    ImageBytes,
    TotalAssetBytes,
    ImagePixels,
    ImageDimension,
    EntryBytes,
    TotalBytes,
    CompressionRatio,
    XmlDepth,
    XmlNodes,
    PartNameBytes,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum DocxError {
    #[error("document IR is invalid")]
    InvalidDocument,
    #[error("multiple pages are not supported by this DOCX subset")]
    MultiplePagesUnsupported,
    #[error("document content is not supported by this DOCX stage")]
    UnsupportedContent,
    #[error("document text is not valid XML 1.0 content")]
    InvalidXmlText,
    #[error("document text style is invalid")]
    InvalidTextStyle,
    #[error("an image asset is unavailable")]
    MissingAsset,
    #[error("image asset access was rejected")]
    RejectedAsset,
    #[error("an asset identifier is used more than once")]
    DuplicateAssetId,
    #[error("an image requires an explicit physical size")]
    MissingImageSize,
    #[error("image media type does not match its encoded content")]
    MediaTypeMismatch,
    #[error("image data is malformed")]
    MalformedImage,
    #[error("image metadata is not allowed")]
    ImageMetadataForbidden,
    #[error("checked DOCX unit conversion overflowed")]
    ArithmeticOverflow,
    #[error("DOCX limit exceeded: {0:?}")]
    LimitExceeded(DocxLimit),
    #[error("DOCX package generation failed")]
    PackageWrite,
    #[error("generated DOCX package failed structural validation")]
    GeneratedPackageInvalid,
}

impl From<DocumentIrValidationError> for DocxError {
    fn from(_: DocumentIrValidationError) -> Self {
        Self::InvalidDocument
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum DocxValidationError {
    #[error("input is not a valid DOCX ZIP package")]
    InvalidZip,
    #[error("DOCX limit exceeded: {0:?}")]
    LimitExceeded(DocxLimit),
    #[error("DOCX contains an unsafe part name")]
    UnsafePartName,
    #[error("DOCX contains duplicate or case-colliding parts")]
    DuplicatePart,
    #[error("DOCX contains an encrypted part")]
    EncryptedPart,
    #[error("DOCX contains a symbolic-link part")]
    SymlinkPart,
    #[error("DOCX contains an unsupported compression method")]
    UnsupportedCompression,
    #[error("DOCX is missing a required part")]
    MissingRequiredPart,
    #[error("DOCX contains a part outside the supported subset")]
    UnexpectedPart,
    #[error("DOCX content types are invalid")]
    InvalidContentTypes,
    #[error("DOCX relationships are invalid")]
    InvalidRelationships,
    #[error("DOCX contains an external relationship")]
    ExternalRelationship,
    #[error("DOCX contains a broken internal relationship")]
    BrokenRelationship,
    #[error("DOCX document XML is invalid")]
    InvalidDocumentXml,
    #[error("DTD declarations are forbidden in DOCX XML")]
    DtdForbidden,
    #[error("DOCX contains forbidden active content")]
    ForbiddenContent,
    #[error("DOCX image declarations are inconsistent")]
    ImageMismatch,
    #[error("DOCX contains a duplicate drawing identifier")]
    DuplicateDrawingId,
}
