mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{tested_count, GitRepo, Staged, ENGINE_NOT_RUN};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Exit code of `testing-conventions unit mutation --language python [--config <cfg>] <project>`.
fn unit_mutation_exit(project: &Path, config: Option<&str>) -> i32 {
    let mut command = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    command.args(["unit", "mutation", "--language", "python"]);
    if let Some(config) = config {
        command.arg("--config").arg(fixtures().join(config));
    }
    command
        .arg(project)
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn killed_project_passes_and_states_the_tested_count() {
    let package = Staged::python("killed");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "python"])
        .arg(package.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "every mutant is caught; stderr: {stderr}"
    );
    assert!(
        tested_count(&stdout) > 0,
        "the engine ran, so the count is non-zero; got: {stdout}"
    );
}

#[test]
fn a_nested_colocated_suite_passes_the_gate() {
    let package = Staged::python_nested("nested_tests");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "python"])
        .arg(package.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("_test.py"),
        "no survivor is reported against the consumer's own suite; got: {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "every mutant is caught; stdout: {stdout} stderr: {stderr}"
    );
    assert!(
        tested_count(&stdout) > 0,
        "the engine ran, so the count is non-zero; got: {stdout}"
    );
}

#[test]
fn a_diff_with_no_mutatable_changed_lines_reports_the_engine_not_run() {
    let repo = GitRepo::new("py-vacuous");
    repo.write("calc.py", "def add(a, b):\n    return a + b\n");
    repo.write(
        "calc_test.py",
        "from calc import add\n\n\ndef test_add():\n    assert add(2, 3) == 5\n",
    );
    repo.commit("baseline");
    let base = repo.head();
    repo.write(
        "calc_test.py",
        "from calc import add\n\n\ndef test_add():\n    assert add(2, 3) == 5\n    assert add(-1, 1) == 0\n",
    );
    repo.commit("tweak only the test file");

    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "python"])
        .args(["--base", &base])
        .arg(repo.path())
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "an empty module set passes; stderr: {stderr}"
    );
    assert!(
        stdout.contains(ENGINE_NOT_RUN),
        "the skip is stated; got: {stdout}"
    );
    assert!(
        !stdout.contains("every mutation was caught"),
        "an engine-skipped pass never claims mutants were caught; got: {stdout}"
    );
}

#[test]
fn a_scan_path_that_is_not_there_names_the_directory_not_the_interpreter() {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "python", "no/such/dir"])
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a scan path that is not there fails the run; stderr: {stderr}"
    );
    assert!(
        !stderr.contains("is `python3` installed?"),
        "a missing working directory must not masquerade as a missing interpreter; got: {stderr}"
    );
    assert!(
        stderr.contains("no/such/dir"),
        "the error names the directory it could not enter; got: {stderr}"
    );
}

#[test]
fn survivors_fail_the_gate_by_default() {
    let package = Staged::python("survivors");
    assert_eq!(unit_mutation_exit(&package.path().join("src"), None), 1);
}

#[test]
fn each_survivor_line_names_the_source_the_mutation_produced() {
    let package = Staged::python("survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "python"])
        .arg(package.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let listed: Vec<&str> = stderr
        .lines()
        .filter(|line| line.trim_start().starts_with("calc.py:"))
        .collect();
    assert!(
        !listed.is_empty(),
        "the assertion-light suite leaves survivors to list; got: {stderr}"
    );
    assert!(
        listed.iter().all(|line| line.contains("(-> ")),
        "each listed survivor names its replacement, not the operator alone; got: {stderr}"
    );
}

#[test]
fn a_loose_tree_fails_the_gate_on_survivors() {
    let project = Staged::python_loose("loose_survivors");
    assert_eq!(unit_mutation_exit(project.path(), None), 1);
}

#[test]
fn an_exempted_survivor_passes_the_gate() {
    let package = Staged::python("survivors");
    assert_eq!(
        unit_mutation_exit(&package.path().join("src"), Some("mutation_exempt_py.toml")),
        0
    );
}
