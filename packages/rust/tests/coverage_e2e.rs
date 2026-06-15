//! E2E tests for the Python coverage rule (#26): drive the built CLI binary
//! end-to-end (no mocks) against the fixture codebases and assert the exit code.
//! Requires `coverage` + `pytest` on PATH.

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage")
}

/// Exit code of `testing-conventions unit coverage --language python --config <cfg> <codebase>`.
fn unit_coverage_exit(codebase: &str, config: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", "python", "--config"])
        .arg(fixtures().join(config))
        .arg(fixtures().join("python").join(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn below_85_exits_nonzero_against_an_85_floor() {
    assert_eq!(unit_coverage_exit("below_85", "floor85.toml"), 1);
}

#[test]
fn above_85_exits_nonzero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("above_85", "floor100.toml"), 1);
}

#[test]
fn full_exits_zero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("full", "floor100.toml"), 0);
}
