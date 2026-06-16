//! The commit-scoped `co-change` check (#33): a source file that changed in a
//! git diff must change its colocated test too.
//!
//! Convention: when a source file is **modified** (e.g. a function removed from
//! `foo.py`) or **deleted** in a commit range, its colocated test — the #15/#18
//! pairing, `foo.py` → `foo_test.py`, `foo.ts` → `foo.test.ts` — must also be in
//! that diff. This catches edits and removals that leave the test silently stale.
//! *Added* source files are not subjects: brand-new code is the coverage floor's
//! job, not this one.
//!
//! [`stale_sources`] walks `git diff --name-status <base>...HEAD` for a
//! [`Language`] and returns every changed source file whose colocated test did
//! not co-change. A file listed in the config `exempt` table (rule `co-change`)
//! is a deliberate, reason-required omission. Rust has no sibling test file —
//! units are inline `#[cfg(test)]` in the same `.rs` — so the rule is
//! Python/TypeScript only (the CLI rejects `--language rust`).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::colocated_test::Language;

/// Every source file changed in `repo`'s `<base>...HEAD` diff whose colocated
/// test did not also change — the stale-test risks — sorted for deterministic
/// output.
///
/// A source file is a subject when it was **modified** (and still holds code) or
/// **deleted** in the diff; an **added** file is not (new code is the coverage
/// floor's concern). A subject whose `repo`-relative path is in `exempt` is a
/// deliberate omission and is skipped. Everything else must have its colocated
/// test (`foo.py` → `foo_test.py`, per `language`) somewhere in the same diff.
pub fn stale_sources(
    _repo: &Path,
    _base: &str,
    _language: Language,
    _exempt: &BTreeSet<String>,
) -> Result<Vec<PathBuf>> {
    // Stub (#33): the command surface, exemption plumbing, and source↔test
    // pairing are wired; the git-diff detection lands once the red integration
    // tests are witnessed failing on CI.
    Ok(Vec::new())
}
