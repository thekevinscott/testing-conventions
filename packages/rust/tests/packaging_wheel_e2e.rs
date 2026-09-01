use std::path::PathBuf;
use std::process::Command;

fn wheel(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging/python_wheel")
        .join(name)
}

/// Exit code of `testing-conventions packaging <wheel> --language python`.
fn packaging_exit(artifact: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("packaging")
        .arg(wheel(artifact))
        .args(["--language", "python"])
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn a_wheel_shipping_a_test_file_exits_nonzero() {
    assert_eq!(packaging_exit("red.whl"), 1);
}

#[test]
fn a_clean_wheel_exits_zero() {
    assert_eq!(packaging_exit("clean.whl"), 0);
}
