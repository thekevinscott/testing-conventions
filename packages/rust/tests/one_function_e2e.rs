//! E2E tests for `unit one-function-per-file`: drive the built CLI binary
//! against the fixture trees and assert the exit code and the reported message.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Absolute path to a fixture tree under `tests/fixtures/one_function/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/one_function")
        .join(name)
}

/// Output of `unit one-function-per-file --language <language> <fixture>`.
fn run(language: &str, fixture_name: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "one-function-per-file", "--language", language])
        .arg(fixture(fixture_name))
        .output()
        .expect("the built binary should run")
}

/// Output of the same run with `--config <config>`.
fn run_with_config(language: &str, fixture_name: &str, config_name: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "one-function-per-file", "--language", language])
        .arg("--config")
        .arg(fixture(config_name))
        .arg(fixture(fixture_name))
        .output()
        .expect("the built binary should run")
}

/// The exit code of an `Output`.
fn code(output: &Output) -> i32 {
    output
        .status
        .code()
        .expect("the process should exit with a code")
}

/// The stderr of an `Output`, as a string.
fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

// Python

#[test]
fn python_red_exits_nonzero() {
    assert_eq!(code(&run("python", "python/red")), 1);
}

#[test]
fn python_red_names_the_extra_function_and_the_one_holding_the_file() {
    let reported = stderr(&run("python", "python/red"));
    assert!(
        reported.contains("two_functions.py:6: one-function-per-file"),
        "the second function's line carries the rule id: {reported}"
    );
    assert!(
        reported.contains("`beta`") && reported.contains("`alpha`"),
        "the message names the extra function and the one holding the file: {reported}"
    );
}

#[test]
fn python_red_counts_every_function_past_the_first() {
    let reported = stderr(&run("python", "python/red"));
    assert!(
        reported.contains("error: 3 function(s)"),
        "two files contribute one and two violations: {reported}"
    );
}

#[test]
fn python_clean_exits_zero() {
    let output = run("python", "python/clean");
    assert_eq!(code(&output), 0);
    assert_eq!(stderr(&output), "");
}

#[test]
fn python_raised_fails_at_the_default_threshold() {
    let output = run("python", "python/raised");
    assert_eq!(code(&output), 1);
    assert!(
        stderr(&output).contains("1-line threshold"),
        "the summary states the threshold in force: {}",
        stderr(&output)
    );
}

#[test]
fn python_raised_passes_at_its_configured_threshold() {
    assert_eq!(
        code(&run_with_config(
            "python",
            "python/raised",
            "python/raised/testing-conventions.toml"
        )),
        0
    );
}

#[test]
fn python_raised_red_reports_the_configured_threshold() {
    let output = run_with_config(
        "python",
        "python/raised_red",
        "python/raised_red/testing-conventions.toml",
    );
    assert_eq!(code(&output), 1);
    assert!(
        stderr(&output).contains("5-line threshold"),
        "the configured threshold reaches the summary: {}",
        stderr(&output)
    );
}

#[test]
fn python_waived_exits_zero() {
    assert_eq!(
        code(&run_with_config(
            "python",
            "python/waived",
            "python/waived/testing-conventions.toml"
        )),
        0
    );
}

// TypeScript

#[test]
fn typescript_red_exits_nonzero() {
    assert_eq!(code(&run("typescript", "typescript/red")), 1);
}

#[test]
fn typescript_red_flags_an_arrow_bound_to_a_module_scope_const() {
    let reported = stderr(&run("typescript", "typescript/red"));
    assert!(
        reported.contains("two-functions.ts:6: one-function-per-file"),
        "the arrow-bound `beta` is a module-scope function: {reported}"
    );
}

#[test]
fn typescript_clean_exits_zero() {
    let output = run("typescript", "typescript/clean");
    assert_eq!(code(&output), 0);
    assert_eq!(stderr(&output), "");
}

#[test]
fn typescript_raised_fails_at_the_default_threshold() {
    assert_eq!(code(&run("typescript", "typescript/raised")), 1);
}

#[test]
fn typescript_raised_passes_at_its_configured_threshold() {
    assert_eq!(
        code(&run_with_config(
            "typescript",
            "typescript/raised",
            "typescript/raised/testing-conventions.toml"
        )),
        0
    );
}

#[test]
fn typescript_raised_red_reports_the_configured_threshold() {
    let output = run_with_config(
        "typescript",
        "typescript/raised_red",
        "typescript/raised_red/testing-conventions.toml",
    );
    assert_eq!(code(&output), 1);
    assert!(
        stderr(&output).contains("5-line threshold"),
        "the configured threshold reaches the summary: {}",
        stderr(&output)
    );
}

#[test]
fn typescript_waived_exits_zero() {
    assert_eq!(
        code(&run_with_config(
            "typescript",
            "typescript/waived",
            "typescript/waived/testing-conventions.toml"
        )),
        0
    );
}

// Rust

#[test]
fn rust_red_exits_nonzero() {
    assert_eq!(code(&run("rust", "rust/red")), 1);
}

#[test]
fn rust_red_names_the_extra_function() {
    let reported = stderr(&run("rust", "rust/red"));
    assert!(
        reported.contains("two_functions.rs:6: one-function-per-file"),
        "the second `fn` item carries the rule id: {reported}"
    );
}

#[test]
fn rust_clean_exits_zero() {
    let output = run("rust", "rust/clean");
    assert_eq!(code(&output), 0);
    assert_eq!(stderr(&output), "");
}

#[test]
fn rust_raised_fails_at_the_default_threshold() {
    assert_eq!(code(&run("rust", "rust/raised")), 1);
}

#[test]
fn rust_raised_passes_at_its_configured_threshold() {
    assert_eq!(
        code(&run_with_config(
            "rust",
            "rust/raised",
            "rust/raised/testing-conventions.toml"
        )),
        0
    );
}

#[test]
fn rust_raised_red_reports_the_configured_threshold() {
    let output = run_with_config(
        "rust",
        "rust/raised_red",
        "rust/raised_red/testing-conventions.toml",
    );
    assert_eq!(code(&output), 1);
    assert!(
        stderr(&output).contains("5-line threshold"),
        "the configured threshold reaches the summary: {}",
        stderr(&output)
    );
}

#[test]
fn rust_waived_exits_zero() {
    assert_eq!(
        code(&run_with_config(
            "rust",
            "rust/waived",
            "rust/waived/testing-conventions.toml"
        )),
        0
    );
}
