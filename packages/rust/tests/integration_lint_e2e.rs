//! E2E tests for the Python integration-test lints (#19): drive the built CLI
//! binary end-to-end (no mocks) against the fixture codebases and assert the
//! exit code.

use std::path::PathBuf;
use std::process::Command;

/// Absolute path to a fixture tree under `tests/fixtures/integration_lint/python/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/integration_lint/python")
        .join(name)
}

/// Exit code of `testing-conventions integration lint --language python <codebase>`.
fn lint_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["integration", "lint", "--language", "python"])
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn red_fixture_exits_nonzero() {
    assert_eq!(lint_exit("red"), 1);
}

#[test]
fn clean_fixture_exits_zero() {
    assert_eq!(lint_exit("clean"), 0);
}
