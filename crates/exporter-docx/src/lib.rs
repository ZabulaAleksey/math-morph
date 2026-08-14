//! Deterministic, fail-closed DOCX export for the supported Document IR subset.

mod error;
mod image;
mod limits;
mod package;
mod validator;
mod xml;

pub use error::{DocxError, DocxLimit, DocxValidationError};
pub use limits::DocxLimits;
pub use package::DocxExporter;
pub use validator::DocxValidator;
