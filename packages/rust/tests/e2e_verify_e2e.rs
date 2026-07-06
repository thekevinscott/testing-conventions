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

// --- #281: `e2e verify <path>` behaves identically to running with cwd
// `<path>` — proven end-to-end by spawning the built binary with cwd fixed at
// the *repo root* while the path argument names the package subdirectory
// carrying the attestation. Before the CLI grows the `path` argument, passing
// an extra positional here is a clap usage error (non-zero exit, no
// attestation-shaped message), so these start red.

#[test]
fn verify_with_path_argument_exits_zero_when_the_package_attestation_is_fresh() {
    let repo = TempRepo::new();
    let package_rel = "packages/widget";
    std::fs::create_dir_all(repo.0.join(package_rel)).unwrap();
    // The package needs its own code commit before an attestation of it can be
    // fresh — a never-committed directory has no code history the `.`
    // pathspec (scoped to the package's cwd) can find.
    repo.commit_code(&format!("{package_rel}/widget.rs"), "pub fn widget() {}\n");
    // Attest scoped to the package subdirectory (cwd = the package).
    assert_eq!(
        run_cli(&repo.0.join(package_rel), &["e2e", "attest", "true"]).0,
        0,
        "attest should record the run"
    );
    // Verify from the repo root, naming the package via the new positional
    // argument — this must behave identically to running with cwd = package.
    let (code, _) = run_cli(&repo.0, &["e2e", "verify", package_rel]);
    assert_eq!(
        code, 0,
        "a fresh package-scoped attestation should pass verify via the path argument"
    );
}

#[test]
fn verify_with_path_argument_exits_nonzero_when_the_package_attestation_is_stale() {
    let repo = TempRepo::new();
    let package_rel = "packages/widget";
    std::fs::create_dir_all(repo.0.join(package_rel)).unwrap();
    repo.commit_code(&format!("{package_rel}/widget.rs"), "pub fn widget() {}\n");
    run_cli(&repo.0.join(package_rel), &["e2e", "attest", "true"]);
    // Move the package's code on without re-attesting.
    repo.commit_code(
        &format!("{package_rel}/widget2.rs"),
        "pub fn widget2() {}\n",
    );

    let (code, stderr) = run_cli(&repo.0, &["e2e", "verify", package_rel]);
    assert_ne!(
        code, 0,
        "a stale package-scoped attestation should fail verify via the path argument"
    );
    assert!(
        stderr.contains("attest"),
        "the failure should hint to re-run attest; got: {stderr}"
    );
}

// --- #294: `e2e verify <path> --scope <dir>` narrows the freshness walk to
// `<dir>` while still reading the attestation from `<path>`.

#[test]
fn verify_with_scope_ignores_a_commit_outside_it() {
    let repo = TempRepo::new();
    let package_rel = "packages/widget";
    std::fs::create_dir_all(repo.0.join(package_rel).join("src")).unwrap();
    std::fs::create_dir_all(repo.0.join(package_rel).join("tests")).unwrap();
    repo.commit_code(
        &format!("{package_rel}/src/widget.rs"),
        "pub fn widget() {}\n",
    );
    run_cli(&repo.0.join(package_rel), &["e2e", "attest", "true"]);
    // A commit outside the scoped src/ dir, but still inside the package root.
    repo.commit_code(&format!("{package_rel}/tests/widget_test.rs"), "// test\n");

    let (code, _) = run_cli(
        &repo.0,
        &[
            "e2e",
            "verify",
            package_rel,
            "--scope",
            &format!("{package_rel}/src"),
        ],
    );
    assert_eq!(
        code, 0,
        "a commit outside --scope should not trip freshness"
    );
}

#[test]
fn verify_with_no_scope_is_unchanged_from_today() {
    // Regression guard: omitting --scope stays byte-identical to #281's
    // whole-path freshness walk.
    let repo = TempRepo::new();
    let package_rel = "packages/widget";
    std::fs::create_dir_all(repo.0.join(package_rel).join("src")).unwrap();
    repo.commit_code(
        &format!("{package_rel}/src/widget.rs"),
        "pub fn widget() {}\n",
    );
    run_cli(&repo.0.join(package_rel), &["e2e", "attest", "true"]);
    repo.commit_code(&format!("{package_rel}/other.rs"), "pub fn other() {}\n");

    let (code, _) = run_cli(&repo.0, &["e2e", "verify", package_rel]);
    assert_ne!(
        code, 0,
        "with no --scope, a commit anywhere under path should still count as code"
    );
}

#[test]
fn verify_with_no_argument_is_unchanged_from_today() {
    // Regression guard: `e2e verify` with no argument stays byte-identical —
    // the default `.` resolves against cwd, exactly like the pre-#281 behavior
    // covered above.
    let repo = TempRepo::new();
    run_cli(&repo.0, &["e2e", "attest", "true"]);
    let (code, _) = run_cli(&repo.0, &["e2e", "verify"]);
    assert_eq!(
        code, 0,
        "a fresh attestation at cwd should still pass with no argument"
    );
}

// --- #319: `e2e verify <path> --scope <dir> --base <ref>` restricts freshness to
// the commits this branch introduced (`<base>..HEAD`), the squash-safe form the
// reusable job needs. A PR that didn't touch the scoped source passes (exit 0)
// even when the attestation is stale against absolute history; a PR that changed
// the scoped source without re-attesting still fails.

