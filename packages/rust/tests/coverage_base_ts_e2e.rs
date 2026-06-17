//! E2E tests for diff-scoped TypeScript coverage — `unit coverage --language
//! typescript --base` (#162): drive the built CLI binary as a real subprocess
//! against throwaway git repos and assert the exit code (and, for a red case, the
//! failure on stderr). Complements the in-process integration tests in
//! `coverage_base_ts.rs`. Requires `git` + a Node toolchain with vitest installed;
//! the repo symlinks the fixtures' `node_modules` so `npx vitest` resolves.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// The fixtures' installed vitest toolchain — symlinked into each throwaway repo
/// so `npx vitest` resolves it via Node's parent lookup without a per-test install.
fn fixtures_node_modules() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_coverage/typescript/node_modules")
}

/// A throwaway git repo, removed on drop. A test writes a baseline, `commit`s it,
/// captures `head()` as the `base`, then mutates and commits the "after".
/// `node_modules` is symlinked to the fixtures' install so vitest resolves.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-cov-base-ts-e2e-{}-{}-{}",
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

/// Exit code + stderr of `unit coverage <repo> --language typescript --base <base>
/// [--config <repo>/<config>]`, run as a real subprocess.
fn coverage_base(repo: &TempRepo, base: &str, config: Option<&str>) -> (i32, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    cmd.arg("unit")
        .arg("coverage")
        .arg(&repo.0)
        .args(["--language", "typescript", "--base", base]);
    if let Some(name) = config {
        cmd.arg("--config").arg(repo.0.join(name));
    }
    let output = cmd.output().expect("the built binary should run");
    (
        output
            .status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
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

/// After: a covered and an uncovered one-line helper → the diff (new lines 5-12)
/// lands at functions 50% / statements 66.67% / branches 100% (see
/// `coverage_base_ts.rs`), so its minimum metric is below the default 80 floor.
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
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    repo.head()
}

#[test]
fn ts_below_floor_diff_exits_nonzero_and_reports_coverage() {
    let repo = TempRepo::new("red");
    let base = baseline(&repo);
    repo.write("widget.ts", WIDGET_TS_75);
    repo.write("widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    let (code, stderr) = coverage_base(&repo, &base, None);
    assert_eq!(
        code, 1,
        "a diff below the floor must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("coverage"),
        "stderr should report the coverage shortfall; got: {stderr}"
    );
}

#[test]
fn ts_covered_change_exits_zero() {
    let repo = TempRepo::new("clean");
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

    let (code, stderr) = coverage_base(&repo, &base, None);
    assert_eq!(code, 0, "a fully covered change passes; stderr: {stderr}");
}

#[test]
fn ts_a_lower_configured_floor_lets_the_same_diff_pass() {
    // The behavior change: the diff that fails the default 80 floor passes once the
    // configured floors are 40 — the floor is the single source of truth. The config
    // is committed so the measurement is deterministic.
    let repo = TempRepo::new("floor40");
    repo.write(
        "testing-conventions.toml",
        "[typescript.coverage]\nlines = 40\nbranches = 40\nfunctions = 40\nstatements = 40\n",
    );
    let base = baseline(&repo);
    repo.write("widget.ts", WIDGET_TS_75);
    repo.write("widget.test.ts", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    let (code, stderr) = coverage_base(&repo, &base, Some("testing-conventions.toml"));
    assert_eq!(
        code, 0,
        "the diff (min 50%) clears a configured 40 floor; stderr: {stderr}"
    );
}

#[test]
fn ts_a_tiny_below_floor_diff_still_exits_nonzero() {
    // No small-diff carve-out (#162): a single untested helper (a brand-new file the
    // suite never imports → 0% on its lines) fails the default 80 floor.
    let repo = TempRepo::new("tiny");
    let base = baseline(&repo);
    repo.write(
        "lonely.ts",
        "export function lonely(): number {\n  return 41;\n}\n",
    );
    repo.commit("add one untested helper");

    let (code, stderr) = coverage_base(&repo, &base, None);
    assert_eq!(
        code, 1,
        "a tiny diff below the floor is not exempted; stderr: {stderr}"
    );
}
