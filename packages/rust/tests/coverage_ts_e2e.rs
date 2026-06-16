//! E2E tests for the TypeScript coverage rule (#31): drive the built CLI binary
//! end-to-end (no mocks) against the fixture codebases and assert the exit code.
//! Requires Node with the fixtures' vitest toolchain installed (see the suite's
//! `package.json`).

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage/typescript")
}

/// Exit code of `testing-conventions unit coverage --language typescript --config <cfg> <codebase>`.
fn unit_coverage_exit(codebase: &str, config: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", "typescript", "--config"])
        .arg(fixtures().join(config))
        .arg(fixtures().join(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn full_exits_zero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("full", "ts_full.toml"), 0);
}

#[test]
fn above_exits_nonzero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("above", "ts_full.toml"), 1);
}

#[test]
fn above_exits_zero_against_the_mid_floor() {
    assert_eq!(unit_coverage_exit("above", "ts_mid.toml"), 0);
}

#[test]
fn below_exits_nonzero_against_the_mid_floor() {
    assert_eq!(unit_coverage_exit("below", "ts_mid.toml"), 1);
}

#[test]
fn exempt_cov_exits_zero_with_the_shim_exempted() {
    // The config exempts shim.ts from coverage, so the built binary omits it from
    // the denominator and clears the 100 floor end-to-end (#32).
    assert_eq!(
        unit_coverage_exit("exempt_cov", "ts_full_exempt_shim.toml"),
        0
    );
}

// Zero-config (#80): a `--config` pointing at a file that doesn't exist falls
// back to the default TypeScript floors (lines/functions/statements 80,
// branches 75) — the same floors as `ts_mid.toml`, so `above` clears them and
// `below` (100% lines but ~66% branches) fails on branches.

#[test]
fn above_exits_zero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("above", "no-such-config.toml"), 0);
}

#[test]
fn below_exits_nonzero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("below", "no-such-config.toml"), 1);
}
