//! E2E tests for the commit-scoped `co-change` check (#33): drive the built CLI
//! binary as a real subprocess against throwaway git repos and assert the exit
//! code (and, for the red case, the named offender). Complements the in-process
//! integration tests in `co_change.rs`.
//!
//! Starts red against the stub in `src/co_change.rs` and goes green once
//! detection is implemented.

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
            "tc-co-change-e2e-{}-{}-{}",
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

    fn remove(&self, rel: &str) {
        std::fs::remove_file(self.0.join(rel)).unwrap();
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

/// Exit code + stderr of `unit co-change <repo> --language <lang> --base <base>
/// [--config <repo>/<config>]`, run as a real subprocess against the built binary.
fn co_change(repo: &TempRepo, language: &str, base: &str, config: Option<&str>) -> (i32, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    cmd.arg("unit")
        .arg("co-change")
        .arg(&repo.0)
        .args(["--language", language, "--base", base]);
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

const WIDGET_PY: &str = "def widget():\n    return 1\n";
const WIDGET_PY_TEST: &str =
    "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 1\n";

#[test]
fn modified_source_without_its_test_exits_nonzero_and_names_it() {
    let repo = TempRepo::new("mod-red");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.commit("edit source only");

    let (code, stderr) = co_change(&repo, "python", &base, None);
    assert_eq!(
        code, 1,
        "a stale source must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("widget.py"),
        "stderr should name the stale source; got: {stderr}"
    );
}

#[test]
fn modified_source_with_its_test_exits_zero() {
    let repo = TempRepo::new("mod-clean");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 2\n",
    );
    repo.commit("edit both");

    let (code, stderr) = co_change(&repo, "python", &base, None);
    assert_eq!(code, 0, "co-changed source and test pass; stderr: {stderr}");
}

#[test]
fn deleted_source_without_deleting_its_test_exits_nonzero() {
    let repo = TempRepo::new("del-red");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.remove("widget.py");
    repo.commit("delete source only");

    let (code, stderr) = co_change(&repo, "python", &base, None);
    assert_eq!(
        code, 1,
        "deleting a source while leaving its test exits non-zero; stderr: {stderr}"
    );
}

#[test]
fn a_co_change_exemption_lifts_the_stale_source() {
    let repo = TempRepo::new("exempt");
    repo.write(
        "testing-conventions.toml",
        "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"co-change\"]\n\
         reason = \"thin launcher; no logic to retest on each edit\"\n",
    );
    repo.write("cli.py", "def main():\n    return 0\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("cli.py", "def main():\n    return 1\n");
    repo.commit("edit the launcher, no test");

    // Stale with no config, lifted once the exemption is supplied.
    assert_eq!(co_change(&repo, "python", &base, None).0, 1);
    assert_eq!(
        co_change(&repo, "python", &base, Some("testing-conventions.toml")).0,
        0
    );
}

#[test]
fn rust_is_rejected() {
    // Rust units are inline `#[cfg(test)]` — no sibling test to go stale.
    let repo = TempRepo::new("rust");
    repo.write("lib.rs", "pub fn f() {}\n");
    repo.commit("base");
    let base = repo.head();

    let (code, stderr) = co_change(&repo, "rust", &base, None);
    assert_eq!(code, 1, "`--language rust` is rejected; stderr: {stderr}");
    assert!(stderr.contains("inline"), "got: {stderr}");
}
