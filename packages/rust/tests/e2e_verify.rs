use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::e2e::{
    attest, verify, verify_extra_scoped, verify_scoped, verify_since, Verification,
};
use testing_conventions::run;

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
    attest(&package, "true").expect("attest should succeed");
    assert_eq!(
        verify(&package).expect("verify should succeed"),
        Verification::Fresh,
    );
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

#[test]
fn verify_scoped_with_scope_equal_to_repo_matches_verify() {
    let repo = TempRepo::new();
    attest(&repo.0, "true").expect("attest should succeed");
    assert_eq!(
        verify_scoped(&repo.0, &repo.0).expect("verify should succeed"),
        verify(&repo.0).expect("verify should succeed"),
    );
}

#[test]
fn verify_extra_scoped_with_no_extra_roots_matches_verify_since() {
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

#[test]
fn verify_since_passes_when_the_branch_left_the_scoped_source_untouched() {
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

#[test]
fn verify_extra_scoped_flags_a_change_under_an_extra_root() {
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");
    repo.commit_code("packages/rust/src/core.rs", "pub fn core() { /* v2 */ }\n");

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
    let argv: Vec<OsString> = vec!["testing-conventions".into(), "e2e".into(), "verify".into()];
    let code = run(argv).expect("`e2e verify` with no argument should still dispatch");
    assert_eq!(code, 1);
}

#[test]
fn cli_verify_with_base_and_scope_ignores_a_change_outside_the_scope() {
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
    let repo = TempRepo::new();
    let package = repo.0.join("packages/python");
    std::fs::create_dir_all(package.join("src")).unwrap();
    repo.commit_code("packages/python/src/lib.rs", "pub fn binding() {}\n");
    attest(&package, "true").expect("attest should succeed");
    let base = rev_parse(&repo.0, "HEAD");

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
