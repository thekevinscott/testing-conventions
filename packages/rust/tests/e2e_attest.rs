//! Integration tests for `e2e attest`.
//!
//! `attest` reads HEAD and the checked-out branch, runs the given command, and
//! — when the command passes — commits the branch's receipt under
//! `e2e-attestations/`. Each test builds a throwaway git repo with one seed
//! commit on a work branch, runs `attest`, and asserts it recorded the run: the
//! clean case writes and commits the receipt, and a failing command reports the
//! failure while leaving the receipts untouched.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::e2e::{attest, Attestation, RECEIPTS_DIR};

/// The work branch every test attests on, and its receipt's committed path.
const BRANCH: &str = "work";
const RECEIPT: &str = "e2e-attestations/work.json";

/// A throwaway git repo with one seed commit on branch `work`, removed on drop.
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
fn attest_records_the_run_writes_the_receipt_and_commits_it() {
    let repo = TempRepo::new();
    let code_commit = repo.head();

    let att = attest(&repo.0, "true").expect("attest should succeed");

    // Records the run against the current code commit and branch.
    assert_eq!(att.command, "true");
    assert_eq!(att.exit_code, 0);
    assert_eq!(att.commit, code_commit);
    assert_eq!(att.branch, BRANCH);

    // Writes the branch's receipt, and the on-disk contents match the return.
    let path = repo.0.join(RECEIPT);
    assert!(path.is_file(), "the receipt should be written");
    let on_disk: Attestation =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(on_disk, att);

    // Commits it on top: HEAD advanced and its parent is the code commit, so the
    // receipt rides as the commit naming the code beneath it.
    let new_head = repo.head();
    assert_ne!(new_head, code_commit, "attest should create a commit");
    assert_eq!(
        rev_parse(&repo.0, &format!("{new_head}^")),
        code_commit,
        "the receipt commit's parent is the attested code commit"
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
fn attest_reports_a_failing_run_and_writes_no_receipt() {
    // A receipt is a passing run's record, so a failing command produces none:
    // the caller reads the failure off the returned exit code, and the tree is
    // exactly as it was.
    let repo = TempRepo::new();
    let code_commit = repo.head();

    let att = attest(&repo.0, "exit 3").expect("attest itself should still succeed");

    assert_eq!(att.exit_code, 3, "the command's exit code is reported");
    assert_eq!(att.commit, code_commit);
    assert!(
        !repo.0.join(RECEIPT).exists(),
        "a failing run leaves no receipt to commit as a passing one"
    );
    assert_eq!(
        repo.head(),
        code_commit,
        "a failing run commits nothing on top"
    );
}

#[test]
fn attest_leaves_the_branchs_earlier_receipt_intact_when_the_command_fails() {
    // A failing re-attest returns before the write, so it does not overwrite the
    // receipt an earlier passing run committed — a failure costs the branch
    // nothing it had already earned.
    let repo = TempRepo::new();
    attest(&repo.0, "true").expect("the passing attest should succeed");
    let recorded = std::fs::read_to_string(repo.0.join(RECEIPT)).unwrap();
    let receipt_commit = repo.head();

    attest(&repo.0, "exit 1").expect("attest itself should still succeed");

    assert_eq!(
        std::fs::read_to_string(repo.0.join(RECEIPT)).unwrap(),
        recorded,
        "the failing run leaves the committed receipt as it was"
    );
    assert_eq!(repo.head(), receipt_commit, "and adds no commit of its own");
}

#[test]
fn attest_runs_the_command_even_when_it_fails() {
    // Skipping the receipt is not skipping the run: the command still executes.
    let repo = TempRepo::new();
    attest(&repo.0, "echo ran > marker; exit 1").expect("attest itself should still succeed");
    assert!(
        repo.0.join("marker").is_file(),
        "attest must run the command whatever its outcome"
    );
}

#[test]
fn attest_leaves_the_retired_single_file_attestation_alone() {
    // Collecting it deleted a file in the same commit that adds this branch's
    // receipt — the same delete/add pairing rename detection turns into a
    // rename, so two branches upgrading in parallel renamed the legacy file to
    // two names and conflicted. It is inert where it sits: never read as a
    // receipt, never counted as scoped source. Sweeping it is the repo's call.
    let repo = TempRepo::new();
    std::fs::write(repo.0.join("e2e-attestation.json"), "{}\n").unwrap();
    git(&repo.0, &["add", "e2e-attestation.json"]);
    git(&repo.0, &["commit", "-q", "-m", "legacy attestation"]);

    attest(&repo.0, "true").expect("attest should succeed");

    assert!(
        repo.0.join("e2e-attestation.json").is_file(),
        "the legacy single file survives; deleting it is what pairs into a rename"
    );
    assert!(repo.0.join(RECEIPT).is_file());
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo.0)
        .output()
        .unwrap();
    assert!(
        status.stdout.is_empty(),
        "the removal is committed, not left dirty: {}",
        String::from_utf8_lossy(&status.stdout)
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
    // `commit.gpgsign` instead of forcing it off: an unsigned receipt commit
    // can't land there. With signing required but unsatisfiable, honoring the
    // policy means the commit (and so `attest`) fails loudly; a forced
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

#[test]
fn attest_leaves_another_branchs_receipt_in_place() {
    // Deleting a sibling's receipt pairs the delete with this branch's add, and
    // git's rename detection turns that pair into a rename whenever the two
    // receipts look alike — which they do, because the `command` field is
    // usually byte-identical across a repo's branches. Two branches off one
    // parent then rename the same source and the merge conflicts.
    let repo = TempRepo::new();
    let foreign = repo.0.join(RECEIPTS_DIR).join("some-other-branch.json");
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

    attest(&repo.0, "true").expect("attest should succeed");

    assert!(
        foreign.is_file(),
        "another branch's receipt must survive; deleting it is what makes \
         sibling branches conflict"
    );
    assert!(
        repo.0.join(RECEIPT).is_file(),
        "this branch's receipt should still be written"
    );
}

#[test]
fn receipts_dir_is_the_public_location() {
    // The committed path is the public contract scripts rely on.
    assert_eq!(RECEIPTS_DIR, "e2e-attestations");
}
