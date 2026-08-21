use std::io::{self, Write};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::{
    DOCUMENT_IR_SCHEMA_VERSION, DOCUMENT_IR_V3_SCHEMA_VERSION, DocumentIrV1, DocumentIrV3,
    DocumentIrValidationError, VersionedDocumentIr,
};

pub const DEFAULT_MAX_SERIALIZED_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum DocumentIrError {
    #[error("document IR serialized input exceeds its configured limit")]
    InputLimitExceeded,
    #[error("document IR serialized output exceeds its configured limit")]
    OutputLimitExceeded,
    #[error("document IR schema version is unsupported")]
    UnsupportedVersion,
    #[error("document IR serialization is malformed")]
    Malformed,
    #[error("document IR violates a model invariant")]
    Invalid(#[from] DocumentIrValidationError),
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EnvelopeV1 {
    schema_version: u16,
    document: DocumentIrV1,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EnvelopeV3 {
    schema_version: u16,
    document: DocumentIrV3,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AnyEnvelope {
    V1(EnvelopeV1),
    V3(EnvelopeV3),
}

#[derive(Deserialize)]
struct VersionProbe {
    schema_version: u16,
}

impl Serialize for VersionedDocumentIr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::V1(document) => EnvelopeV1 {
                schema_version: DOCUMENT_IR_SCHEMA_VERSION,
                document: document.clone(),
            }
            .serialize(serializer),
            Self::V3(document) => EnvelopeV3 {
                schema_version: DOCUMENT_IR_V3_SCHEMA_VERSION,
                document: document.clone(),
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for VersionedDocumentIr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match AnyEnvelope::deserialize(deserializer)? {
            AnyEnvelope::V1(envelope) if envelope.schema_version == DOCUMENT_IR_SCHEMA_VERSION => {
                Ok(Self::V1(envelope.document))
            }
            AnyEnvelope::V3(envelope)
                if envelope.schema_version == DOCUMENT_IR_V3_SCHEMA_VERSION =>
            {
                Ok(Self::V3(envelope.document))
            }
            _ => Err(D::Error::custom("unsupported document IR schema version")),
        }
    }
}

impl VersionedDocumentIr {
    pub fn to_json(&self) -> Result<Vec<u8>, DocumentIrError> {
        self.to_json_with_limit(DEFAULT_MAX_SERIALIZED_BYTES)
    }

    pub fn to_json_with_limit(&self, max_bytes: usize) -> Result<Vec<u8>, DocumentIrError> {
        self.validate()?;
        let mut writer = LimitedWriter::new(max_bytes);
        if serde_json::to_writer(&mut writer, self).is_err() {
            return if writer.limit_exceeded {
                Err(DocumentIrError::OutputLimitExceeded)
            } else {
                Err(DocumentIrError::Malformed)
            };
        }
        Ok(writer.bytes)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, DocumentIrError> {
        Self::from_json_with_limit(bytes, DEFAULT_MAX_SERIALIZED_BYTES)
    }

    pub fn from_json_with_limit(bytes: &[u8], max_bytes: usize) -> Result<Self, DocumentIrError> {
        if bytes.len() > max_bytes {
            return Err(DocumentIrError::InputLimitExceeded);
        }
        let probe: VersionProbe =
            serde_json::from_slice(bytes).map_err(|_| DocumentIrError::Malformed)?;
        if !matches!(
            probe.schema_version,
            DOCUMENT_IR_SCHEMA_VERSION | DOCUMENT_IR_V3_SCHEMA_VERSION
        ) {
            return Err(DocumentIrError::UnsupportedVersion);
        }
        let document: Self =
            serde_json::from_slice(bytes).map_err(|_| DocumentIrError::Malformed)?;
        document.validate()?;
        Ok(document)
    }
}

struct LimitedWriter {
    bytes: Vec<u8>,
    max_bytes: usize,
    limit_exceeded: bool,
}

impl LimitedWriter {
    fn new(max_bytes: usize) -> Self {
        Self {
            bytes: Vec::new(),
            max_bytes,
            limit_exceeded: false,
        }
    }
}

impl Write for LimitedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let Some(next_len) = self.bytes.len().checked_add(buffer.len()) else {
            self.limit_exceeded = true;
            return Err(io::Error::other("document IR output limit exceeded"));
        };
        if next_len > self.max_bytes {
            self.limit_exceeded = true;
            return Err(io::Error::other("document IR output limit exceeded"));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
