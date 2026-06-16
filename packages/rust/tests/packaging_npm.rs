//! Integration tests for the TypeScript packaging slice (#73): exercise the
//! `packaging::inspect` library API over the pre-built `npm pack` **tarball**
//! fixtures (`.tgz`). Rule (README "Packaging"): test files must never ship in
//! the built artifact.
//!
//! The e2e suite (`packaging_npm_e2e.rs`) drives the same tarballs through the
//! built binary; this checks the library contract — `inspect` unpacks the
//! gzipped tar and reports offenders relative to the artifact root.

use std::path::PathBuf;

use testing_conventions::packaging;

fn tarball(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging/typescript_npm")
        .join(name)
}

#[test]
fn a_tarball_shipping_a_test_file_is_flagged() {
    let offenders = packaging::inspect(tarball("red.tgz"), &["*.test.*".to_string()]).unwrap();
    assert_eq!(
        offenders,
        vec![PathBuf::from("package/dist/widget.test.js")]
    );
}

#[test]
fn a_clean_tarball_has_no_offenders() {
    let offenders = packaging::inspect(tarball("clean.tgz"), &["*.test.*".to_string()]).unwrap();
    assert!(offenders.is_empty());
}
