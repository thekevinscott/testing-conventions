//! Integration tests for patch (changed-line) coverage (Python — #132, parent #46).
//!
//! Every line a `<base>...HEAD` diff touches must be covered by the unit suite.
//! Each test builds a throwaway git repo (per the #3 guardrail the codebases are
//! the fixtures — red cases where a changed line is left uncovered, clean cases
//! where it isn't), runs REAL coverage.py over it via the SDK
//! (`patch_coverage::check`) and the CLI (`run`), and asserts the uncovered lines
//! / exit code. Requires `coverage` + `pytest` + `git` on PATH.
//!
//! Opens at RED per AGENTS.md: detection is stubbed (`check` reports nothing), so
//! a repo whose change leaves a line uncovered still comes back clean. The
//! implementation — diffing `<base>...HEAD` and intersecting the changed lines
//! with coverage.py's missing lines/branches — follows once CI witnesses these
//! red.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::patch_coverage::check;
use testing_conventions::run;

/// A throwaway git repo, removed on drop. A test writes a baseline source + its
/// colocated test, `commit`s it, captures `head()` as the `base`, then mutates
/// and commits the "after" so `<base>...HEAD` is the change under test.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-patch-cov-{}-{}-{}",
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

    /// Write `contents` to `rel`, creating parent directories.
    fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    /// Stage everything and commit, advancing HEAD.
    fn commit(&self, message: &str) {
        git(&self.0, &["add", "-A"]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
    }

    /// The current HEAD SHA — captured as the `base` before mutating.
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

/// The uncovered changed lines for `<base>...HEAD` (no exemptions), as
/// `<file>:<line>` strings.
fn uncovered(repo: &TempRepo, base: &str) -> Vec<String> {
    check(&repo.0, base, &[])
        .expect("checking a readable repo should succeed")
        .iter()
        .map(|u| format!("{}:{}", u.file, u.line))
        .collect()
}

/// Result of `unit patch-coverage <repo> --language <lang> --base <base>
/// [--config <repo>/<config>]`, run in-process.
fn run_patch_coverage(
    repo: &TempRepo,
    language: &str,
    base: &str,
    config: Option<&str>,
) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "patch-coverage".into(),
        repo.0.clone().into_os_string(),
        "--language".into(),
        language.into(),
        "--base".into(),
        base.into(),
    ];
    if let Some(name) = config {
        argv.push("--config".into());
        argv.push(repo.0.join(name).into_os_string());
    }
    run(argv)
}

/// Raw CLI invocation (after the program name), so a clap usage error can be
/// asserted rather than unwrapped away.
fn run_cli(args: &[&str]) -> anyhow::Result<i32> {
    let argv: Vec<OsString> = std::iter::once(OsString::from("testing-conventions"))
        .chain(args.iter().copied().map(OsString::from))
        .collect();
    run(argv)
}

/// Fully covered by `WIDGET_TEST_PY` at baseline (both branches taken).
const WIDGET_PY: &str = r#"def widget(n):
    if n > 0:
        return "pos"
    return "neg"
"#;
const WIDGET_TEST_PY: &str = r#"from widget import widget


def test_widget():
    assert widget(1) == "pos"
    assert widget(-1) == "neg"
"#;

/// Adds an `n == 42` branch the baseline test never exercises: line 5
/// (`return "answer"`) is a missing line and line 4 (`if n == 42:`) is the source
/// of a branch never taken — so both changed lines are uncovered.
const WIDGET_PY_UNCOVERED: &str = r#"def widget(n):
    if n > 0:
        return "pos"
    if n == 42:
        return "answer"
    return "neg"
"#;

// ---- Detection via the SDK (`check`) -------------------------------------

#[test]
fn python_uncovered_changed_line_is_reported() {
    // The core red case: the change adds a line (and a branch) the suite never
    // runs, so both new lines come back uncovered.
    let repo = TempRepo::new("uncovered");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", WIDGET_PY_UNCOVERED);
    repo.commit("add an untested branch");

    let reported = uncovered(&repo, &base);
    assert!(
        reported.contains(&"widget.py:5".to_string()),
        "the uncovered new line should be reported; got: {reported:?}"
    );
    assert!(
        reported.contains(&"widget.py:4".to_string()),
        "the uncovered new branch source should be reported; got: {reported:?}"
    );
}

#[test]
fn python_covered_change_is_clean() {
    // Editing a line the suite already exercises keeps the change fully covered.
    let repo = TempRepo::new("covered");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.py",
        r#"def widget(n):
    if n > 0:
        return "positive"
    return "neg"
"#,
    );
    repo.commit("reword a covered line");

    assert!(
        uncovered(&repo, &base).is_empty(),
        "a change to a covered line is clean"
    );
}

