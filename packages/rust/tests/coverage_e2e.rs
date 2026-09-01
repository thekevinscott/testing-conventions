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
    assert_eq!(unit_coverage_exit("conftest_omit", "floor100.toml"), 0);
}

#[test]
fn exempt_cov_exits_zero_against_a_100_floor() {
    assert_eq!(
        unit_coverage_exit("exempt_cov", "floor100_exempt_shim.toml"),
        0
    );
}

#[test]
fn full_exits_zero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("full/src", "no-such-config.toml"), 0);
}

#[test]
fn above_85_exits_nonzero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("above_85/src", "no-such-config.toml"), 1);
}

#[test]
fn below_85_exits_nonzero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("below_85/src", "no-such-config.toml"), 1);
}
