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
