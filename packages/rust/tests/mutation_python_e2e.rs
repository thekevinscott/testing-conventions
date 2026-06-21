//! E2E tests for the Python mutation rule (#203): drive the built CLI binary
//! end-to-end (no mocks) against the fixture projects and assert the exit code.
//! Requires cosmic-ray + pytest on PATH.
//!
//! The gate is **on by default and binary** — parity with the Rust and TypeScript arms:
//! an un-exempted surviving mutant fails the run, and the only way to pass with a
//! survivor present is a reason-required `mutation` exemption. The fixtures are the
//! standard pair: `killed` (every mutant caught) and `survivors` (a coverage-passing but
//! assertion-light suite whose mutants all survive). Each test runs against its own
//! staged copy so the parallel runs never collide in a shared project dir.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::Staged;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Exit code of `testing-conventions unit mutation --language python [--config <cfg>] <project>`.
fn unit_mutation_exit(project: &Path, config: Option<&str>) -> i32 {
    let mut command = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    command.args(["unit", "mutation", "--language", "python"]);
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
fn killed_project_passes_with_no_survivors() {
    // Every mutant is caught, so the project clears the gate.
    let project = Staged::python("killed");
    assert_eq!(unit_mutation_exit(project.path(), None), 0);
}

#[test]
fn survivors_fail_the_gate_by_default() {
    // The gate is on by default and binary: an un-exempted surviving mutant fails the
    // run, no config required.
    let project = Staged::python("survivors");
    assert_eq!(unit_mutation_exit(project.path(), None), 1);
}

#[test]
fn an_exempted_survivor_passes_the_gate() {
    // The survivor's file carries a `mutation` exemption, so the gate clears it (an
    // equivalent / deliberately-defensive mutation, lifted with a reason) — the only
    // way to pass with a survivor present.
    let project = Staged::python("survivors");
    assert_eq!(
        unit_mutation_exit(project.path(), Some("mutation_exempt_py.toml")),
        0
    );
}
