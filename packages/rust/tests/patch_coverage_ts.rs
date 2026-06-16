//! Integration tests for patch (changed-line) coverage (TypeScript — #135, parent #46).
//!
//! The TypeScript twin of `patch_coverage.rs`: every line a `<base>...HEAD` diff
//! touches must be covered by the vitest unit suite. Each test builds a throwaway
//! git repo (per the #3 guardrail the codebases are the fixtures — red cases where
//! a changed line is left uncovered, clean cases where it isn't), runs REAL vitest
//! over it via the SDK (`patch_coverage::check_typescript`) and the CLI (`run`),
//! and asserts the uncovered lines / exit code. Requires `git` + a Node toolchain
//! with vitest installed; the repo symlinks the fixtures' `node_modules` so `npx
//! vitest` resolves (the same install `unit coverage`'s TypeScript tests use).
//!
//! Opens at RED per AGENTS.md: detection is stubbed (`check_typescript` reports
//! nothing), so a repo whose change leaves a line uncovered still comes back clean.
//! The implementation — diffing `<base>...HEAD` and intersecting the changed lines
//! with vitest's per-file coverage — follows once CI witnesses these red.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::patch_coverage::check_typescript;
use testing_conventions::run;

/// The fixtures' installed vitest toolchain — symlinked into each throwaway repo
/// so `npx vitest` resolves it via Node's parent lookup without a per-test install.
fn fixtures_node_modules() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_coverage/typescript/node_modules")
}

/// A throwaway git repo, removed on drop. A test writes a baseline source + its
/// colocated test, `commit`s it, captures `head()` as the `base`, then mutates and
/// commits the "after" so `<base>...HEAD` is the change under test. `node_modules`
/// is symlinked to the fixtures' install so vitest resolves.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-patch-cov-ts-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::os::unix::fs::symlink(fixtures_node_modules(), root.join("node_modules"))
            .expect("symlinking the fixtures' node_modules should succeed");
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
    check_typescript(&repo.0, base, &[])
        .expect("checking a readable repo should succeed")
        .iter()
        .map(|u| format!("{}:{}", u.file, u.line))
        .collect()
}

/// Result of `unit patch-coverage <repo> --language typescript --base <base>
/// [--config <repo>/<config>]`, run in-process.
fn run_patch_coverage(repo: &TempRepo, base: &str, config: Option<&str>) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "patch-coverage".into(),
        repo.0.clone().into_os_string(),
        "--language".into(),
        "typescript".into(),
        "--base".into(),
        base.into(),
    ];
    if let Some(name) = config {
        argv.push("--config".into());
        argv.push(repo.0.join(name).into_os_string());
    }
    run(argv)
}

/// Fully covered by `WIDGET_TEST_TS` at baseline (both branches taken).
const WIDGET_TS: &str = r#"export function widget(n: number): string {
  if (n > 0) return 'pos';
  return 'neg';
}
"#;
const WIDGET_TEST_TS: &str = r#"import { expect, test } from 'vitest';

import { widget } from './widget';

test('widget', () => {
  expect(widget(1)).toBe('pos');
  expect(widget(-1)).toBe('neg');
});
"#;

/// Adds an `n === 42` branch the baseline test never exercises. The diff adds new
/// lines 3-5: line 3 (`if (n === 42) {`) is the source of a branch never taken and
/// line 4 (`return 'answer';`) is an uncovered statement — so both are uncovered.
const WIDGET_TS_UNCOVERED: &str = r#"export function widget(n: number): string {
  if (n > 0) return 'pos';
  if (n === 42) {
    return 'answer';
  }
  return 'neg';
}
"#;

// ---- Detection via the SDK (`check_typescript`) --------------------------

#[test]
fn ts_uncovered_changed_line_is_reported() {
    // The core red case: the change adds a statement (and a branch) the suite never
    // runs, so the new lines come back uncovered.
    let repo = TempRepo::new("uncovered");
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.ts", WIDGET_TS_UNCOVERED);
    repo.commit("add an untested branch");

    let reported = uncovered(&repo, &base);
    assert!(
        reported.contains(&"widget.ts:4".to_string()),
        "the uncovered new statement should be reported; got: {reported:?}"
    );
    assert!(
        reported.contains(&"widget.ts:3".to_string()),
        "the uncovered new branch source should be reported; got: {reported:?}"
    );
}

