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
    let outcome = measure_typescript(&codebase("below").join("src"), MID, &[]).unwrap();
    assert!(
        matches!(&outcome, Outcome::Fail(message) if message.contains("branches")),
        "got: {outcome:?}"
    );
}

#[test]
fn a_coverage_exemption_omits_the_file_and_lets_the_floor_pass() {
    assert_eq!(
        measure_typescript(&codebase("exempt_cov"), FULL, &["shim.ts".to_string()]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn a_missing_toolchain_fails_clean_without_downloading() {
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
    let empty = std::env::temp_dir().join(format!("tc-ts-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let result = measure_typescript(&empty, MID, &[]);
    let _ = std::fs::remove_dir_all(&empty);
    assert!(result.is_err());
}

#[test]
fn a_package_root_vitest_config_governs_a_src_scan() {
    assert_eq!(
        measure_typescript(&codebase("pkg_config").join("src"), FULL, &[]).unwrap(),
        Outcome::Pass
    );
}

#[test]
fn consumer_coverage_thresholds_neither_decide_nor_rewrite() {
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
    assert_eq!(
        measure_typescript(&codebase("full_with_config"), FULL, &[]).unwrap(),
        Outcome::Pass
    );
}
