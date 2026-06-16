//! E2E tests for `e2e verify` (#17, slice #68): drive the built CLI binary in a
//! throwaway git repo (no mocks) and assert it gates on the committed
//! attestation — exit `0` when fresh, non-zero with the run-`attest` hint when
//! the code has moved on. Never runs e2e.
//!
//! Starts red against the stub in `src/e2e.rs` and goes green once `verify` is
//! implemented.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A throwaway git repo with one seed commit, removed on drop.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-e2e-verify-e2e-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        std::fs::write(root.join("README.md"), "seed\n").unwrap();
        git(&root, &["add", "."]);
        git(
            &root,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "seed"],
        );
        TempRepo(root)
    }

    fn commit_code(&self, name: &str, contents: &str) {
        std::fs::write(self.0.join(name), contents).unwrap();
        git(&self.0, &["add", name]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "code"],
        );
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

/// Run the built binary with `args`, cwd set to `repo`; return (exit code, stderr).
fn run_cli(repo: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(args)
        .current_dir(repo)
        .output()
        .expect("the built binary should run");
    (
        out.status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn verify_exits_zero_when_the_attestation_is_fresh() {
    let repo = TempRepo::new();
    assert_eq!(
        run_cli(&repo.0, &["e2e", "attest", "true"]).0,
        0,
        "attest should record the run"
    );
    let (code, _) = run_cli(&repo.0, &["e2e", "verify"]);
    assert_eq!(code, 0, "a fresh attestation should pass verify");
}

#[test]
fn verify_exits_nonzero_with_the_attest_hint_when_stale() {
    let repo = TempRepo::new();
    run_cli(&repo.0, &["e2e", "attest", "true"]);
    // Move the code on without re-attesting.
    repo.commit_code("widget.rs", "pub fn widget() {}\n");

    let (code, stderr) = run_cli(&repo.0, &["e2e", "verify"]);
    assert_ne!(code, 0, "a stale attestation should fail verify");
    assert!(
        stderr.contains("attest"),
        "the failure should hint to re-run attest; got: {stderr}"
    );
}
