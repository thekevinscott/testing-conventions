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

/// Exit code of `unit lint --language python <fixture>`.
fn isolation_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
        "--language".into(),
        "python".into(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

/// Exit code of `unit lint --language python --config <config> <fixture>`.
fn isolation_exit_with_config(fixture_name: &str, config_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
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
    assert_eq!(
        isolation_exit_with_config("waived", "waived/testing-conventions.toml"),
        0
    );
}

#[test]
fn legacy_test_prefix_is_not_scanned() {
    let violations = find_unit_isolation_violations(fixture("legacy_prefix"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "a legacy `test_*.py` is source (not a unit test) and must not be scanned; \
         got {violations:?}"
    );
}

#[test]
fn legacy_test_prefix_exits_zero() {
    assert_eq!(isolation_exit("legacy_prefix"), 0);
}

#[test]
fn external_red_flags_unmocked_external_deps() {
    let violations = find_unit_isolation_violations(fixture("external/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "unmocked-collaborator" && v.message.contains("requests")),
        "an imported, un-mocked third-party package must be flagged; got {violations:?}"
    );
    assert!(
        violations.iter().any(|v| v.message.contains("subprocess")),
        "an imported, un-mocked effectful-stdlib module must be flagged; got {violations:?}"
    );
    assert!(
        !violations.iter().any(|v| v.message.contains("json")),
        "pure stdlib must not be flagged; got {violations:?}"
    );
}

#[test]
fn external_clean_reports_no_violations() {
    let violations = find_unit_isolation_violations(fixture("external/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture mocks the external collaborators by string and uses only pure \
         stdlib; got {violations:?}"
    );
}

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

#[test]
fn barrel_reexport_import_is_the_unit_under_test() {
    let violations = find_unit_isolation_violations(fixture("barrel/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "a barrel test's `from . import …` names its own SUT surface, never a \
         collaborator; got {violations:?}"
    );
}

#[test]
fn barrel_clean_exits_zero() {
    assert_eq!(isolation_exit("barrel/clean"), 0);
}

#[test]
fn barrel_sibling_direct_import_still_flagged() {
    let violations = find_unit_isolation_violations(fixture("barrel/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "unmocked-collaborator" && v.message.contains(".core")),
        "a sibling-direct import from a barrel test must still be flagged; got {violations:?}"
    );
}

#[test]
fn barrel_red_exits_nonzero() {
    assert_eq!(isolation_exit("barrel/red"), 1);
}

#[test]
fn overmatch_red_flags_partly_mocked_import() {
    let violations = find_unit_isolation_violations(fixture("overmatch/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "unmocked-collaborator" && v.message.contains("myproject.ledger")),
        "a multi-symbol import with an un-mocked symbol must be flagged; got {violations:?}"
    );
}

#[test]
fn overmatch_red_exits_nonzero() {
    assert_eq!(isolation_exit("overmatch/red"), 1);
}

#[test]
fn overmatch_clean_reports_no_violations() {
    let violations = find_unit_isolation_violations(fixture("overmatch/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "a multi-symbol import is mocked when every symbol is patched; got {violations:?}"
    );
}

#[test]
fn overmatch_clean_exits_zero() {
    assert_eq!(isolation_exit("overmatch/clean"), 0);
}

#[test]
fn wrong_module_red_flags_last_segment_only_match() {
    let violations = find_unit_isolation_violations(fixture("wrong_module/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "unmocked-collaborator" && v.message.contains("myproject.ledger")),
        "a wrong-module patch must not clear `myproject.ledger`; got {violations:?}"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.message.contains("myproject.formatter")),
        "a stdlib `json.dumps` patch must not clear `myproject.formatter`; got {violations:?}"
    );
}

#[test]
fn wrong_module_red_exits_nonzero() {
    assert_eq!(isolation_exit("wrong_module/red"), 1);
}

#[test]
fn tier_layout_suites_are_not_unit_subjects() {
    assert_eq!(isolation_exit("tier_layout"), 0);
}
