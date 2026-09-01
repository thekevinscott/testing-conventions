use std::path::PathBuf;
use std::process::Command;

fn crate_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging/rust_crate")
        .join(name)
}

/// Exit code of `testing-conventions packaging <crate> --language rust`.
fn packaging_exit(artifact: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("packaging")
        .arg(crate_fixture(artifact))
        .args(["--language", "rust"])
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn a_crate_shipping_the_tests_dir_exits_nonzero() {
    assert_eq!(packaging_exit("widget-0.1.0.crate"), 1);
}

#[test]
fn a_clean_crate_exits_zero() {
    assert_eq!(packaging_exit("clean-0.1.0.crate"), 0);
}
