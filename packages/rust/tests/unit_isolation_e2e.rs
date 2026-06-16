//! E2E tests for the TypeScript unit-isolation check (#43 / #76): drive the
//! built CLI binary end-to-end (no mocks) against the fixtures and assert the
//! exit code.

use std::path::PathBuf;
use std::process::Command;

/// Absolute path to a fixture tree under `tests/fixtures/unit_isolation/typescript/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_isolation/typescript")
        .join(name)
}

/// Exit code of `testing-conventions unit isolation --language typescript <codebase>`.
fn isolation_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "isolation", "--language", "typescript"])
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(isolation_exit("red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(isolation_exit("clean"), 0);
}

// typed `vi.mock` (#77)
#[test]
fn untyped_red_exits_nonzero() {
    assert_eq!(isolation_exit("untyped_mock/red"), 1);
}

#[test]
fn untyped_clean_exits_zero() {
    assert_eq!(isolation_exit("untyped_mock/clean"), 0);
}
