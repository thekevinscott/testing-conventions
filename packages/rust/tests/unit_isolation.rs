use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::run;
use testing_conventions::ts::find_unit_violations;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_isolation/typescript")
        .join(name)
}

/// Exit code of `unit lint --language typescript <fixture>`.
fn isolation_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
        "--language".into(),
        "typescript".into(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

#[test]
fn red_flags_unmocked_collaborators() {
    let violations =
        find_unit_violations(fixture("red")).expect("walking a readable tree should succeed");
    assert_eq!(violations.len(), 2, "got: {violations:?}");
    assert!(violations.iter().all(|v| v.rule == "unmocked-collaborator"));
    let msgs = violations
        .iter()
        .map(|v| v.message.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        msgs.contains("./formatter"),
        "the un-mocked first-party collaborator must be flagged; got {msgs}"
    );
    assert!(
        msgs.contains("lodash"),
        "the un-mocked external collaborator must be flagged; got {msgs}"
    );
    assert!(
        !msgs.contains("./widget"),
        "the unit under test must not be flagged; got {msgs}"
    );
    assert!(
        !msgs.contains("./logger"),
        "a mocked collaborator must not be flagged; got {msgs}"
    );
}

#[test]
fn clean_reports_no_violations() {
    let violations =
        find_unit_violations(fixture("clean")).expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "every collaborator is mocked (type-only imports and the test runner aside); got {violations:?}"
    );
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(isolation_exit("red"), 1);
}

#[test]
fn a_config_waiver_lifts_the_red_fixtures_violations() {
    let dir = std::env::temp_dir().join(format!("tc-ts-lint-waiver-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let config_path = dir.join("testing-conventions.toml");
    std::fs::write(
        &config_path,
        "[[typescript.exempt]]\npath = \"widget.test.ts\"\n\
         rules = [\"unmocked-collaborator\"]\nreason = \"synthetic waiver for this test\"\n",
    )
    .unwrap();
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
        "--language".into(),
        "typescript".into(),
        "--config".into(),
        config_path.clone().into_os_string(),
        fixture("red").into_os_string(),
    ];
    let exit = run(argv);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        exit.expect("a readable tree should not error"),
        0,
        "both violations sit in the exempted file"
    );
}

#[test]
fn clean_exits_zero() {
    assert_eq!(isolation_exit("clean"), 0);
}

#[test]
fn red_flags_untyped_mock() {
    let violations = find_unit_violations(fixture("untyped_mock/red"))
        .expect("walking a readable tree should succeed");
    assert_eq!(violations.len(), 1, "got: {violations:?}");
    assert_eq!(violations[0].rule, "untyped-mock");
    assert!(
        violations[0].message.contains("lodash"),
        "the untyped factory mock must be flagged; got {violations:?}"
    );
    assert!(
        !violations.iter().any(|v| v.message.contains("./formatter")),
        "a typed mock must not be flagged; got {violations:?}"
    );
}

#[test]
fn untyped_clean_reports_no_violations() {
    let violations = find_unit_violations(fixture("untyped_mock/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "a typed factory mock and a bare auto-mock are both fine; got {violations:?}"
    );
}

#[test]
fn untyped_red_exits_nonzero() {
    assert_eq!(isolation_exit("untyped_mock/red"), 1);
}

#[test]
fn untyped_clean_exits_zero() {
    assert_eq!(isolation_exit("untyped_mock/clean"), 0);
}

#[test]
fn spy_option_mock_reports_no_violations() {
    let violations = find_unit_violations(fixture("untyped_mock/spy_clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the options-object spy mock must not be flagged; got {violations:?}"
    );
}

#[test]
fn spy_option_clean_exits_zero() {
    assert_eq!(isolation_exit("untyped_mock/spy_clean"), 0);
}

#[test]
fn ext_normalize_clean_reports_no_violations() {
    let violations = find_unit_violations(fixture("ext_normalize/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "an extension-mismatched import/mock pair must still match; got {violations:?}"
    );
}

#[test]
fn ext_normalize_clean_exits_zero() {
    assert_eq!(isolation_exit("ext_normalize/clean"), 0);
}

#[test]
fn tier_layout_suites_are_not_unit_subjects() {
    assert_eq!(isolation_exit("tier_layout"), 0);
}
