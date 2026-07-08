//! E2E tests for the packaging rule's foundation: drive the built CLI
//! binary against fixture "artifact" trees and assert the exit code. The rule
//! (README "Packaging"): test files must never ship in a built artifact. Each
//! fixture stands in for an unpacked built artifact (a wheel, a `dist/`).
//!
//! These start red — the `packaging` command does not exist yet, so the binary
//! exits with a usage error rather than `1`/`0` — and go green once the command
//! and the scanner land.

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/packaging")
}

/// Exit code of `testing-conventions packaging <fixture> --language <language>`.
fn packaging_exit(fixture: &str, language: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("packaging")
        .arg(fixtures().join(fixture))
        .args(["--language", language])
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn python_artifact_with_a_test_file_exits_nonzero() {
    assert_eq!(packaging_exit("python_red", "python"), 1);
}

#[test]
fn python_clean_artifact_exits_zero() {
    assert_eq!(packaging_exit("python_clean", "python"), 0);
}

#[test]
fn typescript_artifact_with_a_test_file_exits_nonzero() {
    assert_eq!(packaging_exit("typescript_red", "typescript"), 1);
}

#[test]
fn typescript_clean_artifact_exits_zero() {
    assert_eq!(packaging_exit("typescript_clean", "typescript"), 0);
}
