//! Deterministic, bounded Presentation MathML export for the supported scalar AST subset.

mod error;
mod renderer;

pub use error::{MathMlError, MathMlLimit};
pub use renderer::{MathMlFragment, MathMlLimits, MathMlRenderer};
