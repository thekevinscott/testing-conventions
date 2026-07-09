//! Integration tests for `e2e verify`.
//!
//! `verify` confirms a committed receipt answers the branch's e2e nudge: with
//! `--base`, a branch whose content diff leaves the scoped source untouched
//! owes no decision, and one that changed it passes when its diff adds or
//! updates a receipt; without `--base`, receipt presence is the check. The
//! branch-diff semantics themselves are pinned in `e2e_receipts.rs`; this file
//! covers the discovery scoping (a package subdirectory), the `run()` CLI
//! surface (`path`, `--scope`, `--base`, `--extra-scope`, `--exclude`), the
//! entry-point equivalences, and the #391 loud-scope-validation contract.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::e2e::{
    attest, verify, verify_extra_scoped, verify_scoped, verify_since, Verification,
};
use testing_conventions::run;

/// A throwaway git repo with one seed commit on branch `work`, removed on drop.
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

    /// Add and commit a code file.
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

// --- without --base: receipt presence, discovered at `path` ---

#[test]
fn verify_passes_on_a_committed_receipt() {
    let repo = TempRepo::new();
    attest(&repo.0, "true").expect("attest should succeed");
    assert_eq!(
        verify(&repo.0).expect("verify should succeed"),
        Verification::Fresh,
    );
}

#[test]
fn verify_fails_when_no_receipt_is_present() {
    let repo = TempRepo::new();
    assert_eq!(
        verify(&repo.0).expect("verify should succeed"),
        Verification::Missing,
    );
}

#[test]
fn verify_presence_is_indifferent_to_later_code_commits() {
    // No base means no branch diff to read: a receipt stays a receipt however
    // much the tree moves afterward.
    let repo = TempRepo::new();
    attest(&repo.0, "true").expect("attest should succeed");
    repo.commit_code("widget.rs", "pub fn widget() {}\n");
    assert_eq!(
        verify(&repo.0).expect("verify should succeed"),
        Verification::Fresh,
    );
}

#[test]
fn verify_scopes_discovery_to_a_package_subdirectory() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();
    repo.commit_code("packages/widget/widget.rs", "pub fn widget() {}\n");
    // Attest inside the subdirectory: the receipt is written and committed
    // relative to `package`, not the repo root.
    attest(&package, "true").expect("attest should succeed");
    assert_eq!(
        verify(&package).expect("verify should succeed"),
        Verification::Fresh,
    );
    // The repo root itself carries no receipts — verifying it is Missing,
    // proving discovery is scoped to the given directory, not the checkout root.
    assert_eq!(
        verify(&repo.0).expect("verify should succeed"),
        Verification::Missing,
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

// --- entry-point equivalences ---

#[test]
fn verify_scoped_with_scope_equal_to_repo_matches_verify() {
    // `verify_scoped(repo, repo)` is `verify`'s exact definition — a direct
    // regression guard that the two stay in sync.
    let repo = TempRepo::new();
    attest(&repo.0, "true").expect("attest should succeed");
    assert_eq!(
        verify_scoped(&repo.0, &repo.0).expect("verify should succeed"),
        verify(&repo.0).expect("verify should succeed"),
    );
}

#[test]
fn verify_extra_scoped_with_no_extra_roots_matches_verify_since() {
    // No extra roots (and no excludes) is byte-identical to `verify_since` — a
    // package declaring nothing scopes the diff to `--scope` alone.
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code(
        "packages/widget/src/widget.rs",
        "pub fn widget() { /* v2 */ }\n",
    );

    assert_eq!(
        verify_extra_scoped(&package, &package.join("src"), Some(&base), &[], &[]).unwrap(),
        verify_since(&package, &package.join("src"), Some(&base)).unwrap(),
        "no extra roots must be byte-identical to verify_since",
    );
}

// --- with --base: the branch's own diff decides (library surface; the full
// --- matrix is pinned in e2e_receipts.rs) ---

#[test]
fn verify_since_passes_when_the_branch_left_the_scoped_source_untouched() {
    // The unrelated-PR case: the branch touches a *different* package, so it
    // owes the scoped package no decision — however old its receipt is.
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code("packages/other/thing.rs", "pub fn thing() {}\n");

    assert_eq!(
        verify_since(&package, &package.join("src"), Some(&base)).unwrap(),
        Verification::Fresh,
        "a PR that didn't touch the scoped source owes no decision",
    );
}

#[test]
fn verify_since_flags_a_scoped_change_the_branch_did_not_attest() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code(
        "packages/widget/src/widget.rs",
        "pub fn widget() { /* v2 */ }\n",
    );

    assert_eq!(
        verify_since(&package, &package.join("src"), Some(&base)).unwrap(),
        Verification::Missing,
        "a scoped change on the branch without a receipt in its diff must fail",
    );
}

