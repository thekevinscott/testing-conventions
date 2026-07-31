//! E2E tests for `e2e attest`: drive the built CLI binary in a
//! throwaway git repo (no mocks) and assert it runs the command, commits the
//! branch's receipt on a pass, and propagates the command's own exit code —
//! so a wrapping `just` recipe or CI step reads a failing run as a failure.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// The work branch every test attests on, and its receipt's committed path.
const BRANCH: &str = "work";
const RECEIPT: &str = "e2e-attestations/work.json";

/// A throwaway git repo with one seed commit on branch `work`, removed on drop.
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
        git(&root, &["checkout", "-q", "-b", BRANCH]);
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

/// Run the built binary's `e2e attest <command>` with the cwd set to `repo`,
/// returning its exit code and stderr.
fn attest_run(repo: &Path, command: &str) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["e2e", "attest", command])
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

/// The exit code of `e2e attest <command>` run with the cwd set to `repo`.
fn attest_exit(repo: &Path, command: &str) -> i32 {
    attest_run(repo, command).0
}

/// Configure `repo` to require signed commits, but point signing at a program
/// that does not exist — so any *attempted* signature fails. Honoring the repo's
/// `commit.gpgsign` then means the receipt commit is attempted and fails
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
fn attest_exits_zero_and_commits_the_receipt() {
    let repo = TempRepo::new();
    let code_commit = repo.head();

    assert_eq!(attest_exit(&repo.0, "true"), 0, "a passing command exits 0");
    assert!(
        repo.0.join(RECEIPT).is_file(),
        "attest should write the branch's receipt"
    );
    assert_ne!(
        repo.head(),
        code_commit,
        "attest should commit the receipt on top"
    );
}

#[test]
fn attest_propagates_the_commands_exit_code_and_commits_nothing() {
    // The failure a caller must see: a red e2e run exits red, and leaves no
    // committed receipt to push as a passing one.
    let repo = TempRepo::new();
    let code_commit = repo.head();

    let (code, stderr) = attest_run(&repo.0, "exit 3");

    assert_eq!(code, 3, "attest exits with the wrapped command's code");
    assert!(
        !repo.0.join(RECEIPT).exists(),
        "a failing run writes no receipt"
    );
    assert_eq!(repo.head(), code_commit, "and commits nothing");
    assert!(
        stderr.contains("exited 3"),
        "the failure names the command's exit code: {stderr}"
    );
}

#[test]
fn attest_commits_only_an_add_and_keeps_another_branchs_receipt() {
    // The shape of the commit is the whole fix: a delete paired with this
    // branch's add is what git's rename detection turns into a rename, and two
    // branches off one parent renaming the same source is an unresolvable
    // rename/rename conflict. A pure add has nothing to pair with.
    let repo = TempRepo::new();
    let foreign = repo.0.join("e2e-attestations/some-other-branch.json");
    std::fs::create_dir_all(foreign.parent().unwrap()).unwrap();
    std::fs::write(&foreign, "{}\n").unwrap();
    git(&repo.0, &["add", "-A"]);
    git(
        &repo.0,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-m",
            "foreign receipt",
        ],
    );

    assert_eq!(attest_exit(&repo.0, "true"), 0);

    assert!(
        foreign.is_file(),
        "another branch's receipt must survive the attest"
    );
    let out = Command::new("git")
        .args(["show", "--name-status", "--format=", "HEAD"])
        .current_dir(&repo.0)
        .output()
        .expect("git show should run");
    let status = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        status.trim(),
        format!("A\t{RECEIPT}"),
        "the receipt commit must be a single add, with no delete to pair into a rename"
    );
}

#[test]
fn attest_fails_when_required_signing_cannot_be_satisfied() {
    // E2E mirror of the integration check: a repo that requires signed commits but
    // whose signer is unsatisfiable. Honoring `commit.gpgsign` (no forced-off) means
    // the receipt commit is attempted and fails, so the binary exits non-zero —
    // rather than silently committing unsigned and exiting 0.
    let repo = TempRepo::new();
    require_unsatisfiable_signing(&repo.0);

    assert_ne!(
        attest_exit(&repo.0, "true"),
        0,
        "attest must surface the signing failure instead of forcing signing off"
    );
}
