//! The shared `Violation` type emitted by the deterministic test-code lints.
//!
//! Both the Python `lint` module and the Rust `isolation` module report findings
//! as a [`Violation`], so the CLI prints every rule the same way
//! (`path:line: rule — message`). Hoisted here so neither lint module owns
//! the other's type.

use std::path::PathBuf;

/// A single lint violation found in a test file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// File the violation was found in.
    pub file: PathBuf,
    /// 1-based line number of the offending construct.
    pub line: usize,
    /// Short lint identifier (e.g. `no-monkeypatch`, `no-out-of-module-call`).
    pub rule: &'static str,
    /// Human-readable explanation.
    pub message: String,
}
