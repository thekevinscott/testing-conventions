//! E2E tests for patch (changed-line) coverage (TypeScript — #135): drive the
//! built CLI binary as a real subprocess against throwaway git repos and assert
//! the exit code (and, for a red case, the named offender). Complements the
//! in-process integration tests in `patch_coverage_ts.rs`. Requires `git` + a Node
//! toolchain with vitest installed; the repo symlinks the fixtures' `node_modules`
//! so `npx vitest` resolves.
//!
//! Starts red against the stub in `src/patch_coverage.rs` (`check_typescript`
//! reports nothing) and goes green once the diff + vitest detection is implemented.

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
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-patch-cov-ts-e2e-{}-{}-{}",
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

/// Exit code + stderr of `unit patch-coverage <repo> --language typescript --base
/// <base> [--config <repo>/<config>]`, run as a real subprocess.
fn patch_coverage(repo: &TempRepo, base: &str, config: Option<&str>) -> (i32, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    cmd.arg("unit").arg("patch-coverage").arg(&repo.0).args([
        "--language",
        "typescript",
        "--base",
        base,
    ]);
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
const WIDGET_TS_UNCOVERED: &str = r#"export function widget(n: number): string {
  if (n > 0) return 'pos';
  if (n === 42) {
    return 'answer';
  }
  return 'neg';
}
"#;

#[test]
fn uncovered_changed_line_exits_nonzero_and_names_it() {
    let repo = TempRepo::new("red");
    repo.write("widget.ts", WIDGET_TS);
    repo.write("widget.test.ts", WIDGET_TEST_TS);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.ts", WIDGET_TS_UNCOVERED);
    repo.commit("add an untested branch");

    let (code, stderr) = patch_coverage(&repo, &base, None);
    assert_eq!(
        code, 1,
        "an uncovered changed line must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("widget.ts"),
        "stderr should name the uncovered file; got: {stderr}"
    );
}

#[test]
fn covered_change_exits_zero() {
    let repo = TempRepo::new("clean");
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

    let (code, stderr) = patch_coverage(&repo, &base, None);
    assert_eq!(code, 0, "a fully covered change passes; stderr: {stderr}");
}

#[test]
fn added_untested_file_exits_nonzero() {
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

    let (code, stderr) = patch_coverage(&repo, &base, None);
    assert_eq!(
        code, 1,
        "an added file's uncovered lines must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("lonely.ts"),
        "stderr should name the added file; got: {stderr}"
    );
}

#[test]
fn a_coverage_exemption_lifts_the_uncovered_change() {
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

    // Flagged with no config, lifted once the `coverage` exemption is supplied.
    assert_eq!(patch_coverage(&repo, &base, None).0, 1);
    assert_eq!(
        patch_coverage(&repo, &base, Some("testing-conventions.toml")).0,
        0
    );
}
