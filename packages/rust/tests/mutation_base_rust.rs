mod common;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use common::expect_tested;
use testing_conventions::mutation::{measure_rust, Measurement};

const CARGO_TOML: &str =
    "[package]\nname = \"tc_mut_base\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n";

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

    let (count, survivors) = expect_tested(
        measure_rust(
            &repo.0,
            &[],
            &std::collections::BTreeMap::new(),
            Some(&base),
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        count >= survivors.len(),
        "every survivor was judged, so the count covers them"
    );
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
    let repo = TempRepo::bare("subdir-survivor");
    repo.write("crate/Cargo.toml", CARGO_TOML);
    repo.write("crate/src/lib.rs", BASELINE);
    repo.commit("baseline: fully-tested add in a subdir crate");
    let base = repo.head();
    repo.write("crate/src/lib.rs", WITH_SURVIVOR);
    repo.commit("add an assertion-light is_positive");

    let (_, survivors) = expect_tested(
        measure_rust(
            &repo.0.join("crate"),
            &[],
            &std::collections::BTreeMap::new(),
            Some(&base),
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        !survivors.is_empty()
            && survivors
                .iter()
                .all(|s| s.description.contains("is_positive")),
        "the added weak function in the subdir crate should leave a survivor; got {survivors:?}"
    );
}

const MEMBER_CARGO_TOML: &str =
    "[package]\nname = \"tc_mut_member\"\nversion = \"0.0.0\"\nedition = \"2021\"\n";

#[test]
fn base_finds_survivors_in_a_workspace_member_crate() {
    let repo = TempRepo::bare("workspace-member");
    repo.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"member\"]\nresolver = \"2\"\n",
    );
    repo.write("member/Cargo.toml", MEMBER_CARGO_TOML);
    repo.write("member/src/lib.rs", BASELINE);
    repo.commit("baseline: fully-tested add in a workspace member");
    let base = repo.head();
    repo.write("member/src/lib.rs", WITH_SURVIVOR);
    repo.commit("add an assertion-light is_positive");

    let (_, survivors) = expect_tested(
        measure_rust(
            &repo.0.join("member"),
            &[],
            &std::collections::BTreeMap::new(),
            Some(&base),
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        !survivors.is_empty()
            && survivors
                .iter()
                .all(|s| s.description.contains("is_positive")),
        "the added weak function in the workspace member should leave a survivor; got {survivors:?}"
    );
    assert!(
        survivors.iter().all(|s| s.file == "src/lib.rs"),
        "survivor paths are scan-path-relative, not workspace-relative; got {survivors:?}"
    );
}

#[test]
fn base_with_only_non_source_crate_changes_reports_the_engine_not_run() {
    let repo = TempRepo::new("readme-only");
    repo.write("src/lib.rs", WITH_SURVIVOR);
    repo.write("README.md", "before\n");
    repo.commit("baseline");
    let base = repo.head();
    repo.write("README.md", "before\nafter\n");
    repo.commit("edit the crate README, not its source");

    let measurement = measure_rust(
        &repo.0,
        &[],
        &std::collections::BTreeMap::new(),
        Some(&base),
        &[],
    )
    .expect("no run needed");
    assert_eq!(
        measurement,
        Measurement::EngineNotRun,
        "no Rust source changed, so the engine never ran"
    );
}

#[test]
fn base_with_no_changes_under_the_crate_reports_the_engine_not_run() {
    let repo = TempRepo::bare("subdir-nochange");
    repo.write("crate/Cargo.toml", CARGO_TOML);
    repo.write("crate/src/lib.rs", WITH_SURVIVOR);
    repo.write("notes.md", "before\n");
    repo.commit("baseline");
    let base = repo.head();
    repo.write("notes.md", "before\nafter\n");
    repo.commit("tweak a top-level note, not the crate");

    let measurement = measure_rust(
        &repo.0.join("crate"),
        &[],
        &std::collections::BTreeMap::new(),
        Some(&base),
        &[],
    )
    .expect("no run needed");
    assert_eq!(
        measurement,
        Measurement::EngineNotRun,
        "nothing under the crate changed, so the engine never ran"
    );
}
