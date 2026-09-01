mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{tested_count, ts_adapter, GitRepo, Staged, ENGINE_NOT_RUN};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Exit code of `testing-conventions unit mutation --language typescript [--config <cfg>] <project>`.
fn unit_mutation_exit(project: &Path, config: Option<&str>) -> i32 {
    let mut command = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    command
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter());
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
fn run_without_the_adapter_arg_fails_clean() {
    let project = Staged::new("survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg(project.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a missing adapter argument should fail the run; stderr: {stderr}"
    );
    assert!(
        stderr.contains("--ts-mutation-adapter"),
        "the error should name the adapter argument; got: {stderr}"
    );
}

#[test]
fn a_relative_scan_path_runs_from_the_package_root() {
    let project = Staged::new("killed");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .current_dir(project.path())
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter())
        .arg("src")
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("is `node` on PATH?"),
        "a missing working directory must not masquerade as a missing interpreter; got: {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "a relative scan path should clear the gate exactly as the absolute path does; stderr: {stderr}"
    );
}

#[test]
fn a_broken_adapter_path_fails_clean() {
    let project = Staged::new("survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg("/nonexistent/testing-conventions-adapter.js")
        .arg(project.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a broken adapter path should fail the run; stderr: {stderr}"
    );
    assert!(
        stderr.contains("adapter failed"),
        "the error should report the adapter failure; got: {stderr}"
    );
}

#[test]
fn killed_project_passes_and_states_the_tested_count() {
    let package = Staged::new("killed");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter())
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
    let repo = GitRepo::new("ts-vacuous");
    repo.write(
        "index.ts",
        "export function add(a: number, b: number): number {\n  return a + b;\n}\n",
    );
    repo.write(
        "index.test.ts",
        "import { it, expect } from 'vitest';\nimport { add } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n});\n",
    );
    repo.commit("baseline");
    let base = repo.head();
    repo.write(
        "index.test.ts",
        "import { it, expect } from 'vitest';\nimport { add } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n  expect(add(-1, 1)).toBe(0);\n});\n",
    );
    repo.commit("tweak only the test file");

    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter())
        .args(["--base", &base])
        .arg(repo.path())
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "an empty mutate set passes; stderr: {stderr}"
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
fn survivors_fail_the_gate_by_default() {
    let package = Staged::new("survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(ts_adapter())
        .arg(package.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "the assertion-light suite leaves survivors; stderr: {stderr}"
    );
    assert!(
        stderr.contains("unexplained surviving mutant") && stderr.contains("index.ts"),
        "the survivors are listed scan-path-relative; got: {stderr}"
    );
}

#[test]
fn a_loose_tree_fails_the_gate_on_survivors() {
    let project = Staged::loose("loose_survivors");
    assert_eq!(unit_mutation_exit(project.path(), None), 1);
}

#[test]
fn an_exempted_survivor_passes_the_gate() {
    let package = Staged::new("survivors");
    assert_eq!(
        unit_mutation_exit(&package.path().join("src"), Some("mutation_exempt_ts.toml")),
        0
    );
}
