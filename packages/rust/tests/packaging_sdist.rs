//! Integration tests for the Python sdist coverage slice: exercise
//! `packaging::inspect` over pre-built Python **sdist** fixtures (`.tar.gz`).
//! See `packaging_sdist_e2e.rs` for the binary-level checks and the note on why
//! this slice has no red phase (the behavior already shipped).

use std::path::PathBuf;

use testing_conventions::packaging;

fn sdist(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging/python_sdist")
        .join(name)
}

#[test]
fn an_sdist_shipping_a_test_file_is_flagged() {
    let offenders =
        packaging::inspect(sdist("widget-0.1.0.tar.gz"), &["*_test.py".to_string()]).unwrap();
    assert_eq!(
        offenders,
        vec![PathBuf::from("widget-0.1.0/widget/core_test.py")]
    );
}

#[test]
fn a_clean_sdist_has_no_offenders() {
    let offenders =
        packaging::inspect(sdist("clean-0.1.0.tar.gz"), &["*_test.py".to_string()]).unwrap();
    assert!(offenders.is_empty());
}
