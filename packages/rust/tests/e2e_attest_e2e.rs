//! E2E tests for `e2e attest`: drive the built CLI binary in a
//! throwaway git repo (no mocks) and assert it force-runs the command, exits
//! `0`, and commits an attestation on top.
//!
//! Starts red against the stub in `src/e2e.rs` and goes green once `attest` is
//! implemented.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::e2e::ATTESTATION_PATH;

/// A throwaway git repo with one seed commit, removed on drop.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-e2e-attest-e2e-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        // Throwaway repos never sign — keep the suite hermetic regardless of the
        // machine's global `commit.gpgsign`, now that `attest` inherits it instead
        // of forcing it off.
        git(&root, &["config", "commit.gpgsign", "false"]);
        std::fs::write(root.join("README.md"), "seed\n").unwrap();
        git(&root, &["add", "."]);
        git(
            &root,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "seed"],
        );
        TempRepo(root)
    }

    fn head(&self) -> String {
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.0)
            .output()
            .expect("git rev-parse should run");
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

/// Run the built binary's `e2e attest <command>` with the cwd set to `repo`.
fn attest_exit(repo: &Path, command: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["e2e", "attest", command])
        .current_dir(repo)
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

/// Configure `repo` to require signed commits, but point signing at a program
/// that does not exist — so any *attempted* signature fails. Honoring the repo's
/// `commit.gpgsign` then means the attestation commit is attempted and fails
/// (non-zero exit), rather than silently committed unsigned.
fn require_unsatisfiable_signing(repo: &Path) {
    git(repo, &["config", "gpg.format", "ssh"]);
    git(
        repo,
        &["config", "gpg.ssh.program", "/nonexistent/tc-test-signer"],
    );
    git(
        repo,
        &["config", "user.signingkey", "/nonexistent/tc-test-key.pub"],
    );
    git(repo, &["config", "commit.gpgsign", "true"]);
}

#[test]
fn attest_exits_zero_and_commits_an_attestation() {
    let repo = TempRepo::new();
    let code_commit = repo.head();

    assert_eq!(
        attest_exit(&repo.0, "true"),
        0,
        "attest force-runs and exits 0"
    );
    assert!(
        repo.0.join(ATTESTATION_PATH).is_file(),
        "attest should write the attestation file"
    );
    assert_ne!(
        repo.head(),
        code_commit,
        "attest should commit the attestation on top"
    );
}

#[test]
fn attest_exits_zero_even_when_the_command_fails() {
    // Force-run, not force-pass: a failing e2e command still records + commits,
    // so attest itself exits 0.
    let repo = TempRepo::new();
    assert_eq!(attest_exit(&repo.0, "exit 1"), 0);
    assert!(repo.0.join(ATTESTATION_PATH).is_file());
}

#[test]
fn attest_fails_when_required_signing_cannot_be_satisfied() {
    // E2E mirror of the integration check: a repo that requires signed commits but
    // whose signer is unsatisfiable. Honoring `commit.gpgsign` (no forced-off) means
    // the attestation commit is attempted and fails, so the binary exits non-zero —
    // rather than silently committing unsigned and exiting 0.
    let repo = TempRepo::new();
    require_unsatisfiable_signing(&repo.0);

    assert_ne!(
        attest_exit(&repo.0, "true"),
        0,
        "attest must surface the signing failure instead of forcing signing off"
    );
}
