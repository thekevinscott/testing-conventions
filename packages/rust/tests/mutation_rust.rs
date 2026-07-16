//! Integration tests for the Rust mutation rule.
//!
//! These run REAL `cargo mutants` over the fixture crates via the SDK
//! ([`mutation::measure_rust`]) and assert the surviving-mutant set. The *crates
//! themselves* are the fixtures: `killed` (every mutant caught
//! by an asserting test) reports no survivors, and `survivors` (an assertion-light
//! test that runs the code but pins nothing) reports several — the gap mutation
//! testing exposes that coverage can't. Requires only a cargo toolchain — the tool
//! provisions cargo-mutants itself.

mod common;

use std::path::PathBuf;

use common::expect_tested;
use testing_conventions::mutation::measure_rust;

fn crate_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_mutation/rust")
        .join(name)
}

#[test]
fn killed_reports_no_survivors_and_counts_the_tested_mutants() {
    let (count, survivors) = expect_tested(
        measure_rust(
            &crate_dir("killed"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
    assert!(count > 0, "the engine judged the crate's mutants");
}

#[test]
fn survivors_are_reported() {
    let (count, survivors) = expect_tested(
        measure_rust(
            &crate_dir("survivors"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        !survivors.is_empty(),
        "the assertion-light suite should leave survivors"
    );
    assert!(
        survivors.iter().all(|m| m.file == "src/lib.rs"),
        "every survivor is in src/lib.rs; got {survivors:?}"
    );
    assert!(
        count >= survivors.len(),
        "every survivor was judged, so the count covers them"
    );
}

#[test]
fn a_mutation_exemption_drops_the_survivors() {
    // Exempting the survivors' file lifts all of them — an equivalent / deliberately
    // defensive mutation, waived with a reason via `[[rust.exempt]] rules = ["mutation"]`.
    let exempt = vec!["src/lib.rs".to_string()];
    let (_, survivors) = expect_tested(
        measure_rust(
            &crate_dir("survivors"),
            &exempt,
            &std::collections::BTreeMap::new(),
            None,
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        survivors.is_empty(),
        "the exemption should drop every survivor; got {survivors:?}"
    );
}
