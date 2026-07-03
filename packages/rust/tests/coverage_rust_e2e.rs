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
fn padded_exits_nonzero_against_a_100_floor() {
    // `padded`'s `shift` unit is covered only by its integration test; the floor
    // measures the unit suite alone, so the crate fails 100 end-to-end (#265).
    assert_eq!(unit_coverage_exit("padded", "rust_full.toml"), 1);
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

// Zero-config (#206): a `--config` pointing at a file that doesn't exist falls
// back to the default Rust floor — the same way a brand-new crate with no
// `testing-conventions.toml` runs. That default is `lines = 100` with `regions`
// opt-in (#206), so a fully-covered crate clears it while a below-floor crate
// fails — Rust no longer errors out demanding an explicit `[rust].coverage` table.

#[test]
fn above_exits_zero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("above", "no-such-config.toml"), 0);
}

#[test]
fn below_exits_nonzero_with_no_config_via_the_default_floor() {
    // `below` leaves the `else` arm's line uncovered, so it fails the 100 line
    // default even though `regions` isn't part of the zero-config floor.
    assert_eq!(unit_coverage_exit("below", "no-such-config.toml"), 1);
}
