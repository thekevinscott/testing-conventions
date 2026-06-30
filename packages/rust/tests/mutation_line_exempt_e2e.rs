//! E2E tests for line-scoped mutation exemptions (#226): drive the built CLI binary
//! end-to-end (no mocks) against the `survivors` / `killed` fixtures and assert the
//! exit code and message.
//!
//! A `[[<lang>.exempt]]` entry with a `lines` list lifts only the surviving mutants on
//! those lines — not every survivor in the file — with a determinism guard: a listed
//! line whose mutants were all caught (no survivor) is a hard error, and a survivor on
//! an *unlisted* line still fails the gate. The fixtures are the standard pair:
//! `survivors` (assertion-light, every mutant survives) and `killed` (every mutant
//! caught).
//!
//! Red until line-scoped exemptions land: today the `lines` key is rejected by the
//! config self-guard, so each of these exits with an "unknown field" error rather than
//! the line-scoped behavior asserted here. Requires `cargo-mutants` (Rust), the built node
//! adapter + the fixtures' vitest (TypeScript), and cosmic-ray + pytest (Python).

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{ts_adapter, Staged};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

/// Run `unit mutation --language <lang> --config <cfg> <project>` and capture output. The
/// bundled TS adapter path is injected exactly as the npm launcher would (harmless for the
/// Rust / Python arms, which don't read it).
fn run(language: &str, project: &Path, config: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .env("TESTING_CONVENTIONS_TS_MUTATION_ADAPTER", ts_adapter())
        .args(["unit", "mutation", "--language", language, "--config"])
        .arg(fixtures().join(config))
        .arg(project)
        .output()
        .expect("the built binary should run")
}

fn code(output: &Output) -> i32 {
    output
        .status
        .code()
        .expect("the process should exit with a code")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

// ---- Rust -----------------------------------------------------------------

#[test]
fn rust_exempting_the_survivor_line_passes() {
    // Line 7 (`n > 0`) is where every mutant survives; lifting just that line clears
    // the gate.
    let out = run(
        "rust",
        &fixtures().join("rust").join("survivors"),
        "lines_mut_rust_ok.toml",
    );
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn rust_over_exempting_a_caught_line_is_a_hard_error() {
    // In the killed crate line 6's mutants are all caught, so listing it is rejected.
    let out = run(
        "rust",
        &fixtures().join("rust").join("killed"),
        "lines_mut_rust_over.toml",
    );
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("all caught"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}

// ---- TypeScript -----------------------------------------------------------

#[test]
fn typescript_exempting_both_survivor_lines_passes() {
    // Lines 2 and 6 carry the survivors; lifting both clears the gate.
    let project = Staged::new("survivors");
    let out = run("typescript", project.path(), "lines_mut_ts_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn typescript_under_listing_still_fails() {
    // Exempting only line 6 leaves line 2's survivor unexplained, so the gate fails.
    let project = Staged::new("survivors");
    let out = run("typescript", project.path(), "lines_mut_ts_under.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("unexplained"),
        "expected the unlisted survivor to fail the gate, got: {}",
        stderr(&out)
    );
}

#[test]
fn typescript_over_exempting_a_caught_line_is_a_hard_error() {
    // In the killed project line 2's mutants are all caught, so listing it is rejected.
    let project = Staged::new("killed");
    let out = run("typescript", project.path(), "lines_mut_ts_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("all caught"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}

// ---- Python ---------------------------------------------------------------

#[test]
fn python_exempting_both_survivor_lines_passes() {
    // Lines 2 and 6 carry the survivors; lifting both clears the gate.
    let project = Staged::python("survivors");
    let out = run("python", project.path(), "lines_mut_py_ok.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn python_over_exempting_a_caught_line_is_a_hard_error() {
    // In the killed project line 2's mutants are all caught, so listing it is rejected.
    let project = Staged::python("killed");
    let out = run("python", project.path(), "lines_mut_py_over.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("all caught"),
        "expected an over-exemption guard error, got: {}",
        stderr(&out)
    );
}
