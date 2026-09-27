use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/isolation")
        .join(name)
}

/// Exit code of `testing-conventions unit lint --language rust <codebase>`. `--config` names
/// the fixture's own file — usually absent — so the default relative to the crate root cannot
/// apply this crate's exemptions to a fixture tree.
fn iso_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "rust", "--config"])
        .arg(fixture(codebase).join("testing-conventions.toml"))
        .arg(fixture(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(iso_exit("unit/red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(iso_exit("unit/clean"), 0);
}

#[test]
fn cfg_not_test_exits_zero() {
    assert_eq!(iso_exit("unit/cfg_not_test"), 0);
}

#[test]
fn imports_red_exits_nonzero() {
    assert_eq!(iso_exit("imports/red"), 1);
}

#[test]
fn imports_clean_exits_zero() {
    assert_eq!(iso_exit("imports/clean"), 0);
}

#[test]
fn local_build_exits_zero() {
    assert_eq!(iso_exit("unit/local_build"), 0);
}

/// The provisioning test spawns real threads because the lock it proves *is* concurrency;
/// the crate's own config exempts that import, so the shipped binary must not report it.
#[test]
fn this_crates_thread_reach_is_exempted() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "rust", "--config"])
        .arg(root.join("testing-conventions.toml"))
        .arg(&root)
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8(out.stderr).expect("stderr should be utf-8");
    assert!(
        !stderr.contains("std::thread"),
        "the exemption should lift the provisioning test's thread import; got {stderr}"
    );
}

#[test]
fn waived_exits_zero() {
    let code = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "lint", "--language", "rust", "--config"])
        .arg(fixture("unit/waived/testing-conventions.toml"))
        .arg(fixture("unit/waived"))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code");
    assert_eq!(code, 0);
}
