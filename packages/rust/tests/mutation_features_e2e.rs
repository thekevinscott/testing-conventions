mod common;

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation")
}

#[test]
fn a_feature_gated_module_with_killing_tests_passes_the_gate() {
    let status = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust", "--config"])
        .arg(fixtures().join("rust_features.toml"))
        .arg(fixtures().join("rust").join("gated_killed"))
        .status()
        .expect("the built binary should run");
    assert_eq!(status.code(), Some(0));
}

#[test]
fn a_feature_gated_integration_test_target_builds_but_the_gate_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust", "--config"])
        .arg(fixtures().join("rust_features.toml"))
        .arg(fixtures().join("rust").join("gated_ws").join("member"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "the integration tier's kills don't count toward the gate; stdout: {stdout}; stderr: {stderr}"
    );
    assert!(
        stderr.contains("unexplained surviving mutant"),
        "the feature reached the build phase, so the engine judged mutants; stderr: {stderr}"
    );
    assert!(
        stderr.contains("src/boost.rs"),
        "the survivor names the feature-gated module; stderr: {stderr}"
    );
}

#[test]
fn a_baseline_that_cannot_build_fails_loudly() {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .arg(fixtures().join("rust").join("gated_ws").join("member"))
        .output()
        .expect("the built binary should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a baseline that cannot build fails the run; stdout: {stdout}"
    );
    assert!(
        stderr.contains("did not run cleanly"),
        "the failure names the baseline build; stderr: {stderr}"
    );
    assert!(
        !stdout.contains("every mutation was caught"),
        "a run that judged nothing never claims mutants were caught; got: {stdout}"
    );
}
