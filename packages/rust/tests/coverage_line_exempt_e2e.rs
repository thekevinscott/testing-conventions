use std::path::PathBuf;
use std::process::{Command, Output};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage")
}

/// Run `unit coverage --language <lang> --config <cfg> <lang>/<codebase>` and return
/// the captured output (exit code + stderr).
fn run(language: &str, codebase: &str, config: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", language, "--config"])
        .arg(fixtures().join(config))
        .arg(fixtures().join(language).join(codebase))
        .output()
        .expect("the built binary should run")
}

fn code(output: &Output) -> i32 {
    output
        .status
        .code()
        .expect("the process should exit with a code")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn python_minimal_line_exemption_clears_the_floor() {
    let out = run("python", "exempt_cov", "lines_py_shim_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn python_over_exemption_is_a_hard_error() {
    let out = run("python", "exempt_cov", "lines_py_shim_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("uncovered lines"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}

#[test]
fn python_under_listing_still_fails_the_floor() {
    let out = run("python", "exempt_cov", "lines_py_shim_under.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("is below"),
        "expected a floor failure, got: {}",
        stderr(&out)
    );
}

#[test]
fn rust_minimal_line_exemption_clears_the_floor() {
    let out = run("rust", "exempt_cov", "lines_rust_shim_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn rust_over_exemption_is_a_hard_error() {
    let out = run("rust", "exempt_cov", "lines_rust_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("uncovered lines"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}

#[test]
fn typescript_minimal_line_exemption_clears_the_floor() {
    let out = run("typescript", "exempt_cov", "lines_ts_shim_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn typescript_over_exemption_is_a_hard_error() {
    let out = run("typescript", "exempt_cov", "lines_ts_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("uncovered lines"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}
