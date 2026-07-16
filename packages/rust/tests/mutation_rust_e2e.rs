//! E2E tests for the Rust mutation rule: drive the built CLI binary
//! end-to-end (no mocks) against the fixture crates and assert the exit code.
//! Requires only a cargo toolchain — the tool provisions cargo-mutants itself.
//!
//! The gate is **on by default and binary**: an un-exempted surviving mutant fails the
//! run, and the only way to pass with a survivor present is a reason-required
//! `mutation` exemption. The fixtures are the standard pair: `killed` (every mutant
//! caught) and `survivors` (a coverage-passing but assertion-light suite whose mutants
//! all survive).

mod common;

use std::path::PathBuf;
use std::process::Command;

use common::{tested_count, GitRepo, ENGINE_NOT_RUN};

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
fn killed_crate_passes_and_states_the_tested_count() {
    // Every mutant is caught, so the crate clears the gate — and the success line states
    // how many mutants the engine judged, the evidence telling this pass apart from an
    // engine-skipped one.
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .arg(fixtures().join("rust").join("killed"))
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
fn a_diff_without_crate_changes_reports_the_engine_not_run() {
    // The diff touches nothing under the crate (only a top-level note), so the run is
    // skipped — and the output says the engine never ran, distinct from the all-killed
    // success, keeping the vacuous pass visible in the job log. The exit code stays 0:
    // an empty diff owes no run.
    let repo = GitRepo::new("rust-vacuous");
    repo.write(
        "crate/Cargo.toml",
        "[package]\nname = \"tc_mut_vacuous\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n",
    );
    repo.write(
        "crate/src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    );
    repo.write("notes.md", "before\n");
    repo.commit("baseline");
    let base = repo.head();
    repo.write("notes.md", "before\nafter\n");
    repo.commit("tweak a top-level note, not the crate");

    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .args(["--base", &base])
        .arg(repo.path().join("crate"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "an empty crate-relative diff passes; stderr: {stderr}"
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
