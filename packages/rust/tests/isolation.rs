//! Integration tests for the Rust `unit isolation` rule (#44) — the D1 detector: a
//! call out of an inline `#[cfg(test)]` module's own module. Per the #3 guardrail,
//! the rule ships a red fixture (each out-of-module form, must be flagged) and a
//! clean fixture (well-isolated, must pass).

use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::isolation::find_violations;
use testing_conventions::run;

/// Absolute path to a fixture tree under `tests/fixtures/isolation/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/isolation")
        .join(name)
}

/// Exit code of `unit isolation --language rust <fixture>`.
fn iso_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "isolation".into(),
        "--language".into(),
        "rust".into(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

/// `true` when scanning `fixture_name` yields an `no-out-of-module-call` violation
/// in the file ending `file_suffix`.
fn flagged(fixture_name: &str, file_suffix: &str) -> bool {
    find_violations(fixture(fixture_name))
        .expect("walking a readable tree should succeed")
        .iter()
        .any(|v| v.rule == "no-out-of-module-call" && v.file.ends_with(file_suffix))
}

#[test]
fn red_flags_first_party_cross_module_call() {
    assert!(
        flagged("unit/red", "cross_module.rs"),
        "`crate::store::load()` in a unit test must be flagged"
    );
}

#[test]
fn red_flags_effectful_std_call() {
    assert!(
        flagged("unit/red", "effectful_std.rs"),
        "`std::fs::read(...)` in a unit test must be flagged"
    );
}

#[test]
fn red_flags_external_crate_call() {
    assert!(
        flagged("unit/red", "external_crate.rs"),
        "`rand::random()` in a unit test must be flagged"
    );
}

#[test]
fn red_flags_ancestor_module_reach() {
    assert!(
        flagged("unit/red", "ancestor.rs"),
        "`super::super::util::help()` in a unit test must be flagged"
    );
}

#[test]
fn clean_reports_no_violations() {
    let violations =
        find_violations(fixture("unit/clean")).expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture is well-isolated (super:: + injected double + Cursor); got {violations:?}"
    );
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(iso_exit("unit/red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(iso_exit("unit/clean"), 0);
}

#[test]
fn isolation_requires_language() {
    // Omitting `--language` is a usage error, never a silent run.
    let err = run(["testing-conventions", "unit", "isolation", "src"])
        .expect_err("--language is required");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("a missing required flag should surface as a clap::Error");
    assert_eq!(
        clap_err.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}
