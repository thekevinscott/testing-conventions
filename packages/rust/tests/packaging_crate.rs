//! Integration tests for the Rust packaging slice (#74): exercise the
//! `packaging::inspect` library API over pre-built `cargo package` **crate
//! tarball** fixtures (`.crate`). Rule (README "Packaging"): the source tarball
//! must not ship the crate-root `tests/` directory.
//!
//! The e2e suite (`packaging_crate_e2e.rs`) drives the same crates through the
//! built binary; this checks the library contract — `inspect` unpacks the
//! `.crate` (a gzipped tar) and the `tests/` directory pattern flags files under
//! the crate-root `tests/`.

use std::path::PathBuf;

use testing_conventions::packaging;

fn crate_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging/rust_crate")
        .join(name)
}

#[test]
fn a_crate_shipping_the_tests_dir_is_flagged() {
    let offenders =
        packaging::inspect(crate_fixture("widget-0.1.0.crate"), &["tests/".to_string()]).unwrap();
    assert_eq!(
        offenders,
        vec![PathBuf::from("widget-0.1.0/tests/integration.rs")]
    );
}

#[test]
fn a_clean_crate_has_no_offenders() {
    let offenders =
        packaging::inspect(crate_fixture("clean-0.1.0.crate"), &["tests/".to_string()]).unwrap();
    assert!(offenders.is_empty());
}
