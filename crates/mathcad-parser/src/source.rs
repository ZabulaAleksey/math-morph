use std::fmt;
use std::sync::Arc;

pub use math_model::{ExpandedName, SourceSpan};

/// Shared immutable source backing all spans and opaque fragments.
#[derive(Clone, Eq, PartialEq)]
pub struct SourceDocument(Arc<[u8]>);

impl SourceDocument {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(Arc::from(bytes))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn bytes(&self, span: SourceSpan) -> Option<&[u8]> {
        self.0.get(span.start..span.end)
    }
}

impl fmt::Debug for SourceDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceDocument")
            .field("byte_len", &self.0.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct OpaqueFragment {
    pub name: ExpandedName,
    pub span: SourceSpan,
}

impl fmt::Debug for OpaqueFragment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpaqueFragment")
            .field("span", &self.span)
            .finish_non_exhaustive()
    }
}
