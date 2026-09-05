mod common;

use std::path::PathBuf;
use std::process::Command;

use common::{tested_count, GitRepo, ENGINE_NOT_RUN, NOTHING_TESTED};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Exit code of `testing-conventions unit mutation --language rust [--config <cfg>] <crate>`.
fn unit_mutation_exit(crate_name: &str, config: Option<&str>) -> i32 {
    let mut command = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    command.args(["unit", "mutation", "--language", "rust"]);
    if let Some(config) = config {
        command.arg("--config").arg(fixtures().join(config));
    }
    command
        .arg(fixtures().join("rust").join(crate_name))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn killed_crate_passes_and_states_the_tested_count() {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .arg(fixtures().join("rust").join("killed"))
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
fn a_diff_without_crate_changes_reports_the_engine_not_run() {
    let repo = GitRepo::new("rust-vacuous");
    repo.write(
        "crate/Cargo.toml",
        "[package]\nname = \"tc_mut_vacuous\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n",
    );
    repo.write(
        "crate/src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    );
    repo.write("notes.md", "before\n");
    repo.commit("baseline");
    let base = repo.head();
    repo.write("notes.md", "before\nafter\n");
    repo.commit("tweak a top-level note, not the crate");

    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .args(["--base", &base])
        .arg(repo.path().join("crate"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "an empty crate-relative diff passes; stderr: {stderr}"
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
fn a_source_change_without_mutant_sites_reports_nothing_tested() {
    let repo = GitRepo::new("rust-no-sites");
    repo.write(
        "crate/Cargo.toml",
        "[package]\nname = \"tc_mut_no_sites\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n",
    );
    let lib = |answer: &str| {
        format!(
            "pub fn add(a: i32, b: i32) -> i32 {{\n    a + b\n}}\n\n#[cfg(test)]\nmod tests {{\n    use super::*;\n    #[test]\n    fn adds() {{\n        assert_eq!(add(2, 3), 5);\n        assert_eq!(add(10, 1), 11);\n    }}\n}}\n\npub const ANSWER: i32 = {answer};\n"
        )
    };
    repo.write("crate/src/lib.rs", &lib("41"));
    repo.commit("baseline: fully-tested add and a const");
    let base = repo.head();
    repo.write("crate/src/lib.rs", &lib("42"));
    repo.commit("correct the const, touch no function");

    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .args(["--base", &base])
        .arg(repo.path().join("crate"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "a change with no mutant sites passes; stdout: {stdout}; stderr: {stderr}"
    );
    assert!(
        stdout.contains(NOTHING_TESTED),
        "the zero-mutant run says the engine found nothing to test; got: {stdout}"
    );
    assert!(
        !stdout.contains("every mutation was caught"),
        "a run that judged no mutants never claims mutants were caught; got: {stdout}"
    );
}

#[test]
fn base_states_a_nonzero_count_for_a_caught_change_in_a_workspace_member_crate() {
    let repo = GitRepo::new("rust-member-caught");
    repo.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"member\"]\nresolver = \"2\"\n",
    );
    repo.write(
        "member/Cargo.toml",
        "[package]\nname = \"tc_mut_member_caught\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
    );
    repo.write(
        "member/src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn adds() {\n        assert_eq!(add(2, 3), 5);\n        assert_eq!(add(10, 1), 11);\n    }\n}\n",
    );
    repo.commit("baseline: fully-tested add in a workspace member");
    let base = repo.head();
    repo.write(
        "member/src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\npub fn total(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn adds() {\n        assert_eq!(add(2, 3), 5);\n        assert_eq!(add(10, 1), 11);\n    }\n    #[test]\n    fn totals() {\n        assert_eq!(total(2, 3), 5);\n        assert_eq!(total(10, 1), 11);\n    }\n}\n",
    );
    repo.commit("add a fully-tested total");

    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .args(["--base", &base])
        .arg(repo.path().join("member"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "every mutant on the changed lines is caught; stdout: {stdout}; stderr: {stderr}"
    );
    assert!(
        tested_count(&stdout) > 0,
        "the engine tested the member's mutants, so the count is non-zero; got: {stdout}"
    );
}

#[test]
fn base_fails_on_a_survivor_in_a_workspace_member_crate() {
    let repo = GitRepo::new("rust-workspace-member");
    repo.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"member\"]\nresolver = \"2\"\n",
    );
    repo.write(
        "member/Cargo.toml",
        "[package]\nname = \"tc_mut_ws_member\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
    );
    repo.write(
        "member/src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn adds() {\n        assert_eq!(add(2, 3), 5);\n        assert_eq!(add(10, 1), 11);\n    }\n}\n",
    );
    repo.commit("baseline: fully-tested add in a workspace member");
    let base = repo.head();
    repo.write(
        "member/src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\npub fn is_positive(n: i32) -> bool {\n    n > 0\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn adds() {\n        assert_eq!(add(2, 3), 5);\n        assert_eq!(add(10, 1), 11);\n    }\n    #[test]\n    fn runs_is_positive() {\n        let _ = is_positive(1);\n    }\n}\n",
    );
    repo.commit("add an assertion-light is_positive");

    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .args(["--base", &base])
        .arg(repo.path().join("member"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "the added weak function's survivors fail the gate; stdout: {stdout}; stderr: {stderr}"
    );
    assert!(
        stderr.contains("surviving mutant") && stderr.contains("src/lib.rs"),
        "the failure names the survivor by its scan-path-relative file; stderr: {stderr}"
    );
}

#[test]
fn survivors_fail_the_gate_by_default() {
    assert_eq!(unit_mutation_exit("survivors", None), 1);
}

#[test]
fn a_failing_run_lists_each_survivor_with_one_location() {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .arg(fixtures().join("rust").join("survivors"))
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "the survivors fail the gate; stderr: {stderr}"
    );
    assert!(
        stderr.contains("src/lib.rs:"),
        "each survivor line names its location; stderr: {stderr}"
    );
    assert!(
        !stderr.contains(": src/lib.rs:"),
        "a survivor line carries one location; stderr: {stderr}"
    );
}

#[test]
fn an_exempted_survivor_passes_the_gate() {
    assert_eq!(
        unit_mutation_exit("survivors", Some("mutation_exempt.toml")),
        0
    );
}
