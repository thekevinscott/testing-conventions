//! Integration tests for the branch-keyed e2e receipt contract.
//!
//! `attest` writes one receipt per branch under `e2e-attestations/` — keyed by a
//! sanitized slug of the branch name plus a short hash of the raw name — and
//! prunes the receipts other branches left behind. `verify` asks two content
//! questions of the branch's diff: an untouched scoped source owes nothing, and a
//! changed one is answered by a receipt added or updated in that same diff. No
//! commit SHAs are compared, so one receipt covers the branch.
//!
//! These start red against the single-file, exact-match implementation in
//! `src/e2e.rs` and go green once the receipt contract is implemented.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::e2e::{attest, verify, verify_extra_scoped, verify_since, Verification};

/// Where the branch-keyed receipts live, relative to the package root. Spelled
/// out here rather than imported: the committed path is the public contract.
const RECEIPTS_DIR: &str = "e2e-attestations";

/// A throwaway git repo with one seed commit on branch `base`, removed on drop.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-e2e-receipts-{}-{}",
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
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn seed() {}\n").unwrap();
        std::fs::write(root.join("README.md"), "seed\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-q", "-m", "seed"]);
        // A stable name for the seed tip, whatever the default branch is called.
        git(&root, &["branch", "base"]);
        TempRepo(root)
    }

    /// Check out a new branch off the current HEAD.
    fn branch(&self, name: &str) {
        git(&self.0, &["checkout", "-q", "-b", name]);
    }

    /// Write `contents` to `path`, add, and commit.
    fn commit_file(&self, path: &str, contents: &str, message: &str) {
        let full = self.0.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, contents).unwrap();
        git(&self.0, &["add", path]);
        git(&self.0, &["commit", "-q", "-m", message]);
    }

    /// Commit a fixture receipt for `name` — verify's contract is the file's
    /// location, not who wrote it.
    fn commit_receipt(&self, name: &str) {
        self.commit_file(
            &format!("{RECEIPTS_DIR}/{name}.json"),
            "{\"command\":\"true\",\"ran_at\":0,\"exit_code\":0,\"commit\":\"0\",\"branch\":\"x\"}\n",
            "e2e receipt",
        );
    }

    /// The receipt filenames currently on disk.
    fn receipt_names(&self) -> Vec<String> {
        let dir = self.0.join(RECEIPTS_DIR);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return Vec::new();
        };
        let mut names: Vec<String> = entries
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn head(&self) -> String {
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.0)
            .output()
            .expect("git rev-parse should run");
        assert!(out.status.success());
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

// --- attest: one branch-keyed receipt, pruned siblings ---

#[test]
fn attest_writes_a_branch_keyed_receipt_and_no_single_file() {
    let repo = TempRepo::new();
    repo.branch("feature/one");
    let code_commit = repo.head();

    attest(&repo.0, "true").expect("attest should succeed");

    assert!(
        !repo.0.join("e2e-attestation.json").exists(),
        "the single-file attestation is retired"
    );
    let names = repo.receipt_names();
    assert_eq!(names.len(), 1, "one receipt for the branch, got {names:?}");
    let name = &names[0];
    assert!(
        name.starts_with("feature-one-"),
        "the filename leads with the sanitized branch slug: {name}"
    );
    assert!(name.ends_with(".json"), "a JSON receipt: {name}");
    assert!(
        name.len() >= "feature-one-".len() + 6 + ".json".len(),
        "a hash suffix follows the slug: {name}"
    );

    // The receipt records the branch and the run.
    let receipt: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo.0.join(RECEIPTS_DIR).join(name)).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["branch"], "feature/one");
    assert_eq!(receipt["command"], "true");
    assert_eq!(receipt["exit_code"], 0);
    assert_eq!(receipt["commit"], code_commit.as_str());

    // Committed on top of the attested code commit.
    let new_head = repo.head();
    assert_ne!(new_head, code_commit, "attest commits the receipt");
}

#[test]
fn attest_filenames_are_portable_for_any_branch_name() {
    // Slashes, unicode, and length all sanitize to a portable filename; the raw
    // name lives inside the receipt.
    let repo = TempRepo::new();
    let long = format!("wip/Émil's{}", "x".repeat(300));
    repo.branch(&long);

    attest(&repo.0, "true").expect("attest should succeed");

    let names = repo.receipt_names();
    assert_eq!(names.len(), 1);
    let name = &names[0];
    assert!(
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c)),
        "filename restricted to a portable charset: {name}"
    );
    assert!(name.len() <= 120, "filename bounded: {} chars", name.len());
    let receipt: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo.0.join(RECEIPTS_DIR).join(name)).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["branch"], long.as_str());
}

