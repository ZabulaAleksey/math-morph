//! Experimental, bounded MathType integration boundary backed by generated Presentation MathML.
//!
//! This adapter does not call MathType, Microsoft Office, COM, OLE, the filesystem, or a
//! network service. It only turns a supported [`math_model::MathExpression`] into an opaque
//! Presentation MathML payload for a future, separately reviewed bridge.

use std::error::Error;
use std::fmt;

use document_ir::ports::EquationExporter;
use exporter_mathml::{MathMlError, MathMlFragment, MathMlLimits, MathMlRenderer};
use math_model::MathExpression;

/// Media type exposed for the generated Presentation MathML payload.
pub const MATHTYPE_MATHML_MEDIA_TYPE: &str = "application/mathml+xml";

/// The only payload format produced by the experimental stage-092 adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathTypePayloadFormat {
    PresentationMathMl,
}

/// Opaque payload prepared for a future MathType bridge.
///
/// Callers cannot construct this value from arbitrary XML. It can only be produced by
/// [`MathTypeAdapter`], which delegates to the bounded allowlist renderer in `exporter-mathml`.
#[derive(Clone, Eq, PartialEq)]
pub struct MathTypePayload {
    fragment: MathMlFragment,
}

impl MathTypePayload {
    pub const fn format(&self) -> MathTypePayloadFormat {
        MathTypePayloadFormat::PresentationMathMl
    }

    pub const fn media_type(&self) -> &'static str {
        MATHTYPE_MATHML_MEDIA_TYPE
    }

    pub fn as_mathml(&self) -> &str {
        self.fragment.as_str()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.fragment.as_str().as_bytes()
    }

    pub fn byte_len(&self) -> usize {
        self.fragment.byte_len()
    }
}

impl fmt::Debug for MathTypePayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MathTypePayload")
            .field("format", &self.format())
            .field("byte_len", &self.byte_len())
            .finish_non_exhaustive()
    }
}

/// Redacted typed failure from the experimental adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MathTypeError {
    mathml: MathMlError,
}

impl MathTypeError {
    pub const fn mathml_error(&self) -> MathMlError {
        self.mathml
    }
}

impl From<MathMlError> for MathTypeError {
    fn from(mathml: MathMlError) -> Self {
        Self { mathml }
    }
}

impl fmt::Display for MathTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "MathType adapter could not produce bounded Presentation MathML: {}",
            self.mathml
        )
    }
}

impl Error for MathTypeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.mathml)
    }
}

/// Pure, offline adapter from the supported math AST subset to an opaque MathML payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MathTypeAdapter {
    renderer: MathMlRenderer,
}

impl MathTypeAdapter {
    pub const fn new(limits: MathMlLimits) -> Self {
        Self {
            renderer: MathMlRenderer::new(limits),
        }
    }

    pub const fn limits(&self) -> &MathMlLimits {
        self.renderer.limits()
    }

    pub fn adapt_expression(
        &self,
        expression: &MathExpression,
    ) -> Result<MathTypePayload, MathTypeError> {
        let fragment = self.renderer.export_expression(expression)?;
        Ok(MathTypePayload { fragment })
    }
}

impl Default for MathTypeAdapter {
    fn default() -> Self {
        Self::new(MathMlLimits::default())
    }
}

impl EquationExporter for MathTypeAdapter {
    type Output = MathTypePayload;
    type Error = MathTypeError;

    fn export(&self, expression: &MathExpression) -> Result<Self::Output, Self::Error> {
        self.adapt_expression(expression)
    }
}
