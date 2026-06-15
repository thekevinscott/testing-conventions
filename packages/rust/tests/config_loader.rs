//! Integration tests for the config schema + loader (issue #12).
//!
//! These pin the contract from the README's "Configuration" section: one config
//! file is read into the in-memory `Config`, and the self-guard rejects a config
//! that fails its own validation (unknown keys, malformed TOML) rather than
//! silently accepting it.
//!
//! Per the #3 guardrail, the loader ships a clean fixture (`valid.toml`, must
//! load) and red fixtures (`unknown_key.toml` / `malformed.toml`, must fail).

use std::path::PathBuf;

use testing_conventions::config::{
    load_config, Config, PythonConfig, PythonCoverage, Rule, RustConfig, RustCoverage,
    TypeScriptConfig, TypeScriptCoverage,
};

/// Absolute path to a file under `tests/fixtures/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The in-memory shape we expect `valid.toml` to parse into.
fn expected_valid() -> Config {
    Config {
        python: Some(PythonConfig {
            coverage: Some(PythonCoverage {
                branch: true,
                fail_under: 100,
            }),
            exempt: vec![],
        }),
        typescript: Some(TypeScriptConfig {
            coverage: Some(TypeScriptCoverage {
                lines: 100,
                branches: 100,
                functions: 100,
                statements: 100,
            }),
            exempt: vec![],
        }),
        rust: Some(RustConfig {
            coverage: Some(RustCoverage {
                regions: 100,
                lines: 100,
            }),
            exempt: vec![],
        }),
    }
}

#[test]
fn loads_the_canonical_config_into_memory() {
    let config = load_config(fixture("valid.toml")).expect("the canonical config should load");
    assert_eq!(config, expected_valid());
}

#[test]
fn rejects_unknown_keys_self_guard() {
    let result = load_config(fixture("unknown_key.toml"));
    assert!(
        result.is_err(),
        "an unknown config key must be rejected (self-guard), got: {result:?}"
    );
}

#[test]
fn rejects_malformed_toml() {
    let result = load_config(fixture("malformed.toml"));
    assert!(
        result.is_err(),
        "malformed TOML must be rejected, got: {result:?}"
    );
}

#[test]
fn errors_on_a_missing_file() {
    let result = load_config(fixture("does_not_exist.toml"));
    assert!(
        result.is_err(),
        "a missing config file must be an error, got: {result:?}"
    );
}

#[test]
fn loads_exemptions_with_optional_coverage() {
    // `exempt.toml` declares exemptions but no coverage thresholds — both keys
    // are optional (issue #32).
    let config = load_config(fixture("exempt.toml")).expect("an exempt-only config should load");
    let python = config.python.expect("[python] table present");
    assert!(python.coverage.is_none(), "coverage is optional");
    assert_eq!(python.exempt.len(), 1);
    assert_eq!(python.exempt[0].path, "src/cli.py");
    assert_eq!(python.exempt[0].rules, vec![Rule::Location, Rule::Coverage]);
    assert_eq!(
        config.typescript.expect("[typescript] table").exempt[0].rules,
        vec![Rule::Location]
    );
}

#[test]
fn rejects_an_exemption_without_a_reason_self_guard() {
    // The reason is required — a reasonless exemption can never be a silent pass.
    assert!(
        load_config(fixture("exempt_no_reason.toml")).is_err(),
        "an exemption missing its reason must be rejected (self-guard)"
    );
}
