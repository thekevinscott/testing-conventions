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
    // Every mutant is caught, so the project clears the gate — and the success line
    // states how many mutants the engine judged, the evidence telling this pass apart
    // from an engine-skipped one. The default package layout is scanned at `src/`.
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
fn a_diff_with_no_mutatable_changed_lines_reports_the_engine_not_run() {
    // Only a test file changes on the diff, so the run is skipped — and the output says
    // the engine never ran, distinct from the all-killed success, keeping the vacuous
    // pass visible in the job log. The exit code stays 0: an empty diff owes no run.
    // The skip happens before the adapter is spawned, so this holds with no cosmic-ray
    // in the environment.
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
    // `run_py_adapter` spawned with `current_dir(root)` unchecked, and
    // `Command::output()` reports a missing working directory with the same ENOENT as a
    // missing interpreter — so a scan path that isn't there read as ``is `python3`
    // installed?``. That is the #478 mislabel, which was closed for the TypeScript arm
    // and left standing here. The failure must name the directory it could not enter.
    // No adapter is spawned, so this holds with no cosmic-ray in the environment.
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
    // The gate is on by default and binary: an un-exempted surviving mutant fails the
    // run, no config required. The default package layout is scanned at `src/`.
    let package = Staged::python("survivors");
    assert_eq!(unit_mutation_exit(&package.path().join("src"), None), 1);
}

#[test]
fn a_loose_tree_fails_the_gate_on_survivors() {
    // The loose special case: flat scripts, no manifest, scanned at the root. The gate still
    // runs cosmic-ray in place there and fails on the un-exempted survivor.
    let project = Staged::python_loose("loose_survivors");
    assert_eq!(unit_mutation_exit(project.path(), None), 1);
}

#[test]
fn an_exempted_survivor_passes_the_gate() {
    // The survivor's file carries a `mutation` exemption, so the gate clears it (an
    // equivalent / deliberately-defensive mutation, lifted with a reason) — the only
    // way to pass with a survivor present.
    let package = Staged::python("survivors");
    assert_eq!(
        unit_mutation_exit(&package.path().join("src"), Some("mutation_exempt_py.toml")),
        0
    );
}
