//! Integration tests for the commit-scoped `co-change` check (#33).
//!
//! When a source file is **modified** (e.g. a function removed) or **deleted** in
//! a `<base>...HEAD` diff, its colocated test (the #15/#18 pairing — `foo.py` →
//! `foo_test.py`, `foo.ts` → `foo.test.ts`) must change in the same diff;
//! `stale_sources` returns the sources whose test went stale, and the
//! `unit co-change` subcommand turns a non-empty result into a non-zero exit.
//! *Added* source files are not subjects (new code is the coverage floor's job),
//! a test file is never a subject, an empty/comment-only file holds no logic, and
//! a `co-change`-exempt source needn't co-change.
//!
//! Each test builds a throwaway git repo (per the #3 guardrail: red cases — a
//! changed source with no test change — and clean cases). These start red against
//! the stub in `src/co_change.rs` and go green once detection is implemented.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::co_change::stale_sources;
use testing_conventions::colocated_test::Language;
use testing_conventions::run;

/// A throwaway git repo, removed on drop. Starts with no commits; a test writes
/// a baseline, `commit`s it, captures `head()` as the `base`, then mutates and
/// commits the "after" so `<base>...HEAD` is the change under test.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-co-change-{}-{}-{}",
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

    /// Write `contents` to `rel`, creating parent directories.
    fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    /// Delete `rel` from the working tree.
    fn remove(&self, rel: &str) {
        std::fs::remove_file(self.0.join(rel)).unwrap();
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

/// The stale sources reported for `<base>...HEAD` (no exemptions), as `/`-joined
/// relative paths.
fn stale(repo: &TempRepo, base: &str, language: Language) -> Vec<String> {
    stale_sources(&repo.0, base, language, &BTreeSet::new())
        .expect("diffing a readable repo should succeed")
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect()
}

/// Result of `unit co-change <repo> --language <lang> --base <base> [--config
/// <repo>/<config>]`, run in-process.
fn run_co_change(
    repo: &TempRepo,
    language: &str,
    base: &str,
    config: Option<&str>,
) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "co-change".into(),
        repo.0.clone().into_os_string(),
        "--language".into(),
        language.into(),
        "--base".into(),
        base.into(),
    ];
    if let Some(name) = config {
        argv.push("--config".into());
        argv.push(repo.0.join(name).into_os_string());
    }
    run(argv)
}

/// Raw CLI invocation (after the program name), so a clap usage error can be
/// asserted rather than unwrapped away.
fn run_cli(args: &[&str]) -> anyhow::Result<i32> {
    let argv: Vec<OsString> = std::iter::once(OsString::from("testing-conventions"))
        .chain(args.iter().copied().map(OsString::from))
        .collect();
    run(argv)
}

const WIDGET_PY: &str = "def widget():\n    return 1\n";
const WIDGET_PY_TEST: &str =
    "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 1\n";

// ---- Python (#15 pairing) ------------------------------------------------

#[test]
fn python_modified_source_without_its_test_is_stale() {
    // The core red case: widget.py changes, widget_test.py does not.
    let repo = TempRepo::new("py-mod");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.commit("edit the source only");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["widget.py"]);
}

#[test]
fn python_modified_source_with_its_test_is_clean() {
    // Changing both source and its colocated test is exactly what the rule wants.
    let repo = TempRepo::new("py-mod-clean");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 2\n",
    );
    repo.commit("edit both");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_nested_source_is_reported_with_its_relative_path() {
    let repo = TempRepo::new("py-nested");
    repo.write("pkg/helper.py", "def helper():\n    return 1\n");
    repo.write(
        "pkg/helper_test.py",
        "def test_helper():\n    assert True\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write("pkg/helper.py", "def helper():\n    return 2\n");
    repo.commit("edit nested source only");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["pkg/helper.py"]);
}

#[test]
fn python_deleted_source_without_deleting_its_test_is_stale() {
    // A removal that leaves the test behind — the stale orphan this rule targets.
    let repo = TempRepo::new("py-del");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.remove("widget.py");
    repo.commit("delete the source only");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["widget.py"]);
}

