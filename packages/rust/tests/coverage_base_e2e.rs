//! E2E tests for diff-scoped coverage — `unit coverage --base`: drive the
//! built CLI binary as a real subprocess against throwaway git repos and assert
//! the exit code (and, for a red case, the failure on stderr). Complements the
//! in-process integration tests in `coverage_base.rs`. Requires `coverage` +
//! `pytest` + `git` on PATH.
//!
//! Starts red against the stub in `src/patch_coverage.rs` (the diff-scoped ratio
//! reports Pass) and goes green once the ratio-vs-floor measurement is
//! implemented.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A throwaway git repo, removed on drop. A test writes a baseline, `commit`s it,
/// captures `head()` as the `base`, then mutates and commits the "after".
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-cov-base-e2e-{}-{}-{}",
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

/// Exit code + stderr of `unit coverage <repo> --language python --base <base>
/// [--config <repo>/<config>]`, run as a real subprocess.
fn coverage_base(repo: &TempRepo, base: &str, config: Option<&str>) -> (i32, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    cmd.arg("unit")
        .arg("coverage")
        .arg(&repo.0)
        .args(["--language", "python", "--base", base]);
    if let Some(name) = config {
        cmd.arg("--config").arg(repo.0.join(name));
    }
    let output = cmd.output().expect("the built binary should run");
    (
        output
            .status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

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

/// After: a covered and an uncovered one-line helper → 75% covered on the diff
/// (see `coverage_base.rs`).
const WIDGET_PY_75: &str = r#"def widget(n):
    if n > 0:
        return "pos"
    return "neg"


def covered():
    return 1


def uncovered():
    return 2
"#;
const WIDGET_TEST_75: &str = r#"from widget import widget, covered


def test_widget():
    assert widget(1) == "pos"
    assert widget(-1) == "neg"


def test_covered():
    assert covered() == 1
"#;

fn baseline(repo: &TempRepo) -> String {
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    repo.head()
}

#[test]
fn below_floor_diff_exits_nonzero_and_reports_coverage() {
    let repo = TempRepo::new("red");
    let base = baseline(&repo);
    repo.write("widget.py", WIDGET_PY_75);
    repo.write("widget_test.py", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    let (code, stderr) = coverage_base(&repo, &base, None);
    assert_eq!(
        code, 1,
        "a diff below the floor must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("coverage"),
        "stderr should report the coverage shortfall; got: {stderr}"
    );
}

#[test]
fn covered_change_exits_zero() {
    let repo = TempRepo::new("clean");
    let base = baseline(&repo);
    repo.write(
        "widget.py",
        r#"def widget(n):
    if n > 0:
        return "positive"
    return "neg"
"#,
    );
    repo.write(
        "widget_test.py",
        r#"from widget import widget


def test_widget():
    assert widget(1) == "positive"
    assert widget(-1) == "neg"
"#,
    );
    repo.commit("reword a covered line and update its test");

    let (code, stderr) = coverage_base(&repo, &base, None);
    assert_eq!(code, 0, "a fully covered change passes; stderr: {stderr}");
}

#[test]
fn a_lower_configured_floor_lets_the_same_diff_pass() {
    // The behavior change: the 75% diff that fails the default floor passes once
    // the configured floor is 70 — the floor is the single source of truth.
    let repo = TempRepo::new("floor70");
    repo.write(
        "testing-conventions.toml",
        "[python.coverage]\nbranch = true\nfail_under = 70\n",
    );
    let base = baseline(&repo);
    repo.write("widget.py", WIDGET_PY_75);
    repo.write("widget_test.py", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    let (code, stderr) = coverage_base(&repo, &base, Some("testing-conventions.toml"));
    assert_eq!(
        code, 0,
        "75% on the diff clears a configured 70 floor; stderr: {stderr}"
    );
}

#[test]
fn a_tiny_below_floor_diff_still_exits_nonzero() {
    // No small-diff carve-out: a single untested helper (50% on a two-line
    // diff) fails the default floor.
    let repo = TempRepo::new("tiny");
    let base = baseline(&repo);
    repo.write(
        "widget.py",
        &format!("{WIDGET_PY}\n\ndef lonely():\n    return 41\n"),
    );
    repo.commit("add one untested helper");

    let (code, stderr) = coverage_base(&repo, &base, None);
    assert_eq!(
        code, 1,
        "a tiny diff below the floor is not exempted; stderr: {stderr}"
    );
}
