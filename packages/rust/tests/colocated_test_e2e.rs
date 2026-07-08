//! E2E tests for `unit colocated-test` exemptions: drive the built CLI
//! binary end-to-end (a real subprocess) against fixture trees and their configs,
//! and assert the exit code. Complements the in-process integration tests in
//! `colocated_test.rs`.

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/colocated_test")
}

/// Exit code of `unit colocated-test --language <lang> --config <dir>/testing-conventions.toml <dir>`,
/// run as a real subprocess against the built binary.
fn unit_colocated_test_exit(fixture: &str, language: &str) -> i32 {
    let dir = fixtures().join(fixture);
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "colocated-test", "--language", language, "--config"])
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
    assert_eq!(unit_colocated_test_exit("python_exempt", "python"), 0);
}

#[test]
fn a_blank_reason_exemption_makes_the_binary_error() {
    // bad_exempt's config carries an exemption with an empty `reason`. The binary
    // must reject it on load (exit 1), never silently accept a reasonless
    // omission — every exemption must say why.
    assert_eq!(unit_colocated_test_exit("bad_exempt", "python"), 1);
}

#[test]
fn conftest_is_not_an_orphan() {
    // python_conftest holds a conftest.py (pytest fixtures) beside a paired
    // source/test. conftest.py is support, never a subject, so the binary reports
    // no orphans and exits zero.
    assert_eq!(unit_colocated_test_exit("python_conftest", "python"), 0);
}

/// Exit code + stderr of the same invocation, for assertions that inspect the
/// reported orphan (not just the exit code).
fn unit_colocated_test_output(fixture: &str, language: &str) -> (i32, String) {
    let dir = fixtures().join(fixture);
    let output = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "colocated-test", "--language", language, "--config"])
        .arg(dir.join("testing-conventions.toml"))
        .arg(&dir)
        .output()
        .expect("the built binary should run");
    (
        output
            .status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn rust_clean_tree_exits_zero() {
    // Every source module with behavior carries an inline `#[cfg(test)]` module.
    assert_eq!(unit_colocated_test_exit("rust/clean", "rust"), 0);
}

#[test]
fn rust_red_tree_flags_the_untested_module() {
    // `src/untested.rs` has a function but no inline `#[cfg(test)]` module.
    let (code, stderr) = unit_colocated_test_output("rust/red", "rust");
    assert_eq!(
        code, 1,
        "a missing inline test must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("untested.rs"),
        "stderr should name the orphan module; got: {stderr}"
    );
}

#[test]
fn rust_cfg_not_test_module_is_flagged() {
    // #390: `#[cfg(not(test))]` is production code, not a test module. `src/gated.rs`
    // has behavior and only a `not(test)` module, so the binary flags it as an orphan.
    let (code, stderr) = unit_colocated_test_output("rust/cfg_not_test", "rust");
    assert_eq!(
        code, 1,
        "a `not(test)` module is not a test module; stderr: {stderr}"
    );
    assert!(
        stderr.contains("gated.rs"),
        "stderr should name the orphan module; got: {stderr}"
    );
}
