//! Integration test for diff-scoped TypeScript mutation — `unit mutation --language
//! typescript --base` (#202).
//!
//! With `--base`, only mutants on the `<base>...HEAD` changed lines are tested. Stryker
//! has no native git-diff scoping, so the changed lines become `--mutate
//! <file>:<line>-<line>` ranges (line granularity, matching cargo-mutants' `--in-diff`
//! in the Rust arm). Builds a throwaway TypeScript project in a git repo (the codebase
//! is the fixture, per the #3 guardrail): a fully-tested baseline, then a commit that
//! adds an assertion-light function. The diff scopes the run to the added lines, whose
//! mutants survive — while the unchanged, well-tested `add` isn't mutated at all.
//!
//! The project's `node_modules` is symlinked to the fixtures' runner-only toolchain so the
//! out-of-tree repo resolves vitest without a second install; Stryker is bundled with and
//! driven by the Node adapter (#246), pointed at via [`common::ensure_ts_adapter_env`].
//! Requires `git`, the built node adapter, and that toolchain (`npm ci` in
//! `tests/fixtures/unit_mutation/typescript`).

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use common::ensure_ts_adapter_env;
use testing_conventions::mutation::measure_typescript;

/// A baseline whose `add` is fully pinned by its test — no survivors.
const BASELINE: &str = "export function add(a: number, b: number): number {\n  return a + b;\n}\n";

/// The change under test: a new `isPositive` whose test runs it but asserts nothing,
/// so every mutant on the added lines survives. `add` is untouched.
const WITH_SURVIVOR: &str = "export function add(a: number, b: number): number {\n  return a + b;\n}\n\nexport function isPositive(n: number): boolean {\n  return n > 0;\n}\n";

const BASELINE_TEST: &str = "import { it, expect } from 'vitest';\nimport { add } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n  expect(add(-1, 1)).toBe(0);\n});\n";

const WITH_SURVIVOR_TEST: &str = "import { it, expect } from 'vitest';\nimport { add, isPositive } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n  expect(add(-1, 1)).toBe(0);\n});\nit('runs isPositive but asserts nothing', () => {\n  isPositive(1);\n});\n";

const STRYKER_CONF: &str =
    "{ \"testRunner\": \"vitest\", \"reporters\": [\"json\"], \"mutate\": [\"index.ts\"] }\n";

fn toolchain_node_modules() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_mutation/typescript/node_modules")
}

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-mut-base-ts-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        // Resolve vitest from the fixtures' runner-only install rather than a second one.
        std::os::unix::fs::symlink(toolchain_node_modules(), root.join("node_modules")).unwrap();
        let repo = TempRepo(root);
        repo.write("stryker.conf.json", STRYKER_CONF);
        repo
    }

    fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn commit(&self, message: &str) {
        git(&self.0, &["add", "-A"]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
    }

    fn head(&self) -> String {
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.0)
            .output()
            .expect("git rev-parse should run");
        assert!(out.status.success(), "git rev-parse failed");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        // Remove the symlink first so we never recurse into the shared toolchain.
        let _ = std::fs::remove_file(self.0.join("node_modules"));
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git should run");
    assert!(status.success(), "git {args:?} failed");
}

#[test]
fn base_scopes_the_run_to_the_changed_lines() {
    ensure_ts_adapter_env();
    let repo = TempRepo::new("survivor");
    repo.write("index.ts", BASELINE);
    repo.write("index.test.ts", BASELINE_TEST);
    repo.commit("baseline: fully-tested add");
    let base = repo.head();
    repo.write("index.ts", WITH_SURVIVOR);
    repo.write("index.test.ts", WITH_SURVIVOR_TEST);
    repo.commit("add an assertion-light isPositive");

    let survivors = measure_typescript(
        &repo.0,
        &[],
        &std::collections::BTreeMap::new(),
        Some(&base),
    )
    .expect("stryker runs");
    // The added `isPositive` (lines 5-7) is in the diff and assertion-light, so its
    // mutants survive; `add` (lines 1-3) is unchanged, so it's out of scope and never
    // mutated.
    assert!(
        !survivors.is_empty(),
        "the added weak function should leave a survivor on the changed lines"
    );
    assert!(
        survivors
            .iter()
            .all(|s| s.file == "index.ts" && s.line >= 4),
        "only the added lines should be mutated, not the well-tested `add`; got {survivors:?}"
    );
}

#[test]
fn base_with_no_mutatable_changed_files_skips_the_run() {
    // The only change on the diff is to a test file, which is never mutated — so the
    // diff scopes to nothing and the run is skipped entirely (no survivors, no Stryker).
    ensure_ts_adapter_env();
    let repo = TempRepo::new("notests");
    repo.write("index.ts", BASELINE);
    repo.write("index.test.ts", BASELINE_TEST);
    repo.commit("baseline");
    let base = repo.head();
    repo.write(
        "index.test.ts",
        &format!("{BASELINE_TEST}// touch the test file only\n"),
    );
    repo.commit("tweak only the test file");

    let survivors = measure_typescript(
        &repo.0,
        &[],
        &std::collections::BTreeMap::new(),
        Some(&base),
    )
    .expect("no run needed");
    assert!(
        survivors.is_empty(),
        "a test-file-only diff has nothing mutatable; got {survivors:?}"
    );
}
