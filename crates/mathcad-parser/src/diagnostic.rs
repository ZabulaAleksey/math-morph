/// Стабильный код ограниченной диагностики входной границы.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCode {
    FileExtensionMismatch,
    UnknownContainerPart,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileExtensionMismatch => "FILE_EXTENSION_MISMATCH",
            Self::UnknownContainerPart => "UNKNOWN_CONTAINER_PART",
        }
    }
}

/// Severity до появления общего DiagnosticsCollector в этапе 144.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Warning,
}

/// Диагностика не содержит payload или пользовательское имя файла.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub entry_index: Option<usize>,
}

impl Diagnostic {
    pub(crate) const fn warning(code: DiagnosticCode, entry_index: Option<usize>) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            entry_index,
        }
    }
}