#[test]
fn python_added_untested_file_changed_lines_are_uncovered() {
    // Unlike co-change (#33), an *added* file's new lines ARE patch-coverage
    // subjects: a brand-new source the suite never imports is wholly uncovered.
    let repo = TempRepo::new("added");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    let base = repo.head();

    repo.write("lonely.py", "def lonely():\n    return 41\n");
    repo.commit("add a brand-new untested source");

    let reported = uncovered(&repo, &base);
    assert!(
        reported.contains(&"lonely.py:1".to_string()),
        "the added file's uncovered lines should be reported; got: {reported:?}"
    );
}

#[test]
fn python_changed_comment_line_is_not_a_subject() {
    // A comment isn't executable, so coverage has nothing to measure on it — a
    // changed comment line is never flagged.
    let repo = TempRepo::new("comment");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.py",
        r#"def widget(n):
    # branch on the sign of n
    if n > 0:
        return "pos"
    return "neg"
"#,
    );
    repo.commit("add an explanatory comment");

    assert!(
        uncovered(&repo, &base).is_empty(),
        "a changed comment line has nothing to cover"
    );
}

#[test]
fn an_unknown_base_ref_is_an_error() {
    // A base that can't be resolved must surface, never silently pass as "clean".
    let repo = TempRepo::new("bad-base");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");

    assert!(
        check(&repo.0, "no-such-ref", &[]).is_err(),
        "an unresolvable base ref must error"
    );
}

// ---- Exit codes via the CLI (`run`) --------------------------------------

#[test]
fn python_subcommand_exits_nonzero_on_an_uncovered_changed_line() {
    let repo = TempRepo::new("cli-red");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", WIDGET_PY_UNCOVERED);
    repo.commit("add an untested branch");

    assert_eq!(run_patch_coverage(&repo, "python", &base, None).unwrap(), 1);
}

#[test]
fn python_subcommand_exits_zero_when_the_change_is_covered() {
    let repo = TempRepo::new("cli-clean");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    let base = repo.head();
    repo.write(
        "widget.py",
        r#"def widget(n):
    if n > 0:
        return "positive"
    return "neg"
"#,
    );
    repo.commit("reword a covered line");

    assert_eq!(run_patch_coverage(&repo, "python", &base, None).unwrap(), 0);
}

// ---- Exemptions (#32 machinery, rule `coverage`) -------------------------

#[test]
fn python_a_coverage_exemption_lifts_an_uncovered_changed_line() {
    // A `coverage` exemption omits a file from the run, so its changed lines have
    // nothing to cover — the same waiver the floor (#26) honors.
    let repo = TempRepo::new("exempt");
    repo.write(
        "testing-conventions.toml",
        "[[python.exempt]]\npath = \"shim.py\"\nrules = [\"coverage\"]\n\
         reason = \"thin launcher; logic lives in tested modules\"\n",
    );
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.write("shim.py", "def shim():\n    return 0\n");
    repo.commit("base");
    let base = repo.head();

    repo.write("shim.py", "def shim():\n    return 1\n");
    repo.commit("edit the untested launcher");

    // Flagged with no config…
    assert_eq!(run_patch_coverage(&repo, "python", &base, None).unwrap(), 1);
    // …and lifted by the `coverage` exemption.
    assert_eq!(
        run_patch_coverage(&repo, "python", &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

// ---- CLI surface & errors ------------------------------------------------

#[test]
fn patch_coverage_requires_language() {
    let err = run_cli(&["unit", "patch-coverage", "/tmp", "--base", "HEAD"])
        .expect_err("--language is required");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("a missing required flag should surface as a clap::Error");
    assert_eq!(
        clap_err.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn patch_coverage_rejects_rust() {
    // Rust patch coverage (`cargo llvm-cov`) is a separate item.
    let repo = TempRepo::new("rust-reject");
    repo.write("lib.rs", "pub fn f() {}\n");
    repo.commit("base");
    let base = repo.head();

    let err = run_patch_coverage(&repo, "rust", &base, None).unwrap_err();
    assert!(err.to_string().contains("separate item"), "got: {err}");
}

#[test]
fn patch_coverage_rejects_typescript() {
    // The TypeScript twin is a later slice.
    let repo = TempRepo::new("ts-reject");
    repo.write("widget.ts", "export const f = () => 1;\n");
    repo.commit("base");
    let base = repo.head();

    let err = run_patch_coverage(&repo, "typescript", &base, None).unwrap_err();
    assert!(err.to_string().contains("separate item"), "got: {err}");
}
