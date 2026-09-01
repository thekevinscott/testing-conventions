mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{ts_adapter, Staged};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Run `unit mutation --language <lang> --config <cfg> <project>` and capture output. The
/// bundled TS adapter path is passed as `--ts-mutation-adapter` exactly as the npm launcher
/// does (the Rust / Python arms ignore it).
fn run(language: &str, project: &Path, config: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", language, "--config"])
        .arg(fixtures().join(config))
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter())
        .arg(project)
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
fn rust_exempting_the_survivor_line_passes() {
    // Line 7 (`n > 0`) is where every mutant survives; lifting just that line clears
    // the gate.
    let out = run(
        "rust",
        &fixtures().join("rust").join("survivors"),
        "lines_mut_rust_ok.toml",
    );
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn rust_over_exempting_a_caught_line_is_a_hard_error() {
    // In the killed crate line 6's mutants are all caught, so listing it is rejected.
    let out = run(
        "rust",
        &fixtures().join("rust").join("killed"),
        "lines_mut_rust_over.toml",
    );
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("all caught"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}

#[test]
fn typescript_exempting_both_survivor_lines_passes() {
    // Lines 2 and 6 carry the survivors; lifting both clears the gate.
    let project = Staged::loose("loose_survivors");
    let out = run("typescript", project.path(), "lines_mut_ts_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn typescript_under_listing_still_fails() {
    // Exempting only line 6 leaves line 2's survivor unexplained, so the gate fails.
    let project = Staged::loose("loose_survivors");
    let out = run("typescript", project.path(), "lines_mut_ts_under.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("unexplained"),
        "expected the unlisted survivor to fail the gate, got: {}",
        stderr(&out)
    );
}

#[test]
fn typescript_over_exempting_a_caught_line_is_a_hard_error() {
    // In the killed project line 2's mutants are all caught, so listing it is rejected.
    let project = Staged::loose("loose_killed");
    let out = run("typescript", project.path(), "lines_mut_ts_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("all caught"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}

#[test]
fn python_exempting_both_survivor_lines_passes() {
    // Lines 2 and 6 carry the survivors; lifting both clears the gate.
    let project = Staged::python_loose("loose_survivors");
    let out = run("python", project.path(), "lines_mut_py_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn python_over_exempting_a_caught_line_is_a_hard_error() {
    // In the killed project line 2's mutants are all caught, so listing it is rejected.
    let project = Staged::python_loose("loose_killed");
    let out = run("python", project.path(), "lines_mut_py_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("all caught"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}
