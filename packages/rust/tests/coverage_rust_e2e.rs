//! E2E tests for the Rust coverage rule (#37): drive the built CLI binary
//! end-to-end (no mocks) against the fixture crates and assert the exit code.
//! Requires `cargo-llvm-cov`.

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage")
}

/// Exit code of `testing-conventions unit coverage --language rust --config <cfg> <crate>`.
fn unit_coverage_exit(crate_name: &str, config: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", "rust", "--config"])
        .arg(fixtures().join(config))
        .arg(fixtures().join("rust").join(crate_name))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn above_exits_zero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("above", "rust_full.toml"), 0);
}

#[test]
fn below_exits_nonzero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("below", "rust_full.toml"), 1);
}

#[test]
fn below_exits_zero_against_a_lower_floor() {
    assert_eq!(unit_coverage_exit("below", "rust_mid.toml"), 0);
}

#[test]
fn exempt_cov_exits_zero_with_the_shim_exempted() {
    // The config exempts src/shim.rs from coverage, so the built binary omits it
    // from the denominator (via `--ignore-filename-regex`) and clears the 100
    // floor end-to-end (#32).
    assert_eq!(
        unit_coverage_exit("exempt_cov", "rust_full_exempt_shim.toml"),
        0
    );
}
