//! The commit-scoped `co-change` check: a source file that changed in a
//! git diff must change its colocated test too.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::colocated_test::Language;

/// Every source file changed in `repo`'s `<base>...HEAD` diff whose colocated test did not
/// also change, sorted. A subject whose `repo`-relative path is in `exempt` is skipped. Errors
/// when `git diff` fails, so an un-diffable range surfaces rather than passing as "clean".
pub fn stale_sources(
    repo: &Path,
    base: &str,
    language: Language,
    exempt: &BTreeSet<String>,
) -> Result<Vec<PathBuf>> {
    let entries = changed_entries(repo, base)?;
    let fork_point = merge_base(repo, base)?;
    let changed: BTreeSet<&str> = entries.iter().map(|(_, path)| path.as_str()).collect();
    // `<package root>/tests/` belongs to the suite tiers, so nothing under it is a subject.
    let suite_tests = match language {
        Language::Python => crate::tiers::suite_tests_dir(repo, "pyproject.toml"),
        Language::TypeScript => crate::tiers::suite_tests_dir(repo, "package.json"),
        Language::Rust => None,
    };

    let mut stale = Vec::new();
    for (status, rel) in &entries {
        let path = Path::new(rel);
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
        let is_subject = match status {
            Status::Modified => {
                // Deciding on the diff's shape alone would flag a type-only module for
                // having no colocated test — the fact presence uses to skip it.
                let contents = std::fs::read_to_string(repo.join(path))
                    .with_context(|| format!("reading changed source `{rel}`"))?;
                language.is_subject(&contents, path)
                    && !language.same_code(&blob_at(repo, &fork_point, rel)?, &contents, path)
            }
            // HEAD cannot answer this — the file is gone — so `base` is asked instead.
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
    /// `D` — removed; a subject only if the source had a colocated test in base.
    Deleted,
    /// `A` (added) and the rest (`T`, …) — not a co-change subject.
    Other,
}

impl Status {
    /// The status from a `git diff --name-status` field; `--no-renames` makes it one letter.
    fn from_code(code: &str) -> Status {
        match code.chars().next() {
            Some('M') => Status::Modified,
            Some('D') => Status::Deleted,
            _ => Status::Other,
        }
    }
}

/// `true` when `rel` (a `repo`-relative path) exists as a blob in the `base` tree. The `./` in
/// `git cat-file -e <base>:./<rel>` resolves the path relative to `repo`, matching the diff's
/// `--relative` root; a non-zero exit means "no such path in base".
fn test_exists_in_base(repo: &Path, base: &str, rel: &str) -> Result<bool> {
    let spec = format!("{base}:./{rel}");
    let output = Command::new("git")
        .current_dir(repo)
        .args(["cat-file", "-e", &spec])
        .output()
        .with_context(|| format!("running `git cat-file` in `{}`", repo.display()))?;
    Ok(output.status.success())
}

/// The commit `<base>...HEAD` diffs from — the merge base of `base` and HEAD. Comparing against
/// `base`'s tip instead would read a file the branch only commented as a code change, because the
/// tip carries commits this branch never saw.
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

/// The contents of `rel` (a `repo`-relative path) at `commit`. The `./` in
/// `git show <commit>:./<rel>` resolves the path relative to `repo`, matching the diff's
/// `--relative` root.
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

/// The status + `repo`-relative path of every file changed in `<base>...HEAD`, via
/// `git diff --name-status`. `--no-renames` shows a rename as a delete + an add, so the deleted
/// source is still held to its test; `--relative` scopes the diff to `repo`.
fn changed_entries(repo: &Path, base: &str) -> Result<Vec<(Status, String)>> {
    let range = format!("{base}...HEAD");
    // `core.quotepath=off` emits a non-ASCII path raw rather than octal-escaped, so a modified
    // `src/föö.py` reads back as a real file; `--no-ext-diff` blocks a configured external differ.
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
        if let Some((status, path)) = line.split_once('\t') {
            // A name with a `"`, a backslash, or a control byte still comes C-quoted even
            // with `core.quotepath=off`.
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

    const NOWHERE: &str = "/nonexistent-tc-co-change";

    #[test]
    fn test_exists_in_base_reports_a_spawn_failure() {
        let err = test_exists_in_base(Path::new(NOWHERE), "main", "widget_test.py").unwrap_err();
        assert!(format!("{err:#}").contains("running `git cat-file`"));
    }

    #[test]
    fn merge_base_reports_a_spawn_failure() {
        let err = merge_base(Path::new(NOWHERE), "main").unwrap_err();
        assert!(format!("{err:#}").contains("running `git merge-base`"));
    }

    #[test]
    fn blob_at_reports_a_spawn_failure() {
        let err = blob_at(Path::new(NOWHERE), "HEAD", "widget.py").unwrap_err();
        assert!(format!("{err:#}").contains("running `git show "));
    }

    #[test]
    fn changed_entries_reports_a_spawn_failure() {
        let err = changed_entries(Path::new(NOWHERE), "main")
            .err()
            .expect("the missing repo errors");
        assert!(format!("{err:#}").contains("running `git diff`"));
    }

    #[test]
    fn a_changed_source_missing_from_the_worktree_is_an_error() {
        let repo = TempRepo::new("gone");
        repo.commit("widget.py", "def widget():\n    return 1\n");
        repo.git(&["checkout", "-q", "-b", "trunk"]);
        repo.git(&["checkout", "-q", "-b", "feature"]);
        repo.commit("widget.py", "def widget():\n    return 2\n");
        std::fs::remove_file(repo.0.join("widget.py")).unwrap();

        let err = stale_sources(&repo.0, "trunk", Language::Python, &BTreeSet::new())
            .expect_err("the missing worktree file errors");
        assert!(
            format!("{err:#}").contains("reading changed source `widget.py`"),
            "got: {err:#}"
        );
    }
}
