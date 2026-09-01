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
