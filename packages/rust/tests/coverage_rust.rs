//! Integration tests for the Rust coverage rule (#37).
//!
//! These run REAL `cargo llvm-cov` over the fixture crates via the SDK
//! (`coverage::measure_rust`) and assert pass/fail. Per the #3 guardrail the
//! *crates themselves* are the fixtures: `above` (every region and line
//! exercised by colocated inline tests) clears a 100 floor, `below` (one branch
//! arm left uncovered) fails 100 but clears a lower floor, and `exempt_cov`
//! clears 100 only once its untested shim is omitted by a `coverage` exemption.
//! Requires `cargo-llvm-cov`.

use std::path::PathBuf;

use testing_conventions::coverage::{measure_rust, Outcome, RustThresholds};

fn crate_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_coverage/rust")
        .join(name)
}

const FULL: RustThresholds = RustThresholds {
    regions: Some(100),
    lines: 100,
};
const MID: RustThresholds = RustThresholds {
    regions: Some(80),
    lines: 80,
};

#[test]
fn above_passes_a_100_floor() {
    assert_eq!(
        measure_rust(&crate_dir("above"), FULL, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn below_fails_a_100_floor() {
    assert!(matches!(
        measure_rust(&crate_dir("below"), FULL, &[]).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn below_passes_a_lower_floor() {
    // `below` is ~88% regions / ~87% lines — under 100 (the uncovered `else` arm)
    // but comfortably over an 80 floor, so the floor is a real, configurable knob.
    assert_eq!(
        measure_rust(&crate_dir("below"), MID, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn integration_tests_do_not_pad_the_unit_floor() {
    // `padded`'s `shift` unit is exercised only by the crate's integration test
    // (`tests/covers_shift.rs`); the floor measures the unit suite, so the crate
    // reads ~70% regions / ~67% lines and fails 100 (#265). A run that also
    // counted the integration target would read 100% and pass — exactly the
    // padding the Coverage rule forbids.
    assert!(matches!(
        measure_rust(&crate_dir("padded"), FULL, &[]).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn a_coverage_exemption_omits_the_file_and_lets_the_floor_pass() {
    // `exempt_cov` sits at ~75% only because of shim.rs (its `launch` is never
    // exercised); omitting it — the `coverage`-rule exemption the CLI resolves
    // from config — leaves core.rs, fully covered, to clear 100. Without the
    // exemption this crate fails the floor (#32).
    assert_eq!(
        measure_rust(&crate_dir("exempt_cov"), FULL, &["src/shim.rs".to_string()]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_suite_that_cannot_run_is_an_error_not_a_silent_pass() {
    // An empty directory is not a cargo crate; `cargo llvm-cov` exits non-zero, so
    // measuring it must error rather than report a vacuous pass.
    let empty = std::env::temp_dir().join(format!("tc-rust-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let result = measure_rust(&empty, MID, &[]);
    let _ = std::fs::remove_dir_all(&empty);
    assert!(result.is_err());
}
