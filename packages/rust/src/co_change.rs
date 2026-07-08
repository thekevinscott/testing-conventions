//! The commit-scoped `co-change` check: a source file that changed in a
//! git diff must change its colocated test too.
//!
//! Convention: when a source file is **modified** (e.g. a function removed from
//! `foo.py`) or **deleted** in a commit range, its colocated test — the
//! pairing, `foo.py` → `foo_test.py`, `foo.ts` → `foo.test.ts` — must also be in
//! that diff. This catches edits and removals that leave the test silently stale.
//! *Added* source files are not subjects: brand-new code is the coverage floor's
//! job, not this one. A **deletion** is a subject only if the source *had* a
//! colocated test in the base tree — a package barrel (`__init__.py`, `index.ts`)
//! with no sibling test can be deleted without one appearing in the diff, so it is
//! not flagged and needs no exemption.
//!
//! [`stale_sources`] walks `git diff --name-status <base>...HEAD` for a
//! [`Language`] and returns every changed source file whose colocated test did
//! not co-change. A file listed in the config `exempt` table (rule `co-change`)
//! is a deliberate, reason-required omission. Rust has no sibling test file —
//! units are inline `#[cfg(test)]` in the same `.rs` — so the rule is
//! Python/TypeScript only (the CLI rejects `--language rust`).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::colocated_test::Language;

/// Every source file changed in `repo`'s `<base>...HEAD` diff whose colocated
/// test did not also change — the stale-test risks — sorted for deterministic
/// output.
///
/// A source file is a subject when it was **modified** (and still holds code), or
/// **deleted** while it *had* a colocated test in the base tree (the test now at
/// risk of being orphaned); an **added** file is not (new code is the coverage
/// floor's concern), nor is a deleted barrel that never had a sibling test.
/// A subject whose `repo`-relative path is in `exempt` is a deliberate omission and
/// is skipped. Everything else must have its colocated test (`foo.py` →
/// `foo_test.py`, per `language`) somewhere in the same diff.
///
/// Returns an error if `git diff` fails — e.g. `base` names no resolvable ref —
/// so an un-diffable range surfaces rather than silently passing as "clean".
pub fn stale_sources(
    repo: &Path,
    base: &str,
    language: Language,
    exempt: &BTreeSet<String>,
) -> Result<Vec<PathBuf>> {
    let entries = changed_entries(repo, base)?;
    // Every changed path, so a subject's expected test is a set lookup rather
    // than a second walk of the diff.
    let changed: BTreeSet<&str> = entries.iter().map(|(_, path)| path.as_str()).collect();

    let mut stale = Vec::new();
    for (status, rel) in &entries {
        let path = Path::new(rel);
        // A test file, a support file (Python `conftest.py`), or anything this
        // language doesn't track is never a co-change subject.
        if !language.tracks(path) || language.is_test(path) || language.is_support(path) {
            continue;
        }
        let expected = language
            .expected_test_path(path)
            .to_string_lossy()
            .replace('\\', "/");
        // Only an edit or a removal can leave a test stale; a brand-new source is
        // the coverage floor's concern, not this rule's.
        let is_subject = match status {
            Status::Modified => {
                // An empty / comment-only file holds no logic, so editing it needs
                // no test co-change — consistent with the colocated-test rule.
                let contents = std::fs::read_to_string(repo.join(path))
                    .with_context(|| format!("reading changed source `{rel}`"))?;
                language.has_code(&contents)
            }
            // A deletion is a subject only if the source *had* a colocated test in
            // the base tree — the test now at risk of being orphaned. A source that
            // never had a sibling test (a package barrel: `__init__.py`, `index.ts`)
            // can be removed without a test appearing in the diff, so it is not
            // flagged and needs no exemption to delete it. HEAD can't answer
            // this — the file is gone — so we ask `base`.
            Status::Deleted => test_exists_in_base(repo, base, &expected)?,
            Status::Other => false,
        };
        if !is_subject || exempt.contains(rel) {
            continue;
        }
        if !changed.contains(expected.as_str()) {
            stale.push(path.to_path_buf());
        }
    }
    stale.sort();
    Ok(stale)
}

/// The diff status of a changed file, narrowed to what the rule acts on.
enum Status {
    /// `M` — content changed; a subject if it still holds code.
    Modified,
    /// `D` — removed; a subject only if the source had a colocated test in base
    /// (its test should go too), never for a barrel that never had one.
    Deleted,
    /// `A` (added) and the rest (`T`, …) — not a co-change subject.
    Other,
}

impl Status {
    /// The status from a `git diff --name-status` status field. With
    /// `--no-renames` it is a single letter, so only the first char matters.
    fn from_code(code: &str) -> Status {
        match code.chars().next() {
            Some('M') => Status::Modified,
            Some('D') => Status::Deleted,
            _ => Status::Other,
        }
    }
}

/// `true` when `rel` (a `repo`-relative path) exists as a blob in the `base` tree.
///
/// Used to tell a deleted source that once had a colocated test — its test should
/// be removed too, so a stale leftover is worth flagging — from a barrel that never
/// had one, which can be deleted without a test co-changing. Runs
/// `git cat-file -e <base>:./<rel>`: the `./` makes git resolve the path relative to
/// `repo` (the diff's `--relative` root), matching the paths [`changed_entries`]
/// returns, rather than the repo's top level. A missing blob exits non-zero (→
/// `false`); the `base` ref itself already resolved for [`changed_entries`], so a
/// non-zero exit here means "no such path in base", not a bad ref.
fn test_exists_in_base(repo: &Path, base: &str, rel: &str) -> Result<bool> {
    let spec = format!("{base}:./{rel}");
    let output = Command::new("git")
        .current_dir(repo)
        .args(["cat-file", "-e", &spec])
        .output()
        .with_context(|| format!("running `git cat-file` in `{}`", repo.display()))?;
    Ok(output.status.success())
}

/// The status + `repo`-relative path of every file changed in `<base>...HEAD`,
/// via `git diff --name-status`.
///
/// `<base>...HEAD` is the merge-base diff — the changes this branch introduced
/// (what a PR shows), not whatever else moved on `base`. Rename detection is off
/// (`--no-renames`), so a rename shows as a delete + an add (each its own line of
/// `<status>\t<path>`) and the deleted source is still held to its test;
/// `--relative` scopes the diff to `repo` and reports paths relative to it.
fn changed_entries(repo: &Path, base: &str) -> Result<Vec<(Status, String)>> {
    let range = format!("{base}...HEAD");
    let output = Command::new("git")
        .current_dir(repo)
        .args([
            "diff",
            "--name-status",
            "--no-renames",
            "--relative",
            &range,
        ])
        .output()
        .with_context(|| format!("running `git diff` in `{}`", repo.display()))?;
    if !output.status.success() {
        bail!(
            "`git diff {range}` failed in `{}`: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    for line in stdout.lines() {
        // `<status>\t<path>` — the status is a single letter with `--no-renames`.
        if let Some((status, path)) = line.split_once('\t') {
            let path = path.trim_end_matches('\r').replace('\\', "/");
            entries.push((Status::from_code(status), path));
        }
    }
    Ok(entries)
}
