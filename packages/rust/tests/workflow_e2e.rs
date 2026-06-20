//! E2E tests for the workflow guard (#92): drive the built CLI binary against the
//! fixture workflows and assert the exit code. The guard: a `testing-conventions`
//! invocation in a workflow must name a subcommand the binary still exposes, so a
//! rename can't strand the `@v0` consumption path.
//!
//! These start red — the skeleton's `workflow` command reports nothing, so the red
//! fixture exits `0` instead of `1` — and go green once detection lands.

use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/workflow")
        .join(name)
}

/// Exit code of `testing-conventions workflow <fixture>`.
fn workflow_exit(name: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("workflow")
        .arg(fixture(name))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn red_workflow_exits_nonzero() {
    assert_eq!(workflow_exit("red"), 1);
}

#[test]
fn clean_workflow_exits_zero() {
    assert_eq!(workflow_exit("clean"), 0);
}

/// Stdout of `testing-conventions --help`.
fn help_stdout() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("--help")
        .output()
        .expect("the built binary should run");
    String::from_utf8(out.stdout).expect("--help output is utf-8")
}

#[test]
fn help_does_not_list_the_private_workflow_command() {
    // `workflow` is the private #92 drift guard (#191): it must not be listed as a command
    // in `--help`. `clean_workflow_exits_zero` above proves it still runs when invoked
    // explicitly (hidden, not removed).
    let help = help_stdout();
    assert!(
        !help
            .lines()
            .any(|line| line.trim_start().starts_with("workflow")),
        "`workflow` must not be listed in --help (#191):\n{help}"
    );
}
