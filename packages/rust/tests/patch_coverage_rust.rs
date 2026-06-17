//! Integration tests for patch (changed-line) coverage (Rust — #136, parent #46).
//!
//! The Rust twin of `patch_coverage.rs`: every line a `<base>...HEAD` diff touches
//! must be covered by the `cargo llvm-cov` unit suite. Each test builds a throwaway
//! cargo crate in a git repo (per the #3 guardrail the codebases are the fixtures —
//! red cases where a changed line is left uncovered, clean cases where it isn't),
//! runs REAL `cargo llvm-cov` over it via the SDK (`patch_coverage::check_rust`) and
//! the CLI (`run`), and asserts the uncovered lines / exit code. Requires `git` and
//! `cargo-llvm-cov` on PATH.
//!
//! Opens at RED per AGENTS.md: detection is stubbed (`check_rust` reports nothing),
//! so a repo whose change leaves a line uncovered still comes back clean. The
//! implementation — diffing `<base>...HEAD` and intersecting the changed lines with
//! `cargo llvm-cov`'s per-line coverage — follows once CI witnesses these red.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::patch_coverage::check_rust;
use testing_conventions::run;

/// A throwaway cargo crate in a git repo, removed on drop. `new` lays down the
/// `Cargo.toml`; a test writes the baseline source + its inline test, `commit`s it,
/// captures `head()` as the `base`, then mutates and commits the "after" so
/// `<base>...HEAD` is the change under test. The crate carries its own `[workspace]`
/// so `cargo llvm-cov` measures it in isolation.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-patch-cov-rust-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        let repo = TempRepo(root);
        repo.write("Cargo.toml", CARGO_TOML);
        repo
    }

    /// Write `contents` to `rel`, creating parent directories.
    fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    /// Stage everything and commit, advancing HEAD.
    fn commit(&self, message: &str) {
        git(&self.0, &["add", "-A"]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
    }

    /// The current HEAD SHA — captured as the `base` before mutating.
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

/// The uncovered changed lines for `<base>...HEAD` (no exemptions), as
/// `<file>:<line>` strings.
fn uncovered(repo: &TempRepo, base: &str) -> Vec<String> {
    check_rust(&repo.0, base, &[])
        .expect("checking a readable repo should succeed")
        .iter()
        .map(|u| format!("{}:{}", u.file, u.line))
        .collect()
}

/// Result of `unit patch-coverage <repo> --language rust --base <base> [--config
/// <repo>/<config>]`, run in-process.
fn run_patch_coverage(repo: &TempRepo, base: &str, config: Option<&str>) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "patch-coverage".into(),
        repo.0.clone().into_os_string(),
        "--language".into(),
        "rust".into(),
        "--base".into(),
        base.into(),
    ];
    if let Some(name) = config {
        argv.push("--config".into());
        argv.push(repo.0.join(name).into_os_string());
    }
    run(argv)
}

const CARGO_TOML: &str =
    "[package]\nname = \"tc_patch_rust\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n";

/// Fully covered by the inline test at baseline (both arms exercised).
const WIDGET_RS: &str = r#"pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "pos"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "pos");
        assert_eq!(widget(-1), "neg");
    }
}
"#;

/// Adds an `n == -42` arm the baseline test never exercises. The diff adds new
/// lines 4-5; line 5 (`"answer"`) is the body llvm-cov never runs, so it is the
/// uncovered changed line (line 4's condition is still evaluated, so it's covered).
const WIDGET_RS_UNCOVERED: &str = r#"pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "pos"
    } else if n == -42 {
        "answer"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "pos");
        assert_eq!(widget(-1), "neg");
    }
}
"#;

/// Rewords a covered line (`"pos"` → `"positive"`) and updates its test — the
/// change stays fully covered.
const WIDGET_RS_COVERED_EDIT: &str = r#"pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "positive"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "positive");
        assert_eq!(widget(-1), "neg");
    }
}
"#;

// ---- Detection via the SDK (`check_rust`) --------------------------------

#[test]
fn rust_uncovered_changed_line_is_reported() {
    // The core red case: the change adds an arm whose body the suite never runs,
    // so the new line comes back uncovered.
    let repo = TempRepo::new("uncovered");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    let base = repo.head();

    repo.write("src/lib.rs", WIDGET_RS_UNCOVERED);
    repo.commit("add an untested arm");

    let reported = uncovered(&repo, &base);
    assert!(
        reported.contains(&"src/lib.rs:5".to_string()),
        "the uncovered new line should be reported; got: {reported:?}"
    );
}

