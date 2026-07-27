//! E2E tests for cargo-feature passthrough in `unit mutation --language rust`:
//! drive the built CLI binary end-to-end (no mocks) against the
//! feature-gated fixture crate and assert the exit code.
//!
//! A `[rust] features` list names the cargo features the mutation run enables
//! (forwarded to cargo-mutants' build/test invocations), so mutants of
//! `#[cfg(feature = …)]` code are compiled and exercised by the gated module's
//! own tests. Without the feature enabled the module is compiled out: its tests
//! never run, and its mutants survive unexercised.
//!
//! Requires only a cargo toolchain — the tool provisions cargo-mutants itself.

mod common;

use std::path::PathBuf;
use std::process::Command;

use common::tested_count;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

#[test]
fn a_feature_gated_module_with_killing_tests_passes_the_gate() {
    // Every mutant in `gated_killed` — the plain `core` and the feature-gated
    // `boost` — is caught by its colocated test once the `boost` feature is
    // enabled from config, so the crate clears the gate.
    let status = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust", "--config"])
        .arg(fixtures().join("rust_features.toml"))
        .arg(fixtures().join("rust").join("gated_killed"))
        .status()
        .expect("the built binary should run");
    assert_eq!(status.code(), Some(0));
}

#[test]
fn a_feature_gated_integration_test_target_builds_and_the_gate_passes() {
    // `gated_ws/member` is a workspace-member crate whose *integration* test names the
    // feature-gated module — the reported consumer layout. cargo builds a crate's test
    // targets before running them, so the feature has to reach the build phase: a
    // selection that lands only on `cargo test` leaves the unmutated baseline
    // uncompilable, and cargo-mutants judges nothing. With the feature enabled the
    // integration test builds and kills every mutant of the gated module, so the crate
    // clears the gate with a non-zero tested count.
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust", "--config"])
        .arg(fixtures().join("rust_features.toml"))
        .arg(fixtures().join("rust").join("gated_ws").join("member"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "every mutant is caught; stdout: {stdout}; stderr: {stderr}"
    );
    assert!(
        tested_count(&stdout) > 0,
        "the engine judged mutants, so the count is non-zero; got: {stdout}"
    );
}

#[test]
fn a_baseline_that_cannot_build_fails_loudly() {
    // The same crate scanned with no feature list: `tests/boost.rs` names an item that is
    // compiled out, the unmutated baseline never builds, and no mutant is judged. The run
    // fails and says so — a `0 mutant(s) tested` pass would read exactly like an
    // all-killed one.
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .arg(fixtures().join("rust").join("gated_ws").join("member"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a baseline that cannot build fails the run; stdout: {stdout}"
    );
    assert!(
        stderr.contains("did not run cleanly"),
        "the failure names the baseline build; stderr: {stderr}"
    );
    assert!(
        !stdout.contains("every mutation was caught"),
        "a run that judged nothing never claims mutants were caught; got: {stdout}"
    );
}
