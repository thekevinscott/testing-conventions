//! E2E tests for the Rust `integration lint` rule: drive the built CLI
//! binary against the fixture crates and assert the exit code.

use std::path::PathBuf;
use std::process::Command;

/// Absolute path to a fixture crate under `tests/fixtures/isolation/integration/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/isolation/integration")
        .join(name)
}

/// Exit code of `testing-conventions integration lint --language rust <codebase>`.
fn lint_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["integration", "lint", "--language", "rust"])
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(lint_exit("red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(lint_exit("clean"), 0);
}

// waivers
#[test]
fn waived_exits_zero() {
    let code = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["integration", "lint", "--language", "rust", "--config"])
        .arg(fixture("waived/testing-conventions.toml"))
        .arg(fixture("waived"))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code");
    assert_eq!(code, 0);
}

#[test]
fn tier_layout_tests_crate_is_linted_from_a_src_scan() {
    // The integration suite derives from the crate root, so the binary scanning
    // `src/` still lints the crate's `tests/` directory.
    assert_eq!(lint_exit("derived/src"), 1);
}
