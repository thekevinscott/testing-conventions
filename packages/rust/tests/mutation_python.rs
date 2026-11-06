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
fn a_helper_module_under_tests_is_never_mutated() {
    let project = Staged::python_loose_test_helpers("loose_test_helpers");
    let (count, survivors) = expect_tested(
        measure_python(
            project.path(),
            &[],
            &std::collections::BTreeMap::new(),
            None,
        )
        .expect("cosmic-ray runs"),
    );
    assert!(count > 0, "calc.py is mutated, so the count is non-zero");
    assert!(
        survivors.is_empty(),
        "tests/integration/served.py is test code and yields no mutants; got {survivors:?}"
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
fn a_declaration_only_module_leaves_no_survivors() {
    let package = Staged::python_declaration_only("declaration_only");
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
        "settings.py has no function or control flow, so its mutants are not subjects; got {survivors:?}"
    );
}

#[test]
fn a_mutant_inside_an_annotation_leaves_no_survivors() {
    let package = Staged::python("annotations");
    let (count, survivors) = expect_tested(
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
        "an annotation is never evaluated, so its mutants are not judged; got {survivors:?}"
    );
    assert!(
        count > 0,
        "the body and the parameter default are still judged, so the count is non-zero"
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

#[test]
fn a_mutant_its_colocated_test_kills_never_runs_the_rest_of_the_suite() {
    let project = Staged::python_loose_with_neighbor("colocated_first");
    let (count, survivors) = expect_tested(
        measure_python(
            project.path(),
            &[],
            &std::collections::BTreeMap::new(),
            None,
        )
        .expect("cosmic-ray runs"),
    );
    assert!(count > 0, "calc.py is mutated, so the count is non-zero");
    assert!(
        survivors.is_empty(),
        "calc_test.py kills every mutant; got {survivors:?}"
    );
    let runs = std::fs::read_to_string(project.path().join("runs.log")).unwrap_or_default();
    assert_eq!(
        runs.lines().count(),
        1,
        "a_test.py runs only in the clean baseline, never for a mutant calc_test.py kills"
    );
}

#[test]
fn a_mutant_only_another_test_kills_is_still_killed() {
    let project = Staged::python_loose_with_neighbor("colocated_fallback");
    let (count, survivors) = expect_tested(
        measure_python(
            project.path(),
            &[],
            &std::collections::BTreeMap::new(),
            None,
        )
        .expect("cosmic-ray runs"),
    );
    assert!(count > 0, "calc.py is mutated, so the count is non-zero");
    assert!(
        survivors.is_empty(),
        "a mutant the colocated test misses falls back to the full suite; got {survivors:?}"
    );
}

#[test]
fn an_adapter_failure_surfaces_its_output() {
    let dir = std::env::temp_dir().join(format!("tc-mut-py-fail-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let err = measure_python(&dir, &[], &std::collections::BTreeMap::new(), None).unwrap_err();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        format!("{err:#}").contains("the Python mutation adapter failed"),
        "got: {err:#}"
    );
}
