//! Integration tests for the packaging rule's foundation: exercise the
//! `packaging::scan` library API over fixture "artifact" trees. The rule (README
//! "Packaging"): test files must never ship in a built artifact. Each fixture
//! stands in for an unpacked built artifact (a wheel, a `dist/`).
//!
//! The e2e suite (`packaging_e2e.rs`) drives the same fixtures through the built
//! binary; this checks the library contract directly.

use std::path::PathBuf;

use testing_conventions::packaging;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging")
        .join(name)
}

#[test]
fn python_artifact_with_a_test_file_is_flagged() {
    let offenders = packaging::scan(fixture("python_red"), &["*_test.py".to_string()]).unwrap();
    assert_eq!(
        offenders,
        vec![fixture("python_red").join("widget_test.py")]
    );
}

#[test]
fn clean_python_artifact_has_no_offenders() {
    let offenders = packaging::scan(fixture("python_clean"), &["*_test.py".to_string()]).unwrap();
    assert!(offenders.is_empty());
}

#[test]
fn typescript_artifact_with_a_test_file_is_flagged() {
    let offenders = packaging::scan(fixture("typescript_red"), &["*.test.*".to_string()]).unwrap();
    assert_eq!(
        offenders,
        vec![fixture("typescript_red").join("button.test.ts")]
    );
}

#[test]
fn clean_typescript_artifact_has_no_offenders() {
    let offenders =
        packaging::scan(fixture("typescript_clean"), &["*.test.*".to_string()]).unwrap();
    assert!(offenders.is_empty());
}
