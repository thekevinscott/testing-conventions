//! E2E test for the Rust mutation arm's engine provisioning.
//!
//! The consumer installs nothing and never names cargo-mutants: the tool provisions the
//! engine itself on first use — a pinned `cargo install` into its own cache directory —
//! and drives the binary from there. This drives the built CLI over the `killed` fixture
//! with **no ambient cargo-mutants** (CI no longer installs one), asserting the run clears
//! the gate, and runs it a second time to assert the provisioning is reused (idempotent —
//! the cached binary is invoked, not reinstalled). Requires only a cargo toolchain.

use std::path::PathBuf;
use std::process::Command;

fn killed_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation/rust/killed")
}

/// Exit code of `testing-conventions unit mutation --language rust <crate>`.
fn unit_mutation_exit() -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .arg(killed_crate())
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn provisions_the_engine_and_reuses_it() {
    // First run provisions cargo-mutants (no ambient engine); the clean crate clears the gate.
    assert_eq!(
        unit_mutation_exit(),
        0,
        "the tool should provision cargo-mutants and run the gate"
    );
    // Second run finds the provisioned binary in the cache and reuses it — same clean result.
    assert_eq!(
        unit_mutation_exit(),
        0,
        "the provisioned engine should be reused on the next run"
    );
}
