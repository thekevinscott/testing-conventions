//! Integration tests for diff-scoped TypeScript coverage — `unit coverage
//! --language typescript --base` (#162).
//!
//! The TypeScript twin of `coverage_base.rs`: with `--base`, the SAME configured
//! vitest floors (lines / branches / functions / statements) are measured over the
//! `<base>...HEAD` diff (the changed lines) instead of the whole tree. Unlike the
//! implicit-100% `unit patch-coverage` it replaces, a changed line is judged
//! against the configured floor — a diff that clears it passes even with an
//! uncovered line, and one below it fails however small the diff (no small-diff
//! carve-out, per the #162 decision).
//!
//! Each test builds a throwaway git repo (the codebases are the fixtures, per the
//! #3 guardrail) and runs REAL vitest over it via the SDK
//! (`patch_coverage::measure_typescript`) and the CLI (`run`). Requires `git` + a
//! Node toolchain with vitest installed; the repo symlinks the fixtures'
//! `node_modules` so `npx vitest` resolves (the same install `unit coverage`'s
//! TypeScript tests use).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::coverage::{Outcome, TypeScriptThresholds};
use testing_conventions::{patch_coverage, run};

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
            "tc-cov-base-ts-{}-{}-{}",
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

/// A uniform floor across all four metrics — the bracket the known-ratio diff is
/// judged against (its minimum metric is 50%, so an 80 floor fails and a 40 floor
/// clears).
fn floors(level: u8) -> TypeScriptThresholds {
    TypeScriptThresholds {
        lines: level,
        branches: level,
        functions: level,
        statements: level,
    }
}

/// The diff-scoped outcome for `<base>...HEAD` at a uniform `level` floor (no
/// exemptions) via the SDK.
fn measure_base(repo: &TempRepo, base: &str, level: u8) -> Outcome {
    patch_coverage::measure_typescript(&repo.0, base, floors(level), &[])
        .expect("measuring a readable repo should succeed")
}

/// Exit code of `unit coverage <repo> --language typescript --base <base> [--config
/// <repo>/<config>]`, run in-process.
fn run_coverage_base(repo: &TempRepo, base: &str, config: Option<&str>) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "coverage".into(),
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

/// Baseline: `widget` is fully covered (both branches taken) by `WIDGET_TEST_TS`.
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

/// After: appends two one-line functions — `covered` (the test calls it) and
/// `uncovered` (it doesn't). The diff adds new lines 5-12; restricted to them the
/// four metrics land at a known shape (verified against real vitest):
///   - functions: `covered` (line 6) is called, `uncovered` (line 10) isn't → 1/2 = **50%**
///   - statements / lines: the `covered` body + braces run, `uncovered`'s don't → 4/6 = **66.67%**
///   - branches: the only branch arm in the diff is `covered`'s, taken → **100%**
///
/// So the minimum metric is 50%: the same diff fails an 80 floor (functions 50,
/// lines/statements 66.67 all below) and clears a 40 floor.
const WIDGET_TS_75: &str = r#"export function widget(n: number): string {
  if (n > 0) return 'pos';
  return 'neg';
}

export function covered(): number {
  return 1;
}

export function uncovered(): number {
  return 2;
}
"#;
const WIDGET_TEST_75: &str = r#"import { expect, test } from 'vitest';

import { widget, covered } from './widget';

test('widget', () => {
  expect(widget(1)).toBe('pos');
  expect(widget(-1)).toBe('neg');
});

test('covered', () => {
  expect(covered()).toBe(1);
});
"#;

/// Writes the fully-covered baseline + its test and returns its commit as the base.
fn baseline(repo: &TempRepo) -> String {
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    repo.head()
}

// ---- The floor is measured over the diff (SDK `measure_typescript`) -------

#[test]
fn ts_a_diff_below_the_floor_fails() {
    // The core red case: the known-ratio diff (min metric 50%) is below an 80 floor,
    // so `--base` fails it — even though the whole tree is still well covered.
    let repo = TempRepo::new("below");
    let base = baseline(&repo);
    repo.write("widget.ts", WIDGET_TS_75);
    repo.write("widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert!(
        matches!(measure_base(&repo, &base, 80), Outcome::Fail(_)),
        "the diff's 50% functions / 66.67% statements are below an 80 floor"
    );
}

