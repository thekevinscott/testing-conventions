//! Integration tests for the TypeScript coverage rule (#31).
//!
//! These run REAL vitest over the fixture codebases via the SDK
//! (`coverage::measure_typescript`) and assert pass/fail. Per the #3 guardrail
//! the *codebases themselves* are the fixtures: `full` (100% on all four metrics)
//! clears a 100 floor, `above` (~83% lines / 87% branches) fails 100 but clears a
//! mid floor, `below` (100% lines but only ~66% branches) fails the mid floor on
//! branches — the branch floor catching what line coverage misses. Requires Node
//! with the fixtures' vitest toolchain installed (see the suite's `package.json`).

use std::path::PathBuf;

use testing_conventions::coverage::{measure_typescript, Outcome, TypeScriptThresholds};

fn codebase(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_coverage/typescript")
        .join(name)
}

const FULL: TypeScriptThresholds = TypeScriptThresholds {
    lines: 100,
    branches: 100,
    functions: 100,
    statements: 100,
};
const MID: TypeScriptThresholds = TypeScriptThresholds {
    lines: 80,
    branches: 75,
    functions: 80,
    statements: 80,
};

#[test]
fn full_passes_a_100_floor() {
    assert_eq!(
        measure_typescript(&codebase("full"), FULL, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn above_fails_a_100_floor() {
    assert!(matches!(
        measure_typescript(&codebase("above"), FULL, &[]).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn above_passes_the_mid_floor() {
    assert_eq!(
        measure_typescript(&codebase("above"), MID, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn below_fails_the_mid_floor_on_branches() {
    // `below` has 100% lines but only ~66% branches; the mid floor's branch
    // threshold (75) is what fails it — the whole point of measuring branches.
    let outcome = measure_typescript(&codebase("below"), MID, &[]).unwrap();
    assert!(
        matches!(&outcome, Outcome::Fail(message) if message.contains("branches")),
        "got: {outcome:?}"
    );
}

#[test]
fn a_coverage_exemption_omits_the_file_and_lets_the_floor_pass() {
    // `exempt_cov` sits below 100 only because of shim.ts (its `launch` is never
    // exercised); omitting it — the `coverage`-rule exemption the CLI resolves
    // from config — leaves core.ts, fully covered, to clear 100. Without the
    // exemption this codebase fails the floor (#32).
    assert_eq!(
        measure_typescript(&codebase("exempt_cov"), FULL, &["shim.ts".to_string()]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_suite_that_cannot_run_is_an_error_not_a_silent_pass() {
    // An empty directory has no test files; vitest exits non-zero, so measuring it
    // must error rather than report a vacuous pass.
    let empty = std::env::temp_dir().join(format!("tc-ts-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let result = measure_typescript(&empty, MID, &[]);
    let _ = std::fs::remove_dir_all(&empty);
    assert!(result.is_err());
}
