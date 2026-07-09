//! E2E tests for the TypeScript integration-isolation lint: drive
//! the built CLI binary end-to-end (no mocks) against the fixture codebases and
//! assert the exit code.

use std::path::PathBuf;
use std::process::Command;

/// Absolute path to a fixture tree under
/// `tests/fixtures/integration_lint/typescript/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/integration_lint/typescript")
        .join(name)
}

/// Exit code of `testing-conventions integration lint --language typescript <codebase>`.
fn lint_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["integration", "lint", "--language", "typescript"])
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(lint_exit("no_first_party_mock/red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(lint_exit("no_first_party_mock/clean"), 0);
}

// The suite tiers derive from the package root, so the binary scanning the
// package's source directory still lints the sibling suites.

#[test]
fn tier_layout_integration_suite_is_linted_from_a_src_scan() {
    assert_eq!(lint_exit("tier_layout/red/src"), 1);
}

#[test]
fn tier_layout_test_outside_a_standard_tier_is_flagged() {
    assert_eq!(lint_exit("tier_layout/unknown_tier/src"), 1);
}
