use std::path::PathBuf;
use std::process::Command;

fn sdist(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging/python_sdist")
        .join(name)
}

/// Exit code of `testing-conventions packaging <sdist> --language python`.
fn packaging_exit(artifact: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("packaging")
        .arg(sdist(artifact))
        .args(["--language", "python"])
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn an_sdist_shipping_a_test_file_exits_nonzero() {
    assert_eq!(packaging_exit("widget-0.1.0.tar.gz"), 1);
}

#[test]
fn a_clean_sdist_exits_zero() {
    assert_eq!(packaging_exit("clean-0.1.0.tar.gz"), 0);
}
