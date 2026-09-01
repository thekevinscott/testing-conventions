use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::coverage::{Outcome, TypeScriptThresholds};
use testing_conventions::{patch_coverage, run};

fn fixtures_node_modules() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_coverage/typescript/node_modules")
}

const PACKAGE_JSON: &str =
    "{ \"name\": \"tc-cov-base\", \"private\": true, \"version\": \"1.0.0\" }\n";

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
        let repo = TempRepo(root);
        repo.write("package.json", PACKAGE_JSON);
        repo
    }

    fn src(&self) -> PathBuf {
        self.0.join("src")
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
    patch_coverage::measure_typescript(
        &repo.src(),
        base,
        floors(level),
        &[],
        &std::collections::BTreeMap::new(),
    )
    .expect("measuring a readable repo should succeed")
}

/// Exit code of `unit coverage <repo> --language typescript --base <base> [--config
/// <repo>/<config>]`, run in-process.
fn run_coverage_base(repo: &TempRepo, base: &str, config: Option<&str>) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "coverage".into(),
        repo.src().into_os_string(),
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

fn baseline(repo: &TempRepo) -> String {
    repo.write("src/widget.ts", WIDGET_TS);
    repo.write("src/widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    repo.head()
}

#[test]
fn ts_a_diff_below_the_floor_fails() {
    let repo = TempRepo::new("below");
    let base = baseline(&repo);
    repo.write("src/widget.ts", WIDGET_TS_75);
    repo.write("src/widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert!(
        matches!(measure_base(&repo, &base, 80), Outcome::Fail(_)),
        "the diff's 50% functions / 66.67% statements are below an 80 floor"
    );
}

#[test]
fn ts_the_same_diff_clears_a_lower_floor() {
    let repo = TempRepo::new("clears");
    let base = baseline(&repo);
    repo.write("src/widget.ts", WIDGET_TS_75);
    repo.write("src/widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert_eq!(
        measure_base(&repo, &base, 40),
        Outcome::Pass,
        "every metric (min 50%) clears a 40 floor despite the uncovered helper"
    );
}

#[test]
fn ts_a_fully_covered_change_passes() {
    let repo = TempRepo::new("covered");
    let base = baseline(&repo);
    repo.write(
        "src/widget.ts",
        r#"export function widget(n: number): string {
  if (n > 0) return 'positive';
  return 'neg';
}
"#,
    );
    repo.write(
        "src/widget.test.ts",
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
    let repo = TempRepo::new("tiny");
    let base = baseline(&repo);
    repo.write(
        "src/lonely.ts",
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
    let repo = TempRepo::new("no-ts");
    repo.write("src/widget.ts", WIDGET_TS);
    repo.write("src/widget.test.ts", WIDGET_TEST_TS);
    repo.write("README.md", "# project\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("README.md", "# project\n\nnow with docs\n");
    repo.commit("docs only");

    assert_eq!(measure_base(&repo, &base, 100), Outcome::Pass);
}

#[test]
fn ts_an_unknown_base_ref_is_an_error() {
    let repo = TempRepo::new("bad-base");
    let _ = baseline(&repo);
    assert!(
        patch_coverage::measure_typescript(
            &repo.0,
            "no-such-ref",
            floors(80),
            &[],
            &std::collections::BTreeMap::new()
        )
        .is_err(),
        "an unresolvable base ref must error"
    );
}

#[test]
fn ts_cli_exits_nonzero_on_a_below_floor_diff() {
    let repo = TempRepo::new("cli-red");
    let base = baseline(&repo);
    repo.write("src/widget.ts", WIDGET_TS_75);
    repo.write("src/widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert_eq!(run_coverage_base(&repo, &base, None).unwrap(), 1);
}

#[test]
fn ts_cli_exits_zero_when_the_diff_clears_the_floor() {
    let repo = TempRepo::new("cli-clean");
    let base = baseline(&repo);
    repo.write(
        "src/widget.ts",
        r#"export function widget(n: number): string {
  if (n > 0) return 'positive';
  return 'neg';
}
"#,
    );
    repo.write(
        "src/widget.test.ts",
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
    let repo = TempRepo::new("cli-floor40");
    repo.write(
        "testing-conventions.toml",
        "[typescript.coverage]\nlines = 40\nbranches = 40\nfunctions = 40\nstatements = 40\n",
    );
    let base = baseline(&repo);
    repo.write("src/widget.ts", WIDGET_TS_75);
    repo.write("src/widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

#[test]
fn ts_a_coverage_exemption_lifts_a_below_floor_change() {
    let repo = TempRepo::new("exempt");
    repo.write(
        "testing-conventions.toml",
        "[[typescript.exempt]]\npath = \"shim.ts\"\nrules = [\"coverage\"]\n\
         lines = [\"1-3\"]\nreason = \"thin launcher; logic lives in tested modules\"\n",
    );
    repo.write("src/widget.ts", WIDGET_TS);
    repo.write("src/widget.test.ts", WIDGET_TEST_TS);
    repo.write(
        "src/shim.ts",
        "export function shim(): number {\n  return 0;\n}\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "src/shim.ts",
        "export function shim(): number {\n  return 1;\n}\n",
    );
    repo.commit("edit the untested launcher");

    assert_eq!(run_coverage_base(&repo, &base, None).unwrap(), 1);
    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}
