//! E2E tests for `e2e attest` (#17, slice #67): drive the built CLI binary in a
//! throwaway git repo (no mocks) and assert it force-runs the command, exits
//! `0`, and commits an attestation on top.
//!
//! Starts red against the stub in `src/e2e.rs` and goes green once `attest` is
//! implemented.

use std::os::unix::fs::PermissionsExt;
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

/// Write an executable stand-in signer into `dir` and return its path. git's
/// OpenPGP path runs `gpg --status-fd=2 -bsau <key>`, reads the signature from
/// stdout, and treats `[GNUPG:] SIG_CREATED` on the status fd as success — so
/// this records a signature with no real key material or network.
fn write_fake_gpg(dir: &Path) -> PathBuf {
    let path = dir.join("fake-gpg");
    std::fs::write(
        &path,
        "#!/bin/sh\n\
         echo '[GNUPG:] SIG_CREATED S' >&2\n\
         printf '%s\\n' '-----BEGIN PGP SIGNATURE-----' '' 'ZmFrZQ==' '-----END PGP SIGNATURE-----'\n",
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// Configure `repo` to require an OpenPGP-signed commit, signed by `fake_gpg`.
fn require_signing(repo: &Path, fake_gpg: &Path) {
    git(repo, &["config", "gpg.format", "openpgp"]);
    git(repo, &["config", "gpg.program", fake_gpg.to_str().unwrap()]);
    git(repo, &["config", "user.signingkey", "fake"]);
    git(repo, &["config", "commit.gpgsign", "true"]);
}

/// Whether the commit at `rev` carries a signature (a `gpgsig` header).
fn commit_is_signed(dir: &Path, rev: &str) -> bool {
    let out = Command::new("git")
        .args(["cat-file", "commit", rev])
        .current_dir(dir)
        .output()
        .expect("git cat-file should run");
    assert!(out.status.success(), "git cat-file {rev} failed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|line| line.starts_with("gpgsig"))
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
fn attest_signs_the_attestation_commit_when_the_repo_requires_signing() {
    // #128: driving the built binary, attest in a signing-required repo must
    // produce a *signed* attestation commit (inheriting commit.gpgsign) so it can
    // merge where verified signatures are required — and still exit 0.
    let repo = TempRepo::new();
    let fake_gpg = write_fake_gpg(&repo.0);
    require_signing(&repo.0, &fake_gpg);

    assert_eq!(
        attest_exit(&repo.0, "true"),
        0,
        "attest force-runs and exits 0"
    );
    assert!(
        commit_is_signed(&repo.0, "HEAD"),
        "the attestation commit should be signed when the repo requires signing"
    );
}
