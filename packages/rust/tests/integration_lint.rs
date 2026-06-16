//! Integration tests for the Python integration-test lints
//! (#19; rules #48–#52). Per the #3 guardrail, each lint ships a red fixture
//! (a violation — must be reported) and a clean fixture (must pass).

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

/// Exit code of `integration lint --language python --config <config> <fixture>`.
fn lint_exit_with_config(fixture_name: &str, config_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "integration".into(),
        "lint".into(),
        "--language".into(),
        "python".into(),
        "--config".into(),
        fixture(config_name).into_os_string(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

// ---- R1: forbid `monkeypatch` (#49) --------------------------------------

#[test]
fn monkeypatch_red_reports_a_violation() {
    let violations = find_violations(fixture("monkeypatch/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.iter().any(|v| v.rule == "no-monkeypatch"),
        "the red fixture uses pytest's `monkeypatch` and must be flagged; got {violations:?}"
    );
}

#[test]
fn monkeypatch_clean_reports_no_violations() {
    let violations = find_violations(fixture("monkeypatch/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture patches via a fixture (no monkeypatch); got {violations:?}"
    );
}

#[test]
fn monkeypatch_red_exits_nonzero() {
    assert_eq!(lint_exit("monkeypatch/red"), 1);
}

#[test]
fn monkeypatch_clean_exits_zero() {
    assert_eq!(lint_exit("monkeypatch/clean"), 0);
}

// ---- R2: patches must live in fixtures, not inline (#50) -----------------

#[test]
fn inline_patch_red_flags_the_with_form() {
    let violations = find_violations(fixture("inline_patch/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "no-inline-patch" && v.file.ends_with("inline_with_patch_test.py")),
        "an inline `with patch(...)` in a test body must be flagged; got {violations:?}"
    );
}

#[test]
fn inline_patch_red_flags_the_bare_call() {
    let violations = find_violations(fixture("inline_patch/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "no-inline-patch" && v.file.ends_with("bare_patch_call_test.py")),
        "a bare `patch(...)` call in a test body must be flagged; got {violations:?}"
    );
}

#[test]
fn inline_patch_clean_reports_no_violations() {
    let violations = find_violations(fixture("inline_patch/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture wraps the patch in a fixture; got {violations:?}"
    );
}

#[test]
fn inline_patch_red_exits_nonzero() {
    assert_eq!(lint_exit("inline_patch/red"), 1);
}

#[test]
fn inline_patch_clean_exits_zero() {
    assert_eq!(lint_exit("inline_patch/clean"), 0);
}

// ---- R3: env via patch.dict(os.environ, …) (#51) -------------------------

#[test]
fn environ_red_flags_subscript_assignment() {
    let violations =
        find_violations(fixture("environ/red")).expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "no-environ-mutation"
                && v.file.ends_with("subscript_assignment_test.py")),
        "`os.environ[...] = ...` must be flagged; got {violations:?}"
    );
}

#[test]
fn environ_red_flags_del_statement() {
    let violations =
        find_violations(fixture("environ/red")).expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "no-environ-mutation" && v.file.ends_with("del_statement_test.py")),
        "`del os.environ[...]` must be flagged; got {violations:?}"
    );
}

#[test]
fn environ_red_flags_mutating_method() {
    let violations =
        find_violations(fixture("environ/red")).expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "no-environ-mutation"
                && v.file.ends_with("mutating_method_test.py")),
        "`os.environ.update(...)` must be flagged; got {violations:?}"
    );
}

#[test]
fn environ_clean_reports_no_violations() {
    let violations =
        find_violations(fixture("environ/clean")).expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture sets env via patch.dict in a fixture; got {violations:?}"
    );
}

#[test]
fn environ_red_exits_nonzero() {
    assert_eq!(lint_exit("environ/red"), 1);
}

#[test]
fn environ_clean_exits_zero() {
    assert_eq!(lint_exit("environ/clean"), 0);
}

// ---- R4: don't patch module-global config constants (#52, waivable) -------

#[test]
fn constant_patch_red_reports_a_violation() {
    let violations = find_violations(fixture("constant_patch/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.iter().any(|v| v.rule == "no-constant-patch"),
        "patching a module-global UPPER_CASE constant must be flagged; got {violations:?}"
    );
}

#[test]
fn constant_patch_clean_reports_no_violations() {
    let violations = find_violations(fixture("constant_patch/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture injects config explicitly; got {violations:?}"
    );
}

#[test]
fn constant_patch_red_exits_nonzero() {
    assert_eq!(lint_exit("constant_patch/red"), 1);
}

#[test]
fn constant_patch_waived_exits_zero() {
    // Same patch as the red fixture, but the file is waived in the config.
    assert_eq!(
        lint_exit_with_config(
            "constant_patch/waived",
            "constant_patch/waived/testing-conventions.toml"
        ),
        0
    );
}

// ---- Integration isolation: no first-party patch (#42) -------------------

#[test]
fn first_party_patch_red_reports_a_violation() {
    let violations = find_violations(fixture("no_first_party_patch/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.rule == "no-first-party-patch" && v.file.ends_with("charge_test.py")),
        "patching a first-party target (`myproject.ledger.record`) in an integration test \
         must be flagged; got {violations:?}"
    );
}

#[test]
fn first_party_patch_clean_reports_no_violations() {
    let violations = find_violations(fixture("no_first_party_patch/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture patches only third-party / effectful-stdlib targets \
         (`requests.post`, `subprocess.run`); got {violations:?}"
    );
}

#[test]
fn first_party_patch_red_exits_nonzero() {
    assert_eq!(lint_exit("no_first_party_patch/red"), 1);
}

#[test]
fn first_party_patch_clean_exits_zero() {
    assert_eq!(lint_exit("no_first_party_patch/clean"), 0);
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