#[test]
fn python_deleting_source_and_test_together_is_clean() {
    let repo = TempRepo::new("py-del-both");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.remove("widget.py");
    repo.remove("widget_test.py");
    repo.commit("delete both");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_added_source_is_not_a_subject() {
    // Brand-new code is the coverage floor's concern, not co-change's; a new
    // source with no colocated test is not flagged here.
    let repo = TempRepo::new("py-add");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("fresh.py", "def fresh():\n    return 9\n");
    repo.commit("add a brand-new source");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_modifying_only_the_test_is_allowed() {
    // A test file is never a co-change subject — tightening a test on its own is fine.
    let repo = TempRepo::new("py-test-only");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 1\n    assert widget() != 0\n",
    );
    repo.commit("strengthen the test only");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_modified_empty_file_is_not_a_subject() {
    // An empty / comment-only file carries no logic, so editing it needs no test
    // co-change — consistent with the colocated-test rule (#32).
    let repo = TempRepo::new("py-empty");
    repo.write("pkg/__init__.py", "");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("pkg/__init__.py", "# a comment, still no code\n");
    repo.commit("touch the empty package init");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_conftest_is_not_a_subject() {
    // conftest.py is pytest support, never a colocated-test subject (#112).
    let repo = TempRepo::new("py-conftest");
    repo.write("conftest.py", "import pytest\n");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "conftest.py",
        "import pytest\n\n# a new fixture is coming\n",
    );
    repo.commit("edit conftest only");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_subcommand_exits_nonzero_when_a_source_is_stale() {
    let repo = TempRepo::new("py-cli-red");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.commit("edit the source only");

    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 1);
}

#[test]
fn python_subcommand_exits_zero_when_every_change_co_changes() {
    let repo = TempRepo::new("py-cli-clean");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 2\n",
    );
    repo.commit("edit both");

    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 0);
}

// ---- Exemptions (#32 machinery, rule `co-change`) ------------------------

#[test]
fn python_a_co_change_exemption_lifts_a_stale_source() {
    let repo = TempRepo::new("py-exempt");
    repo.write(
        "testing-conventions.toml",
        "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"co-change\"]\n\
         reason = \"thin launcher; no logic to retest on each edit\"\n",
    );
    repo.write("cli.py", "def main():\n    return 0\n");
    repo.commit("base");
    let base = repo.head();

    repo.write("cli.py", "def main():\n    return 1\n");
    repo.commit("edit the launcher, no test");

    // Stale with no config…
    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 1);
    // …and lifted by the `co-change` exemption.
    assert_eq!(
        run_co_change(&repo, "python", &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

#[test]
fn a_stale_exempt_entry_is_an_error() {
    // The exempt list can't silently rot: a path that names no file is rejected.
    let repo = TempRepo::new("py-stale-exempt");
    repo.write(
        "testing-conventions.toml",
        "[[python.exempt]]\npath = \"ghost.py\"\nrules = [\"co-change\"]\nreason = \"gone\"\n",
    );
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.commit("edit source only");

    assert!(run_co_change(&repo, "python", &base, Some("testing-conventions.toml")).is_err());
}

// ---- TypeScript (#18 pairing) --------------------------------------------

#[test]
fn typescript_modified_source_without_its_test_is_stale() {
    let repo = TempRepo::new("ts-mod");
    repo.write("widget.ts", "export const widget = () => 1;\n");
    repo.write(
        "widget.test.ts",
        "import { widget } from './widget';\nit('works', () => expect(widget()).toBe(1));\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.ts", "export const widget = () => 2;\n");
    repo.commit("edit the source only");

    assert_eq!(stale(&repo, &base, Language::TypeScript), vec!["widget.ts"]);
}

#[test]
fn typescript_modified_source_with_its_test_is_clean() {
    let repo = TempRepo::new("ts-mod-clean");
    repo.write("widget.ts", "export const widget = () => 1;\n");
    repo.write(
        "widget.test.ts",
        "import { widget } from './widget';\nit('works', () => expect(widget()).toBe(1));\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.ts", "export const widget = () => 2;\n");
    repo.write(
        "widget.test.ts",
        "import { widget } from './widget';\nit('works', () => expect(widget()).toBe(2));\n",
    );
    repo.commit("edit both");

    assert!(stale(&repo, &base, Language::TypeScript).is_empty());
}

// ---- CLI surface & errors ------------------------------------------------

#[test]
fn an_unknown_base_ref_is_an_error() {
    // A base that can't be resolved must surface, never silently pass as "clean".
    let repo = TempRepo::new("bad-base");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");

    assert!(
        stale_sources(&repo.0, "no-such-ref", Language::Python, &BTreeSet::new()).is_err(),
        "an unresolvable base ref must error"
    );
}

#[test]
fn co_change_requires_language() {
    let err = run_cli(&["unit", "co-change", "/tmp", "--base", "HEAD"])
        .expect_err("--language is required");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("a missing required flag should surface as a clap::Error");
    assert_eq!(
        clap_err.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn co_change_requires_base() {
    let err = run_cli(&["unit", "co-change", "/tmp", "--language", "python"])
        .expect_err("--base is required");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("a missing required flag should surface as a clap::Error");
    assert_eq!(
        clap_err.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn co_change_rejects_rust() {
    // Rust units are inline `#[cfg(test)]` in the same file, so a sibling test
    // can't go stale — the command rejects `--language rust`.
    let repo = TempRepo::new("rust-reject");
    repo.write("lib.rs", "pub fn f() {}\n");
    repo.commit("base");
    let base = repo.head();

    let err = run_co_change(&repo, "rust", &base, None).unwrap_err();
    assert!(err.to_string().contains("inline"), "got: {err}");
}
