//! Integration tests for the Python coverage rule (#26): parse a coverage.py
//! JSON report and enforce the configured floor.
//!
//! Per the #3 guardrail, a clean fixture report (meets the floor — must pass)
//! and a red fixture report (below the floor — must fail) drive the check. These
//! are coverage.py reports, so the deterministic enforcement is exercised here
//! without needing a Python toolchain in CI.

use std::path::PathBuf;

use testing_conventions::coverage::{evaluate, parse_report, Outcome, Thresholds};

/// Read a coverage report fixture under `tests/fixtures/unit_coverage/python/`.
fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_coverage/python")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("fixture `{}` must exist", path.display()))
}

const FLOOR: Thresholds = Thresholds {
    fail_under: 100,
    branch: true,
};

#[test]
fn clean_report_meets_the_floor() {
    let report = parse_report(&fixture("clean.json")).expect("a valid coverage.py report");
    assert_eq!(evaluate(&report, FLOOR), Outcome::Pass);
}

#[test]
fn red_report_is_below_the_floor() {
    let report = parse_report(&fixture("red.json")).expect("a valid coverage.py report");
    assert!(
        matches!(evaluate(&report, FLOOR), Outcome::Fail(_)),
        "80% coverage must fail a 100% floor"
    );
}
