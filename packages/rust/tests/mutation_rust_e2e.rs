//! E2E tests for the Rust mutation rule (#201): drive the built CLI binary
//! end-to-end (no mocks) against the fixture crates and assert the exit code.
//! Requires `cargo-mutants`.
//!
//! The rule is **report-only by default**: it lists surviving mutants but exits `0`
//! unless a `[rust].mutation` table opts into the hard gate. The fixtures are the
//! standard pair: `killed` (every mutant caught) and `survivors` (a coverage-passing
//! but assertion-light suite whose mutants all survive).

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
    // Every mutant is caught, so the crate passes whether or not the gate is on.
    assert_eq!(unit_mutation_exit("killed", None), 0);
    assert_eq!(unit_mutation_exit("killed", Some("mutation_gate.toml")), 0);
}

#[test]
fn survivors_report_only_exits_zero() {
    // No `[rust].mutation` table: report-only, so surviving mutants are listed but the
    // command still exits 0 — a signal, not a gate.
    assert_eq!(unit_mutation_exit("survivors", None), 0);
}

#[test]
fn survivors_under_the_gate_exit_nonzero() {
    // With the `[rust].mutation` table the hard gate bites: an un-exempted surviving
    // mutant fails the run.
    assert_eq!(unit_mutation_exit("survivors", Some("mutation_gate.toml")), 1);
}

#[test]
fn an_exempted_survivor_passes_the_gate() {
    // The survivor's file carries a `mutation` exemption, so the gate clears it (an
    // equivalent / deliberately-defensive mutation, lifted with a reason).
    assert_eq!(unit_mutation_exit("survivors", Some("mutation_exempt.toml")), 0);
}
