//! Integration tests for the Python coverage rule.
//!
//! These run REAL coverage.py over the fixture codebases via the SDK
//! (`coverage::measure`) and assert pass/fail. Per the guardrail the
//! *codebases themselves* are the fixtures: `full` (100% branch) clears a 100
//! floor, `above_85` (~86%) fails 100, `below_85` (~71%) fails 85. Each is the
//! prescribed consumer package layout — `{pyproject.toml, src/**}` scanned at
//! `src/`. Requires `coverage` + `pytest` on PATH.

use std::path::PathBuf;

use testing_conventions::coverage::{measure, Outcome, Thresholds};

fn codebase(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_coverage/python")
        .join(name)
}

const FLOOR_85: Thresholds = Thresholds {
    fail_under: 85,
    branch: true,
};
const FLOOR_100: Thresholds = Thresholds {
    fail_under: 100,
    branch: true,
};

// `full`, `above_85`, and `below_85` are the default package layout —
// `{pyproject.toml, src/**}` scanned at `src/`. The package-root pyproject anchors
// pytest's rootdir so the colocated `<module>_test.py` resolves its `from <module>
// import ...` when coverage runs at the scan path.

#[test]
fn below_85_fails_an_85_floor() {
    assert!(matches!(
        measure(&codebase("below_85").join("src"), FLOOR_85, &[]).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn above_85_fails_a_100_floor() {
    assert!(matches!(
        measure(&codebase("above_85").join("src"), FLOOR_100, &[]).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn full_passes_a_100_floor() {
    assert_eq!(
        measure(&codebase("full").join("src"), FLOOR_100, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_package_root_conftest_governs_a_src_scan() {
    // The standard package layout scanned at `src/`: pytest resolves its rootdir and
    // conftest files with its own upward search, so the package-root `conftest.py`'s
    // fixture is available to the colocated suite below the scan path — the Python arm's
    // documented anchoring answer, pinned. The suite passes only if that fixture loads.
    assert_eq!(
        measure(&codebase("pkg_config").join("src"), FLOOR_100, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn conftest_is_omitted_from_the_denominator() {
    // conftest.py is pytest support, never a coverage subject. `conftest_omit`'s
    // widget.py is fully covered, but its conftest.py has an unused fixture body
    // (uncovered) — so the 100 floor passes only because conftest.py is omitted
    // from the denominator alongside the test files.
    assert_eq!(
        measure(&codebase("conftest_omit"), FLOOR_100, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_coverage_exemption_omits_the_file_and_lets_the_floor_pass() {
    // `exempt_cov` sits at ~58% only because of shim.py; omitting it (the
    // `coverage`-rule exemption the CLI resolves from config) leaves core.py,
    // fully covered, to clear 100. The exemption is doing real work — without it
    // this codebase fails the floor.
    assert_eq!(
        measure(&codebase("exempt_cov"), FLOOR_100, &["shim.py".to_string()]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_suite_that_cannot_run_is_an_error_not_a_silent_pass() {
    // An empty directory collects no tests; measuring it must error rather than
    // report a vacuous pass.
    let empty = std::env::temp_dir().join(format!("tc-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let result = measure(&empty, FLOOR_85, &[]);
    let _ = std::fs::remove_dir_all(&empty);
    assert!(result.is_err());
}
