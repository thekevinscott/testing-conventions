use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::run;

/// Absolute path to a fixture tree under `tests/fixtures/one_function/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/one_function")
        .join(name)
}

/// Exit code of `unit one-function-per-file --language <language> <fixture>`,
/// with no config — the zero-config default threshold.
fn exit(language: &str, fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "one-function-per-file".into(),
        "--language".into(),
        language.into(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

/// Exit code of the same run with `--config <config>`.
fn exit_with_config(language: &str, fixture_name: &str, config_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "one-function-per-file".into(),
        "--language".into(),
        language.into(),
        "--config".into(),
        fixture(config_name).into_os_string(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

#[test]
fn python_red_exits_nonzero() {
    assert_eq!(exit("python", "python/red"), 1);
}

#[test]
fn python_clean_exits_zero() {
    assert_eq!(exit("python", "python/clean"), 0);
}

#[test]
fn python_raised_fails_at_the_default_threshold() {
    assert_eq!(exit("python", "python/raised"), 1);
}

#[test]
fn python_raised_passes_at_its_configured_threshold() {
    assert_eq!(
        exit_with_config(
            "python",
            "python/raised",
            "python/raised/testing-conventions.toml"
        ),
        0
    );
}

#[test]
fn python_waived_exits_zero() {
    assert_eq!(
        exit_with_config(
            "python",
            "python/waived",
            "python/waived/testing-conventions.toml"
        ),
        0
    );
}

#[test]
fn python_raised_still_flags_a_function_over_its_configured_threshold() {
    assert_eq!(
        exit_with_config(
            "python",
            "python/raised_red",
            "python/raised_red/testing-conventions.toml"
        ),
        1
    );
}

#[test]
fn typescript_red_exits_nonzero() {
    assert_eq!(exit("typescript", "typescript/red"), 1);
}

#[test]
fn typescript_clean_exits_zero() {
    assert_eq!(exit("typescript", "typescript/clean"), 0);
}

#[test]
fn typescript_raised_fails_at_the_default_threshold() {
    assert_eq!(exit("typescript", "typescript/raised"), 1);
}

#[test]
fn typescript_raised_passes_at_its_configured_threshold() {
    assert_eq!(
        exit_with_config(
            "typescript",
            "typescript/raised",
            "typescript/raised/testing-conventions.toml"
        ),
        0
    );
}

#[test]
fn typescript_waived_exits_zero() {
    assert_eq!(
        exit_with_config(
            "typescript",
            "typescript/waived",
            "typescript/waived/testing-conventions.toml"
        ),
        0
    );
}

#[test]
fn typescript_raised_still_flags_a_function_over_its_configured_threshold() {
    assert_eq!(
        exit_with_config(
            "typescript",
            "typescript/raised_red",
            "typescript/raised_red/testing-conventions.toml"
        ),
        1
    );
}

#[test]
fn rust_is_off_until_a_config_opts_in() {
    assert_eq!(exit("rust", "rust/red"), 0);
}

#[test]
fn rust_red_exits_nonzero_once_a_config_opts_in() {
    assert_eq!(
        exit_with_config("rust", "rust/red", "rust/red/testing-conventions.toml"),
        1
    );
}

#[test]
fn rust_clean_exits_zero() {
    assert_eq!(exit("rust", "rust/clean"), 0);
}

#[test]
fn rust_raised_is_unjudged_without_a_config() {
    assert_eq!(exit("rust", "rust/raised"), 0);
}

#[test]
fn rust_raised_passes_at_its_configured_threshold() {
    assert_eq!(
        exit_with_config(
            "rust",
            "rust/raised",
            "rust/raised/testing-conventions.toml"
        ),
        0
    );
}

#[test]
fn rust_waived_exits_zero() {
    assert_eq!(
        exit_with_config(
            "rust",
            "rust/waived",
            "rust/waived/testing-conventions.toml"
        ),
        0
    );
}

#[test]
fn rust_raised_still_flags_a_function_over_its_configured_threshold() {
    assert_eq!(
        exit_with_config(
            "rust",
            "rust/raised_red",
            "rust/raised_red/testing-conventions.toml"
        ),
        1
    );
}
