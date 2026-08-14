use std::fmt;
use std::sync::Arc;

/// Half-open byte range in the immutable source XML buffer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    pub const fn new(start: usize, end: usize) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub const fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

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

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ExpandedName {
    pub namespace_uri: Option<Arc<str>>,
    pub local_name: String,
}

impl fmt::Debug for ExpandedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExpandedName")
            .field("has_namespace", &self.namespace_uri.is_some())
            .field("local_name_bytes", &self.local_name.len())
            .finish()
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
