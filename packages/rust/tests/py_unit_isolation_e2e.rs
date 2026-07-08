//! E2E tests for the Python unit-isolation check: drive the built
//! CLI binary end-to-end (no mocks) against the fixtures and assert the exit code.

use std::path::PathBuf;
use std::process::Command;

/// Absolute path to a fixture tree under `tests/fixtures/unit_isolation/python/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_isolation/python")
        .join(name)
}

/// Exit code of `testing-conventions unit lint --language python <codebase>`.
fn isolation_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "python"])
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

/// Exit code of the built binary with `--config`.
fn isolation_exit_with_config(codebase: &str, config: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "python", "--config"])
        .arg(fixture(config))
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

#[test]
fn waived_exits_zero() {
    assert_eq!(
        isolation_exit_with_config("waived", "waived/testing-conventions.toml"),
        0
    );
}

// A legacy `test_*.py` is source (not scanned), so the tree is clean
#[test]
fn legacy_test_prefix_exits_zero() {
    assert_eq!(isolation_exit("legacy_prefix"), 0);
}

// external & effectful-stdlib deps
#[test]
fn external_red_exits_nonzero() {
    assert_eq!(isolation_exit("external/red"), 1);
}

#[test]
fn external_clean_exits_zero() {
    assert_eq!(isolation_exit("external/clean"), 0);
}

#[test]
fn external_waived_exits_zero() {
    assert_eq!(
        isolation_exit_with_config(
            "external/waived",
            "external/waived/testing-conventions.toml"
        ),
        0
    );
}

// A barrel test's `from . import …` names the SUT's own surface, not a
// collaborator; a sibling-direct import (`from .core import …`) is still flagged
#[test]
fn barrel_clean_exits_zero() {
    assert_eq!(isolation_exit("barrel/clean"), 0);
}

#[test]
fn barrel_red_exits_nonzero() {
    assert_eq!(isolation_exit("barrel/red"), 1);
}

// #393: a multi-symbol import is mocked only when every symbol is patched at its own
// module path; a last-segment match against a different module does not mock it
#[test]
fn overmatch_red_exits_nonzero() {
    assert_eq!(isolation_exit("overmatch/red"), 1);
}

#[test]
fn overmatch_clean_exits_zero() {
    assert_eq!(isolation_exit("overmatch/clean"), 0);
}

#[test]
fn wrong_module_red_exits_nonzero() {
    assert_eq!(isolation_exit("wrong_module/red"), 1);
}
