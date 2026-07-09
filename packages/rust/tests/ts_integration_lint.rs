//! Integration tests for the TypeScript integration-isolation lint
//! (`no-first-party-mock`). The rule ships a red fixture (a first-party `vi.mock`
//! — must be flagged) and a clean fixture (only third-party / built-in mocks —
//! must pass).

use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::run;
use testing_conventions::ts::find_integration_violations;

/// Absolute path to a fixture tree under
/// `tests/fixtures/integration_lint/typescript/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/integration_lint/typescript")
        .join(name)
}

/// Exit code of `integration lint --language typescript <fixture>`.
fn lint_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "integration".into(),
        "lint".into(),
        "--language".into(),
        "typescript".into(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

#[test]
fn red_flags_first_party_mocks() {
    let violations = find_integration_violations(fixture("no_first_party_mock/red"))
        .expect("walking a readable tree should succeed");
    // Both red files mock a first-party module — one via `vi.mock`, one via `vi.doMock`.
    assert_eq!(violations.len(), 2, "got: {violations:?}");
    assert!(violations.iter().all(|v| v.rule == "no-first-party-mock"));
    assert!(
        violations
            .iter()
            .any(|v| v.file.ends_with("charge.test.ts")),
        "the `vi.mock('../src/ledger')` form must be flagged; got {violations:?}"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.file.ends_with("notify.test.mts")),
        "the `vi.doMock('./mailer')` form must be flagged; got {violations:?}"
    );
}

#[test]
fn clean_reports_no_violations() {
    let violations = find_integration_violations(fixture("no_first_party_mock/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture mocks only third-party packages and Node built-ins; got {violations:?}"
    );
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(lint_exit("no_first_party_mock/red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(lint_exit("no_first_party_mock/clean"), 0);
}

// The suite tiers derive from the package root — the nearest `package.json` at
// or above the scanned `path` — so a call whose `path` is the package's source
// directory still lints the sibling `tests/integration/` and `tests/e2e/` suites.

#[test]
fn tier_layout_integration_suite_is_linted_from_a_src_scan() {
    assert_eq!(lint_exit("tier_layout/red/src"), 1);
}

#[test]
fn tier_layout_test_outside_a_standard_tier_is_flagged() {
    // `tests/loose.test.ts` sits under the package's `tests/` but in neither
    // `tests/integration/` nor `tests/e2e/` — the `unknown-tier` violation.
    assert_eq!(lint_exit("tier_layout/unknown_tier/src"), 1);
}
