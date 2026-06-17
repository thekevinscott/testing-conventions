//! E2E tests for patch (changed-line) coverage (Rust — #136): drive the built CLI
//! binary as a real subprocess against throwaway cargo crates and assert the exit
//! code (and, for a red case, the named offender). Complements the in-process
//! integration tests in `patch_coverage_rust.rs`. Requires `git` and
//! `cargo-llvm-cov` on PATH.
//!
//! Starts red against the stub in `src/patch_coverage.rs` (`check_rust` reports
//! nothing) and goes green once the diff + `cargo llvm-cov` detection is
//! implemented.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A throwaway cargo crate in a git repo, removed on drop. `new` lays down the
/// `Cargo.toml`; a test writes a baseline, `commit`s it, captures `head()` as the
/// `base`, then mutates and commits the "after".
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-patch-cov-rust-e2e-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        let repo = TempRepo(root);
        repo.write("Cargo.toml", CARGO_TOML);
        repo
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

/// Exit code + stderr of `unit patch-coverage <repo> --language rust --base <base>
/// [--config <repo>/<config>]`, run as a real subprocess.
fn patch_coverage(repo: &TempRepo, base: &str, config: Option<&str>) -> (i32, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    cmd.arg("unit")
        .arg("patch-coverage")
        .arg(&repo.0)
        .args(["--language", "rust", "--base", base]);
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

const CARGO_TOML: &str =
    "[package]\nname = \"tc_patch_rust\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n";

const WIDGET_RS: &str = r#"pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "pos"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "pos");
        assert_eq!(widget(-1), "neg");
    }
}
"#;
const WIDGET_RS_UNCOVERED: &str = r#"pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "pos"
    } else if n == -42 {
        "answer"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "pos");
        assert_eq!(widget(-1), "neg");
    }
}
"#;
const WIDGET_RS_COVERED_EDIT: &str = r#"pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "positive"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "positive");
        assert_eq!(widget(-1), "neg");
    }
}
"#;

#[test]
fn uncovered_changed_line_exits_nonzero_and_names_it() {
    let repo = TempRepo::new("red");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    let base = repo.head();
    repo.write("src/lib.rs", WIDGET_RS_UNCOVERED);
    repo.commit("add an untested arm");

    let (code, stderr) = patch_coverage(&repo, &base, None);
    assert_eq!(
        code, 1,
        "an uncovered changed line must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("src/lib.rs"),
        "stderr should name the uncovered file; got: {stderr}"
    );
}

#[test]
fn covered_change_exits_zero() {
    let repo = TempRepo::new("clean");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    let base = repo.head();
    repo.write("src/lib.rs", WIDGET_RS_COVERED_EDIT);
    repo.commit("reword a covered line and update its test");

    let (code, stderr) = patch_coverage(&repo, &base, None);
    assert_eq!(code, 0, "a fully covered change passes; stderr: {stderr}");
}

#[test]
fn added_untested_file_exits_nonzero() {
    let repo = TempRepo::new("added");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    let base = repo.head();
    repo.write("src/lib.rs", &format!("{WIDGET_RS}pub mod extra;\n"));
    repo.write("src/extra.rs", "pub fn extra() -> i64 {\n    41\n}\n");
    repo.commit("add a brand-new untested module");

    let (code, stderr) = patch_coverage(&repo, &base, None);
    assert_eq!(
        code, 1,
        "an added file's uncovered lines must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("src/extra.rs"),
        "stderr should name the added file; got: {stderr}"
    );
}

#[test]
fn a_coverage_exemption_lifts_the_uncovered_change() {
    let repo = TempRepo::new("exempt");
    repo.write(
        "testing-conventions.toml",
        "[[rust.exempt]]\npath = \"src/shim.rs\"\nrules = [\"coverage\"]\n\
         reason = \"thin launcher; logic lives in tested modules\"\n",
    );
    repo.write("src/lib.rs", &format!("{WIDGET_RS}pub mod shim;\n"));
    repo.write("src/shim.rs", "pub fn shim() -> i64 {\n    0\n}\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("src/shim.rs", "pub fn shim() -> i64 {\n    1\n}\n");
    repo.commit("edit the untested launcher");

    // Flagged with no config, lifted once the `coverage` exemption is supplied.
    assert_eq!(patch_coverage(&repo, &base, None).0, 1);
    assert_eq!(
        patch_coverage(&repo, &base, Some("testing-conventions.toml")).0,
        0
    );
}
