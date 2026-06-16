//! Integration tests for the TypeScript unit-isolation check
//! (#43 / #76: `unmocked-collaborator`). Per the #3 guardrail, the rule ships a
//! red fixture (an un-mocked collaborator — must be flagged) and a clean fixture
//! (every collaborator mocked — must pass).

use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::run;
use testing_conventions::ts::find_unit_violations;

/// Absolute path to a fixture tree under `tests/fixtures/unit_isolation/typescript/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_isolation/typescript")
        .join(name)
}

/// Exit code of `unit isolation --language typescript <fixture>`.
fn isolation_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "isolation".into(),
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
    // `./formatter` (first-party) and `lodash` (external) are imported but not mocked.
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
    // The unit under test (`./widget`) and the mocked `./logger` must NOT be flagged.
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
fn clean_exits_zero() {
    assert_eq!(isolation_exit("clean"), 0);
}

// ---- typed `vi.mock` (#77): a mock factory must anchor to the real module ----

#[test]
fn red_flags_untyped_mock() {
    let violations = find_unit_violations(fixture("untyped_mock/red"))
        .expect("walking a readable tree should succeed");
    // The `lodash` mock has a factory but no `vi.importActual<…>` anchor.
    assert_eq!(violations.len(), 1, "got: {violations:?}");
    assert_eq!(violations[0].rule, "untyped-mock");
    assert!(
        violations[0].message.contains("lodash"),
        "the untyped factory mock must be flagged; got {violations:?}"
    );
    // The typed `./formatter` mock (`importActual<typeof import(...)>`) is fine.
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
