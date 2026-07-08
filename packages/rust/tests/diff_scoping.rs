//! Integration tests for the shared diff/path-scoping machinery (#392): the
//! `<base>...HEAD` diff parser that backs changed-line coverage AND TypeScript /
//! Python mutation (both consume [`patch_coverage::changed_lines`]), plus the
//! co-change name-status walk ([`co_change::stale_sources`]).
//!
//! Two defects both dropped changed files from scoping — a false green:
//!   1. an added body line beginning `++ ` renders as `+++ …` and was consumed by
//!      the `+++` file-header branch, diverting the file's later added lines to a
//!      bogus key;
//!   2. a git-quoted / non-ASCII path (default `core.quotepath=on`) never matched a
//!      report key (coverage) or read back as a file (co-change).
//!
//! Each test builds a throwaway git repo under **default** git config (the reported
//! bug's condition — no `core.quotepath=off`) and drives the real `git diff`.
//! Requires `git` on PATH.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::co_change::stale_sources;
use testing_conventions::colocated_test::Language;
use testing_conventions::patch_coverage::changed_lines;

/// A throwaway git repo, removed on drop. A test writes a baseline, `commit`s it,
/// captures `head()` as the `base`, then mutates and commits the "after".
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-diff-scope-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        TempRepo(root)
    }

    fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn commit(&self, message: &str) {
        git(&self.0, &["add", "-A"]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
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

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git should run");
    assert!(status.success(), "git {args:?} failed");
}

#[test]
fn a_plus_plus_body_line_keeps_the_files_later_changed_lines_in_scope() {
    // A hunk that adds a `++ 1` line (rendered `+++ 1` in the diff) followed by more
    // added lines: the parser that backs coverage and TS/Python mutation must keep
    // those later lines attributed to `calc.py`, not divert them to a bogus key that
    // drops them from scoping.
    let repo = TempRepo::new("plusplus");
    repo.write("calc.py", "def calc(n):\n    return n\n");
    repo.commit("base");
    let base = repo.head();
    repo.write(
        "calc.py",
        "def calc(n):\n    return n\n\n\n++ 1\n\n\ndef never_run():\n    return 999\n",
    );
    repo.commit("append a ++ line and an untested helper");

    let changed = changed_lines(&repo.0, &base).expect("diffing a readable repo succeeds");
    let lines = changed
        .get("calc.py")
        .unwrap_or_else(|| panic!("calc.py must be in scope; got keys {:?}", changed.keys()));
    // The `def never_run():` (line 8) and `return 999` (line 9) come *after* the `++ 1`
    // line — under the bug they vanish; they must stay in scope.
    assert!(
        lines.contains(&8) && lines.contains(&9),
        "the lines after the ++ line must stay in scope; got {lines:?}"
    );
    // And no bogus single-token key ("1", from `++ 1`) leaks in.
    assert!(
        !changed.contains_key("1"),
        "the ++ body line must not create a phantom file key; got keys {:?}",
        changed.keys()
    );
}

#[test]
fn a_non_ascii_path_is_scoped_under_default_git_config() {
    // Under git's default `core.quotepath=on`, a changed `src/föö.py` is emitted as a
    // C-quoted `"b/src/f\303\266\303\266.py"`. `changed_lines` must decode it to the
    // real UTF-8 key so it matches the coverage report; left quoted, every changed line
    // in the file is silently skipped (a vacuous pass).
    let repo = TempRepo::new("nonascii");
    repo.write("src/föö.py", "def foo(n):\n    return n\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("src/föö.py", "def foo(n):\n    return n + 1\n");
    repo.commit("edit the non-ASCII source");

    let changed = changed_lines(&repo.0, &base).expect("diffing a readable repo succeeds");
    assert!(
        changed.contains_key("src/föö.py"),
        "the non-ASCII path must decode to its real UTF-8 key; got keys {:?}",
        changed.keys()
    );
}

#[test]
fn co_change_scopes_a_non_ascii_modified_source() {
    // The co-change name-status walk inherits the same quoting: a `Modified`
    // `src/föö.py` under default config was mis-keyed (and hard-errored when read
    // back). Editing it while leaving its colocated test must flag it stale — not
    // error, and not silently pass.
    let repo = TempRepo::new("cochange-nonascii");
    repo.write("src/föö.py", "def foo(n):\n    return n\n");
    repo.write(
        "src/föö_test.py",
        "from föö import foo\n\n\ndef test_foo():\n    assert foo(1) == 1\n",
    );
    repo.commit("base");
    let base = repo.head();
    repo.write("src/föö.py", "def foo(n):\n    return n + 1\n");
    repo.commit("edit the non-ASCII source only");

    let stale: Vec<String> = stale_sources(&repo.0, &base, Language::Python, &Default::default())
        .expect("diffing a readable repo succeeds")
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        stale,
        vec!["src/föö.py".to_string()],
        "the non-ASCII modified source with a stale test must be flagged"
    );
}
