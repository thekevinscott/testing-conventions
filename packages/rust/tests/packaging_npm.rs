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
