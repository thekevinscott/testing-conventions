use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/colocated_test")
        .join(name)
}

fn stdout_of(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(args)
        .output()
        .expect("the built binary should run");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr_of(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(args)
        .output()
        .expect("the built binary should run");
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn unit_mutation_help_names_a_check_not_a_gate() {
    let help = stdout_of(&["unit", "mutation", "--help"]);
    assert!(help.contains("The check is on by default"), "got: {help}");
}

#[test]
fn e2e_verify_help_names_checks_not_gates_for_diff_scoping() {
    let help = stdout_of(&["e2e", "verify", "--help"]);
    assert!(
        help.contains("the changed-line coverage/mutation checks read the diff"),
        "got: {help}"
    );
}

#[test]
fn a_scopable_exemption_without_lines_names_the_check_or_rule_boundary() {
    let dir = fixture("scopable_no_lines");
    let stderr = stderr_of(&[
        "unit",
        "colocated-test",
        "--language",
        "python",
        "--config",
        dir.join("testing-conventions.toml").to_str().unwrap(),
        dir.to_str().unwrap(),
    ]);
    assert!(
        stderr.contains(
            "only `coverage` and `mutation` are line-scoped; every other check or rule is \
             whole-file"
        ),
        "got: {stderr}"
    );
}

#[test]
fn lines_alongside_a_whole_file_rule_says_move_the_rest() {
    let dir = fixture("lines_on_whole_file");
    let stderr = stderr_of(&[
        "unit",
        "colocated-test",
        "--language",
        "python",
        "--config",
        dir.join("testing-conventions.toml").to_str().unwrap(),
        dir.to_str().unwrap(),
    ]);
    assert!(
        stderr.contains("move the rest to a separate entry"),
        "got: {stderr}"
    );
}
