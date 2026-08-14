//! Versioned, backend-neutral document interchange model.

mod model;
pub mod ports;
mod serialization;

pub use model::*;
pub use serialization::{DEFAULT_MAX_SERIALIZED_BYTES, DocumentIrError};
