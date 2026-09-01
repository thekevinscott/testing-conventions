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
    functions: None,
    branch: None,
};
const MID: RustThresholds = RustThresholds {
    regions: Some(80),
    lines: 80,
    functions: None,
    branch: None,
};

#[test]
fn above_passes_a_100_floor() {
    assert_eq!(
        measure_rust(&crate_dir("above"), FULL, &[], &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn below_fails_a_100_floor() {
    assert!(matches!(
        measure_rust(&crate_dir("below"), FULL, &[], &[]).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn below_passes_a_lower_floor() {
    assert_eq!(
        measure_rust(&crate_dir("below"), MID, &[], &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn integration_tests_do_not_pad_the_unit_floor() {
    assert!(matches!(
        measure_rust(&crate_dir("padded"), FULL, &[], &[]).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn a_coverage_exemption_omits_the_file_and_lets_the_floor_pass() {
    assert_eq!(
        measure_rust(
            &crate_dir("exempt_cov"),
            FULL,
            &["src/shim.rs".to_string()],
            &[]
        )
        .unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_suite_that_cannot_run_is_an_error_not_a_silent_pass() {
    let empty = std::env::temp_dir().join(format!("tc-rust-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let result = measure_rust(&empty, MID, &[], &[]);
    let _ = std::fs::remove_dir_all(&empty);
    assert!(result.is_err());
}
