mod common;

use common::{expect_tested, ts_adapter, Staged};
use testing_conventions::mutation::measure_typescript;

#[test]
fn killed_reports_no_survivors() {
    let package = Staged::new("killed");
    let (_, survivors) = expect_tested(
        measure_typescript(
            &package.path().join("src"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &ts_adapter(),
        )
        .expect("stryker runs"),
    );
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn survivors_are_reported() {
    let package = Staged::new("survivors");
    let (_, survivors) = expect_tested(
        measure_typescript(
            &package.path().join("src"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &ts_adapter(),
        )
        .expect("stryker runs"),
    );
    assert!(
        !survivors.is_empty(),
        "the assertion-light suite should leave survivors under the scan path"
    );
    assert!(
        survivors.iter().all(|m| m.file == "index.ts"),
        "survivors are reported relative to the scan path; got {survivors:?}"
    );
}

#[test]
fn a_package_root_relative_vitest_include_still_reaches_the_colocated_suite() {
    let package = Staged::configured("config_include");
    let (count, survivors) = expect_tested(
        measure_typescript(
            &package.path().join("src"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &ts_adapter(),
        )
        .expect("the colocated suite judges the mutants"),
    );
    assert!(
        count > 0,
        "the config's `include` resolves against the package root, so the suite runs"
    );
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn a_loose_tree_with_no_manifest_reports_root_relative_survivors() {
    let project = Staged::loose("loose_survivors");
    let (_, survivors) = expect_tested(
        measure_typescript(
            project.path(),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &ts_adapter(),
        )
        .expect("stryker runs"),
    );
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
fn a_loose_tree_with_no_manifest_passes_when_all_mutants_are_killed() {
    let project = Staged::loose("loose_killed");
    let (_, survivors) = expect_tested(
        measure_typescript(
            project.path(),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &ts_adapter(),
        )
        .expect("stryker runs"),
    );
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn a_mutation_exemption_drops_the_survivors() {
    let package = Staged::new("survivors");
    let exempt = vec!["index.ts".to_string()];
    let (_, survivors) = expect_tested(
        measure_typescript(
            &package.path().join("src"),
            &exempt,
            &std::collections::BTreeMap::new(),
            None,
            &ts_adapter(),
        )
        .expect("stryker runs"),
    );
    assert!(
        survivors.is_empty(),
        "the exemption should drop every survivor; got {survivors:?}"
    );
}