#[test]
fn verify_since_passes_when_the_branch_attested_its_scoped_change() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code(
        "packages/widget/src/widget.rs",
        "pub fn widget() { /* v2 */ }\n",
    );
    attest(&package, "true").expect("re-attest should succeed");

    assert_eq!(
        verify_since(&package, &package.join("src"), Some(&base)).unwrap(),
        Verification::Fresh,
        "a scoped change the branch attested must pass",
    );
}

// --- extra scopes: a shared source tree beside the package joins the scoped
// --- diff, with feature-gated subtrees carved back out ---

#[test]
fn verify_extra_scoped_flags_a_change_under_an_extra_root() {
    // The dirsql shape: a binding package whose e2e artifact is compiled from a
    // shared core in a *sibling* tree. A core-only PR leaves the binding's own
    // diff empty — so `--base` alone would pass it — yet the binding's e2e is
    // exactly what the change puts at risk. Declaring the core as an extra
    // scope makes the change owe the binding a decision.
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    // The PR touches only the shared core — outside the binding's own subtree.
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() { /* v2 */ }\n");

    // Sanity: without the extra root, the binding's own diff is empty and the
    // branch passes — exactly the gap the extra scope closes.
    assert_eq!(
        verify_since(&package, &package, Some(&base)).unwrap(),
        Verification::Fresh,
        "scope-only --base can't see a sibling-tree change",
    );
    let extra = [PathBuf::from("packages/rust/src")];
    assert_eq!(
        verify_extra_scoped(&package, &package, Some(&base), &extra, &[]).unwrap(),
        Verification::Missing,
        "a non-excluded change under an extra root owes the binding a decision",
    );
}

#[test]
fn verify_extra_scoped_passes_once_the_extra_root_change_is_attested() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() { /* v2 */ }\n");
    attest(&package, "true").expect("re-attest should succeed");

    let extra = [PathBuf::from("packages/rust/src")];
    assert_eq!(
        verify_extra_scoped(&package, &package, Some(&base), &extra, &[]).unwrap(),
        Verification::Fresh,
        "attesting after the extra-root change must pass",
    );
}

#[test]
fn verify_extra_scoped_ignores_a_change_under_an_excluded_subtree() {
    // The feature-gated carve-out: `packages/rust/src/cli` is compiled out of the
    // binding, so a cli-only core change owes it nothing, even though cli lives
    // under the declared extra root.
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    repo.commit_code("packages/rust/src/cli/main.rs", "pub fn cli() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code(
        "packages/rust/src/cli/main.rs",
        "pub fn cli() { /* v2 */ }\n",
    );

    let extra = [PathBuf::from("packages/rust/src")];
    let exclude = [PathBuf::from("packages/rust/src/cli")];
    assert_eq!(
        verify_extra_scoped(&package, &package, Some(&base), &extra, &exclude).unwrap(),
        Verification::Fresh,
        "a change only under an excluded subtree owes no decision",
    );
}

// --- the `run()` CLI surface ---

/// `testing-conventions e2e verify …` exit code, dispatched in-process.
fn e2e_verify_cli(path: &Path, flags: &[(&str, &str)]) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "e2e".into(),
        "verify".into(),
        path.as_os_str().to_owned(),
    ];
    for (flag, value) in flags {
        argv.push((*flag).into());
        argv.push((*value).into());
    }
    run(argv)
}

#[test]
fn cli_verify_with_path_argument_passes_on_a_receipt() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();
    repo.commit_code("packages/widget/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");

    assert_eq!(
        e2e_verify_cli(&package, &[]).expect("dispatch should succeed"),
        0,
        "a receipt at the given path should pass",
    );
}

#[test]
fn cli_verify_with_path_argument_fails_when_missing() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(&package).unwrap();

    assert_eq!(
        e2e_verify_cli(&package, &[]).expect("dispatch should succeed"),
        1,
        "no receipt at the given path should fail",
    );
}

#[test]
fn cli_verify_with_no_argument_defaults_to_the_current_directory() {
    // `run()` dispatches in-process, so cwd here really is the test binary's own
    // working directory (the crate root) — asserting that the no-arg form still
    // parses and dispatches is the regression this locks down; the crate root
    // carries no receipts, so the outcome is `1` (Missing).
    let argv: Vec<OsString> = vec!["testing-conventions".into(), "e2e".into(), "verify".into()];
    let code = run(argv).expect("`e2e verify` with no argument should still dispatch");
    assert_eq!(code, 1);
}

