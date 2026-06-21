//! Integration test for diff-scoped Rust mutation — `unit mutation --language rust
//! --base` (#201).
//!
//! With `--base`, only mutants on the `<base>...HEAD` changed lines are tested, via
//! cargo-mutants' own `--in-diff`. Builds a throwaway cargo crate in a git repo (the
//! codebase is the fixture, per the #3 guardrail): a fully-tested baseline, then a
//! commit that adds an assertion-light function. The diff scopes the run to the added
//! function, whose mutants all survive — while the unchanged, well-tested code isn't
//! mutated at all. Requires `git` + `cargo-mutants` (the run builds the crate from
//! scratch, so it's slow).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::mutation::measure_rust;

const CARGO_TOML: &str =
    "[package]\nname = \"tc_mut_base\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n";

/// A baseline whose `add` is fully pinned by its inline test — no survivors.
const BASELINE: &str = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adds() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(10, 1), 11);
    }
}
"#;

/// The change under test: a new `is_positive` whose test runs it but asserts nothing,
/// so every mutant on the added lines survives. `add` is untouched.
const WITH_SURVIVOR: &str = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn is_positive(n: i32) -> bool {
    n > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adds() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(10, 1), 11);
    }
    #[test]
    fn runs_is_positive() {
        let _ = is_positive(1);
    }
}
"#;

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        let repo = Self::bare(slug);
        repo.write("Cargo.toml", CARGO_TOML);
        repo
    }

    /// A git repo with no crate at its root — for placing the crate in a subdirectory.
    fn bare(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-mut-base-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        TempRepo(root)
    }

    fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn commit(&self, message: &str) {
        git(&self.0, &["add", "-A"]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
    }

    fn head(&self) -> String {
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.0)
            .output()
            .expect("git rev-parse should run");
        assert!(out.status.success(), "git rev-parse failed");
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

#[test]
fn base_scopes_the_run_to_the_changed_function() {
    let repo = TempRepo::new("survivor");
    repo.write("src/lib.rs", BASELINE);
    repo.commit("baseline: fully-tested add");
    let base = repo.head();
    repo.write("src/lib.rs", WITH_SURVIVOR);
    repo.commit("add an assertion-light is_positive");

    let survivors = measure_rust(&repo.0, &[], Some(&base)).expect("cargo-mutants runs");
    // The added `is_positive` is in the diff and assertion-light, so its mutants
    // survive; `add` is unchanged, so it's out of scope and never mutated.
    assert!(
        !survivors.is_empty(),
        "the added weak function should leave a survivor on the changed lines"
    );
    assert!(
        survivors
            .iter()
            .all(|s| s.description.contains("is_positive")),
        "only the changed `is_positive` should be mutated; got {survivors:?}"
    );
}

#[test]
fn base_finds_survivors_in_a_subdir_crate() {
    // The crate is a subdirectory of the git repo — the common consumer layout. The
    // diff must be made crate-relative or cargo-mutants' `--in-diff` (which runs in the
    // crate dir) matches nothing; with `--relative` the added weak function's mutants
    // are found.
    let repo = TempRepo::bare("subdir-survivor");
    repo.write("crate/Cargo.toml", CARGO_TOML);
    repo.write("crate/src/lib.rs", BASELINE);
    repo.commit("baseline: fully-tested add in a subdir crate");
    let base = repo.head();
    repo.write("crate/src/lib.rs", WITH_SURVIVOR);
    repo.commit("add an assertion-light is_positive");

    let survivors =
        measure_rust(&repo.0.join("crate"), &[], Some(&base)).expect("cargo-mutants runs");
    assert!(
        !survivors.is_empty()
            && survivors
                .iter()
                .all(|s| s.description.contains("is_positive")),
        "the added weak function in the subdir crate should leave a survivor; got {survivors:?}"
    );
}

#[test]
fn base_with_no_changes_under_the_crate_reports_no_survivors() {
    // A PR that changes nothing under the crate (here, only a top-level note) yields an
    // empty crate-relative diff — nothing to mutate, so no survivors and, crucially, no
    // error (cargo-mutants writes no outcomes for a zero-mutant run).
    let repo = TempRepo::bare("subdir-nochange");
    repo.write("crate/Cargo.toml", CARGO_TOML);
    repo.write("crate/src/lib.rs", WITH_SURVIVOR); // a would-be survivor, left unchanged
    repo.write("notes.md", "before\n");
    repo.commit("baseline");
    let base = repo.head();
    repo.write("notes.md", "before\nafter\n"); // only a non-crate file changes
    repo.commit("tweak a top-level note, not the crate");

    let survivors = measure_rust(&repo.0.join("crate"), &[], Some(&base)).expect("no run needed");
    assert!(
        survivors.is_empty(),
        "nothing under the crate changed, so there are no survivors; got {survivors:?}"
    );
}
