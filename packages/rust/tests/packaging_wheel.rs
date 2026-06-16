//! Integration tests for the Python packaging slice (#72): exercise the
//! `packaging::inspect` library API over the pre-built **wheel** fixtures. The
//! rule (README "Packaging"): test files must never ship in the built artifact.
//!
//! The e2e suite (`packaging_wheel_e2e.rs`) drives the same wheels through the
//! built binary; this checks the library contract — `inspect` unpacks the wheel
//! and reports offenders relative to the artifact root.

use std::path::PathBuf;

use testing_conventions::packaging;

fn wheel(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging/python_wheel")
        .join(name)
}

#[test]
fn a_wheel_shipping_a_test_file_is_flagged() {
    let offenders = packaging::inspect(wheel("red.whl"), &["*_test.py".to_string()]).unwrap();
    assert_eq!(offenders, vec![PathBuf::from("widget/core_test.py")]);
}

#[test]
fn a_clean_wheel_has_no_offenders() {
    let offenders = packaging::inspect(wheel("clean.whl"), &["*_test.py".to_string()]).unwrap();
    assert!(offenders.is_empty());
}