#[test]
fn ts_the_same_diff_clears_a_lower_floor() {
    // The behavior change from the implicit-100% patch-coverage: the SAME diff, with
    // its uncovered helper, PASSES once the configured floor is 40 — the changed
    // lines are judged against the number you set, not against 100%.
    let repo = TempRepo::new("clears");
    let base = baseline(&repo);
    repo.write("widget.ts", WIDGET_TS_75);
    repo.write("widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert_eq!(
        measure_base(&repo, &base, 40),
        Outcome::Pass,
        "every metric (min 50%) clears a 40 floor despite the uncovered helper"
    );
}

#[test]
fn ts_a_fully_covered_change_passes() {
    // Editing a line the suite already exercises keeps the diff at 100% → any floor
    // is met.
    let repo = TempRepo::new("covered");
    let base = baseline(&repo);
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

    assert_eq!(measure_base(&repo, &base, 80), Outcome::Pass);
}

#[test]
fn ts_a_tiny_below_floor_diff_is_not_exempted() {
    // The #162 decision: there is no small-diff carve-out. A single untested helper
    // (a brand-new file the suite never imports → 0% on its few lines) fails the 80
    // floor just like a large diff would.
    let repo = TempRepo::new("tiny");
    let base = baseline(&repo);
    repo.write(
        "lonely.ts",
        "export function lonely(): number {\n  return 41;\n}\n",
    );
    repo.commit("add one untested helper");

    assert!(
        matches!(measure_base(&repo, &base, 80), Outcome::Fail(_)),
        "a tiny 0%-covered diff still fails an 80 floor"
    );
}

#[test]
fn ts_a_change_touching_no_typescript_passes() {
    // A diff with no TypeScript source has no changed line to measure — vacuously
    // passes (the suite isn't even run), at any floor.
    let repo = TempRepo::new("no-ts");
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.write("README.md", "# project\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("README.md", "# project\n\nnow with docs\n");
    repo.commit("docs only");

    assert_eq!(measure_base(&repo, &base, 100), Outcome::Pass);
}

#[test]
fn ts_an_unknown_base_ref_is_an_error() {
    // A base that can't be resolved must surface, never silently pass as "clean".
    let repo = TempRepo::new("bad-base");
    let _ = baseline(&repo);
    assert!(
        patch_coverage::measure_typescript(&repo.0, "no-such-ref", floors(80), &[]).is_err(),
        "an unresolvable base ref must error"
    );
}

// ---- Exit codes via the CLI (`run`) --------------------------------------

#[test]
fn ts_cli_exits_nonzero_on_a_below_floor_diff() {
    // No config, so the diff is judged against the default TypeScript floors — now all
    // four at 100 (#194); the known-ratio diff (functions 50%, statements 66.67%) is
    // below them → exit 1.
    let repo = TempRepo::new("cli-red");
    let base = baseline(&repo);
    repo.write("widget.ts", WIDGET_TS_75);
    repo.write("widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert_eq!(run_coverage_base(&repo, &base, None).unwrap(), 1);
}

#[test]
fn ts_cli_exits_zero_when_the_diff_clears_the_floor() {
    let repo = TempRepo::new("cli-clean");
    let base = baseline(&repo);
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

    assert_eq!(run_coverage_base(&repo, &base, None).unwrap(), 0);
}

#[test]
fn ts_cli_a_lower_configured_floor_lets_the_same_diff_pass() {
    // A `[typescript.coverage]` table with all four floors at 40 re-scopes the floor:
    // the known-ratio diff that fails the default floor now passes — the floor is the
    // single source of truth, whole-tree or diff. The config is committed so the
    // measurement is deterministic.
    let repo = TempRepo::new("cli-floor40");
    repo.write(
        "testing-conventions.toml",
        "[typescript.coverage]\nlines = 40\nbranches = 40\nfunctions = 40\nstatements = 40\n",
    );
    let base = baseline(&repo);
    repo.write("widget.ts", WIDGET_TS_75);
    repo.write("widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

// ---- Exemptions (#32 machinery, rule `coverage`) -------------------------

#[test]
fn ts_a_coverage_exemption_lifts_a_below_floor_change() {
    // A `coverage` exemption excludes a file from the run, so its changed lines drop
    // out of the diff ratios — the same waiver the whole-tree floor (#31) honors.
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

    // Below the floor with no config…
    assert_eq!(run_coverage_base(&repo, &base, None).unwrap(), 1);
    // …and lifted by the `coverage` exemption.
    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}
