//! E2E tests for patch (changed-line) coverage (Python — #132): drive the built
//! CLI binary as a real subprocess against throwaway git repos and assert the
//! exit code (and, for a red case, the named offender). Complements the
//! in-process integration tests in `patch_coverage.rs`. Requires `coverage` +
//! `pytest` + `git` on PATH.
//!
//! Starts red against the stub in `src/patch_coverage.rs` (detection reports
//! nothing) and goes green once the diff + coverage detection is implemented.

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
            "tc-patch-cov-e2e-{}-{}-{}",
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

/// Exit code + stderr of `unit patch-coverage <repo> --language <lang> --base
/// <base> [--config <repo>/<config>]`, run as a real subprocess.
fn patch_coverage(
    repo: &TempRepo,
    language: &str,
    base: &str,
    config: Option<&str>,
) -> (i32, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    cmd.arg("unit").arg("patch-coverage").arg(&repo.0).args([
        "--language",
        language,
        "--base",
        base,
    ]);
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
const WIDGET_PY_UNCOVERED: &str = r#"def widget(n):
    if n > 0:
        return "pos"
    if n == 42:
        return "answer"
    return "neg"
"#;

#[test]
fn uncovered_changed_line_exits_nonzero_and_names_it() {
    let repo = TempRepo::new("red");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", WIDGET_PY_UNCOVERED);
    repo.commit("add an untested branch");

    let (code, stderr) = patch_coverage(&repo, "python", &base, None);
    assert_eq!(
        code, 1,
        "an uncovered changed line must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("widget.py"),
        "stderr should name the uncovered file; got: {stderr}"
    );
}

#[test]
fn covered_change_exits_zero() {
    let repo = TempRepo::new("clean");
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
    repo.write(
        "widget_test.py",
        r#"from widget import widget


def test_widget():
    assert widget(1) == "positive"
    assert widget(-1) == "neg"
"#,
    );
    repo.commit("reword a covered line and update its test");

    let (code, stderr) = patch_coverage(&repo, "python", &base, None);
    assert_eq!(code, 0, "a fully covered change passes; stderr: {stderr}");
}

#[test]
fn added_untested_file_exits_nonzero() {
    let repo = TempRepo::new("added");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    let base = repo.head();
    repo.write("lonely.py", "def lonely():\n    return 41\n");
    repo.commit("add a brand-new untested source");

    let (code, stderr) = patch_coverage(&repo, "python", &base, None);
    assert_eq!(
        code, 1,
        "an added file's uncovered lines must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("lonely.py"),
        "stderr should name the added file; got: {stderr}"
    );
}

#[test]
fn a_coverage_exemption_lifts_the_uncovered_change() {
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

    // Flagged with no config, lifted once the `coverage` exemption is supplied.
    assert_eq!(patch_coverage(&repo, "python", &base, None).0, 1);
    assert_eq!(
        patch_coverage(&repo, "python", &base, Some("testing-conventions.toml")).0,
        0
    );
}

#[test]
fn rust_is_rejected() {
    // Rust patch coverage (`cargo llvm-cov`) is a separate item.
    let repo = TempRepo::new("rust");
    repo.write("lib.rs", "pub fn f() {}\n");
    repo.commit("base");
    let base = repo.head();

    let (code, stderr) = patch_coverage(&repo, "rust", &base, None);
    assert_eq!(code, 1, "`--language rust` is rejected; stderr: {stderr}");
    assert!(stderr.contains("separate item"), "got: {stderr}");
}

#[test]
fn typescript_is_rejected() {
    // The TypeScript twin is a later slice.
    let repo = TempRepo::new("ts");
    repo.write("widget.ts", "export const f = () => 1;\n");
    repo.commit("base");
    let base = repo.head();

    let (code, stderr) = patch_coverage(&repo, "typescript", &base, None);
    assert_eq!(
        code, 1,
        "`--language typescript` is rejected; stderr: {stderr}"
    );
    assert!(stderr.contains("separate item"), "got: {stderr}");
}
