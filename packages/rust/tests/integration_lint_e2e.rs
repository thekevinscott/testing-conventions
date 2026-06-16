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

/// Exit code of the built binary with `--config`.
fn lint_exit_with_config(codebase: &str, config: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["integration", "lint", "--language", "python", "--config"])
        .arg(fixture(config))
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

// R1: forbid `monkeypatch` (#49)
#[test]
fn monkeypatch_red_exits_nonzero() {
    assert_eq!(lint_exit("monkeypatch/red"), 1);
}

#[test]
fn monkeypatch_clean_exits_zero() {
    assert_eq!(lint_exit("monkeypatch/clean"), 0);
}

#[test]
fn monkeypatch_waived_exits_zero() {
    // Same monkeypatch use as the red fixture, but the file is waived in the config (#123).
    assert_eq!(
        lint_exit_with_config(
            "monkeypatch/waived",
            "monkeypatch/waived/testing-conventions.toml"
        ),
        0
    );
}

// R2: patches must live in fixtures, not inline (#50)
#[test]
fn inline_patch_red_exits_nonzero() {
    assert_eq!(lint_exit("inline_patch/red"), 1);
}

#[test]
fn inline_patch_clean_exits_zero() {
    assert_eq!(lint_exit("inline_patch/clean"), 0);
}

#[test]
fn inline_patch_waived_exits_zero() {
    // Same inline `with patch(...)` as the red fixture, but the file is waived (#123).
    assert_eq!(
        lint_exit_with_config(
            "inline_patch/waived",
            "inline_patch/waived/testing-conventions.toml"
        ),
        0
    );
}

// R3: env via patch.dict(os.environ, …) (#51)
#[test]
fn environ_red_exits_nonzero() {
    assert_eq!(lint_exit("environ/red"), 1);
}

#[test]
fn environ_clean_exits_zero() {
    assert_eq!(lint_exit("environ/clean"), 0);
}

#[test]
fn environ_waived_exits_zero() {
    // Same os.environ mutation as the red fixture, but the file is waived (#123).
    assert_eq!(
        lint_exit_with_config("environ/waived", "environ/waived/testing-conventions.toml"),
        0
    );
}

// R4: don't patch module-global config constants (#52, waivable)
#[test]
fn constant_patch_red_exits_nonzero() {
    assert_eq!(lint_exit("constant_patch/red"), 1);
}

#[test]
fn constant_patch_waived_exits_zero() {
    assert_eq!(
        lint_exit_with_config(
            "constant_patch/waived",
            "constant_patch/waived/testing-conventions.toml"
        ),
        0
    );
}

// #145: a legacy `test_*.py` is source (not scanned), so the tree is clean
#[test]
fn legacy_test_prefix_exits_zero() {
    assert_eq!(lint_exit("legacy_prefix"), 0);
}

// Integration isolation: no first-party patch (#42)
#[test]
fn first_party_patch_red_exits_nonzero() {
    assert_eq!(lint_exit("no_first_party_patch/red"), 1);
}

#[test]
fn first_party_patch_clean_exits_zero() {
    assert_eq!(lint_exit("no_first_party_patch/clean"), 0);
}

#[test]
fn first_party_patch_waived_exits_zero() {
    assert_eq!(
        lint_exit_with_config(
            "no_first_party_patch/waived",
            "no_first_party_patch/waived/testing-conventions.toml"
        ),
        0
    );
}
