//! Integration-test lints (issue #19; rules #48–#52) — the `integration lint`
//! command.
//!
//! A *lint* here is a deterministic style/mechanism check on test code, as
//! opposed to the structural `location` / `coverage` rules. This module hosts
//! the mocking mechanism & style lints; more lints will join them under the
//! same command.
//!
//! **Skeleton (#48):** the lint set is empty, so [`find_violations`] reports
//! nothing yet. The first lint — forbid `monkeypatch` (#49) — lands next and
//! turns the red tests in `tests/integration_lint.rs` /
//! `tests/integration_lint_e2e.rs` green.

use std::path::{Path, PathBuf};

use anyhow::Result;

/// A single lint violation found in a test file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// File the violation was found in.
    pub file: PathBuf,
    /// 1-based line number of the offending construct.
    pub line: usize,
    /// Short lint identifier (e.g. `no-monkeypatch`).
    pub rule: &'static str,
    /// Human-readable explanation.
    pub message: String,
}

/// Scan the Python test files under `root` and return every lint violation,
/// sorted for deterministic output.
///
/// **Skeleton (#48):** no lints are wired yet, so this always returns an empty
/// list. The bright-line lints (#49–#52) populate it. The red tests asserting
/// the `red` fixture is flagged therefore fail until #49 lands — which is the
/// point.
pub fn find_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    // Bound so the skeleton compiles with the eventual signature; detection
    // (which actually walks `root`) arrives with #49.
    let _ = root.as_ref();
    Ok(Vec::new())
}
