use std::path::{Path, PathBuf};
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage/typescript")
}

/// Exit code + captured stderr of `testing-conventions unit coverage --language typescript <dir>`.
fn unit_coverage_output(dir: &Path) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", "typescript"])
        .arg(dir)
        .output()
        .expect("the built binary should run");
    (
        out.status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn missing_toolchain_fails_clean_without_downloading() {
    let dir =
        std::env::temp_dir().join(format!("tc-ts-cov-e2e-notoolchain-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (code, stderr) = unit_coverage_output(&dir);
    let _ = std::fs::remove_dir_all(&dir);
    assert_ne!(
        code, 0,
        "a missing toolchain should fail the run; stderr: {stderr}"
    );
    assert!(
        stderr.contains("npx --no-install"),
        "the error should name the no-download invocation; got: {stderr}"
    );
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
    assert_eq!(unit_coverage_exit("full/src", "ts_full.toml"), 0);
}

#[test]
fn a_src_scan_below_a_package_root_config_exits_zero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("pkg_config/src", "ts_full.toml"), 0);
}

#[test]
fn above_exits_nonzero_against_a_100_floor() {
    assert_eq!(unit_coverage_exit("above/src", "ts_full.toml"), 1);
}

#[test]
fn above_exits_zero_against_the_mid_floor() {
    assert_eq!(unit_coverage_exit("above/src", "ts_mid.toml"), 0);
}

#[test]
fn below_exits_nonzero_against_the_mid_floor() {
    assert_eq!(unit_coverage_exit("below/src", "ts_mid.toml"), 1);
}

#[test]
fn exempt_cov_exits_zero_with_the_shim_exempted() {
    assert_eq!(
        unit_coverage_exit("exempt_cov", "ts_full_exempt_shim.toml"),
        0
    );
}

#[test]
fn full_exits_zero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("full/src", "no-such-config.toml"), 0);
}

#[test]
fn above_exits_nonzero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("above/src", "no-such-config.toml"), 1);
}

#[test]
fn below_exits_nonzero_with_no_config_via_the_default_floor() {
    assert_eq!(unit_coverage_exit("below/src", "no-such-config.toml"), 1);
}
