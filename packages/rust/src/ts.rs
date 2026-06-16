//! TypeScript isolation analysis (issue #43) — skeleton.
//!
//! The integration direction (#75) is wired here but **reports nothing yet**:
//! detection (an `oxc` AST walk for first-party `vi.mock()` targets) lands in
//! the follow-up `feat` commit and turns the red fixtures green.

use std::path::Path;

use anyhow::Result;

use crate::lint::Violation;

/// Scan the TypeScript test files under `root` for integration-isolation
/// violations.
///
/// Skeleton: reports nothing yet. The `no-first-party-mock` detection lands
/// next (#75).
pub fn find_integration_violations(_root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    Ok(Vec::new())
}