#[test]
fn attest_keys_case_only_branch_twins_to_distinct_portable_names() {
    // `Foo` and `foo` are distinct branches; their receipts must not collide as
    // paths on a case-insensitive filesystem, so the sanitized slugs share a
    // lowercase prefix and the hash of the raw name separates them.
    let upper = TempRepo::new();
    upper.branch("CaseTwin");
    attest(&upper.0, "true").expect("attest should succeed");
    let upper_name = upper.receipt_names().remove(0);

    let lower = TempRepo::new();
    lower.branch("casetwin");
    attest(&lower.0, "true").expect("attest should succeed");
    let lower_name = lower.receipt_names().remove(0);

    assert_ne!(
        upper_name.to_lowercase(),
        lower_name.to_lowercase(),
        "case-only twins must differ even case-folded: {upper_name} vs {lower_name}"
    );
}

#[test]
fn attest_overwrites_its_own_receipt_in_place() {
    let repo = TempRepo::new();
    repo.branch("feature/one");

    attest(&repo.0, "true").expect("first attest should succeed");
    let first = repo.receipt_names();
    repo.commit_file("src/lib.rs", "pub fn seed2() {}\n", "more code");
    attest(&repo.0, "exit 3").expect("second attest should succeed");
    let second = repo.receipt_names();

    assert_eq!(first, second, "re-attesting rewrites the same file");
    assert_eq!(second.len(), 1);
    let receipt: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo.0.join(RECEIPTS_DIR).join(&second[0])).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["exit_code"], 3, "the receipt records the latest run");
}

#[test]
fn attest_prunes_receipts_other_branches_left_behind() {
    let repo = TempRepo::new();
    repo.commit_receipt("merged-branch-0123abcd45");
    repo.branch("feature/two");

    attest(&repo.0, "true").expect("attest should succeed");

    let names = repo.receipt_names();
    assert_eq!(names.len(), 1, "stale sibling receipts are pruned: {names:?}");
    assert!(names[0].starts_with("feature-two-"));
}

#[test]
fn attest_errors_on_a_detached_head() {
    // The receipt is keyed by branch; with no branch checked out there is
    // nothing to key it by, and the error names the fix.
    let repo = TempRepo::new();
    git(&repo.0, &["checkout", "-q", "--detach"]);
    let result = attest(&repo.0, "true");
    assert!(result.is_err(), "attest on a detached HEAD should error");
}

// --- verify --base: two content questions of the branch's diff ---

#[test]
fn verify_base_passes_when_the_branch_leaves_scoped_source_untouched() {
    let repo = TempRepo::new();
    repo.branch("feature/docs");
    repo.commit_file("README.md", "docs only\n", "docs");
    let scope = repo.0.join("src");
    let result = verify_since(&repo.0, &scope, Some("base")).expect("verify should run");
    assert_eq!(result, Verification::Fresh, "no scoped change owes no decision");
}

#[test]
fn verify_base_requires_a_receipt_when_the_branch_changes_scoped_source() {
    let repo = TempRepo::new();
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");
    let result = verify_since(&repo.0, &repo.0, Some("base")).expect("verify should run");
    assert!(
        !matches!(result, Verification::Fresh),
        "a scoped change with no receipt fails"
    );
}

#[test]
fn verify_base_passes_on_a_receipt_added_by_the_branch() {
    let repo = TempRepo::new();
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");
    repo.commit_receipt("feature-code-abcd012345");
    let result = verify_since(&repo.0, &repo.0, Some("base")).expect("verify should run");
    assert_eq!(result, Verification::Fresh, "the branch's receipt answers the nudge");
}

#[test]
fn verify_base_stays_fresh_after_further_scoped_pushes() {
    // One decision covers the branch: pushing more scoped commits after the
    // receipt does not re-demand it.
    let repo = TempRepo::new();
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");
    repo.commit_receipt("feature-code-abcd012345");
    repo.commit_file("src/lib.rs", "pub fn changed_again() {}\n", "more code");
    let result = verify_since(&repo.0, &repo.0, Some("base")).expect("verify should run");
    assert_eq!(result, Verification::Fresh, "later pushes stay green");
}

#[test]
fn verify_base_passes_on_a_receipt_updated_by_the_branch() {
    // A receipt inherited from the merge base counts only when this branch
    // updates it — and an update is as good as an add.
    let repo = TempRepo::new();
    repo.commit_receipt("feature-code-abcd012345");
    git(&repo.0, &["branch", "-f", "base"]);
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");
    repo.commit_file(
        &format!("{RECEIPTS_DIR}/feature-code-abcd012345.json"),
        "{\"command\":\"true\",\"ran_at\":1,\"exit_code\":0,\"commit\":\"1\",\"branch\":\"x\"}\n",
        "re-attest",
    );
    let result = verify_since(&repo.0, &repo.0, Some("base")).expect("verify should run");
    assert_eq!(result, Verification::Fresh);
}

#[test]
fn verify_base_ignores_a_receipt_inherited_from_the_merge_base() {
    // A receipt that predates the branch answered someone else's nudge.
    let repo = TempRepo::new();
    repo.commit_receipt("earlier-branch-abcd012345");
    git(&repo.0, &["branch", "-f", "base"]);
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");
    let result = verify_since(&repo.0, &repo.0, Some("base")).expect("verify should run");
    assert!(
        !matches!(result, Verification::Fresh),
        "an inherited receipt is not this branch's decision"
    );
}

