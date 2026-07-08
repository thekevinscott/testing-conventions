//! Integration tests for the Rust `unit lint` rule. Two detectors: D1
//! flags a call out of an inline `#[cfg(test)]` module's own module
//! (`no-out-of-module-call`); D2 flags a foreign `use` import
//! (`no-out-of-module-import`). Each ships a red fixture (every out-of-module
//! form, must be flagged) and a clean fixture (must pass).

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

/// Exit code of `unit lint --language rust <fixture>`.
fn iso_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
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
fn cfg_not_test_module_is_not_linted_as_test_code() {
    // A `#[cfg(not(test))]` module compiles in non-test builds — it is production
    // code, not a unit test — so its first-party cross-module call must not be
    // flagged. Treating `not(test)` as a test module is a false red.
    let violations = find_violations(fixture("unit/cfg_not_test"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "production code under `#[cfg(not(test))]` must not be linted as a unit test; got {violations:?}"
    );
}

#[test]
fn cfg_not_test_exits_zero() {
    assert_eq!(iso_exit("unit/cfg_not_test"), 0);
}

/// `true` when scanning `fixture_name` yields a `no-out-of-module-import`
/// violation in the file ending `file_suffix`.
fn import_flagged(fixture_name: &str, file_suffix: &str) -> bool {
    find_violations(fixture(fixture_name))
        .expect("walking a readable tree should succeed")
        .iter()
        .any(|v| v.rule == "no-out-of-module-import" && v.file.ends_with(file_suffix))
}

#[test]
fn imports_red_flags_first_party_glob() {
    assert!(
        import_flagged("imports/red", "first_party_glob.rs"),
        "`use crate::other::*` in a unit test must be flagged"
    );
}

#[test]
fn imports_red_flags_first_party_named() {
    assert!(
        import_flagged("imports/red", "first_party_named.rs"),
        "`use crate::other::Thing` in a unit test must be flagged"
    );
}

#[test]
fn imports_red_flags_external_crate() {
    assert!(
        import_flagged("imports/red", "external_named.rs"),
        "`use rand::Rng` in a unit test must be flagged"
    );
}

#[test]
fn imports_red_flags_effectful_std() {
    assert!(
        import_flagged("imports/red", "effectful_std.rs"),
        "`use std::fs` in a unit test must be flagged"
    );
}

#[test]
fn imports_clean_reports_no_violations() {
    let violations =
        find_violations(fixture("imports/clean")).expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture imports only super:: and pure std; got {violations:?}"
    );
}

#[test]
fn imports_red_exits_nonzero() {
    assert_eq!(iso_exit("imports/red"), 1);
}

#[test]
fn imports_clean_exits_zero() {
    assert_eq!(iso_exit("imports/clean"), 0);
}

#[test]
fn isolation_requires_language() {
    // Omitting `--language` is a usage error, never a silent run.
    let err =
        run(["testing-conventions", "unit", "lint", "src"]).expect_err("--language is required");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("a missing required flag should surface as a clap::Error");
    assert_eq!(
        clap_err.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

/// Exit code of `unit lint --language rust --config <config> <fixture>`.
fn iso_exit_config(fixture_name: &str, config_rel: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
        "--language".into(),
        "rust".into(),
        "--config".into(),
        fixture(config_rel).into_os_string(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

#[test]
fn waived_out_of_module_call_exits_zero() {
    // The out-of-module call in `unit/waived` is lifted by its testing-conventions.toml.
    assert_eq!(
        iso_exit_config("unit/waived", "unit/waived/testing-conventions.toml"),
        0
    );
}

#[test]
fn stale_exempt_entry_errors() {
    // A stale exempt path must make the run error, not silently pass.
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
        "--language".into(),
        "rust".into(),
        "--config".into(),
        fixture("unit/stale_exempt.toml").into_os_string(),
        fixture("unit/waived").into_os_string(),
    ];
    assert!(
        run(argv).is_err(),
        "a stale exempt entry must error, not silently pass"
    );
}

// ---- #393: the source walk skips tests/ and target/ ----------------------

#[test]
fn local_build_crate_neither_aborts_nor_false_flags() {
    // A locally-built crate carries `target/` (built artifacts) and `tests/fixtures/`
    // (an intentionally-broken `.rs` plus a `#[cfg(test)]` module that reaches out of
    // module). The unit-isolation walk scans only the crate's own unit source, so it
    // neither aborts on the unparsable fixture nor false-flags the non-unit trees.
    let violations = find_violations(fixture("unit/local_build"))
        .expect("a locally-built crate must not abort the rule on a tests/ or target/ file");
    assert!(
        violations.is_empty(),
        "target/ and tests/ must be skipped, so nothing is flagged; got {violations:?}"
    );
}

#[test]
fn local_build_crate_exits_zero() {
    assert_eq!(iso_exit("unit/local_build"), 0);
}
