mod common;

use common::{expect_tested, Staged};
use testing_conventions::mutation::measure_python;

#[test]
fn killed_reports_no_survivors() {
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
    assert!(
        survivors.iter().all(|m| m.description.contains("(-> ")),
        "each survivor names the source its mutation produced; got {survivors:?}"
    );
}

#[test]
fn a_test_file_nested_below_the_scan_path_is_never_mutated() {
    let package = Staged::python_nested("nested_tests");
    let (count, survivors) = expect_tested(
        measure_python(
            &package.path().join("src"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
        )
        .expect("cosmic-ray runs"),
    );
    assert!(count > 0, "the engine ran, so the count is non-zero");
    assert!(
        survivors
            .iter()
            .all(|m| !m.file.ends_with("_test.py") && !m.file.starts_with("test_")),
        "the suite judges the mutants and is never mutated itself; got {survivors:?}"
    );
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn a_loose_tree_with_no_manifest_reports_root_relative_survivors() {
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