#[test]
fn verify_base_does_not_count_a_receipt_deletion() {
    // Pruning a merged branch's receipt is hygiene, not a decision.
    let repo = TempRepo::new();
    repo.commit_receipt("merged-branch-abcd012345");
    git(&repo.0, &["branch", "-f", "base"]);
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");
    git(&repo.0, &["rm", "-q", &format!("{RECEIPTS_DIR}/merged-branch-abcd012345.json")]);
    git(&repo.0, &["commit", "-q", "-m", "prune"]);
    let result = verify_since(&repo.0, &repo.0, Some("base")).expect("verify should run");
    assert!(
        !matches!(result, Verification::Fresh),
        "a deletion-only receipt diff does not answer the nudge"
    );
}

#[test]
fn verify_base_receipt_only_branch_passes() {
    // Receipts are not scoped source: a branch that only adds its receipt has
    // changed nothing that owes a decision.
    let repo = TempRepo::new();
    repo.branch("feature/attest-only");
    repo.commit_receipt("feature-attest-only-abcd012345");
    let result = verify_since(&repo.0, &repo.0, Some("base")).expect("verify should run");
    assert_eq!(result, Verification::Fresh);
}

#[test]
fn verify_base_ignores_the_legacy_single_file_attestation() {
    // The retired `e2e-attestation.json` is neither a receipt nor scoped source:
    // it never answers the nudge, and deleting it owes nothing.
    let repo = TempRepo::new();
    repo.branch("feature/code");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code");
    let head = repo.head();
    repo.commit_file(
        "e2e-attestation.json",
        &format!("{{\"command\":\"true\",\"ran_at\":0,\"exit_code\":0,\"commit\":\"{head}\"}}\n"),
        "legacy attest",
    );
    let result = verify_since(&repo.0, &repo.0, Some("base")).expect("verify should run");
    assert!(
        !matches!(result, Verification::Fresh),
        "the legacy single file no longer answers the nudge"
    );

    let deleter = TempRepo::new();
    deleter.commit_file("e2e-attestation.json", "{}\n", "legacy attest");
    git(&deleter.0, &["branch", "-f", "base"]);
    deleter.branch("chore/drop-legacy");
    git(&deleter.0, &["rm", "-q", "e2e-attestation.json"]);
    git(&deleter.0, &["commit", "-q", "-m", "drop legacy attestation"]);
    let result = verify_since(&deleter.0, &deleter.0, Some("base")).expect("verify should run");
    assert_eq!(result, Verification::Fresh, "deleting the legacy file owes nothing");
}

#[test]
fn verify_base_extra_scope_change_owes_a_decision_answered_by_a_receipt() {
    // A shared tree beside the package joins the scoped diff (#333): a change
    // there puts the question to the branch, and the branch's receipt answers it.
    let repo = TempRepo::new();
    repo.commit_file("core/src/lib.rs", "pub fn core() {}\n", "core seed");
    std::fs::create_dir_all(repo.0.join("packages/binding")).unwrap();
    repo.commit_file("packages/binding/manifest.toml", "name = \"binding\"\n", "binding seed");
    git(&repo.0, &["branch", "-f", "base"]);
    repo.branch("feature/core");
    repo.commit_file("core/src/lib.rs", "pub fn core_changed() {}\n", "core change");

    let package = repo.0.join("packages/binding");
    let extra = vec![PathBuf::from("core/src")];
    let result = verify_extra_scoped(&package, &package, Some("base"), &extra, &[])
        .expect("verify should run");
    assert!(
        !matches!(result, Verification::Fresh),
        "a shared-tree change owes the binding a decision"
    );

    repo.commit_file(
        &format!("packages/binding/{RECEIPTS_DIR}/feature-core-abcd012345.json"),
        "{\"command\":\"true\",\"ran_at\":0,\"exit_code\":0,\"commit\":\"0\",\"branch\":\"x\"}\n",
        "binding receipt",
    );
    let result = verify_extra_scoped(&package, &package, Some("base"), &extra, &[])
        .expect("verify should run");
    assert_eq!(result, Verification::Fresh, "the binding's receipt answers it");
}

// --- verify without --base: receipt presence ---

#[test]
fn verify_without_base_passes_on_a_committed_receipt() {
    // With no branch diff to read, presence is the check — later code commits
    // do not stale a receipt.
    let repo = TempRepo::new();
    repo.commit_receipt("some-branch-abcd012345");
    repo.commit_file("src/lib.rs", "pub fn changed() {}\n", "code after receipt");
    let result = verify(&repo.0).expect("verify should run");
    assert_eq!(result, Verification::Fresh);
}

#[test]
fn verify_without_base_missing_when_no_receipts() {
    let repo = TempRepo::new();
    let result = verify(&repo.0).expect("verify should run");
    assert_eq!(result, Verification::Missing);
}
