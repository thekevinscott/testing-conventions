//! E2E tests for cargo-feature passthrough in `unit coverage --language rust`
//! (#266): drive the built CLI binary end-to-end (no mocks) against the
//! feature-gated fixture crates and assert the exit code and message.
//!
//! A `[rust] features` list names the cargo features the coverage run enables
//! (`cargo llvm-cov --features …`), so `#[cfg(feature = …)]` code is compiled
//! and measured: covered gated code clears the floor, and untested gated code
//! fails it — the floor gates the full configured source tree, with nothing
//! compiled out of the denominator.
//!
//! Red until feature passthrough lands: today the `features` key is rejected by
//! the config self-guard, so every one of these exits non-zero with an "unknown
//! field" error rather than the feature-aware behavior asserted here. Requires
//! `cargo-llvm-cov`.

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

#[test]
fn a_covered_feature_gated_module_clears_the_floor() {
    // `gated`'s `boost` module is fully covered by its inline test; with the
    // `boost` feature enabled from config, the whole crate is measured and the
    // 100 floor passes.
    let out = run("gated", "rust_features_full.toml");
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
}

#[test]
fn an_untested_feature_gated_module_fails_the_floor() {
    // `gated_untested`'s `boost` module carries no tests; with the feature
    // enabled from config, its uncovered regions and lines are measured and the
    // 100 floor fails — the floor's verdict, so the failure is a threshold
    // shortfall, never a config or invocation error.
    let out = run("gated_untested", "rust_features_full.toml");
    assert_eq!(code(&out), 1, "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("coverage below thresholds"),
        "expected a floor failure, got: {}",
        stderr(&out)
    );
}
