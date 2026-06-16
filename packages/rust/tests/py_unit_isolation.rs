//! Integration tests for the Python unit-isolation check
//! (#42 slice 2: `unmocked-collaborator`). Per the #3 guardrail, the rule ships a
//! red fixture (an imported, un-mocked first-party collaborator — must be flagged)
//! and a clean fixture (the canonical patched-by-string form — must pass).

use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::lint::find_unit_isolation_violations;
use testing_conventions::run;

/// Absolute path to a fixture tree under `tests/fixtures/unit_isolation/python/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_isolation/python")
        .join(name)
}

/// Exit code of `unit isolation --language python <fixture>`.
fn isolation_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "isolation".into(),
        "--language".into(),
        "python".into(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

/// Exit code of `unit isolation --language python --config <config> <fixture>`.
fn isolation_exit_with_config(fixture_name: &str, config_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "isolation".into(),
        "--language".into(),
        "python".into(),
        "--config".into(),
        fixture(config_name).into_os_string(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

#[test]
fn red_flags_unmocked_first_party_collaborator() {
    let violations = find_unit_isolation_violations(fixture("red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "unmocked-collaborator" && v.message.contains("myproject.ledger")),
        "an imported, un-mocked first-party collaborator must be flagged; got {violations:?}"
    );
    // The unit under test (`myproject.widget`) is never a collaborator.
    assert!(
        !violations
            .iter()
            .any(|v| v.message.contains("myproject.widget")),
        "the unit under test must not be flagged; got {violations:?}"
    );
}

#[test]
fn clean_reports_no_violations() {
    let violations = find_unit_isolation_violations(fixture("clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture imports only the unit under test and patches its collaborator \
         by string; got {violations:?}"
    );
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
    // Same un-mocked collaborator as the red fixture, but the file is waived.
    assert_eq!(
        isolation_exit_with_config("waived", "waived/testing-conventions.toml"),
        0
    );
}
