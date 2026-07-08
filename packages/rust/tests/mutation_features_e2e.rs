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
//! Red until feature passthrough lands: today the `features` key is rejected by
//! the config self-guard, so the run exits non-zero with an "unknown field"
//! error rather than the clean pass asserted here. Requires only a cargo
//! toolchain — the tool provisions cargo-mutants itself.

use std::path::PathBuf;
use std::process::Command;

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
