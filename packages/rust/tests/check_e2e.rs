//! E2E tests for the `check` umbrella (#56): drive the built CLI binary against
//! fixture trees and assert the aggregate exit code. `check` reads the fixture's
//! `testing-conventions.toml` and runs every rule its language tables enable.
//!
//! These fixtures deliberately set no coverage floor, so they exercise the
//! toolchain-free rules (`unit colocated-test` + `integration lint`) and need no
//! Python on `PATH`; the coverage rule's own wiring is covered in `coverage.rs`.

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/check")
}

/// Run `check --config <dir>/testing-conventions.toml <dir>` against the built
/// binary; return its exit code and captured stderr.
fn check(fixture: &str) -> (i32, String) {
    let dir = fixtures().join(fixture);
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("check")
        .arg("--config")
        .arg(dir.join("testing-conventions.toml"))
        .arg(&dir)
        .output()
        .expect("the built binary should run");
    let code = out
        .status
        .code()
        .expect("the process should exit with a code");
    (code, String::from_utf8_lossy(&out.stderr).into_owned())
}

#[test]
fn a_clean_tree_passes() {
    // Python + TypeScript sources each have their colocated test, and the one
    // Python test file is lint-clean → every enabled rule passes, so the umbrella
    // exits 0.
    let (code, stderr) = check("clean");
    assert_eq!(code, 0, "stderr: {stderr}");
}

#[test]
fn a_violation_in_any_rule_fails_the_umbrella() {
    // orphan.py has no colocated test → `unit colocated-test (python)` fails, so
    // `check` fails (exit 1) even though the TypeScript pair is clean.
    let (code, stderr) = check("red");
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(stderr.contains("orphan.py"), "stderr: {stderr}");
}

#[test]
fn a_config_that_enables_no_checks_is_an_error() {
    // A config that names no language table gives `check` nothing to run; that is a
    // misconfiguration reported as an error, not a silent pass.
    let (code, stderr) = check("no_checks");
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(stderr.contains("no checks"), "stderr: {stderr}");
}
