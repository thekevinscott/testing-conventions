//! Integration tests for the Python coverage rule (#26).
//!
//! These run REAL coverage.py over the fixture codebases via the SDK
//! (`coverage::measure`) and assert pass/fail. Per the #3 guardrail the
//! *codebases themselves* are the fixtures: `full` (100% branch) clears a 100
//! floor, `above_85` (~86%) fails 100, `below_85` (~71%) fails 85. Requires
//! `coverage` + `pytest` on PATH.

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

#[test]
fn below_85_fails_an_85_floor() {
    assert!(matches!(
        measure(&codebase("below_85"), FLOOR_85).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn above_85_fails_a_100_floor() {
    assert!(matches!(
        measure(&codebase("above_85"), FLOOR_100).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn full_passes_a_100_floor() {
    assert_eq!(
        measure(&codebase("full"), FLOOR_100).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_suite_that_cannot_run_is_an_error_not_a_silent_pass() {
    // An empty directory collects no tests; measuring it must error rather than
    // report a vacuous pass.
    let empty = std::env::temp_dir().join(format!("tc-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let result = measure(&empty, FLOOR_85);
    let _ = std::fs::remove_dir_all(&empty);
    assert!(result.is_err());
}
