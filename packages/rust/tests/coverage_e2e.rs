//! E2E tests for the Python coverage rule: drive the built CLI binary
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

// `full`, `above_85`, and `below_85` are the default package layout —
// `{pyproject.toml, src/**}` — so the codebase handed to the CLI is the `src/` scan path.

#[test]
fn below_85_exits_nonzero_against_an_85_floor() {
    assert_eq!(unit_coverage_exit("below_85/src", "floor85.toml"), 1);
}

#[test]
fn above_85_exits_nonzero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("above_85/src", "floor100.toml"), 1);
}

#[test]
fn full_exits_zero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("full/src", "floor100.toml"), 0);
}

#[test]
fn conftest_omitted_exits_zero_against_a_100_floor() {
    // conftest_omit's conftest.py has an uncovered fixture body; the binary clears
    // the 100 floor only by omitting conftest.py from the denominator.
    assert_eq!(unit_coverage_exit("conftest_omit", "floor100.toml"), 0);
}

#[test]
fn exempt_cov_exits_zero_against_a_100_floor() {
    // The config exempts shim.py from coverage, so the built binary omits it
    // from the denominator and clears the 100 floor end-to-end.
    assert_eq!(
        unit_coverage_exit("exempt_cov", "floor100_exempt_shim.toml"),
        0
    );
}

// Zero-config: a `--config` pointing at a file that doesn't exist falls
// back to the default Python floor — the same way a brand-new library with no
// `testing-conventions.toml` runs. That default is now 100, so only a
// fully-covered suite clears it: `full` (100%) passes, while `above_85` (~86%,
// which cleared the old 85 default) and `below_85` (~71%) both fail.

#[test]
fn full_exits_zero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("full/src", "no-such-config.toml"), 0);
}

#[test]
fn above_85_exits_nonzero_with_no_config_via_the_default_floor() {
    // ~86% cleared the old 85 default; the strict 100 default fails it.
    assert_eq!(unit_coverage_exit("above_85/src", "no-such-config.toml"), 1);
}

#[test]
fn below_85_exits_nonzero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("below_85/src", "no-such-config.toml"), 1);
}
