//! Integration tests for the commit-scoped `co-change` check (#33), folded into
//! `unit colocated-test --base` (#161).
//!
//! When a source file is **modified** (e.g. a function removed) or **deleted** in
//! a `<base>...HEAD` diff, its colocated test (the #15/#18 pairing — `foo.py` →
//! `foo_test.py`, `foo.ts` → `foo.test.ts`) must change in the same diff;
//! `stale_sources` returns the sources whose test went stale, and
//! `unit colocated-test --base` turns a non-empty result into a non-zero exit on
//! top of the tree-wide presence check. *Added* source files are not subjects
//! (new code is the coverage floor's job), a test file is never a subject, an
//! empty/comment-only file holds no logic, and a `co-change`-exempt source needn't
//! co-change.
//!
//! Each test builds a throwaway git repo (per the #3 guardrail: red cases — a
//! changed source with no test change — and clean cases).

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

/// Result of `unit colocated-test <repo> --language <lang> --base <base> [--config
/// <repo>/<config>]`, run in-process. Since #161 the commit-scoped co-change check
/// rides on `colocated-test`'s opt-in `--base` flag (presence + co-change), so
/// these cases drive that command.
fn run_co_change(
    repo: &TempRepo,
    language: &str,
    base: &str,
    config: Option<&str>,
) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "colocated-test".into(),
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

/// Result of `unit colocated-test <repo> --language <lang>` with **no** `--base`:
/// the presence-only scope. `--base` is opt-in (#161), so this ignores a
/// stale-but-present source that the `--base` form flags.
fn run_colocated_presence(repo: &TempRepo, language: &str) -> anyhow::Result<i32> {
    run([
        OsString::from("testing-conventions"),
        "unit".into(),
        "colocated-test".into(),
        repo.0.clone().into_os_string(),
        "--language".into(),
        language.into(),
    ])
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
    // cli.py has its colocated test (so presence is satisfied), but a change edits
    // cli.py and leaves cli_test.py — stale under `--base`, unless waived.
    let repo = TempRepo::new("py-exempt");
    repo.write(
        "testing-conventions.toml",
        "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"co-change\"]\n\
         reason = \"thin launcher; no logic to retest on each edit\"\n",
    );
    repo.write("cli.py", "def main():\n    return 0\n");
    repo.write(
        "cli_test.py",
        "from cli import main\n\n\ndef test_main():\n    assert main() == 0\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write("cli.py", "def main():\n    return 1\n");
    repo.commit("edit the launcher, leave its test");

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
fn co_change_rejects_rust() {
    // Rust units are inline `#[cfg(test)]` in the same file, so a sibling test
    // can't go stale — `--base --language rust` is rejected (presence without
    // `--base` still supports Rust).
    let repo = TempRepo::new("rust-reject");
    repo.write("lib.rs", "pub fn f() {}\n");
    repo.commit("base");
    let base = repo.head();

    let err = run_co_change(&repo, "rust", &base, None).unwrap_err();
    assert!(err.to_string().contains("inline"), "got: {err}");
}

// ---- `--base` folds co-change into colocated-test (#161) -----------------

#[test]
fn base_adds_co_change_on_top_of_presence() {
    // The defining merge: with every source paired on disk, editing a source and
    // leaving its test is stale under `--base` (the co-change scope) — yet clean
    // without it, since presence alone sees both files still present.
    let repo = TempRepo::new("base-additive");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.commit("edit the source only");

    // Stale under `--base`…
    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 1);
    // …but presence-only (no `--base`) passes: the test still exists on disk.
    assert_eq!(run_colocated_presence(&repo, "python").unwrap(), 0);
}

#[test]
fn base_still_enforces_tree_wide_presence() {
    // `--base` *adds* co-change; it doesn't drop presence. An orphan source (no
    // colocated test at all) is flagged even when the diff co-changes cleanly —
    // you can't slip an orphan past `--base` by leaving it untouched.
    let repo = TempRepo::new("base-presence");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.write("orphan.py", "def orphan():\n    return 9\n");
    repo.commit("base");
    let base = repo.head();
    // A co-change-clean edit: the source and its test move together.
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 2\n",
    );
    repo.commit("edit widget and its test together");

    // Co-change is clean, but presence still flags the untouched orphan.
    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 1);
}
