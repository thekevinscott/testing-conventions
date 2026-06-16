//! E2E tests for `unit location` exemptions (#32): drive the built CLI binary
//! end-to-end (a real subprocess) against fixture trees and their configs, and
//! assert the exit code. Complements the in-process integration tests in
//! `unit_location.rs`.

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_location")
}

/// Exit code of `unit location --language <lang> --config <dir>/testing-conventions.toml <dir>`,
/// run as a real subprocess against the built binary.
fn unit_location_exit(fixture: &str, language: &str) -> i32 {
    let dir = fixtures().join(fixture);
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "location", "--language", language, "--config"])
        .arg(dir.join("testing-conventions.toml"))
        .arg(&dir)
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn config_exemptions_clear_the_tree() {
    // python_exempt's config exempts cli.py + pkg/__init__.py, so the binary
    // reports no orphans.
    assert_eq!(unit_location_exit("python_exempt", "python"), 0);
}

#[test]
fn a_blank_reason_exemption_makes_the_binary_error() {
    // bad_exempt's config carries an exemption with an empty `reason`. The binary
    // must reject it on load (exit 1), never silently accept a reasonless
    // omission. (#32 — "every exemption must say why".)
    assert_eq!(unit_location_exit("bad_exempt", "python"), 1);
}
