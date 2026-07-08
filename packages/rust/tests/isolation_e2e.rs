//! E2E tests for the Rust `unit lint` rule: drive the built CLI binary
//! against the fixture crates and assert the exit code.

use std::path::PathBuf;
use std::process::Command;

/// Absolute path to a fixture tree under `tests/fixtures/isolation/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/isolation")
        .join(name)
}

/// Exit code of `testing-conventions unit lint --language rust <codebase>`.
fn iso_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "rust"])
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(iso_exit("unit/red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(iso_exit("unit/clean"), 0);
}

// D2: foreign imports
#[test]
fn imports_red_exits_nonzero() {
    assert_eq!(iso_exit("imports/red"), 1);
}

#[test]
fn imports_clean_exits_zero() {
    assert_eq!(iso_exit("imports/clean"), 0);
}

// waivers
#[test]
fn waived_exits_zero() {
    let code = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "rust", "--config"])
        .arg(fixture("unit/waived/testing-conventions.toml"))
        .arg(fixture("unit/waived"))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code");
    assert_eq!(code, 0);
}
