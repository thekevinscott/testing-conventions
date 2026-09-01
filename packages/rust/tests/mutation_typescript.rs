mod common;

use common::{expect_tested, ts_adapter, Staged};
use testing_conventions::mutation::measure_typescript;

#[test]
fn killed_reports_no_survivors() {
    // The default package layout clears the gate: the package-root run resolves
    // `../package.json`, and every mutant under the `src/` scan path is caught.
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
    // The default package layout with an assertion-light suite. The fixture installs only
    // vitest, yet the gate runs Stryker over it via the bundled adapter, roots the sandbox at
    // the package root (so `../package.json` resolves), mutates only the `src/` scan path, and
    // reports its survivors scan-path-relative.
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
fn a_loose_tree_with_no_manifest_reports_root_relative_survivors() {
    // The loose special case: flat scripts, no manifest, sources at the scanned root. The
    // gate runs Stryker in place at that root and reports survivors root-relative.
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
    // The loose killed twin: the same flat shape clears the gate.
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
    // Exempting the survivors' file lifts all of them — an equivalent / deliberately
    // defensive mutation, waived with a reason via `[[typescript.exempt]] rules = ["mutation"]`.
    // The exempt path is scan-path-relative, matching the reported survivors.
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
