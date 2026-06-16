//! E2E tests for the Python coverage rule (#26): drive the built CLI binary
//! end-to-end (no mocks) against the fixture codebases and assert the exit code.
//! Requires `coverage` + `pytest` on PATH.

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage")
}

/// Exit code of `testing-conventions unit coverage --language python --config <cfg> <codebase>`.
fn unit_coverage_exit(codebase: &str, config: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", "python", "--config"])
        .arg(fixtures().join(config))
        .arg(fixtures().join("python").join(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn below_85_exits_nonzero_against_an_85_floor() {
    assert_eq!(unit_coverage_exit("below_85", "floor85.toml"), 1);
}

#[test]
fn above_85_exits_nonzero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("above_85", "floor100.toml"), 1);
}

#[test]
fn full_exits_zero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("full", "floor100.toml"), 0);
}

#[test]
fn conftest_omitted_exits_zero_against_a_100_floor() {
    // conftest_omit's conftest.py has an uncovered fixture body; the binary clears
    // the 100 floor only by omitting conftest.py from the denominator (#112).
    assert_eq!(unit_coverage_exit("conftest_omit", "floor100.toml"), 0);
}

#[test]
fn exempt_cov_exits_zero_against_a_100_floor() {
    // The config exempts shim.py from coverage, so the built binary omits it
    // from the denominator and clears the 100 floor end-to-end (#32).
    assert_eq!(
        unit_coverage_exit("exempt_cov", "floor100_exempt_shim.toml"),
        0
    );
}

// Zero-config (#80): a `--config` pointing at a file that doesn't exist falls
// back to the default Python floor (branch on, 85) — the same way a brand-new
// library with no `testing-conventions.toml` runs. The default is specifically
// 85, not 100: `above_85` (over 85, under 100) clears it.

#[test]
fn full_exits_zero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("full", "no-such-config.toml"), 0);
}

#[test]
fn above_85_exits_zero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("above_85", "no-such-config.toml"), 0);
}

#[test]
fn below_85_exits_nonzero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("below_85", "no-such-config.toml"), 1);
}
