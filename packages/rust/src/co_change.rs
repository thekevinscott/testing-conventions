//! The commit-scoped `co-change` check: a source file that changed in a
//! git diff must change its colocated test too.
//!
//! Convention: when a source file is **modified** (e.g. a function removed from
//! `foo.py`) or **deleted** in a commit range, its colocated test — the
//! pairing, `foo.py` → `foo_test.py`, `foo.ts` → `foo.test.ts` — must also be in
//! that diff. This catches edits and removals that leave the test silently stale.
//! A modification counts when it changes the code the compiler sees: the file at the
//! merge base and the file at HEAD are compared with comments and formatting
//! whitespace normalized away ([`Language::same_code`]), so a comment reword or a
//! whitespace sweep leaves the test current and passes.
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
/// A source file is a subject when it was **modified** into a different program while
/// still declaring behavior ([`Language::is_subject`], the predicate the presence rule
/// reads, and [`Language::same_code`], which reads what the edit changed), or **deleted**
/// while it *had* a colocated test in the base tree (the test now at risk of being
/// orphaned); an **added** file is not (new code is the coverage floor's concern),
/// nor is a deleted barrel that never had a sibling test.
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
    let fork_point = merge_base(repo, base)?;
    // Every changed path, so a subject's expected test is a set lookup rather
    // than a second walk of the diff.
    let changed: BTreeSet<&str> = entries.iter().map(|(_, path)| path.as_str()).collect();
    // `<package root>/tests/` belongs to the suite tiers (integration / e2e),
    // so nothing under it is a co-change subject.
    let suite_tests = match language {
        Language::Python => crate::tiers::suite_tests_dir(repo, "pyproject.toml"),
        Language::TypeScript => crate::tiers::suite_tests_dir(repo, "package.json"),
        Language::Rust => None,
    };

    let mut stale = Vec::new();
    for (status, rel) in &entries {
        let path = Path::new(rel);
        // A test file, a support file (Python `conftest.py`), or anything this
        // language doesn't track is never a co-change subject.
        if !language.tracks(path) || language.is_test(path) || language.is_support(path) {
            continue;
        }
        if suite_tests
            .as_ref()
            .is_some_and(|tests| repo.join(path).starts_with(tests))
        {
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
                // The file's own contents decide, through the predicate presence reads:
                // an empty / comment-only file and a type-only TypeScript module hold no
                // behavior, so editing one cannot leave a test stale. Deciding on the
                // diff's shape alone would flag a module for having no colocated test —
                // the fact presence uses to skip it.
                let contents = std::fs::read_to_string(repo.join(path))
                    .with_context(|| format!("reading changed source `{rel}`"))?;
                // What the edit did decides the rest: a comment reword or a whitespace
                // sweep leaves the compiler the same program, so the colocated test still
                // pins the behavior the file has.
                language.is_subject(&contents, path)
                    && !language.same_code(&blob_at(repo, &fork_point, rel)?, &contents, path)
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
    /// `M` — content changed; a subject if the file still declares behavior.
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

/// The commit `<base>...HEAD` diffs from — the merge base of `base` and HEAD.
///
/// The modify arm compares a subject against its contents *here*, not at `base`'s tip: the
/// tip carries commits this branch never saw, so a file the branch only commented would read
/// as a code change. [`changed_entries`] already resolved the three-dot range, which needs the
/// same merge base, so a failure here names a repo whose history moved underfoot.
fn merge_base(repo: &Path, base: &str) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(["merge-base", base, "HEAD"])
        .output()
        .with_context(|| format!("running `git merge-base` in `{}`", repo.display()))?;
    if !output.status.success() {
        bail!(
            "`git merge-base {base} HEAD` failed in `{}`: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// The contents of `rel` (a `repo`-relative path) at `commit`.
///
/// `git show <commit>:./<rel>` resolves the path relative to `repo` — the diff's `--relative`
/// root — matching the paths [`changed_entries`] returns. The diff named `rel` as modified, so
/// a blob that fails to read is a real error, never a file to skip quietly.
fn blob_at(repo: &Path, commit: &str, rel: &str) -> Result<String> {
    let spec = format!("{commit}:./{rel}");
    let output = Command::new("git")
        .current_dir(repo)
        .args(["show", &spec])
        .output()
        .with_context(|| format!("running `git show {spec}` in `{}`", repo.display()))?;
    if !output.status.success() {
        bail!(
            "reading `{rel}` at `{commit}` in `{}`: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
    // `-c core.quotepath=off --no-ext-diff` pins the walk against the caller's git
    // config (#392): a non-ASCII path is emitted raw rather than octal-escaped, so a
    // `Modified` `src/föö.py` keys correctly (and reads back as a real file) instead of
    // hard-erroring; a configured external differ is blocked. `--name-status` carries no
    // `a/`/`b/` prefix, so no prefix pinning is needed here.
    let output = Command::new("git")
        .current_dir(repo)
        .args([
            "-c",
            "core.quotepath=off",
            "diff",
            "--name-status",
            "--no-ext-diff",
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
            // Decode a residual C-quoted path (a name with a `"` / backslash / control
            // byte still comes quoted even with `core.quotepath=off`) before normalizing
            // separators (#392).
            let path = crate::patch_coverage::unquote_c_path(path.trim_end_matches('\r'));
            let path = path.replace('\\', "/");
            entries.push((Status::from_code(status), path));
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    /// A throwaway git repo, removed on drop.
    struct TempRepo(PathBuf);

    impl TempRepo {
        fn new(slug: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "tc-co-change-git-{}-{}-{}",
                slug,
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            std::fs::create_dir_all(&root).unwrap();
            let repo = TempRepo(root);
            repo.git(&["init", "-q"]);
            repo.git(&["config", "user.email", "test@example.com"]);
            repo.git(&["config", "user.name", "Test"]);
            repo
        }

        fn git(&self, args: &[&str]) {
            let status = Command::new("git")
                .args(args)
                .current_dir(&self.0)
                .status()
                .expect("git should run");
            assert!(status.success(), "git {args:?} failed");
        }

        /// Write `contents` to `rel` and commit it, advancing HEAD.
        fn commit(&self, rel: &str, contents: &str) {
            std::fs::write(self.0.join(rel), contents).unwrap();
            self.git(&["add", "-A"]);
            self.git(&["-c", "commit.gpgsign=false", "commit", "-q", "-m", rel]);
        }

        fn head(&self) -> String {
            let out = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&self.0)
                .output()
                .expect("git rev-parse should run");
            assert!(out.status.success(), "git rev-parse failed");
            String::from_utf8(out.stdout).unwrap().trim().to_string()
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn merge_base_answers_where_the_branch_left_trunk() {
        let repo = TempRepo::new("mb");
        repo.commit("widget.py", "x = 1\n");
        repo.git(&["checkout", "-q", "-b", "trunk"]);
        let fork_point = repo.head();
        repo.git(&["checkout", "-q", "-b", "feature"]);
        repo.commit("widget.py", "x = 2\n");
        repo.git(&["checkout", "-q", "trunk"]);
        repo.commit("widget.py", "x = 3\n");
        let trunk_tip = repo.head();
        repo.git(&["checkout", "-q", "feature"]);

        // The commit the branch forked from, not the tip trunk has since reached.
        assert_eq!(merge_base(&repo.0, "trunk").unwrap(), fork_point);
        assert_ne!(fork_point, trunk_tip);
    }

    #[test]
    fn merge_base_errors_when_the_histories_never_met() {
        let repo = TempRepo::new("mb-orphan");
        repo.commit("widget.py", "x = 1\n");
        repo.git(&["checkout", "-q", "-b", "trunk"]);
        repo.git(&["checkout", "-q", "--orphan", "stranger"]);
        repo.commit("widget.py", "x = 2\n");

        let err = merge_base(&repo.0, "trunk").unwrap_err();
        assert!(err.to_string().contains("git merge-base"), "got: {err}");
    }

    #[test]
    fn blob_at_reads_the_file_as_it_stood() {
        let repo = TempRepo::new("blob");
        repo.commit("widget.py", "x = 1\n");
        let first = repo.head();
        repo.commit("widget.py", "x = 2\n");

        assert_eq!(blob_at(&repo.0, &first, "widget.py").unwrap(), "x = 1\n");
        assert_eq!(
            blob_at(&repo.0, &repo.head(), "widget.py").unwrap(),
            "x = 2\n"
        );
    }

    #[test]
    fn blob_at_errors_when_the_path_is_absent() {
        let repo = TempRepo::new("blob-missing");
        repo.commit("widget.py", "x = 1\n");

        let err = blob_at(&repo.0, &repo.head(), "ghost.py").unwrap_err();
        assert!(err.to_string().contains("ghost.py"), "got: {err}");
    }
}
