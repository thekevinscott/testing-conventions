mod common;

use common::{expect_tested, Staged};
use testing_conventions::mutation::measure_python;

#[test]
fn killed_reports_no_survivors() {
    // The default package layout clears the gate: every mutant under the `src/` scan path is
    // caught by the colocated suite.
    let package = Staged::python("killed");
    let (_, survivors) = expect_tested(
        measure_python(
            &package.path().join("src"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
        )
        .expect("cosmic-ray runs"),
    );
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn survivors_are_reported() {
    // The default package layout with an assertion-light suite: the gate mutates only the
    // `src/` scan path and reports its survivors scan-path-relative.
    let package = Staged::python("survivors");
    let (_, survivors) = expect_tested(
        measure_python(
            &package.path().join("src"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
        )
        .expect("cosmic-ray runs"),
    );
    assert!(
        !survivors.is_empty(),
        "the assertion-light suite should leave survivors under the scan path"
    );
    assert!(
        survivors.iter().all(|m| m.file == "calc.py"),
        "survivors are reported relative to the scan path; got {survivors:?}"
    );
}

#[test]
fn a_loose_tree_with_no_manifest_reports_root_relative_survivors() {
    // The loose special case: flat scripts, no manifest, sources at the scanned root. The
    // gate runs cosmic-ray in place at that root and reports survivors root-relative.
    let project = Staged::python_loose("loose_survivors");
    let (_, survivors) = expect_tested(
        measure_python(
            project.path(),
            &[],
            &std::collections::BTreeMap::new(),
            None,
        )
        .expect("cosmic-ray runs"),
    );
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
fn a_loose_tree_with_no_manifest_passes_when_all_mutants_are_killed() {
    // The loose killed twin: the same flat shape clears the gate.
    let project = Staged::python_loose("loose_killed");
    let (_, survivors) = expect_tested(
        measure_python(
            project.path(),
            &[],
            &std::collections::BTreeMap::new(),
            None,
        )
        .expect("cosmic-ray runs"),
    );
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn a_mutation_exemption_drops_the_survivors() {
    // Exempting the survivors' file lifts all of them — an equivalent / deliberately
    // defensive mutation, waived with a reason via `[[python.exempt]] rules = ["mutation"]`.
    // The exempt path is scan-path-relative, matching the reported survivors.
    let package = Staged::python("survivors");
    let exempt = vec!["calc.py".to_string()];
    let (_, survivors) = expect_tested(
        measure_python(
            &package.path().join("src"),
            &exempt,
            &std::collections::BTreeMap::new(),
            None,
        )
        .expect("cosmic-ray runs"),
    );
    assert!(
        survivors.is_empty(),
        "the exemption should drop every survivor; got {survivors:?}"
    );
}
