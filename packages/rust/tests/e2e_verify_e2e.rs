//! E2E tests for `e2e verify`: drive the built CLI binary in a throwaway git
//! repo (no mocks) and assert the exit codes and the actionable failure hint.
//! The branch-diff semantics are pinned end-to-end in `e2e_receipts_e2e.rs`;
//! this file covers presence (no `--base`), the `path` argument, the caller
//! `--scope`, the `--extra-scope`/`--exclude` pair, and the #391 loud error.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A throwaway git repo with one seed commit on branch `work`, removed on drop.
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
        // Throwaway repos never sign — keep the suite hermetic regardless of the
        // machine's global `commit.gpgsign`.
        git(&root, &["config", "commit.gpgsign", "false"]);
        std::fs::write(root.join("README.md"), "seed\n").unwrap();
        git(&root, &["add", "."]);
        git(
            &root,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "seed"],
        );
        git(&root, &["checkout", "-q", "-b", "work"]);
        TempRepo(root)
    }

    fn commit_code(&self, name: &str, contents: &str) {
        let full = self.0.join(name);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, contents).unwrap();
        git(&self.0, &["add", name]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "code"],
        );
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

/// Run the built binary with `args` in `cwd`, returning (exit code, stdout+stderr).
fn run_cli(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("the built binary should run");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code().expect("an exit code"), text)
}

#[test]
fn verify_exits_zero_on_a_committed_receipt() {
    let repo = TempRepo::new();
    let (code, _) = run_cli(&repo.0, &["e2e", "attest", "true"]);
    assert_eq!(code, 0);

    let (code, text) = run_cli(&repo.0, &["e2e", "verify"]);
    assert_eq!(code, 0, "a committed receipt should pass: {text}");
}

#[test]
fn verify_exits_nonzero_with_the_attest_hint_when_missing() {
    let repo = TempRepo::new();
    let (code, text) = run_cli(&repo.0, &["e2e", "verify"]);
    assert_eq!(code, 1, "no receipt should fail");
    assert!(
        text.contains("e2e attest"),
        "the failure names the fix: {text}"
    );
}

#[test]
fn verify_with_path_argument_reads_the_package_receipts() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();
    repo.commit_code("packages/widget/widget.rs", "pub fn widget() {}\n");
    let (code, _) = run_cli(&package, &["e2e", "attest", "true"]);
    assert_eq!(code, 0);

    // Verified from the repo root, with the package as the path argument.
    let (code, text) = run_cli(&repo.0, &["e2e", "verify", "packages/widget"]);
    assert_eq!(code, 0, "the package's receipt should pass: {text}");

    // The repo root itself carries no receipts.
    let (code, _) = run_cli(&repo.0, &["e2e", "verify"]);
    assert_eq!(code, 1, "discovery is scoped to the path argument");
}

#[test]
fn verify_with_base_and_scope_ignores_a_change_outside_the_scope() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    let (code, _) = run_cli(&package, &["e2e", "attest", "true"]);
    assert_eq!(code, 0);
    let base = repo.head();
    repo.commit_code("packages/widget/tests/widget_test.rs", "// test\n");

    let (code, text) = run_cli(
        &repo.0,
        &[
            "e2e",
            "verify",
            "packages/widget",
            "--scope",
            "packages/widget/src",
            "--base",
            &base,
        ],
    );
    assert_eq!(code, 0, "a change outside --scope owes no decision: {text}");
}

#[test]
fn verify_with_base_demands_a_receipt_for_a_scoped_change() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    let (code, _) = run_cli(&package, &["e2e", "attest", "true"]);
    assert_eq!(code, 0);
    let base = repo.head();
    repo.commit_code(
        "packages/widget/src/widget.rs",
        "pub fn widget() { /* v2 */ }\n",
    );

    let (code, text) = run_cli(
        &repo.0,
        &["e2e", "verify", "packages/widget", "--base", &base],
    );
    assert_eq!(code, 1, "an unanswered scoped change fails");
    assert!(
        text.contains("e2e attest"),
        "the failure names the fix: {text}"
    );

    // Attest on the branch and the same call passes.
    let (code, _) = run_cli(&package, &["e2e", "attest", "true"]);
    assert_eq!(code, 0);
    let (code, text) = run_cli(
        &repo.0,
        &["e2e", "verify", "packages/widget", "--base", &base],
    );
    assert_eq!(code, 0, "the branch's receipt answers the nudge: {text}");
}

#[test]
fn verify_with_extra_scope_demands_a_receipt_for_a_core_change() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() {}\n");
    repo.commit_code("packages/rust/src/cli/main.rs", "pub fn cli() {}\n");
    let (code, _) = run_cli(&package, &["e2e", "attest", "true"]);
    assert_eq!(code, 0);
    let base = repo.head();
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() { /* v2 */ }\n");

    let (code, _) = run_cli(
        &repo.0,
        &[
            "e2e",
            "verify",
            "packages/python",
            "--base",
            &base,
            "--extra-scope",
            "packages/rust/src",
            "--exclude",
            "packages/rust/src/cli",
        ],
    );
    assert_eq!(
        code, 1,
        "a non-excluded core change owes the binding a decision"
    );
}

#[test]
fn verify_with_extra_scope_exits_zero_on_an_excluded_change() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    repo.commit_code("packages/rust/src/cli/main.rs", "pub fn cli() {}\n");
    let (code, _) = run_cli(&package, &["e2e", "attest", "true"]);
    assert_eq!(code, 0);
    let base = repo.head();
    repo.commit_code(
        "packages/rust/src/cli/main.rs",
        "pub fn cli() { /* v2 */ }\n",
    );

    let (code, text) = run_cli(
        &repo.0,
        &[
            "e2e",
            "verify",
            "packages/python",
            "--base",
            &base,
            "--extra-scope",
            "packages/rust/src",
            "--exclude",
            "packages/rust/src/cli",
        ],
    );
    assert_eq!(code, 0, "an excluded change owes no decision: {text}");
}

#[test]
fn verify_with_a_scope_matching_no_tracked_path_errors_loudly() {
    // #391: a typo'd scope must be a loud error naming the bad scope, never a
    // silent pass over an empty diff.
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    let (code, _) = run_cli(&package, &["e2e", "attest", "true"]);
    assert_eq!(code, 0);
    let base = repo.head();
    repo.commit_code(
        "packages/widget/src/widget.rs",
        "pub fn widget() { /* v2 */ }\n",
    );

    let (code, text) = run_cli(
        &repo.0,
        &[
            "e2e",
            "verify",
            "packages/widget",
            "--scope",
            "packages/widget/ghost",
            "--base",
            &base,
        ],
    );
    assert_ne!(code, 0, "a bogus --scope must fail");
    assert!(
        text.contains("scope"),
        "the error should name the bad scope: {text}"
    );
}
