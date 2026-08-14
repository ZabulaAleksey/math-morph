use std::fmt;

use thiserror::Error;

use crate::{AssetRefIr, MediaTypeIr};
use math_model::MathExpression;

/// Backend-neutral equation export boundary. The output type belongs to an adapter.
pub trait EquationExporter {
    type Output;
    type Error;

    fn export(&self, expression: &MathExpression) -> Result<Self::Output, Self::Error>;
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AssetResolveError {
    #[error("asset is unavailable")]
    Unavailable,
    #[error("asset access was rejected")]
    Rejected,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ResolvedAsset {
    pub media_type: MediaTypeIr,
    pub bytes: Vec<u8>,
}

impl fmt::Debug for ResolvedAsset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedAsset")
            .field("media_type", &self.media_type)
            .field("byte_len", &self.bytes.len())
            .finish()
    }
}

pub trait AssetResolver {
    fn resolve(&self, reference: &AssetRefIr) -> Result<ResolvedAsset, AssetResolveError>;
}