#[test]
fn rust_covered_change_is_clean() {
    // Editing a line the suite already exercises keeps the change fully covered.
    let repo = TempRepo::new("covered");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    let base = repo.head();

    repo.write("src/lib.rs", WIDGET_RS_COVERED_EDIT);
    repo.commit("reword a covered line and update its test");

    assert!(
        uncovered(&repo, &base).is_empty(),
        "a change to a covered line is clean"
    );
}

#[test]
fn a_change_touching_no_rust_is_clean() {
    // A diff with no `.rs` source returns clean immediately — there's no changed
    // line to measure, so the suite isn't even run.
    let repo = TempRepo::new("no-rs");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.write("README.md", "# project\n");
    repo.commit("base");
    let base = repo.head();

    repo.write("README.md", "# project\n\nnow with docs\n");
    repo.commit("docs only");

    assert!(
        uncovered(&repo, &base).is_empty(),
        "a change that touches no Rust source is clean"
    );
}

#[test]
fn rust_added_untested_file_changed_lines_are_uncovered() {
    // Unlike co-change (#33), an *added* file's new lines ARE patch-coverage
    // subjects: a brand-new module no test exercises is wholly uncovered.
    let repo = TempRepo::new("added");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    let base = repo.head();

    repo.write("src/lib.rs", &format!("{WIDGET_RS}pub mod extra;\n"));
    repo.write("src/extra.rs", "pub fn extra() -> i64 {\n    41\n}\n");
    repo.commit("add a brand-new untested module");

    let reported = uncovered(&repo, &base);
    assert!(
        reported.iter().any(|r| r.starts_with("src/extra.rs:")),
        "the added file's uncovered lines should be reported; got: {reported:?}"
    );
}

#[test]
fn rust_changed_comment_line_is_not_a_subject() {
    // A comment isn't an executable line, so coverage has nothing to measure on it
    // — a changed comment line is never flagged.
    let repo = TempRepo::new("comment");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "src/lib.rs",
        r#"// classify the sign of n
pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "pos"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "pos");
        assert_eq!(widget(-1), "neg");
    }
}
"#,
    );
    repo.commit("add an explanatory comment");

    assert!(
        uncovered(&repo, &base).is_empty(),
        "a changed comment line has nothing to cover"
    );
}

#[test]
fn an_unknown_base_ref_is_an_error() {
    // A base that can't be resolved must surface, never silently pass as "clean".
    let repo = TempRepo::new("bad-base");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");

    assert!(
        check_rust(&repo.0, "no-such-ref", &[]).is_err(),
        "an unresolvable base ref must error"
    );
}

// ---- Exit codes via the CLI (`run`) --------------------------------------

#[test]
fn rust_subcommand_exits_nonzero_on_an_uncovered_changed_line() {
    let repo = TempRepo::new("cli-red");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    let base = repo.head();
    repo.write("src/lib.rs", WIDGET_RS_UNCOVERED);
    repo.commit("add an untested arm");

    assert_eq!(run_patch_coverage(&repo, &base, None).unwrap(), 1);
}

#[test]
fn rust_subcommand_exits_zero_when_the_change_is_covered() {
    let repo = TempRepo::new("cli-clean");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    let base = repo.head();
    repo.write("src/lib.rs", WIDGET_RS_COVERED_EDIT);
    repo.commit("reword a covered line and update its test");

    assert_eq!(run_patch_coverage(&repo, &base, None).unwrap(), 0);
}

// ---- Exemptions (#32 machinery, rule `coverage`) -------------------------

#[test]
fn rust_a_coverage_exemption_lifts_an_uncovered_changed_line() {
    // A `coverage` exemption drops a file from the run, so its changed lines have
    // nothing to cover — the same waiver the floor (#37) honors.
    let repo = TempRepo::new("exempt");
    repo.write(
        "testing-conventions.toml",
        "[[rust.exempt]]\npath = \"src/shim.rs\"\nrules = [\"coverage\"]\n\
         reason = \"thin launcher; logic lives in tested modules\"\n",
    );
    repo.write("src/lib.rs", &format!("{WIDGET_RS}pub mod shim;\n"));
    repo.write("src/shim.rs", "pub fn shim() -> i64 {\n    0\n}\n");
    repo.commit("base");
    let base = repo.head();

    repo.write("src/shim.rs", "pub fn shim() -> i64 {\n    1\n}\n");
    repo.commit("edit the untested launcher");

    // Flagged with no config…
    assert_eq!(run_patch_coverage(&repo, &base, None).unwrap(), 1);
    // …and lifted by the `coverage` exemption.
    assert_eq!(
        run_patch_coverage(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}
