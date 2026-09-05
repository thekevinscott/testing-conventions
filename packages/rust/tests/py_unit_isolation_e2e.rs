use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_isolation/python")
        .join(name)
}

/// Exit code of `testing-conventions unit lint --language python <codebase>`.
fn isolation_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "python"])
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

/// Exit code of the built binary with `--config`.
fn isolation_exit_with_config(codebase: &str, config: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "python", "--config"])
        .arg(fixture(config))
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
fn waived_exits_zero() {
    assert_eq!(
        isolation_exit_with_config("waived", "waived/testing-conventions.toml"),
        0
    );
}

#[test]
fn legacy_test_prefix_exits_zero() {
    assert_eq!(isolation_exit("legacy_prefix"), 0);
}

#[test]
fn external_red_exits_nonzero() {
    assert_eq!(isolation_exit("external/red"), 1);
}

#[test]
fn external_clean_exits_zero() {
    assert_eq!(isolation_exit("external/clean"), 0);
}

#[test]
fn external_waived_exits_zero() {
    assert_eq!(
        isolation_exit_with_config(
            "external/waived",
            "external/waived/testing-conventions.toml"
        ),
        0
    );
}

#[test]
fn stdlib_private_clean_exits_zero() {
    assert_eq!(isolation_exit("stdlib_private/clean"), 0);
}

#[test]
fn barrel_clean_exits_zero() {
    assert_eq!(isolation_exit("barrel/clean"), 0);
}

#[test]
fn barrel_red_exits_nonzero() {
    assert_eq!(isolation_exit("barrel/red"), 1);
}

#[test]
fn overmatch_red_exits_nonzero() {
    assert_eq!(isolation_exit("overmatch/red"), 1);
}

#[test]
fn overmatch_clean_exits_zero() {
    assert_eq!(isolation_exit("overmatch/clean"), 0);
}

#[test]
fn wrong_module_red_exits_nonzero() {
    assert_eq!(isolation_exit("wrong_module/red"), 1);
}

#[test]
fn tier_layout_suites_are_not_unit_subjects() {
    assert_eq!(isolation_exit("tier_layout"), 0);
}
