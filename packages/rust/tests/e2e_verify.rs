//! Integration tests for `e2e verify` (#17, slice #68).
//!
//! `verify` reads the committed attestation and confirms it names the *latest
//! code commit* — the newest commit touching any path other than the attestation
//! file. Each test builds a throwaway git repo, optionally attests, and asserts
//! the [`Verification`] outcome. Per the #3 guardrail: the clean case (a fresh
//! attestation passes) and the red cases (no attestation; code changed since).
//!
//! These start red against the stub in `src/e2e.rs` and go green once `verify`
//! is implemented.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::e2e::{attest, verify, Verification};

/// A throwaway git repo with one seed commit, removed on drop.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-e2e-verify-{}-{}",
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

    /// Add and commit a code file, advancing HEAD to a new code commit (so a
    /// prior attestation goes stale).
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

fn rev_parse(dir: &Path, rev: &str) -> String {
    let out = Command::new("git")
        .args(["rev-parse", rev])
        .current_dir(dir)
        .output()
        .expect("git rev-parse should run");
    assert!(out.status.success(), "git rev-parse {rev} failed");
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

#[test]
fn verify_passes_when_the_attestation_names_the_latest_code_commit() {
    let repo = TempRepo::new();
    // Attest against the current code commit: writes the attestation and commits
    // it on top, so it names the code commit beneath it.
    attest(&repo.0, "true").expect("attest should succeed");
    assert_eq!(
        verify(&repo.0).expect("verify should succeed"),
        Verification::Fresh,
    );
}

#[test]
fn verify_fails_when_no_attestation_is_present() {
    let repo = TempRepo::new();
    assert_eq!(
        verify(&repo.0).expect("verify should succeed"),
        Verification::Missing,
    );
}

#[test]
fn verify_fails_when_code_changed_since_the_attestation() {
    let repo = TempRepo::new();
    attest(&repo.0, "true").expect("attest should succeed");
    // The attestation names the code commit it rode on top of.
    let attested = rev_parse(&repo.0, "HEAD^");
    // A new code commit on top makes the attestation stale.
    repo.commit_code("widget.rs", "pub fn widget() {}\n");
    let latest = rev_parse(&repo.0, "HEAD");

    assert_eq!(
        verify(&repo.0).expect("verify should succeed"),
        Verification::Stale { attested, latest },
    );
}
