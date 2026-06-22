//! E2E tests for line-scoped coverage exemptions (#226): drive the built CLI binary
//! end-to-end (no mocks) against the `exempt_cov` fixtures and assert the exit code
//! and message.
//!
//! A `[[<lang>.exempt]]` entry with a `lines` list excuses only those lines from the
//! coverage floor — not the whole file — with a determinism guard: a listed line that
//! is actually covered (or carries no measured code) is a hard error, and a *missing*
//! uncovered line still fails the floor. The fixtures are the standard `exempt_cov`
//! codebases (a fully-covered `core` plus an uncovered launcher `shim`), now lifted at
//! line granularity instead of whole-file.
//!
//! Red until line-scoped exemptions land: today the `lines` key is rejected by the
//! config self-guard, so every one of these exits non-zero with an "unknown field"
//! error rather than the line-scoped behavior asserted here. Requires `coverage` +
//! `pytest` (Python), `cargo-llvm-cov` (Rust), and the vitest toolchain (TypeScript).

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage")
}

/// Run `unit coverage --language <lang> --config <cfg> <lang>/<codebase>` and return
/// the captured output (exit code + stderr).
fn run(language: &str, codebase: &str, config: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", language, "--config"])
        .arg(fixtures().join(config))
        .arg(fixtures().join(language).join(codebase))
        .output()
        .expect("the built binary should run")
}

fn code(output: &Output) -> i32 {
    output
        .status
        .code()
        .expect("the process should exit with a code")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

// ---- Python ---------------------------------------------------------------

#[test]
fn python_minimal_line_exemption_clears_the_floor() {
    // Only shim.py's uncovered body (lines 2-4) is exempt; core.py is fully covered, so
    // the 100 floor passes with just those lines lifted.
    let out = run("python", "exempt_cov", "lines_py_shim_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn python_over_exemption_is_a_hard_error() {
    // Line 1 (`def launch`) runs on import and is covered, so listing it is rejected —
    // a line-scoped exemption may only name uncovered lines.
    let out = run("python", "exempt_cov", "lines_py_shim_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("uncovered lines"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}

#[test]
fn python_under_listing_still_fails_the_floor() {
    // Exempting only lines 2-3 leaves line 4 uncovered, so the floor still bites.
    let out = run("python", "exempt_cov", "lines_py_shim_under.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("is below"),
        "expected a floor failure, got: {}",
        stderr(&out)
    );
}

// ---- Rust -----------------------------------------------------------------

#[test]
fn rust_minimal_line_exemption_clears_the_floor() {
    // Only src/shim.rs's uncovered `launch` region (lines 6-8) is exempt; core.rs is
    // fully covered, so the 100 floor passes.
    let out = run("rust", "exempt_cov", "lines_rust_shim_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn rust_over_exemption_is_a_hard_error() {
    // src/core.rs line 6 is fully covered, so listing it is rejected.
    let out = run("rust", "exempt_cov", "lines_rust_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("uncovered lines"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}

// ---- TypeScript -----------------------------------------------------------

#[test]
fn typescript_minimal_line_exemption_clears_the_floor() {
    // Only shim.ts's uncovered `launch` (lines 1-3) is exempt; core.ts is fully
    // covered, so the 100 floor passes.
    let out = run("typescript", "exempt_cov", "lines_ts_shim_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn typescript_over_exemption_is_a_hard_error() {
    // core.ts line 2 is fully covered, so listing it is rejected.
    let out = run("typescript", "exempt_cov", "lines_ts_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("uncovered lines"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}
