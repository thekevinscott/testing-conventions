use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const RECEIPTS_DIR: &str = "e2e-attestations";

struct TempRepo(PathBuf);

impl TempRepo {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-e2e-receipts-e2e-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        git(&root, &["config", "commit.gpgsign", "false"]);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn seed() {}\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-q", "-m", "seed"]);
        git(&root, &["branch", "base"]);
        TempRepo(root)
    }

    fn branch(&self, name: &str) {
        git(&self.0, &["checkout", "-q", "-b", name]);
    }

    fn commit_file(&self, path: &str, contents: &str, message: &str) {
        let full = self.0.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, contents).unwrap();
        git(&self.0, &["add", path]);
        git(&self.0, &["commit", "-q", "-m", message]);
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

fn run(repo: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(args)
        .current_dir(repo)
        .output()
        .expect("the built binary should run");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code().expect("an exit code"), text)
}

/// `e2e slug` is read by command substitution, so its contract is stdout alone —
/// merging stderr in would pin diagnostics the substitution never sees.
fn run_stdout(repo: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(args)
        .current_dir(repo)
        .output()
        .expect("the built binary should run");
    (
        out.status.code().expect("an exit code"),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn attest_writes_the_branch_receipt_and_exits_zero() {
    let repo = TempRepo::new();
    repo.branch("feature/one");

    let (code, _) = run(&repo.0, &["e2e", "attest", "true"]);

    assert_eq!(code, 0, "attest force-runs and exits 0");
    assert!(
        !repo.0.join("e2e-attestation.json").exists(),
        "the single-file attestation is retired"
    );
    let receipts: Vec<_> = std::fs::read_dir(repo.0.join(RECEIPTS_DIR))
        .expect("the receipts directory should exist")
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        receipts.len(),
        1,
        "one receipt for the branch: {receipts:?}"
    );
    assert_eq!(
        receipts[0], "feature-one.json",
        "branch-keyed: {receipts:?}"
    );
}

#[test]
fn verify_base_passes_a_branch_whose_diff_carries_a_receipt() {
    let repo = TempRepo::new();
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");
    repo.commit_file(
        &format!("{RECEIPTS_DIR}/feature-code-abcd012345.json"),
        "{\"command\":\"true\",\"ran_at\":0,\"exit_code\":0,\"commit\":\"0\",\"branch\":\"x\"}\n",
        "receipt",
    );

    let (code, text) = run(&repo.0, &["e2e", "verify", "--base", "base"]);
    assert_eq!(code, 0, "a receipt in the branch diff passes: {text}");
}

#[test]
fn attest_then_later_pushes_stay_green() {
    let repo = TempRepo::new();
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");

    let (code, _) = run(&repo.0, &["e2e", "attest", "true"]);
    assert_eq!(code, 0);
    repo.commit_file("src/lib.rs", "pub fn changed_again() {}\n", "more code");

    let (code, text) = run(&repo.0, &["e2e", "verify", "--base", "base"]);
    assert_eq!(code, 0, "later pushes stay green: {text}");
}

#[test]
fn slug_prints_the_standardized_name_for_an_argument() {
    let repo = TempRepo::new();
    let (code, text) = run_stdout(&repo.0, &["e2e", "slug", "Feature/One"]);
    assert_eq!(code, 0);
    assert_eq!(text.trim(), "feature-one");
}

#[test]
fn slug_defaults_to_the_checked_out_branch() {
    let repo = TempRepo::new();
    repo.branch("feature/two");
    let (code, text) = run_stdout(&repo.0, &["e2e", "slug"]);
    assert_eq!(code, 0);
    assert_eq!(text.trim(), "feature-two");
}

#[test]
fn slug_with_no_argument_errors_on_a_detached_head() {
    let repo = TempRepo::new();
    git(&repo.0, &["checkout", "-q", "--detach"]);
    let (code, text) = run(&repo.0, &["e2e", "slug"]);
    assert_ne!(code, 0, "no branch to standardize: {text}");
}

#[test]
fn verify_base_fails_a_scoped_change_with_no_receipt_naming_attest() {
    let repo = TempRepo::new();
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");

    let (code, text) = run(&repo.0, &["e2e", "verify", "--base", "base"]);
    assert_ne!(code, 0, "a scoped change with no receipt fails");
    assert!(
        text.contains("e2e attest"),
        "the failure names the fix: {text}"
    );
}
