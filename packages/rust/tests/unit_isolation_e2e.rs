//! E2E tests for the TypeScript unit-isolation check: drive the
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

/// Exit code of `testing-conventions unit lint --language typescript <codebase>`.
fn isolation_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "typescript"])
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

// typed `vi.mock`
#[test]
fn untyped_red_exits_nonzero() {
    assert_eq!(isolation_exit("untyped_mock/red"), 1);
}

#[test]
fn untyped_clean_exits_zero() {
    assert_eq!(isolation_exit("untyped_mock/clean"), 0);
}

// Vitest options-object mock (`{ spy: true }`) — not a factory, must pass.
#[test]
fn spy_option_clean_exits_zero() {
    assert_eq!(isolation_exit("untyped_mock/spy_clean"), 0);
}

// #393: a `.js` import mocked bare (and the inverse) resolves to the same module, so
// the collaborator is mocked and the tree passes.
#[test]
fn ext_normalize_clean_exits_zero() {
    assert_eq!(isolation_exit("ext_normalize/clean"), 0);
}

#[test]
fn tier_layout_suites_are_not_unit_subjects() {
    // `<package root>/tests/` belongs to the suite tiers; the unit-suite
    // isolation rule reports nothing there.
    assert_eq!(isolation_exit("tier_layout"), 0);
}
