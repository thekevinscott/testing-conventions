//! E2E tests for the Rust mutation rule (#201): drive the built CLI binary
//! end-to-end (no mocks) against the fixture crates and assert the exit code.
//! Requires only a cargo toolchain — the tool provisions cargo-mutants itself (#242).
//!
//! The gate is **on by default and binary**: an un-exempted surviving mutant fails the
//! run, and the only way to pass with a survivor present is a reason-required
//! `mutation` exemption. The fixtures are the standard pair: `killed` (every mutant
//! caught) and `survivors` (a coverage-passing but assertion-light suite whose mutants
//! all survive).

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Exit code of `testing-conventions unit mutation --language rust [--config <cfg>] <crate>`.
fn unit_mutation_exit(crate_name: &str, config: Option<&str>) -> i32 {
    let mut command = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    command.args(["unit", "mutation", "--language", "rust"]);
    if let Some(config) = config {
        command.arg("--config").arg(fixtures().join(config));
    }
    command
        .arg(fixtures().join("rust").join(crate_name))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn killed_crate_passes_with_no_survivors() {
    // Every mutant is caught, so the crate clears the gate.
    assert_eq!(unit_mutation_exit("killed", None), 0);
}

#[test]
fn survivors_fail_the_gate_by_default() {
    // The gate is on by default and binary: an un-exempted surviving mutant fails the
    // run, no config required.
    assert_eq!(unit_mutation_exit("survivors", None), 1);
}

#[test]
fn an_exempted_survivor_passes_the_gate() {
    // The survivor's file carries a `mutation` exemption, so the gate clears it (an
    // equivalent / deliberately-defensive mutation, lifted with a reason) — the only
    // way to pass with a survivor present.
    assert_eq!(
        unit_mutation_exit("survivors", Some("mutation_exempt.toml")),
        0
    );
}
