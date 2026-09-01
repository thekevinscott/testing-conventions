use std::path::PathBuf;

use testing_conventions::colocated_test::Language;
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
            one_function_per_file: None,
            exempt: vec![],
            build_command: None,
            reason: String::new(),
        }),
        typescript: Some(TypeScriptConfig {
            coverage: Some(TypeScriptCoverage {
                lines: 100,
                branches: 100,
                functions: 100,
                statements: 100,
            }),
            one_function_per_file: None,
            exempt: vec![],
            build_command: None,
            reason: String::new(),
        }),
        rust: Some(RustConfig {
            coverage: Some(RustCoverage {
                regions: Some(100),
                lines: 100,
                functions: None,
                branch: None,
            }),
            features: vec![],
            one_function_per_file: None,
            exempt: vec![],
            build_command: None,
            reason: String::new(),
        }),
        e2e: None,
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
    let config = load_config(fixture("exempt.toml")).expect("an exempt-only config should load");
    let python = config.python.expect("[python] table present");
    assert!(python.coverage.is_none(), "coverage is optional");
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
    assert!(
        load_config(fixture("exempt_no_reason.toml")).is_err(),
        "an exemption missing its reason must be rejected (self-guard)"
    );
}

#[test]
fn rejects_an_exemption_with_a_blank_reason_self_guard() {
    assert!(
        load_config(fixture("exempt_empty_reason.toml")).is_err(),
        "an exemption with a blank reason must be rejected (self-guard)"
    );
}

#[test]
fn loads_a_python_build_command_with_a_reason() {
    let config = load_config(fixture("python_build_command.toml"))
        .expect("a [python].build_command with a reason should load");
    let python = config.python.expect("[python] table present");
    assert_eq!(
        python.build_command.as_deref(),
        Some("uv run maturin develop")
    );
    assert!(!python.reason.trim().is_empty(), "the reason must survive");
}

#[test]
fn a_python_build_command_with_no_reason_loads() {
    assert!(
        load_config(fixture("python_build_command_no_reason.toml")).is_ok(),
        "a [python].build_command with no reason must load (reason is optional)"
    );
    assert!(
        load_config(fixture("python_build_command_blank_reason.toml")).is_ok(),
        "a [python].build_command with a blank reason must load (reason is optional)"
    );
}

#[test]
fn loads_a_typescript_build_command_with_a_reason() {
    let config = load_config(fixture("typescript_build_command.toml"))
        .expect("a [typescript].build_command must load once the schema generalizes");
    let ts = config.typescript.expect("[typescript] table present");
    assert_eq!(ts.build_command.as_deref(), Some("pnpm build"));
    assert!(
        !ts.reason.trim().is_empty(),
        "the reason note survives when present"
    );
}

#[test]
fn loads_a_rust_build_command_with_a_reason() {
    let config = load_config(fixture("rust_build_command.toml"))
        .expect("a [rust].build_command with a reason must load once the schema generalizes");
    let rust = config.rust.expect("[rust] table present");
    assert_eq!(
        rust.build_command.as_deref(),
        Some("cargo run --bin codegen")
    );
    assert!(!rust.reason.trim().is_empty(), "the reason must survive");
}

#[test]
fn a_typescript_build_command_with_a_blank_reason_loads() {
    assert!(
        load_config(fixture("typescript_build_command_blank_reason.toml")).is_ok(),
        "a [typescript].build_command with a blank reason must load (reason is optional)"
    );
}

#[test]
fn loads_an_e2e_extra_scope_and_exclude_table() {
    let config = load_config(fixture("e2e_extra_scope.toml"))
        .expect("an [e2e] extra_scope/exclude config must load (the schema must accept the table)");
    let e2e = config.e2e.expect("[e2e] table present");
    assert_eq!(e2e.extra_scope, vec!["packages/rust/src"]);
    assert_eq!(
        e2e.exclude,
        vec!["packages/rust/src/cli", "packages/rust/src/bin"]
    );
}

#[test]
fn e2e_table_keys_are_optional() {
    let config = load_config(fixture("e2e_extra_scope_only.toml"))
        .expect("an [e2e] table with only extra_scope should load");
    let e2e = config.e2e.expect("[e2e] table present");
    assert_eq!(e2e.extra_scope, vec!["packages/rust/src"]);
    assert!(e2e.exclude.is_empty(), "exclude defaults to empty");
}

#[test]
fn rejects_an_unknown_e2e_key_self_guard() {
    assert!(
        load_config(fixture("e2e_unknown_key.toml")).is_err(),
        "an unknown key under [e2e] must be rejected (self-guard)"
    );
}

#[test]
fn partial_coverage_tables_inherit_defaults() {
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
    assert!(
        load_config(fixture("unknown_coverage_field.toml")).is_err(),
        "an unknown key inside a coverage table must still be rejected"
    );
}

#[test]
fn an_unknown_key_error_points_at_migrations() {
    let err =
        load_config(fixture("unknown_key.toml")).expect_err("an unknown key must be rejected");
    let chain = format!("{err:#}");
    assert!(
        chain.contains("MIGRATIONS.md"),
        "the unknown-key error must point at MIGRATIONS.md, got: {chain}"
    );
}

#[test]
fn an_unknown_nested_key_error_points_at_migrations() {
    let err = load_config(fixture("e2e_unknown_key.toml"))
        .expect_err("an unknown [e2e] key must be rejected");
    let chain = format!("{err:#}");
    assert!(
        chain.contains("MIGRATIONS.md"),
        "the nested unknown-key error must point at MIGRATIONS.md, got: {chain}"
    );
}

#[test]
fn one_function_per_file_thresholds_are_per_language_and_default_to_one() {
    let config = load_config(fixture("one_function_per_file.toml"))
        .expect("a one_function_per_file table should load");
    assert_eq!(config.one_function_threshold(Language::Python), Some(5));
    assert_eq!(
        config.one_function_threshold(Language::TypeScript),
        Some(20)
    );
    assert_eq!(config.one_function_threshold(Language::Rust), None);
}

#[test]
fn python_and_typescript_default_to_one_line_without_a_table() {
    let config = Config::default();
    assert_eq!(config.one_function_threshold(Language::Python), Some(1));
    assert_eq!(config.one_function_threshold(Language::TypeScript), Some(1));
    assert_eq!(config.one_function_threshold(Language::Rust), None);
}

#[test]
fn a_rust_table_opts_the_language_in() {
    let config = load_config(fixture("one_function_per_file_rust.toml"))
        .expect("a rust one_function_per_file table should load");
    assert_eq!(config.one_function_threshold(Language::Rust), Some(8));
}

#[test]
fn one_function_per_file_is_a_waivable_rule_id() {
    assert_eq!(
        Rule::from_id("one-function-per-file"),
        Some(Rule::OneFunctionPerFile)
    );
    assert_eq!(Rule::OneFunctionPerFile.id(), "one-function-per-file");
    assert!(!Rule::OneFunctionPerFile.is_line_scopable());
}