#[test]
fn verify_with_base_exits_zero_on_an_unrelated_branch() {
    let repo = TempRepo::new();
    let package_rel = "packages/widget";
    std::fs::create_dir_all(repo.0.join(package_rel).join("src")).unwrap();
    std::fs::create_dir_all(repo.0.join("packages/other")).unwrap();
    repo.commit_code(
        &format!("{package_rel}/src/widget.rs"),
        "pub fn widget() {}\n",
    );
    run_cli(&repo.0.join(package_rel), &["e2e", "attest", "true"]);
    // A later scoped commit the attestation does not name (stale vs history).
    repo.commit_code(
        &format!("{package_rel}/src/widget.rs"),
        "pub fn widget() { /* v2 */ }\n",
    );
    let base = rev_parse(&repo.0, "HEAD");
    // The PR touches a different package.
    repo.commit_code("packages/other/thing.rs", "pub fn thing() {}\n");

    let (code, _) = run_cli(
        &repo.0,
        &[
            "e2e",
            "verify",
            package_rel,
            "--scope",
            &format!("{package_rel}/src"),
            "--base",
            &base,
        ],
    );
    assert_eq!(
        code, 0,
        "--base must make an unrelated PR pass despite a stale-on-base attestation"
    );
}

#[test]
fn verify_with_base_fails_when_the_branch_changed_scoped_source() {
    let repo = TempRepo::new();
    let package_rel = "packages/widget";
    std::fs::create_dir_all(repo.0.join(package_rel).join("src")).unwrap();
    repo.commit_code(
        &format!("{package_rel}/src/widget.rs"),
        "pub fn widget() {}\n",
    );
    run_cli(&repo.0.join(package_rel), &["e2e", "attest", "true"]);
    let base = rev_parse(&repo.0, "HEAD");
    // The PR changes the scoped source without re-attesting.
    repo.commit_code(
        &format!("{package_rel}/src/widget.rs"),
        "pub fn widget() { /* v2 */ }\n",
    );

    let (code, stderr) = run_cli(
        &repo.0,
        &[
            "e2e",
            "verify",
            package_rel,
            "--scope",
            &format!("{package_rel}/src"),
            "--base",
            &base,
        ],
    );
    assert_ne!(code, 0, "a scoped change on the branch should fail --base");
    assert!(
        stderr.contains("attest"),
        "the failure should hint to re-run attest; got: {stderr}"
    );
}

// --- #333: `e2e verify <path> --base <ref> [--extra-scope <dir>]...
// [--exclude <dir>]...` joins extra freshness roots (a shared source tree that is
// a sibling of the package) into the walk. A non-excluded change under an extra
// root stales the attestation; a change only under an excluded subtree stays
// fresh. Before the binary grows the flags, passing them is a clap usage error
// (non-zero exit, no attestation-shaped message), so these start red.

#[test]
fn verify_with_extra_scope_fails_on_a_non_excluded_core_change() {
    let repo = TempRepo::new();
    let package_rel = "packages/python";
    std::fs::create_dir_all(repo.0.join(package_rel).join("src")).unwrap();
    std::fs::create_dir_all(repo.0.join("packages/rust/src")).unwrap();
    repo.commit_code(
        &format!("{package_rel}/src/lib.rs"),
        "pub fn binding() {}\n",
    );
    run_cli(&repo.0.join(package_rel), &["e2e", "attest", "true"]);
    let base = rev_parse(&repo.0, "HEAD");
    // The PR touches only the shared core, outside the binding's own subtree.
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() {}\n");

    let (code, stderr) = run_cli(
        &repo.0,
        &[
            "e2e",
            "verify",
            package_rel,
            "--base",
            &base,
            "--extra-scope",
            "packages/rust/src",
            "--exclude",
            "packages/rust/src/cli",
        ],
    );
    assert_ne!(
        code, 0,
        "a non-excluded change under --extra-scope should fail verify"
    );
    assert!(
        stderr.contains("attest"),
        "the failure should hint to re-run attest; got: {stderr}"
    );
}

#[test]
fn verify_with_extra_scope_exits_zero_on_an_excluded_change() {
    let repo = TempRepo::new();
    let package_rel = "packages/python";
    std::fs::create_dir_all(repo.0.join(package_rel).join("src")).unwrap();
    std::fs::create_dir_all(repo.0.join("packages/rust/src/cli")).unwrap();
    repo.commit_code(
        &format!("{package_rel}/src/lib.rs"),
        "pub fn binding() {}\n",
    );
    run_cli(&repo.0.join(package_rel), &["e2e", "attest", "true"]);
    let base = rev_parse(&repo.0, "HEAD");
    // A change only under the feature-gated cli/ subtree of the extra root.
    repo.commit_code("packages/rust/src/cli/main.rs", "pub fn cli() {}\n");

    let (code, _) = run_cli(
        &repo.0,
        &[
            "e2e",
            "verify",
            package_rel,
            "--base",
            &base,
            "--extra-scope",
            "packages/rust/src",
            "--exclude",
            "packages/rust/src/cli",
        ],
    );
    assert_eq!(
        code, 0,
        "a change only under --exclude should not trip freshness"
    );
}
