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
//! reason-required `mutation` exemption. The fixtures are the standard pair: `killed`
//! (every mutant caught) and `survivors` (a coverage-passing but assertion-light suite
//! whose mutants all survive). Each test runs against its own staged copy so the
//! parallel Stryker runs never collide in a shared project dir.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{ts_adapter, Staged};

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
        .arg(project.path())
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
        .arg(project.path())
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
fn killed_project_passes_with_no_survivors() {
    // Every mutant is caught, so the project clears the gate.
    let project = Staged::new("killed");
    assert_eq!(unit_mutation_exit(project.path(), None), 0);
}

#[test]
fn survivors_fail_the_gate_by_default() {
    // The gate is on by default and binary: an un-exempted surviving mutant fails the
    // run, no config required.
    let project = Staged::new("survivors");
    assert_eq!(unit_mutation_exit(project.path(), None), 1);
}

#[test]
fn a_src_scan_path_with_an_upward_import_fails_on_survivors() {
    // The standard `{package.json, src/**}` layout whose source imports `../package.json`,
    // scanned at `src/`: the gate reaches the survivors and lists them scan-path-relative —
    // the sandbox is rooted at the package root, so the upward import resolves and the run
    // is judged on mutants, never on module resolution.
    let package = Staged::upward("upward_survivors");
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
fn a_src_scan_path_with_an_upward_import_passes_when_all_mutants_are_killed() {
    // The killed twin clears the gate: the upward import resolves in the sandbox and every
    // mutant under the scan path is caught.
    let package = Staged::upward("upward_killed");
    assert_eq!(unit_mutation_exit(&package.path().join("src"), None), 0);
}

#[test]
fn an_exempted_survivor_passes_the_gate() {
    // The survivor's file carries a `mutation` exemption, so the gate clears it (an
    // equivalent / deliberately-defensive mutation, lifted with a reason) — the only
    // way to pass with a survivor present.
    let project = Staged::new("survivors");
    assert_eq!(
        unit_mutation_exit(project.path(), Some("mutation_exempt_ts.toml")),
        0
    );
}
