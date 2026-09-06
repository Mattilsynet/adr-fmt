//! Diagnostic reporting for ADR validation.
//!
//! Per AFM-0003, all rule findings are warnings; the tool exits 0 on
//! lint completion regardless of warning count. Exit 1 is reserved for
//! infrastructure failures (missing config, unreadable files, invalid
//! configuration), which surface as errors returned from `run` and are
//! mapped to an exit code at the `src/main.rs` boundary — the only
//! authorised `process::exit` site per AFM-0026:R4 — not through this
//! diagnostic channel.

use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DiagnosticSource {
    Global,
    Document(PathBuf),
}

#[derive(Debug)]
pub(crate) struct SourcedDiagnostic {
    pub source: DiagnosticSource,
    pub diagnostic: Diagnostic,
}

#[derive(Debug, Default)]
pub(crate) struct DiagnosticBatch {
    diagnostics: Vec<Diagnostic>,
    sources: Vec<DiagnosticSource>,
}

impl DiagnosticBatch {
    pub(crate) fn push(&mut self, item: SourcedDiagnostic) {
        self.diagnostics.push(item.diagnostic);
        self.sources.push(item.source);
    }

    pub(crate) fn document(&mut self, path: &Path, diagnostics: Vec<Diagnostic>) {
        for item in document_diagnostics(path, diagnostics) {
            self.push(item);
        }
    }

    pub(crate) fn into_sourced(self) -> Vec<SourcedDiagnostic> {
        self.sources
            .into_iter()
            .zip(self.diagnostics)
            .map(|(source, diagnostic)| SourcedDiagnostic { source, diagnostic })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

impl std::ops::Deref for DiagnosticBatch {
    type Target = [Diagnostic];

    fn deref(&self) -> &Self::Target {
        &self.diagnostics
    }
}

pub(crate) fn document_diagnostics(
    path: &Path,
    diagnostics: Vec<Diagnostic>,
) -> Vec<SourcedDiagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| SourcedDiagnostic {
            source: DiagnosticSource::Document(path.to_path_buf()),
            diagnostic,
        })
        .collect()
}

/// Diagnostic severity. Only `Warning` is emitted today per AFM-0003
/// advisory-only semantics. The enum is kept as a single-variant type
/// to leave room for a future `--error-on-warning` mode without breaking
/// the diagnostic API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Warning => f.write_str("warning"),
        }
    }
}

/// A single diagnostic message attached to a file and optional line.
#[derive(Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub rule: &'static str,
    pub file: String,
    pub line: usize,
    pub message: String,
    /// Internal diagnostics are not shown to users.
    pub internal: bool,
}

impl Diagnostic {
    #[must_use]
    pub fn warning(rule: &'static str, file: &Path, line: usize, message: String) -> Self {
        Self {
            severity: Severity::Warning,
            rule,
            file: file.display().to_string(),
            line,
            message,
            internal: false,
        }
    }
}
