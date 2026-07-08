//! Integration tests for the `functions` and `branch` floors on the Rust
//! coverage config: both keys are part of the `[rust].coverage` schema,
//! accepted by the loader's self-guard alongside `regions` and `lines`.
//!
//! Red until the floors land: today the config self-guard
//! (`deny_unknown_fields`) rejects both keys, so loading these configs errors
//! instead of parsing.

use std::path::PathBuf;

use testing_conventions::config::load_config;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage")
}

#[test]
fn a_functions_floor_is_part_of_the_config_schema() {
    // `[rust] coverage = { lines = 50, functions = 100 }` parses — the loader's
    // self-guard knows the key — and carries the floor.
    let config = load_config(fixtures().join("rust_functions_full.toml"))
        .expect("the `functions` floor should parse");
    let coverage = config
        .rust
        .expect("the [rust] table should load")
        .coverage
        .expect("the coverage table should load");
    assert_eq!(coverage.functions, Some(100));
    assert_eq!(coverage.lines, 50);
    assert_eq!(coverage.branch, None);
}

#[test]
fn a_branch_floor_is_part_of_the_config_schema() {
    // `[rust] coverage = { lines = 50, branch = 100 }` parses — the loader's
    // self-guard knows the key — and carries the floor.
    let config = load_config(fixtures().join("rust_branch_full.toml"))
        .expect("the `branch` floor should parse");
    let coverage = config
        .rust
        .expect("the [rust] table should load")
        .coverage
        .expect("the coverage table should load");
    assert_eq!(coverage.branch, Some(100));
    assert_eq!(coverage.lines, 50);
    assert_eq!(coverage.functions, None);
}
