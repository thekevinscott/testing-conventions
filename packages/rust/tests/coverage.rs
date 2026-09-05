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
    assert_eq!(
        measure(&codebase("pkg_config").join("src"), FLOOR_100, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn conftest_is_omitted_from_the_denominator() {
    assert_eq!(
        measure(&codebase("conftest_omit"), FLOOR_100, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_coverage_exemption_omits_the_file_and_lets_the_floor_pass() {
    assert_eq!(
        measure(&codebase("exempt_cov"), FLOOR_100, &["shim.py".to_string()]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn an_omit_that_swallows_every_source_is_an_error() {
    let err = measure(&codebase("full").join("src"), FLOOR_100, &["*".to_string()])
        .expect_err("with every file omitted the report step has nothing to report");
    assert!(format!("{err:#}").contains("coverage json"), "got: {err:#}");
}

#[test]
fn a_suite_that_cannot_run_is_an_error_not_a_silent_pass() {
    let empty = std::env::temp_dir().join(format!("tc-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let result = measure(&empty, FLOOR_85, &[]);
    let _ = std::fs::remove_dir_all(&empty);
    assert!(result.is_err());
}
