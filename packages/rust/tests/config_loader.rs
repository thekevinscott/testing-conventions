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
                regions: Some(100),
                lines: 100,
                functions: None,
                branch: None,
            }),
            features: vec![],
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
    // A whole-file presence exemption and a separate line-scoped coverage exemption for
    // the same file — `coverage` requires `lines`, so the two can't share one entry (#226).
    assert_eq!(python.exempt.len(), 2);
    assert_eq!(python.exempt[0].path, "src/cli.py");
    assert_eq!(python.exempt[0].rules, vec![Rule::ColocatedTest]);
    assert!(python.exempt[0].lines.is_empty());
    assert_eq!(python.exempt[1].rules, vec![Rule::Coverage]);
    assert_eq!(
        python.exempt[1].lines,
        vec![testing_conventions::config::LineSpec::Range(5, 6)]
    );
    assert_eq!(
        config.typescript.expect("[typescript] table").exempt[0].rules,
        vec![Rule::ColocatedTest]
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

#[test]
fn rejects_an_exemption_with_a_blank_reason_self_guard() {
    // Distinct from a *missing* reason: the `reason` key is present but blank.
    // The loader's validation step must still reject it on load.
    assert!(
        load_config(fixture("exempt_empty_reason.toml")).is_err(),
        "an exemption with a blank reason must be rejected (self-guard)"
    );
}

#[test]
fn partial_coverage_tables_inherit_defaults() {
    // Each table sets only one field; the rest fall back to the language's default
    // floor (#216). Previously a partial table errored on the required fields.
    let config = load_config(fixture("partial_coverage.toml"))
        .expect("a partial coverage table should load, filling defaults");
    assert_eq!(
        config.python.expect("[python]").coverage.expect("coverage"),
        PythonCoverage {
            branch: true,
            fail_under: 90,
        }
    );
    assert_eq!(
        config
            .typescript
            .expect("[typescript]")
            .coverage
            .expect("coverage"),
        TypeScriptCoverage {
            lines: 100,
            branches: 90,
            functions: 100,
            statements: 100,
        }
    );
    assert_eq!(
        config.rust.expect("[rust]").coverage.expect("coverage"),
        RustCoverage {
            regions: Some(90),
            lines: 100,
            functions: None,
            branch: None,
        }
    );
}

#[test]
fn an_unknown_field_in_a_coverage_table_still_errors() {
    // Field defaults fill *missing* keys; a typo'd key is still rejected.
    assert!(
        load_config(fixture("unknown_coverage_field.toml")).is_err(),
        "an unknown key inside a coverage table must still be rejected"
    );
}
