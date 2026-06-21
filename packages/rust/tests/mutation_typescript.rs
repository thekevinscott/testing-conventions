//! Integration tests for the TypeScript mutation rule (#202).
//!
//! These run REAL Stryker over the fixture projects via the SDK
//! ([`mutation::measure_typescript`]) and assert the surviving-mutant set — the TS
//! parallel of the Rust vertical (#201). Per the #3 guardrail the *projects
//! themselves* are the fixtures: `killed` (every mutant caught by an asserting test)
//! reports no survivors, and `survivors` (an assertion-light test that runs the code
//! but pins nothing) reports several — the gap mutation testing exposes that coverage
//! can't. Requires the fixtures' Stryker toolchain (`npm ci` in
//! `tests/fixtures/unit_mutation/typescript`).

use std::path::PathBuf;

use testing_conventions::mutation::measure_typescript;

fn project(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_mutation/typescript")
        .join(name)
}

#[test]
fn killed_reports_no_survivors() {
    let survivors = measure_typescript(&project("killed"), &[], None).expect("stryker runs");
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn survivors_are_reported() {
    let survivors = measure_typescript(&project("survivors"), &[], None).expect("stryker runs");
    assert!(
        !survivors.is_empty(),
        "the assertion-light suite should leave survivors"
    );
    assert!(
        survivors.iter().all(|m| m.file == "index.ts"),
        "every survivor is in index.ts; got {survivors:?}"
    );
}

#[test]
fn a_mutation_exemption_drops_the_survivors() {
    // Exempting the survivors' file lifts all of them — an equivalent / deliberately
    // defensive mutation, waived with a reason via `[[typescript.exempt]] rules = ["mutation"]`.
    let exempt = vec!["index.ts".to_string()];
    let survivors = measure_typescript(&project("survivors"), &exempt, None).expect("stryker runs");
    assert!(
        survivors.is_empty(),
        "the exemption should drop every survivor; got {survivors:?}"
    );
}
