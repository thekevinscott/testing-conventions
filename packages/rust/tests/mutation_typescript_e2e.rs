//! E2E tests for the TypeScript mutation rule (#202): drive the built CLI binary
//! end-to-end (no mocks) against the fixture projects and assert the exit code.
//! Requires the fixtures' Stryker toolchain (`npm ci` in
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

use common::Staged;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Exit code of `testing-conventions unit mutation --language typescript [--config <cfg>] <project>`.
fn unit_mutation_exit(project: &Path, config: Option<&str>) -> i32 {
    let mut command = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    command.args(["unit", "mutation", "--language", "typescript"]);
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

/// Exit code + captured stderr of `testing-conventions unit mutation --language typescript <project>`.
fn unit_mutation_output(project: &Path) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg(project)
        .output()
        .expect("the built binary should run");
    (
        out.status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn missing_toolchain_fails_clean_without_downloading() {
    // End-to-end: with no Stryker installed, the binary must fail (exit 1) with a clear
    // error and never download the deprecated `stryker` package — it runs only the
    // project's own pinned `@stryker-mutator/core` via `npx --no-install`.
    let project = Staged::typescript_without_toolchain("survivors");
    let (code, stderr) = unit_mutation_output(project.path());
    assert_eq!(
        code, 1,
        "a missing toolchain should fail the run; stderr: {stderr}"
    );
    assert!(
        stderr.contains("npx --no-install"),
        "the error should name the no-download invocation; got: {stderr}"
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
