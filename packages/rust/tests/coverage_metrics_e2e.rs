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
fn an_uncalled_function_fails_a_functions_floor() {
    let out = run("funcs", "rust_functions_full.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("coverage below thresholds"),
        "expected a floor failure, got: {}",
        stderr(&out)
    );
}

#[test]
fn the_same_functions_coverage_clears_a_lower_floor() {
    let out = run("funcs", "rust_functions_mid.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn the_branch_floor_gates_the_measured_branches() {
    // One test drives both runs: rustup installs the fixture's pinned nightly on first use,
    // and two tests racing that install corrupt each other's downloads.
    let out = run("branchy", "rust_branch_full.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("coverage below thresholds"),
        "expected a floor failure, got: {}",
        stderr(&out)
    );

    let out = run("branchy", "rust_branch_mid.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn a_branch_floor_on_a_stable_toolchain_names_the_nightly_requirement() {
    let out = run("below", "rust_branch_full.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("nightly"),
        "expected the nightly requirement to be named, got: {}",
        stderr(&out)
    );
}