#[test]
fn ts_covered_change_is_clean() {
    // Editing a line the suite already exercises keeps the change fully covered.
    let repo = TempRepo::new("covered");
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.ts",
        r#"export function widget(n: number): string {
  if (n > 0) return 'positive';
  return 'neg';
}
"#,
    );
    repo.write(
        "widget.test.ts",
        r#"import { expect, test } from 'vitest';

import { widget } from './widget';

test('widget', () => {
  expect(widget(1)).toBe('positive');
  expect(widget(-1)).toBe('neg');
});
"#,
    );
    repo.commit("reword a covered line and update its test");

    assert!(
        uncovered(&repo, &base).is_empty(),
        "a change to a covered line is clean"
    );
}

#[test]
fn a_change_touching_no_typescript_is_clean() {
    // A diff with no TypeScript source returns clean immediately — there's no
    // changed line to measure, so the suite isn't even run.
    let repo = TempRepo::new("no-ts");
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.write("README.md", "# project\n");
    repo.commit("base");
    let base = repo.head();

    repo.write("README.md", "# project\n\nnow with docs\n");
    repo.commit("docs only");

    assert!(
        uncovered(&repo, &base).is_empty(),
        "a change that touches no TypeScript source is clean"
    );
}

#[test]
fn ts_added_untested_file_changed_lines_are_uncovered() {
    // Unlike co-change (#33), an *added* file's new lines ARE patch-coverage
    // subjects: a brand-new source the suite never imports is wholly uncovered.
    let repo = TempRepo::new("added");
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "lonely.ts",
        "export function lonely(): number {\n  return 41;\n}\n",
    );
    repo.commit("add a brand-new untested source");

    let reported = uncovered(&repo, &base);
    assert!(
        reported.contains(&"lonely.ts:2".to_string()),
        "the added file's uncovered lines should be reported; got: {reported:?}"
    );
}

#[test]
fn ts_changed_comment_line_is_not_a_subject() {
    // A comment isn't a statement or branch, so coverage has nothing to measure on
    // it — a changed comment line is never flagged.
    let repo = TempRepo::new("comment");
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.ts",
        r#"export function widget(n: number): string {
  // branch on the sign of n
  if (n > 0) return 'pos';
  return 'neg';
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
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");

    assert!(
        check_typescript(&repo.0, "no-such-ref", &[]).is_err(),
        "an unresolvable base ref must error"
    );
}

// ---- Exit codes via the CLI (`run`) --------------------------------------

#[test]
fn ts_subcommand_exits_nonzero_on_an_uncovered_changed_line() {
    let repo = TempRepo::new("cli-red");
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.ts", WIDGET_TS_UNCOVERED);
    repo.commit("add an untested branch");

    assert_eq!(run_patch_coverage(&repo, &base, None).unwrap(), 1);
}

#[test]
fn ts_subcommand_exits_zero_when_the_change_is_covered() {
    let repo = TempRepo::new("cli-clean");
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    let base = repo.head();
    repo.write(
        "widget.ts",
        r#"export function widget(n: number): string {
  if (n > 0) return 'positive';
  return 'neg';
}
"#,
    );
    repo.write(
        "widget.test.ts",
        r#"import { expect, test } from 'vitest';

import { widget } from './widget';

test('widget', () => {
  expect(widget(1)).toBe('positive');
  expect(widget(-1)).toBe('neg');
});
"#,
    );
    repo.commit("reword a covered line and update its test");

    assert_eq!(run_patch_coverage(&repo, &base, None).unwrap(), 0);
}

// ---- Exemptions (#32 machinery, rule `coverage`) -------------------------

#[test]
fn ts_a_coverage_exemption_lifts_an_uncovered_changed_line() {
    // A `coverage` exemption excludes a file from the run, so its changed lines
    // have nothing to cover — the same waiver the floor (#31) honors.
    let repo = TempRepo::new("exempt");
    repo.write(
        "testing-conventions.toml",
        "[[typescript.exempt]]\npath = \"shim.ts\"\nrules = [\"coverage\"]\n\
         reason = \"thin launcher; logic lives in tested modules\"\n",
    );
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.write(
        "shim.ts",
        "export function shim(): number {\n  return 0;\n}\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "shim.ts",
        "export function shim(): number {\n  return 1;\n}\n",
    );
    repo.commit("edit the untested launcher");

    // Flagged with no config…
    assert_eq!(run_patch_coverage(&repo, &base, None).unwrap(), 1);
    // …and lifted by the `coverage` exemption.
    assert_eq!(
        run_patch_coverage(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}
