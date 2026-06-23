//! Integration tests for the TypeScript mutation rule (#202).
//!
//! These run REAL Stryker over the fixture projects via the SDK
//! ([`mutation::measure_typescript`]) and assert the surviving-mutant set — the TS
//! parallel of the Rust vertical (#201). Per the #3 guardrail the *projects
//! themselves* are the fixtures: `killed` (every mutant caught by an asserting test)
//! reports no survivors, and `survivors` (an assertion-light test that runs the code
//! but pins nothing) reports several — the gap mutation testing exposes that coverage
//! can't.
//!
//! Each test runs against its own staged copy of the fixture (node_modules symlinked to
//! the shared toolchain) so Stryker's in-project report/sandbox never collides between
//! the parallel tests. Requires the fixtures' Stryker toolchain (`npm ci` in
//! `tests/fixtures/unit_mutation/typescript`).

mod common;

use common::Staged;
use testing_conventions::mutation::measure_typescript;

#[test]
fn killed_reports_no_survivors() {
    let project = Staged::new("killed");
    let survivors = measure_typescript(
        project.path(),
        &[],
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect("stryker runs");
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn survivors_are_reported() {
    let project = Staged::new("survivors");
    let survivors = measure_typescript(
        project.path(),
        &[],
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect("stryker runs");
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
fn a_missing_toolchain_fails_clean_without_downloading() {
    // No `node_modules`: the TS arm must surface a clear error via `npx --no-install`
    // and never silently fetch the long-deprecated standalone `stryker` package (renamed
    // to `@stryker-mutator/core` in 2019). Parity with the cosmic-ray / cargo-mutants
    // arms, which invoke the binary directly and fail clean when it's absent.
    let project = Staged::typescript_without_toolchain("killed");
    let err = measure_typescript(
        project.path(),
        &[],
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect_err("a project with no Stryker installed must error, not download one");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("npx --no-install"),
        "the error should name the no-download invocation; got: {msg}"
    );
}

#[test]
fn a_mutation_exemption_drops_the_survivors() {
    // Exempting the survivors' file lifts all of them — an equivalent / deliberately
    // defensive mutation, waived with a reason via `[[typescript.exempt]] rules = ["mutation"]`.
    let project = Staged::new("survivors");
    let exempt = vec!["index.ts".to_string()];
    let survivors = measure_typescript(
        project.path(),
        &exempt,
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect("stryker runs");
    assert!(
        survivors.is_empty(),
        "the exemption should drop every survivor; got {survivors:?}"
    );
}
