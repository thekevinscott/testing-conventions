use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/colocated_test")
}

/// Exit code of `unit colocated-test --language <lang> --config <dir>/testing-conventions.toml <dir>`,
/// run as a real subprocess against the built binary.
fn unit_colocated_test_exit(fixture: &str, language: &str) -> i32 {
    let dir = fixtures().join(fixture);
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "colocated-test", "--language", language, "--config"])
        .arg(dir.join("testing-conventions.toml"))
        .arg(&dir)
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn config_exemptions_clear_the_tree() {
    assert_eq!(unit_colocated_test_exit("python_exempt", "python"), 0);
}

#[test]
fn a_blank_reason_exemption_makes_the_binary_error() {
    assert_eq!(unit_colocated_test_exit("bad_exempt", "python"), 1);
}

#[test]
fn conftest_is_not_an_orphan() {
    assert_eq!(unit_colocated_test_exit("python_conftest", "python"), 0);
}

/// Exit code + stderr of the same invocation, for assertions that inspect the
/// reported orphan (not just the exit code).
fn unit_colocated_test_output(fixture: &str, language: &str) -> (i32, String) {
    let dir = fixtures().join(fixture);
    let output = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "colocated-test", "--language", language, "--config"])
        .arg(dir.join("testing-conventions.toml"))
        .arg(&dir)
        .output()
        .expect("the built binary should run");
    (
        output
            .status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn rust_clean_tree_exits_zero() {
    assert_eq!(unit_colocated_test_exit("rust/clean", "rust"), 0);
}

#[test]
fn rust_red_tree_flags_the_untested_module() {
    let (code, stderr) = unit_colocated_test_output("rust/red", "rust");
    assert_eq!(
        code, 1,
        "a missing inline test must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("untested.rs"),
        "stderr should name the orphan module; got: {stderr}"
    );
}

#[test]
fn rust_cfg_not_test_module_is_flagged() {
    let (code, stderr) = unit_colocated_test_output("rust/cfg_not_test", "rust");
    assert_eq!(
        code, 1,
        "a `not(test)` module is not a test module; stderr: {stderr}"
    );
    assert!(
        stderr.contains("gated.rs"),
        "stderr should name the orphan module; got: {stderr}"
    );
}

#[test]
fn python_suite_helpers_are_not_colocated_subjects() {
    assert_eq!(unit_colocated_test_exit("python_tiers", "python"), 0);
}

#[test]
fn typescript_suite_helpers_are_not_colocated_subjects() {
    assert_eq!(
        unit_colocated_test_exit("typescript_tiers", "typescript"),
        0
    );
}

#[test]
fn type_only_typescript_modules_do_not_fail_the_run() {
    let (code, stderr) = unit_colocated_test_output("typescript/type_only", "typescript");
    assert_eq!(
        code, 0,
        "type-only modules are not subjects; stderr: {stderr}"
    );
}

#[test]
fn a_mixed_type_and_runtime_module_still_fails_the_run() {
    let (code, stderr) = unit_colocated_test_output("typescript/type_only_mixed", "typescript");
    assert_eq!(
        code, 1,
        "a module with runtime code stays a subject; stderr: {stderr}"
    );
    assert!(
        stderr.contains("mixed.ts"),
        "stderr should name the orphan module; got: {stderr}"
    );
}
