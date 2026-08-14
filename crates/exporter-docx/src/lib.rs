//! Deterministic, fail-closed DOCX export for the supported Document IR subset.

mod error;
mod image;
mod limits;
mod omml;
mod package;
mod validator;
mod xml;

pub use error::{DocxError, DocxLimit, DocxValidationError, OmmlError, OmmlLimit};
pub use limits::DocxLimits;
pub use omml::{OmmlFragment, OmmlLimits, WordEquationExporter};
pub use package::DocxExporter;
pub use validator::DocxValidator;
