//! E2E tests for the unknown-config-key upgrade path: drive the built CLI binary
//! (no mocks) with a config carrying a key the schema doesn't know, and assert the
//! run fails naming the key and pointing at `MIGRATIONS.md` — the record of every
//! key a release renamed or removed, alongside its replacement.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// An empty scan directory: the config self-guard fires on load, before any scan,
/// so the run's outcome is the config error regardless of what the directory holds.
fn empty_scan_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("tc-config-unknown-key-e2e-scan");
    std::fs::create_dir_all(&dir).expect("the scan dir should be creatable");
    dir
}

fn run_with_config(config: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "colocated-test", "--language", "python", "--config"])
        .arg(fixture(config))
        .arg(empty_scan_dir())
        .output()
        .expect("the built binary should run")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn an_unknown_config_key_fails_the_run_naming_the_key() {
    let out = run_with_config("unknown_key.toml");
    assert_ne!(
        out.status.code(),
        Some(0),
        "an unknown key must fail the run"
    );
    assert!(
        stderr(&out).contains("unknown field `unknown`"),
        "the failure must name the rejected key, got: {}",
        stderr(&out)
    );
}

#[test]
fn an_unknown_config_key_error_points_at_migrations() {
    let out = run_with_config("unknown_key.toml");
    assert!(
        stderr(&out).contains("MIGRATIONS.md"),
        "the failure must point at MIGRATIONS.md — the record of renamed/removed keys, got: {}",
        stderr(&out)
    );
}
