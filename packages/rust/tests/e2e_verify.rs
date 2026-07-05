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

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::e2e::{attest, verify, Verification};
use testing_conventions::run;

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

// --- #281: `e2e verify` accepts a directory argument, scoping attestation
// discovery to it instead of always reading the checkout root. `e2e::verify`
// already takes a `&Path` (this whole file exercises it that way); these cases
// pin the *library* behavior a subdirectory argument depends on: attesting and
// verifying against a package subdirectory of a larger repo behaves the same as
// attesting/verifying at the repo root — fresh, stale, and missing all scope to
// the given directory rather than some ambient root.

#[test]
fn verify_scopes_fresh_to_a_package_subdirectory() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();
    // The package needs its own code commit before it can be "fresh" — a
    // freshly created, never-committed directory has no code history for the
    // `.` pathspec (scoped to the package's cwd) to find.
    repo.commit_code("packages/widget/widget.rs", "pub fn widget() {}\n");
    // Attest inside the subdirectory: the attestation is written and committed
    // relative to `package`, not the repo root.
    attest(&package, "true").expect("attest should succeed");
    assert_eq!(
        verify(&package).expect("verify should succeed"),
        Verification::Fresh,
    );
    // The repo root itself carries no attestation — verifying it is Missing,
    // proving discovery is scoped to the given directory, not the checkout root.
    assert_eq!(
        verify(&repo.0).expect("verify should succeed"),
        Verification::Missing,
    );
}

#[test]
fn verify_scopes_stale_to_a_package_subdirectory() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();
    repo.commit_code("packages/widget/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let attested = rev_parse(&repo.0, "HEAD^");
    repo.commit_code("packages/widget/widget2.rs", "pub fn widget2() {}\n");
    let latest = rev_parse(&repo.0, "HEAD");

    assert_eq!(
        verify(&package).expect("verify should succeed"),
        Verification::Stale { attested, latest },
    );
}

#[test]
fn verify_scopes_missing_to_a_package_subdirectory() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();
    assert_eq!(
        verify(&package).expect("verify should succeed"),
        Verification::Missing,
    );
}

// --- #281: the `testing-conventions e2e verify <path>` CLI surface. `run()`
// dispatches in-process, so these never touch the test binary's own working
// directory — the path argument alone must drive discovery. Before `lib.rs`
// grows the `Verify { path }` field these fail to parse at all (clap rejects
// the unexpected positional argument on the current unit-variant `Verify`).

/// `testing-conventions e2e verify <path>` exit code, dispatched in-process.
fn e2e_verify_run(path: &Path) -> anyhow::Result<i32> {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "e2e".into(),
        "verify".into(),
        path.as_os_str().to_owned(),
    ];
    run(argv)
}

#[test]
fn cli_verify_with_path_argument_passes_when_fresh() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();
    repo.commit_code("packages/widget/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");

    assert_eq!(
        e2e_verify_run(&package).expect("dispatch should succeed"),
        0,
        "a fresh attestation at the given path should pass",
    );
}

#[test]
fn cli_verify_with_path_argument_fails_when_missing() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();

    assert_eq!(
        e2e_verify_run(&package).expect("dispatch should succeed"),
        1,
        "no attestation at the given path should fail",
    );
}

#[test]
fn cli_verify_with_path_argument_fails_when_stale() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();
    repo.commit_code("packages/widget/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    repo.commit_code("packages/widget/widget2.rs", "pub fn widget2() {}\n");

    assert_eq!(
        e2e_verify_run(&package).expect("dispatch should succeed"),
        1,
        "a stale attestation at the given path should fail",
    );
}

#[test]
fn cli_verify_with_no_argument_defaults_to_the_current_directory() {
    // Regression guard (#281): `e2e verify` with *no* argument must stay
    // byte-identical to today — the default `.` resolves against whatever the
    // process's current directory is, exactly like the pre-#281 `current_dir()`
    // call did. `run()` dispatches in-process, so cwd here really is the test
    // binary's own working directory (the crate root) — asserting only that the
    // no-arg form still parses and dispatches (rather than erroring as an
    // unrecognized invocation) is the regression this locks down; the
    // fresh/stale/missing behavior at cwd is already covered end-to-end by
    // `e2e_verify_e2e.rs`.
    let argv: Vec<OsString> = vec!["testing-conventions".into(), "e2e".into(), "verify".into()];
    let code = run(argv).expect("`e2e verify` with no argument should still dispatch");
    // The crate root itself carries no attestation, so this is `1` (Missing) —
    // the point is that it dispatches at all, not which outcome cwd produces.
    assert_eq!(code, 1);
}
