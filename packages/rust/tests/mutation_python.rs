//! Integration tests for the Python mutation rule.
//!
//! These run REAL cosmic-ray over the fixture projects via the SDK
//! ([`mutation::measure_python`]), which spawns the bundled Python adapter (`python3 -m
//! testing_conventions.mutation.main`) to drive cosmic-ray in-process, and assert the
//! surviving-mutant set — the Python parallel of the Rust and TypeScript arms. The
//! *projects themselves* are the fixtures: `killed` (every mutant caught by an asserting
//! test) reports no survivors, and `survivors` (an assertion-light test that runs the
//! code but pins nothing) reports several — the gap mutation testing exposes that
//! coverage can't.
//!
//! Each test runs against its own staged copy (cosmic-ray mutates in place) so the
//! parallel runs never collide. Requires a `python3` with cosmic-ray + pytest installed and
//! the source package importable (`PYTHONPATH=packages/python/python`).

mod common;

use common::Staged;
use testing_conventions::mutation::measure_python;

#[test]
fn killed_reports_no_survivors() {
    let project = Staged::python("killed");
    let survivors = measure_python(
        project.path(),
        &[],
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect("cosmic-ray runs");
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn survivors_are_reported() {
    let project = Staged::python("survivors");
    let survivors = measure_python(
        project.path(),
        &[],
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect("cosmic-ray runs");
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
    let survivors = measure_python(
        project.path(),
        &exempt,
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect("cosmic-ray runs");
    assert!(
        survivors.is_empty(),
        "the exemption should drop every survivor; got {survivors:?}"
    );
}
