//! E2E tests for the TypeScript mutation rule (#202): drive the built CLI binary
//! end-to-end (no mocks) against the fixture projects and assert the exit code.
//!
//! The binary spawns the bundled Node mutation adapter (#246); in production the npm
//! launcher injects its path, so these tests pass the freshly-built adapter via
//! [`common::ts_adapter`] on each invocation. The fixtures are **runner-only** (vitest,
//! no Stryker) — the consumer installs nothing Stryker-related; the tool bundles and
//! drives it. Requires the built node adapter and the fixtures' vitest (`npm ci` in
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

const ADAPTER_ENV: &str = "TESTING_CONVENTIONS_TS_MUTATION_ADAPTER";

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Exit code of `testing-conventions unit mutation --language typescript [--config <cfg>] <project>`,
/// with the bundled adapter path injected exactly as the npm launcher would.
fn unit_mutation_exit(project: &Path, config: Option<&str>) -> i32 {
    let mut command = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    command
        .env(ADAPTER_ENV, ts_adapter())
        .args(["unit", "mutation", "--language", "typescript"]);
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
fn run_outside_the_launcher_fails_clean() {
    // The binary is meant to be run through the npm launcher, which sets the adapter env
    // var. Invoked directly without it, the TS arm must fail (exit 1) with a clear error
    // naming the var — never guess at a Node entry on disk. `env_remove` guarantees the
    // var is unset regardless of the ambient environment.
    let project = Staged::new("survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .env_remove(ADAPTER_ENV)
        .args(["unit", "mutation", "--language", "typescript"])
        .arg(project.path())
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "an unset adapter path should fail the run; stderr: {stderr}"
    );
    assert!(
        stderr.contains(ADAPTER_ENV),
        "the error should name the adapter env var; got: {stderr}"
    );
}

#[test]
fn a_broken_adapter_path_fails_clean() {
    // The env var points at a Node entry that doesn't exist (node can't find the module):
    // the run must fail (exit 1) with the adapter's captured output surfaced, not hang or
    // pass. Covers the non-zero-exit path of the adapter spawn.
    let project = Staged::new("survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .env(ADAPTER_ENV, "/nonexistent/testing-conventions-adapter.js")
        .args(["unit", "mutation", "--language", "typescript"])
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
