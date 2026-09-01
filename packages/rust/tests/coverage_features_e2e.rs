use std::path::PathBuf;
use std::process::{Command, Output};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage")
}

/// Run `unit coverage --language rust --config <cfg> rust/<crate>` and return the
/// captured output (exit code + stderr).
fn run(crate_name: &str, config: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", "rust", "--config"])
        .arg(fixtures().join(config))
        .arg(fixtures().join("rust").join(crate_name))
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
fn a_covered_feature_gated_module_clears_the_floor() {
    let out = run("gated", "rust_features_full.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn an_untested_feature_gated_module_fails_the_floor() {
    let out = run("gated_untested", "rust_features_full.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("coverage below thresholds"),
        "expected a floor failure, got: {}",
        stderr(&out)
    );
}
