//! E2E tests for the TypeScript mutation rule: drive the built CLI binary
//! end-to-end (no mocks) against the fixture projects and assert the exit code.
//!
//! The binary spawns the bundled Node mutation adapter; in production the npm
//! launcher appends its path as `--ts-mutation-adapter`, so these tests pass the freshly-built
//! adapter ([`common::ts_adapter`]) the same way on each invocation. The fixtures are
//! **runner-only** (vitest): the tool bundles and drives Stryker; the project provides only its
//! own test runner. Requires the built node adapter and the fixtures' vitest (`npm ci` in
//! `tests/fixtures/unit_mutation/typescript`).
//!
//! The gate is **on by default and binary** — parity with the Rust arm: an un-exempted
//! surviving mutant fails the run, and the only way to pass with a survivor present is a
//! reason-required `mutation` exemption. The default fixtures are the prescribed consumer
//! package layout — `{package.json, tsconfig.json, src/**, tests/**}`, scanned at `src/`,
//! whose source imports `../package.json`: `killed` (every mutant caught) and `survivors` (a
//! coverage-passing but assertion-light suite whose mutants all survive). The flat, no-manifest
//! shape is the `loose_*` special case. Each test runs against its own staged copy so the
//! parallel Stryker runs never collide in a shared project dir.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{tested_count, ts_adapter, GitRepo, Staged, ENGINE_NOT_RUN};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Exit code of `testing-conventions unit mutation --language typescript [--config <cfg>] <project>`,
/// passing the bundled adapter path as `--ts-mutation-adapter`, exactly as the npm launcher does.
fn unit_mutation_exit(project: &Path, config: Option<&str>) -> i32 {
    let mut command = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    command
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter());
    if let Some(config) = config {
        command.arg("--config").arg(fixtures().join(config));
    }
    command
        .arg(project)
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn run_without_the_adapter_arg_fails_clean() {
    // The npm launcher appends `--ts-mutation-adapter`; run directly without it, the TS arm
    // must fail (exit 1) with a clear error naming the argument — never guess at a Node entry
    // on disk.
    let project = Staged::new("survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg(project.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a missing adapter argument should fail the run; stderr: {stderr}"
    );
    assert!(
        stderr.contains("--ts-mutation-adapter"),
        "the error should name the adapter argument; got: {stderr}"
    );
}

#[test]
fn a_broken_adapter_path_fails_clean() {
    // The argument points at a Node entry that doesn't exist (node can't find the module):
    // the run must fail (exit 1) with the adapter's captured output surfaced, not hang or
    // pass. Covers the non-zero-exit path of the adapter spawn.
    let project = Staged::new("survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg("/nonexistent/testing-conventions-adapter.js")
        .arg(project.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a broken adapter path should fail the run; stderr: {stderr}"
    );
    assert!(
        stderr.contains("adapter failed"),
        "the error should report the adapter failure; got: {stderr}"
    );
}

#[test]
fn killed_project_passes_and_states_the_tested_count() {
    // Every mutant is caught, so the project clears the gate — and the success line
    // states how many mutants the engine judged, the evidence telling this pass apart
    // from an engine-skipped one. The default package layout is scanned at `src/`.
    let package = Staged::new("killed");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter())
        .arg(package.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "every mutant is caught; stderr: {stderr}"
    );
    assert!(
        tested_count(&stdout) > 0,
        "the engine ran, so the count is non-zero; got: {stdout}"
    );
}

#[test]
fn a_diff_with_no_mutatable_changed_lines_reports_the_engine_not_run() {
    // Only a test file changes on the diff, so the run is skipped — and the output says
    // the engine never ran, distinct from the all-killed success, keeping the vacuous
    // pass visible in the job log. The exit code stays 0: an empty diff owes no run.
    let repo = GitRepo::new("ts-vacuous");
    repo.write(
        "index.ts",
        "export function add(a: number, b: number): number {\n  return a + b;\n}\n",
    );
    repo.write(
        "index.test.ts",
        "import { it, expect } from 'vitest';\nimport { add } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n});\n",
    );
    repo.commit("baseline");
    let base = repo.head();
    repo.write(
        "index.test.ts",
        "import { it, expect } from 'vitest';\nimport { add } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n  expect(add(-1, 1)).toBe(0);\n});\n",
    );
    repo.commit("tweak only the test file");

    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter())
        .args(["--base", &base])
        .arg(repo.path())
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "an empty mutate set passes; stderr: {stderr}"
    );
    assert!(
        stdout.contains(ENGINE_NOT_RUN),
        "the skip is stated; got: {stdout}"
    );
    assert!(
        !stdout.contains("every mutation was caught"),
        "an engine-skipped pass never claims mutants were caught; got: {stdout}"
    );
}

#[test]
fn survivors_fail_the_gate_by_default() {
    // The gate is on by default and binary: an un-exempted surviving mutant fails the run, no
    // config required. The default `{package.json, src/**}` layout is scanned at `src/`, the
    // run rooted at the package root so the upward `../package.json` import resolves, and the
    // survivors are listed scan-path-relative.
    let package = Staged::new("survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter())
        .arg(package.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "the assertion-light suite leaves survivors; stderr: {stderr}"
    );
    assert!(
        stderr.contains("unexplained surviving mutant") && stderr.contains("index.ts"),
        "the survivors are listed scan-path-relative; got: {stderr}"
    );
}

#[test]
fn a_loose_tree_fails_the_gate_on_survivors() {
    // The loose special case: flat scripts, no manifest, scanned at the root. The gate still
    // runs Stryker in place there and fails on the un-exempted survivor.
    let project = Staged::loose("loose_survivors");
    assert_eq!(unit_mutation_exit(project.path(), None), 1);
}

#[test]
fn an_exempted_survivor_passes_the_gate() {
    // The survivor's file carries a `mutation` exemption, so the gate clears it (an
    // equivalent / deliberately-defensive mutation, lifted with a reason) — the only
    // way to pass with a survivor present.
    let package = Staged::new("survivors");
    assert_eq!(
        unit_mutation_exit(&package.path().join("src"), Some("mutation_exempt_ts.toml")),
        0
    );
}
