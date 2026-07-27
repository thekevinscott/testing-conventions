//! Integration tests for cargo-feature passthrough in `unit mutation --language rust`,
//! at the measurement boundary ([`measure_rust`]) with the real engine.
//!
//! A `[rust] features` list names the cargo features the mutation run enables. cargo
//! builds the crate's test targets **before** it runs them, so a feature that reaches
//! only the test phase leaves the unmutated baseline build broken for any crate whose
//! integration test names a `#[cfg(feature = …)]` item: cargo-mutants stops at
//! `cargo build failed in an unmutated tree` and judges nothing. The fixture is the
//! reported layout — a workspace-member crate whose `tests/boost.rs` uses the gated
//! module — and the feature has to reach every cargo invocation for it to build.
//!
//! Requires a cargo toolchain — the tool provisions cargo-mutants itself; the run builds
//! the crate from scratch, so it's slow.

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use common::expect_tested;
use testing_conventions::mutation::measure_rust;

/// The feature-gated workspace-member fixture crate: `gated_ws/member`, whose cargo
/// workspace root is the `gated_ws` directory above it.
fn member() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_mutation/rust/gated_ws/member")
}

#[test]
fn a_feature_gated_integration_test_target_builds_and_kills_its_mutants() {
    let (count, survivors) = expect_tested(
        measure_rust(
            &member(),
            &[],
            &BTreeMap::new(),
            None,
            &["boost".to_string()],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        count > 0,
        "the engine judged mutants, so the pass carries its evidence; got {count}"
    );
    assert!(
        survivors.is_empty(),
        "the gated module's integration test kills every mutant; got {survivors:?}"
    );
}

#[test]
fn a_baseline_that_cannot_build_is_an_error_not_a_vacuous_pass() {
    // The same crate with the feature left off: `tests/boost.rs` names an item that is
    // compiled out, so the unmutated baseline never builds and no mutant is judged. That
    // is a hard error — reporting it as a `Tested { count: 0 }` pass would read exactly
    // like an all-killed run.
    let err = measure_rust(&member(), &[], &BTreeMap::new(), None, &[])
        .expect_err("a baseline that cannot build fails the measurement");
    assert!(
        err.to_string().contains("did not run cleanly"),
        "the failure names the baseline build; got: {err}"
    );
}
