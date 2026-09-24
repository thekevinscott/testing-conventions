use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/workflow_lint")
        .join(name)
}

/// `testing-conventions workflow-lint <fixture>`.
fn workflow_lint(name: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("workflow-lint")
        .arg(fixture(name))
        .output()
        .expect("the built binary should run")
}

#[test]
fn a_clean_tree_exits_zero() {
    let out = workflow_lint("clean");
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn a_red_tree_exits_one_and_names_every_step() {
    let out = workflow_lint("red");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).expect("stderr is utf-8");
    assert!(
        stderr.contains("workflows/publish.yml:13:") && stderr.contains("Publish bootstrap stubs"),
        "{stderr}"
    );
    assert!(
        stderr.contains("action.yml:9:") && stderr.contains("Rewrite the version"),
        "{stderr}"
    );
    assert!(
        stderr.contains("2 step(s) encode logic in CI YAML"),
        "{stderr}"
    );
}

#[test]
fn the_default_path_is_dot_github() {
    // Run with no path argument in a directory that has no `.github`: the default must be a pass,
    // not a missing-path error.
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("workflow-lint")
        .current_dir(fixture("clean/selftest"))
        .output()
        .expect("the built binary should run");
    assert_eq!(out.status.code(), Some(0));
}
