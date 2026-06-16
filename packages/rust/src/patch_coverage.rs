//! Patch (changed-line) coverage (Python — #132, parent #46).
//!
//! Enforces the README Coverage rule's changed-line guarantee: every line a diff
//! touches must be covered by the unit suite. Where [`crate::coverage`] measures
//! the *whole* suite against a floor (#26) and the #131 ratchet against a
//! baseline, this measures only the lines `<base>...HEAD` added or modified —
//! failing when any changed, executable line is left uncovered.
//!
//! Two inputs are combined:
//!   - the **diff** — [`changed_lines`] runs `git diff --unified=0 <base>...HEAD`
//!     and returns the new-side line numbers each file gained. This is the diff
//!     machinery established here, shared by the forthcoming TypeScript / Rust
//!     twins.
//!   - the **coverage** — coverage.py's per-file `missing_lines` /
//!     `missing_branches` ([`crate::coverage::measure_patch_report`]). A changed
//!     line is uncovered when it is a missing line, or the source of a branch the
//!     suite never took. Non-executable changed lines (comments, blanks) and
//!     `coverage`-exempt files have nothing to cover and are skipped.
//!
//! Relationship to the commit-scoped co-change rule ([`crate::co_change`], #33):
//! co-change enforces that a changed source and its colocated *test* move
//! together; patch coverage enforces that the changed *lines* are actually
//! exercised. They are complementary, not overlapping — co-change can pass (the
//! test file changed) while patch coverage fails (the change isn't covered), and
//! vice versa.

use std::path::Path;

use anyhow::Result;

/// A changed source line the unit suite doesn't cover — a `root`-relative path
/// and the 1-based new-side line number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uncovered {
    /// `root`-relative path of the changed file.
    pub file: String,
    /// The 1-based new-side line number that isn't covered.
    pub line: u64,
}

/// Every line added or modified in `root`'s `<base>...HEAD` diff that the unit
/// suite doesn't cover, sorted for deterministic output. `omit` is the
/// `coverage`-rule exemptions (as in [`crate::coverage::measure`]) — an exempt
/// file's changed lines needn't be covered. Requires coverage.py + pytest + git.
pub fn check(_root: &Path, _base: &str, _omit: &[String]) -> Result<Vec<Uncovered>> {
    // Stub (#132): the `unit patch-coverage` command surface, the `--base` diff
    // source, and the `coverage`-exemption plumbing are wired; the git-diff +
    // coverage detection lands once the red integration tests are witnessed
    // failing on CI (and the e2e tests fail locally).
    Ok(Vec::new())
}
