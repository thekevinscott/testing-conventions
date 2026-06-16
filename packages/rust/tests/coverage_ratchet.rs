//! Integration tests for the coverage non-regression ratchet (Python — #131,
//! parent #46).
//!
//! The unit-coverage floor can't regress: a committed `coverage-baseline.json`
//! records the last total per language, and a run that drops below it fails even
//! when it still clears the configured floor. These drive the `unit coverage`
//! CLI through `run()` (REAL coverage.py over the fixtures) and assert the exit
//! code. The baseline lives beside the measured tree, so each fixture carries its
//! own.
//!
//! Opens at RED per AGENTS.md: the baseline is ignored today, so the regressed
//! fixture (~86%, baseline 100%) still exits 0. The implementation — reading the
//! baseline and enforcing no-regression alongside the floor — follows once CI
//! witnesses these red. Requires `coverage` + `pytest` on PATH.

use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::run;

/// Absolute path to a fixture codebase under `tests/fixtures/unit_coverage/python/`.
fn codebase(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_coverage/python")
        .join(name)
}

/// Exit code of `unit coverage --language python --config floor85.toml <codebase>`.
/// The 85 floor lets the regressed fixture (~86%) clear the floor, so the only
/// thing that can fail it is the baseline ratchet.
fn ratchet_exit(codebase_name: &str) -> i32 {
    let config =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_coverage/floor85.toml");
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "coverage".into(),
        "--language".into(),
        "python".into(),
        "--config".into(),
        config.into_os_string(),
        codebase(codebase_name).into_os_string(),
    ];
    run(argv).expect("`unit coverage` should run to an exit code")
}

#[test]
fn regression_below_the_baseline_fails_even_when_the_floor_passes() {
    // `ratchet_regressed` is ~86%: it clears the 85 floor, but its committed
    // baseline records 100%, so the run regressed and must exit non-zero.
    assert_eq!(ratchet_exit("ratchet_regressed"), 1);
}

#[test]
fn meeting_the_baseline_passes() {
    // `ratchet_clean` is 100% and its baseline records 100% — no regression, and
    // the floor is met, so it exits zero.
    assert_eq!(ratchet_exit("ratchet_clean"), 0);
}
