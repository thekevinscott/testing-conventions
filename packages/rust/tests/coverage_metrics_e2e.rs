//! E2E tests for the `functions` and `branch` floors on `unit coverage
//! --language rust` (#267): drive the built CLI binary end-to-end (no mocks)
//! against the fixture crates and assert the exit code and message.
//!
//! `[rust].coverage` takes two more opt-in floors alongside `regions`:
//! **`functions`** gates the export's functions total (stable toolchain), and
//! **`branch`** gates the branches total — the run adds `--branch`, which needs
//! a nightly toolchain (the `branchy` fixture pins one via its own
//! `rust-toolchain.toml`; on a stable toolchain the run fails with the nightly
//! requirement named). Both floors are the verdict of a measured run, so a
//! shortfall is a threshold message, never a config or invocation error.
//!
//! Red until the floors land: today both keys are rejected by the config
//! self-guard, so every one of these exits non-zero with an "unknown field"
//! error rather than the floor behavior asserted here. Requires
//! `cargo-llvm-cov`; the branch tests fetch the fixture's pinned nightly via
//! rustup on first run, and the stable-toolchain test assumes the repo's own
//! toolchain is stable (as in CI).

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage")
}

/// Run `unit coverage --language rust --config <cfg> rust/<crate>` and return the
/// captured output (exit code + stderr).
fn run(crate_name: &str, config: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", "rust", "--config"])
        .arg(fixtures().join(config))
        .arg(fixtures().join("rust").join(crate_name))
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

// ---- functions (stable) ----------------------------------------------------

#[test]
fn an_uncalled_function_fails_a_functions_floor() {
    // `funcs`'s `triple` is never called: functions coverage is 2/3 while lines
    // clear the low line floor, so the functions floor is the failing metric —
    // a threshold shortfall, never a config error.
    let out = run("funcs", "rust_functions_full.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("coverage below thresholds"),
        "expected a floor failure, got: {}",
        stderr(&out)
    );
}

#[test]
fn the_same_functions_coverage_clears_a_lower_floor() {
    // 2/3 functions covered clears a 60 floor — the floor is a real,
    // configurable knob.
    let out = run("funcs", "rust_functions_mid.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

// ---- branch (nightly via the fixture's rust-toolchain.toml) ----------------

#[test]
fn the_branch_floor_gates_the_measured_branches() {
    // `branchy`'s inline test takes one of the branch's two outcomes: branch
    // coverage is 50%, so a 100 floor fails on the measured number while a 50
    // floor clears — the floor is a real, configurable knob. One test drives
    // both runs sequentially: the fixture's pinned nightly is auto-installed by
    // rustup on first use, and two tests hitting that first install
    // concurrently race and corrupt each other's downloads.
    let out = run("branchy", "rust_branch_full.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("coverage below thresholds"),
        "expected a floor failure, got: {}",
        stderr(&out)
    );

    let out = run("branchy", "rust_branch_mid.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn a_branch_floor_on_a_stable_toolchain_names_the_nightly_requirement() {
    // `below` carries no toolchain pin, so the run uses the repo's stable
    // toolchain, where `--branch` cannot instrument — the run errors and the
    // message names the nightly requirement instead of reporting a floor pass.
    let out = run("below", "rust_branch_full.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("nightly"),
        "expected the nightly requirement to be named, got: {}",
        stderr(&out)
    );
}
