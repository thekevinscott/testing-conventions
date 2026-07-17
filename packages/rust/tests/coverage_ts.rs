//! Integration tests for the TypeScript coverage rule.
//!
//! These run REAL vitest over the fixture codebases via the SDK
//! (`coverage::measure_typescript`) and assert pass/fail. Per the guardrail
//! the *codebases themselves* are the fixtures, in the prescribed consumer package
//! layout `{package.json, src/**}` scanned at `src/`: `full` (100% on all four
//! metrics) clears a 100 floor, `above` (~83% lines / 87% branches) fails 100 but
//! clears a mid floor, `below` (100% lines but only ~66% branches) fails the mid
//! floor on branches — the branch floor catching what line coverage misses. The
//! flat, no-manifest shape is the named special case (`exempt_cov`,
//! `full_with_config`). Requires Node with the fixtures' vitest toolchain installed
//! (see the suite's `package.json`).

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
        measure_typescript(&codebase("full").join("src"), FULL, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn above_fails_a_100_floor() {
    assert!(matches!(
        measure_typescript(&codebase("above").join("src"), FULL, &[]).unwrap(),
        Outcome::Fail(_)
    ));
}

#[test]
fn above_passes_the_mid_floor() {
    assert_eq!(
        measure_typescript(&codebase("above").join("src"), MID, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn below_fails_the_mid_floor_on_branches() {
    // `below` has 100% lines but only ~66% branches; the mid floor's branch
    // threshold (75) is what fails it — the whole point of measuring branches.
    let outcome = measure_typescript(&codebase("below").join("src"), MID, &[]).unwrap();
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
    // exemption this codebase fails the floor.
    assert_eq!(
        measure_typescript(&codebase("exempt_cov"), FULL, &["shim.ts".to_string()]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_missing_toolchain_fails_clean_without_downloading() {
    // No `node_modules`: the coverage arm must surface a clear error via `npx
    // --no-install` and never silently fetch vitest. Parity with the cosmic-ray /
    // cargo-llvm-cov arms, which invoke their binary directly and fail clean when absent.
    let dir = std::env::temp_dir().join(format!("tc-ts-cov-notoolchain-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let result = measure_typescript(&dir, MID, &[]);
    let _ = std::fs::remove_dir_all(&dir);
    let err = result.expect_err("a project with no vitest installed must error, not download one");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("npx --no-install"),
        "the error should name the no-download invocation; got: {msg}"
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

#[test]
fn a_package_root_vitest_config_governs_a_src_scan() {
    // The standard package layout — `{package.json, vitest.config.ts, src/**, tests/**}`,
    // scanned at `src/` — where the package-root config is load-bearing: its setup file is
    // the only thing that covers `src/boot.ts`, and the `tests/` tier fails loudly if the
    // run ever collects it. Pins the anchoring answer the docs state: vitest resolves the
    // package-root config with its own upward search from the scan path (as pytest does),
    // config-file-relative paths (the setup file) resolve beside the config, and discovery
    // and measurement stay scoped to the scan path.
    assert_eq!(
        measure_typescript(&codebase("pkg_config").join("src"), FULL, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn consumer_coverage_thresholds_neither_decide_nor_rewrite() {
    // A consumer config's own `coverage.thresholds` must not decide the gate's outcome,
    // and `autoUpdate` must never rewrite the consumer's file during a gate run. The
    // staged package pins both at once: its config demands `lines: 99` with `autoUpdate`
    // while its sources sit at ~66% — above the gate's configured floor, below the
    // consumer's own. The gate's floor is the only floor (Pass), and the config file is
    // left byte-identical. Staged into a temp copy so the vitest run never touches a
    // committed fixture.
    let staged = std::env::temp_dir().join(format!("tc-ts-cov-thresholds-{}", std::process::id()));
    let src = staged.join("src");
    std::fs::create_dir_all(&src).unwrap();
    for file in ["package.json", "vitest.setup.ts"] {
        std::fs::copy(codebase("pkg_config").join(file), staged.join(file)).unwrap();
    }
    for file in ["boot.ts", "widget.ts", "widget.test.ts"] {
        std::fs::copy(
            codebase("pkg_config").join("src").join(file),
            src.join(file),
        )
        .unwrap();
    }
    std::fs::write(
        staged.join("vitest.config.ts"),
        "import { defineConfig } from 'vitest/config';\n\nexport default defineConfig({\n  test: {\n    setupFiles: ['./vitest.setup.ts'],\n    coverage: {\n      thresholds: { lines: 99, autoUpdate: true },\n    },\n  },\n});\n",
    )
    .unwrap();
    // Uncovered source drags the measurement below the consumer's 99 (and keeps it above
    // the gate's floor below).
    std::fs::write(
        src.join("extra.ts"),
        "export function unused(n: number): string {\n  if (n > 0) return 'positive';\n  return 'other';\n}\n",
    )
    .unwrap();
    std::os::unix::fs::symlink(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/unit_coverage/typescript/node_modules"),
        staged.join("node_modules"),
    )
    .unwrap();

    let floor = TypeScriptThresholds {
        lines: 50,
        branches: 50,
        functions: 50,
        statements: 50,
    };
    let config_before = std::fs::read(staged.join("vitest.config.ts")).unwrap();
    let outcome = measure_typescript(&src, floor, &[]);
    let config_after = std::fs::read(staged.join("vitest.config.ts")).unwrap();
    let _ = std::fs::remove_file(staged.join("node_modules"));
    let _ = std::fs::remove_dir_all(&staged);
    assert_eq!(
        outcome
            .expect("the gate's own floor decides; the consumer threshold must not error the run"),
        Outcome::Pass,
        "above the gate's floor, below the consumer's own threshold"
    );
    assert_eq!(
        config_before, config_after,
        "the consumer's vitest.config.ts must be left byte-identical"
    );
}

#[test]
fn a_package_root_config_file_is_not_counted_as_uncovered_source() {
    // `full_with_config/` is fully tested (identical to `full/`) but also
    // carries its own `vitest.config.ts` — the shape a per-package monorepo
    // `uses:` call produces (`path` names the whole package root, not just
    // `src/`). vitest's own default excludes already keep config files out of
    // the coverage denominator; the rule must not clobber those defaults with
    // its own `--coverage.exclude` flags.
    assert_eq!(
        measure_typescript(&codebase("full_with_config"), FULL, &[]).unwrap(),
        Outcome::Pass
    );
}
