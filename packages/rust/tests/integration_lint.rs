//! Integration tests for the Python integration-test lints
//! (#19; rules #48–#52). Per the #3 guardrail, each lint ships a red fixture
//! (a violation — must be reported) and a clean fixture (must pass).
//!
//! These **start RED**: the skeleton (#48) wires `integration lint` but no
//! detection, so the red-fixture assertions below fail until R1 (#49) lands.

use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::lint::find_violations;
use testing_conventions::run;

/// Absolute path to a fixture tree under `tests/fixtures/integration_lint/python/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/integration_lint/python")
        .join(name)
}

/// Raw result of invoking the CLI with `args` after the program name.
fn run_cli(args: &[&str]) -> anyhow::Result<i32> {
    let argv: Vec<OsString> = std::iter::once(OsString::from("testing-conventions"))
        .chain(args.iter().copied().map(OsString::from))
        .collect();
    run(argv)
}

/// Exit code of `integration lint --language python <fixture>`.
fn lint_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "integration".into(),
        "lint".into(),
        "--language".into(),
        "python".into(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

// ---- R1: forbid `monkeypatch` (#49) --------------------------------------

#[test]
fn red_fixture_reports_a_monkeypatch_violation() {
    let violations =
        find_violations(fixture("red")).expect("walking a readable tree should succeed");
    assert!(
        violations.iter().any(|v| v.rule == "no-monkeypatch"),
        "the red fixture uses pytest's `monkeypatch` and must be flagged; got {violations:?}"
    );
}

#[test]
fn clean_fixture_reports_no_violations() {
    let violations =
        find_violations(fixture("clean")).expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture patches via a fixture (no monkeypatch); got {violations:?}"
    );
}

#[test]
fn red_fixture_exits_nonzero() {
    assert_eq!(lint_exit("red"), 1);
}

#[test]
fn clean_fixture_exits_zero() {
    assert_eq!(lint_exit("clean"), 0);
}

// ---- CLI surface ---------------------------------------------------------

#[test]
fn integration_lint_requires_language() {
    // Omitting `--language` is a usage error, never a silent `python` run.
    let err = run_cli(&["integration", "lint", "src"]).expect_err("--language is required");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("a missing required flag should surface as a clap::Error");
    assert_eq!(
        clap_err.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}
