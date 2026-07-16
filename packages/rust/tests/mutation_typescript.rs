//! Integration tests for the TypeScript mutation rule.
//!
//! These run REAL Stryker over the fixture projects via the SDK
//! ([`mutation::measure_typescript`]) — which spawns the bundled Node adapter — and
//! assert the surviving-mutant set, the TS parallel of the Rust vertical. The
//! *projects themselves* are the fixtures: `killed` (every mutant caught by an
//! asserting test) reports no survivors, and `survivors` (an assertion-light test that runs
//! the code but pins nothing) reports several — the gap mutation testing exposes that
//! coverage can't.
//!
//! The fixtures are **runner-only**: they install just vitest. That the gate still runs
//! Stryker over them proves the tool bundles and drives the engine; the
//! project provides only its own test runner. Each test runs against its own staged copy
//! (vitest `node_modules` symlinked) so the parallel in-place Stryker runs never collide, and
//! passes the freshly-built adapter path ([`common::ts_adapter`]) straight to the rule.
//! Requires the built node adapter and the fixtures' vitest (`npm ci` in
//! `tests/fixtures/unit_mutation/typescript`).

mod common;

use common::{expect_tested, ts_adapter, Staged};
use testing_conventions::mutation::measure_typescript;

#[test]
fn killed_reports_no_survivors() {
    let project = Staged::new("killed");
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
fn survivors_are_reported() {
    // The fixture installs only vitest, yet the gate runs Stryker over it via the bundled
    // adapter and finds the assertion-light suite's survivors. The tool drives
    // the engine; the project supplies only its test runner.
    let project = Staged::new("survivors");
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
fn a_src_scan_path_with_an_upward_import_reports_scan_relative_survivors() {
    // The standard package layout: `{package.json, src/**}`, scanned at `src/`, where a
    // source imports `../package.json`. The gate roots Stryker's in-place run at the package
    // root (so the upward import resolves), mutates only the scan path, judges mutants by
    // the scan path's colocated suite alone (the fixture's `tests/` tier fails loudly if
    // ever run), and reports survivors scan-path-relative.
    let package = Staged::upward("upward_survivors");
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
fn a_src_scan_path_with_an_upward_import_passes_when_all_mutants_are_killed() {
    // The killed twin: the same layout clears the gate — the package-root run resolves
    // `../package.json`, and every mutant under the scan path is caught.
    let package = Staged::upward("upward_killed");
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
fn a_mutation_exemption_drops_the_survivors() {
    // Exempting the survivors' file lifts all of them — an equivalent / deliberately
    // defensive mutation, waived with a reason via `[[typescript.exempt]] rules = ["mutation"]`.
    let project = Staged::new("survivors");
    let exempt = vec!["index.ts".to_string()];
    let (_, survivors) = expect_tested(
        measure_typescript(
            project.path(),
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
