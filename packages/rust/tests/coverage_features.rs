//! Integration tests for cargo-feature passthrough in the config: a
//! `[rust] features` list is part of the schema, accepted by the loader's
//! self-guard alongside `coverage` and `exempt`.
//!
//! Red until feature passthrough lands: today the config self-guard
//! (`deny_unknown_fields`) rejects the `features` key, so loading these
//! configs errors instead of parsing.

use std::path::PathBuf;

use testing_conventions::config::load_config;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn a_rust_features_list_is_part_of_the_config_schema() {
    // `[rust] features = ["boost"]` next to a `coverage` table parses — the
    // loader's self-guard knows the key — and carries the listed features.
    let config = load_config(fixtures().join("unit_coverage/rust_features_full.toml"))
        .expect("the `features` key should parse");
    assert_eq!(
        config.rust.expect("the [rust] table should load").features,
        vec!["boost".to_string()]
    );
}

#[test]
fn a_rust_features_list_parses_without_a_coverage_table() {
    // `features` stands on its own — the mutation config carries only the list.
    let config = load_config(fixtures().join("unit_mutation/rust_features.toml"))
        .expect("the `features` key should parse");
    assert_eq!(
        config.rust.expect("the [rust] table should load").features,
        vec!["boost".to_string()]
    );
}
