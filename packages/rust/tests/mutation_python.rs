//! Integration tests for the Python mutation rule (#203).
//!
//! These run REAL cosmic-ray over the fixture projects via the SDK
//! ([`mutation::measure_python`]) and assert the surviving-mutant set — the Python
//! parallel of the Rust (#201) and TypeScript (#202) arms. Per the #3 guardrail the
//! *projects themselves* are the fixtures: `killed` (every mutant caught by an asserting
//! test) reports no survivors, and `survivors` (an assertion-light test that runs the
//! code but pins nothing) reports several — the gap mutation testing exposes that
//! coverage can't.
//!
//! Each test runs against its own staged copy (cosmic-ray mutates in place) so the
//! parallel runs never collide. Requires cosmic-ray + pytest on PATH.

mod common;

use common::Staged;
use testing_conventions::mutation::measure_python;

#[test]
fn killed_reports_no_survivors() {
    let project = Staged::python("killed");
    let survivors = measure_python(project.path(), &[], None).expect("cosmic-ray runs");
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn survivors_are_reported() {
    let project = Staged::python("survivors");
    let survivors = measure_python(project.path(), &[], None).expect("cosmic-ray runs");
    assert!(
        !survivors.is_empty(),
        "the assertion-light suite should leave survivors"
    );
    assert!(
        survivors.iter().all(|m| m.file == "calc.py"),
        "every survivor is in calc.py; got {survivors:?}"
    );
}

#[test]
fn a_mutation_exemption_drops_the_survivors() {
    // Exempting the survivors' file lifts all of them — an equivalent / deliberately
    // defensive mutation, waived with a reason via `[[python.exempt]] rules = ["mutation"]`.
    let project = Staged::python("survivors");
    let exempt = vec!["calc.py".to_string()];
    let survivors = measure_python(project.path(), &exempt, None).expect("cosmic-ray runs");
    assert!(
        survivors.is_empty(),
        "the exemption should drop every survivor; got {survivors:?}"
    );
}
