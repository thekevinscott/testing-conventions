use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_isolation/typescript")
        .join(name)
}

/// Exit code of `testing-conventions unit lint --language typescript <codebase>`.
fn isolation_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "typescript"])
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(isolation_exit("red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(isolation_exit("clean"), 0);
}

#[test]
fn untyped_red_exits_nonzero() {
    assert_eq!(isolation_exit("untyped_mock/red"), 1);
}

#[test]
fn untyped_clean_exits_zero() {
    assert_eq!(isolation_exit("untyped_mock/clean"), 0);
}

#[test]
fn spy_option_clean_exits_zero() {
    assert_eq!(isolation_exit("untyped_mock/spy_clean"), 0);
}

#[test]
fn ext_normalize_clean_exits_zero() {
    assert_eq!(isolation_exit("ext_normalize/clean"), 0);
}

#[test]
fn tier_layout_suites_are_not_unit_subjects() {
    assert_eq!(isolation_exit("tier_layout"), 0);
}
