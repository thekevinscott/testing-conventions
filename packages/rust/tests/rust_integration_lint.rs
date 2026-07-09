//! Integration tests for the Rust `integration lint` rule —
//! `no-first-party-double`: a `tests/` integration crate runs first-party code for
//! real, so doubling a first-party item with `#[double]` is a violation. Ships a
//! red fixture (must be flagged) and a clean fixture (must pass).

use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::isolation::find_integration_violations;
use testing_conventions::run;

/// Absolute path to a fixture crate under `tests/fixtures/isolation/integration/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/isolation/integration")
        .join(name)
}

/// Exit code of `integration lint --language rust <fixture>`.
fn lint_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "integration".into(),
        "lint".into(),
        "--language".into(),
        "rust".into(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

#[test]
fn red_flags_first_party_double() {
    let violations = find_integration_violations(fixture("red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.iter().any(|v| v.rule == "no-first-party-double"),
        "`#[double] use widget::Renderer` doubles the crate under test and must be flagged; got {violations:?}"
    );
}

#[test]
fn clean_reports_no_violations() {
    let violations = find_integration_violations(fixture("clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture runs first-party for real and doubles only `rand`; got {violations:?}"
    );
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(lint_exit("red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(lint_exit("clean"), 0);
}

#[test]
fn waived_first_party_double_exits_zero() {
    // The first-party double in `waived/` is lifted by its testing-conventions.toml.
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "integration".into(),
        "lint".into(),
        "--language".into(),
        "rust".into(),
        "--config".into(),
        fixture("waived/testing-conventions.toml").into_os_string(),
        fixture("waived").into_os_string(),
    ];
    assert_eq!(run(argv).expect("a readable tree should not error"), 0);
}

#[test]
fn tier_layout_tests_crate_is_linted_from_a_src_scan() {
    // The integration suite derives from the crate root — the nearest
    // `Cargo.toml` at or above the scanned `path` — so scanning `src/` still
    // lints the crate's `tests/` directory (cargo's own suite layout).
    assert_eq!(lint_exit("derived/src"), 1);
}
