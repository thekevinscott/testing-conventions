//! Integration tests for `e2e attest` (#17, slice #67).
//!
//! `attest` reads HEAD and commits, so each test builds a throwaway git repo
//! with one seed commit (the "code commit"), runs `attest`, and asserts it
//! recorded the run against that commit, wrote the attestation file, and
//! committed it on top. Per the #3 guardrail these are the clean (passing
//! command) and red (failing command) cases.
//!
//! These start red against the stub in `src/e2e.rs` and go green once `attest`
//! is implemented.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::e2e::{attest, Attestation, ATTESTATION_PATH};

/// A throwaway git repo with one seed commit, removed on drop.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-e2e-attest-{}-{}",
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

    fn head(&self) -> String {
        rev_parse(&self.0, "HEAD")
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

/// Configure `repo` to require signed commits, but point signing at a program
/// that does not exist — so any *attempted* signature fails. This lets a test
/// prove `attest` honors the repo's `commit.gpgsign` without a working signer
/// (a real signature isn't what's under test, and isn't portably available
/// here): honoring the policy means the commit is *attempted* and fails, rather
/// than silently skipped.
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
fn attest_names_head_writes_the_file_and_commits_it() {
    let repo = TempRepo::new();
    let code_commit = repo.head();

    let att = attest(&repo.0, "true").expect("attest should succeed");

    // Records the run against the current code commit.
    assert_eq!(att.command, "true");
    assert_eq!(att.exit_code, 0);
    assert_eq!(att.commit, code_commit);

    // Writes the attestation file, and the on-disk contents match the return.
    let path = repo.0.join(ATTESTATION_PATH);
    assert!(path.is_file(), "the attestation file should be written");
    let on_disk: Attestation =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(on_disk, att);

    // Commits it on top: HEAD advanced and its parent is the code commit, so the
    // attestation rides as the commit naming the code beneath it.
    let new_head = repo.head();
    assert_ne!(new_head, code_commit, "attest should create a commit");
    assert_eq!(
        rev_parse(&repo.0, &format!("{new_head}^")),
        code_commit,
        "the attestation commit's parent is the attested code commit"
    );
}

#[test]
fn attest_runs_the_command() {
    let repo = TempRepo::new();
    attest(&repo.0, "echo ran > marker").expect("attest should succeed");
    assert!(
        repo.0.join("marker").is_file(),
        "attest must actually run the command"
    );
}

#[test]
fn attest_records_a_failing_run_and_still_commits() {
    // Force a run, not a pass: a non-zero command still produces a committed
    // attestation recording the failure.
    let repo = TempRepo::new();
    let code_commit = repo.head();

    let att = attest(&repo.0, "exit 3").expect("attest itself should still succeed");

    assert_eq!(att.exit_code, 3, "the command's exit code is recorded");
    assert_eq!(att.commit, code_commit);
    assert!(repo.0.join(ATTESTATION_PATH).is_file());
    assert_ne!(
        repo.head(),
        code_commit,
        "a failing run is still committed (force-run, not force-pass)"
    );
}

#[test]
fn attest_errors_outside_a_git_repo() {
    // No git repo → no HEAD to attest against → a clear error, not a panic.
    let dir = std::env::temp_dir().join(format!(
        "tc-e2e-attest-nogit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let result = attest(&dir, "true");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(result.is_err(), "attest outside a git repo should error");
}

#[test]
fn attest_honors_repo_commit_signing() {
    // The nudge targets locked-down repos — exactly the ones whose branch
    // protection requires *verified* signatures. So `attest` must honor the repo's
    // `commit.gpgsign` instead of forcing it off: an unsigned attestation commit
    // can't land there. With signing required but unsatisfiable, honoring the
    // policy means the commit (and so `attest`) fails loudly; the old forced
    // `commit.gpgsign=false` instead skips signing and wrongly succeeds.
    let repo = TempRepo::new();
    require_unsatisfiable_signing(&repo.0);

    let result = attest(&repo.0, "true");

    assert!(
        result.is_err(),
        "attest must honor the repo's commit.gpgsign (attempt the signature) \
         instead of forcing it off and committing unsigned"
    );
}
