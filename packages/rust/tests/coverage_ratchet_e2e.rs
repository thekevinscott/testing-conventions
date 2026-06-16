//! E2E tests for the coverage non-regression ratchet (Python — #131): drive the
//! built CLI binary end-to-end (no mocks) against the ratchet fixtures and assert
//! the exit code.
//!
//! Opens at RED per AGENTS.md: the baseline is ignored today, so the regressed
//! fixture (~86%, baseline 100%) still exits 0 — the implementation follows once
//! CI witnesses these red. Requires `coverage` + `pytest` on PATH.

use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage")
}

/// Exit code of `testing-conventions unit coverage --language python --config
/// floor85.toml <codebase>`.
fn ratchet_exit(codebase: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "coverage", "--language", "python", "--config"])
        .arg(fixtures().join("floor85.toml"))
        .arg(fixtures().join("python").join(codebase))
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn regression_below_the_baseline_exits_nonzero() {
    // ~86% clears the 85 floor but regressed below the committed 100% baseline.
    assert_eq!(ratchet_exit("ratchet_regressed"), 1);
}

#[test]
fn meeting_the_baseline_exits_zero() {
    // 100% meets both the 85 floor and the committed 100% baseline.
    assert_eq!(ratchet_exit("ratchet_clean"), 0);
}