#[test]
fn cli_verify_with_base_and_scope_ignores_a_change_outside_the_scope() {
    // The reusable e2e-verify job's shape: `e2e verify <path> --scope <dir>
    // --base <ref>`. A commit touching only the package's `tests/` — outside the
    // caller's scoped `src/` — owes no decision.
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code("packages/widget/tests/widget_test.rs", "// test\n");

    let scope = package.join("src");
    assert_eq!(
        e2e_verify_cli(
            &package,
            &[
                ("--scope", scope.to_str().unwrap()),
                ("--base", base.as_str()),
            ],
        )
        .expect("dispatch should succeed"),
        0,
        "a change outside --scope owes no decision",
    );
}

#[test]
fn cli_verify_with_base_and_no_scope_reads_the_whole_path() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code("packages/widget/other.rs", "pub fn other() {}\n");

    assert_eq!(
        e2e_verify_cli(&package, &[("--base", base.as_str())]).expect("dispatch should succeed"),
        1,
        "with no --scope, a change anywhere under path owes a decision",
    );
}

#[test]
fn cli_verify_with_extra_scope_fails_on_a_non_excluded_core_change() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() {}\n");
    repo.commit_code("packages/rust/src/cli/main.rs", "pub fn cli() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() { /* v2 */ }\n");

    assert_eq!(
        e2e_verify_cli(
            &package,
            &[
                ("--base", base.as_str()),
                ("--extra-scope", "packages/rust/src"),
                ("--exclude", "packages/rust/src/cli"),
            ],
        )
        .expect("dispatch should succeed"),
        1,
        "a non-excluded change under --extra-scope should fail verify",
    );
}

#[test]
fn cli_verify_with_extra_scope_passes_on_an_excluded_change() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() {}\n");
    repo.commit_code("packages/rust/src/cli/main.rs", "pub fn cli() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code(
        "packages/rust/src/cli/main.rs",
        "pub fn cli() { /* v2 */ }\n",
    );

    assert_eq!(
        e2e_verify_cli(
            &package,
            &[
                ("--base", base.as_str()),
                ("--extra-scope", "packages/rust/src"),
                ("--exclude", "packages/rust/src/cli"),
            ],
        )
        .expect("dispatch should succeed"),
        0,
        "a change only under --exclude should pass verify",
    );
}

// --- #391: a `--scope` (or `--extra-scope`) that resolves to no tracked path is
// rejected loudly instead of silently diffing nothing — a diff over nothing is
// always empty, so a branch that changed real source would pass forever.

#[test]
fn verify_since_errors_on_a_scope_below_path_that_matches_no_tracked_path() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code(
        "packages/widget/src/widget.rs",
        "pub fn widget() { /* v2 */ }\n",
    );

    let bogus = package.join("ghost");
    let err = verify_since(&package, &bogus, Some(&base))
        .expect_err("a --scope matching no tracked path must error, not pass silently");
    assert!(
        err.to_string().contains("scope"),
        "the error should name --scope; got: {err}",
    );
}

#[test]
fn verify_since_errors_on_a_scope_outside_the_repo() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");

    let outside = std::env::temp_dir().join("tc-391-outside-any-repo");
    let err = verify_since(&package, &outside, Some(&base))
        .expect_err("a --scope outside the repo must error");
    assert!(
        err.to_string().contains("scope"),
        "the error should name --scope; got: {err}",
    );
}

#[test]
fn verify_extra_scoped_errors_on_an_extra_root_that_matches_no_tracked_path() {
    // A typo'd `--extra-scope` (the shared core's path misspelled) would silently
    // drop out of the scoped diff, so a core change never owes the binding a
    // decision — it must error instead.
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");

    // `packages/rust/src` is never created — a repo-root-relative root matching
    // no tracked path.
    let extra = [PathBuf::from("packages/rust/src")];
    let err = verify_extra_scoped(&package, &package, Some(&base), &extra, &[])
        .expect_err("an --extra-scope matching no tracked path must error");
    assert!(
        err.to_string().contains("extra-scope"),
        "the error should name --extra-scope; got: {err}",
    );
}

#[test]
fn verify_since_still_fails_for_a_valid_descendant_scope_with_no_receipt() {
    // Guard the other direction: validation must not over-reject. A real
    // descendant scope whose source the branch changed still demands a receipt.
    let repo = TempRepo::new();
    let package = repo.0.join("packages/widget");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/widget/src/widget.rs", "pub fn widget() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code(
        "packages/widget/src/widget.rs",
        "pub fn widget() { /* v2 */ }\n",
    );

    assert_eq!(
        verify_since(&package, &package.join("src"), Some(&base)).unwrap(),
        Verification::Missing,
        "a valid descendant scope with an unanswered change must still fail",
    );
}
