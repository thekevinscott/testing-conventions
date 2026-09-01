//! The shared `Violation` type emitted by the deterministic test-code lints.

use std::path::PathBuf;

/// A single lint violation found in a test file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub file: PathBuf,
    /// 1-based line number of the offending construct.
    pub line: usize,
    /// Short lint identifier (e.g. `no-monkeypatch`, `no-out-of-module-call`).
    pub rule: &'static str,
    pub message: String,
}
