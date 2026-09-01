mod common;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use common::{expect_tested, ts_adapter};
use testing_conventions::mutation::{measure_typescript, Measurement};

/// The default package: `package.json` at the repo root, sources under `src/`, `src/index.ts`
/// reading the manifest one level above the scan path — the standard layout the gate is
/// pointed at as `<repo>/src`.
const PACKAGE_JSON: &str = "{ \"name\": \"tc-base\", \"private\": true, \"version\": \"1.2.3\" }\n";

/// A baseline whose `add` and `VERSION` are fully pinned by their tests — no survivors.
const BASELINE: &str = "import pkg from '../package.json';\n\nexport const VERSION: string = pkg.version;\n\nexport function add(a: number, b: number): number {\n  return a + b;\n}\n";

/// The change under test: a new `isPositive` whose test runs it but asserts nothing, so
/// every mutant on the added lines survives. `add` and `VERSION` are untouched.
const WITH_SURVIVOR: &str = "import pkg from '../package.json';\n\nexport const VERSION: string = pkg.version;\n\nexport function add(a: number, b: number): number {\n  return a + b;\n}\n\nexport function isPositive(n: number): boolean {\n  return n > 0;\n}\n";

const BASELINE_TEST: &str = "import { it, expect } from 'vitest';\nimport { add, VERSION } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n  expect(add(-1, 1)).toBe(0);\n});\nit('pins the manifest version', () => {\n  expect(VERSION).toBe('1.2.3');\n});\n";

const WITH_SURVIVOR_TEST: &str = "import { it, expect } from 'vitest';\nimport { add, isPositive, VERSION } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n  expect(add(-1, 1)).toBe(0);\n});\nit('pins the manifest version', () => {\n  expect(VERSION).toBe('1.2.3');\n});\nit('runs isPositive but asserts nothing', () => {\n  expect(typeof isPositive(1)).toBe('boolean');\n});\n";

/// The loose special case: flat scripts at the repo root with a `stryker.conf.json`, no manifest.
const LOOSE_BASELINE: &str =
    "export function add(a: number, b: number): number {\n  return a + b;\n}\n";

const LOOSE_WITH_SURVIVOR: &str = "export function add(a: number, b: number): number {\n  return a + b;\n}\n\nexport function isPositive(n: number): boolean {\n  return n > 0;\n}\n";

const LOOSE_BASELINE_TEST: &str = "import { it, expect } from 'vitest';\nimport { add } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n  expect(add(-1, 1)).toBe(0);\n});\n";

const LOOSE_WITH_SURVIVOR_TEST: &str = "import { it, expect } from 'vitest';\nimport { add, isPositive } from './index';\nit('pins add', () => {\n  expect(add(2, 3)).toBe(5);\n  expect(add(-1, 1)).toBe(0);\n});\nit('runs isPositive but asserts nothing', () => {\n  isPositive(1);\n});\n";

const STRYKER_CONF: &str =
    "{ \"testRunner\": \"vitest\", \"reporters\": [\"json\"], \"mutate\": [\"index.ts\"] }\n";

fn toolchain_node_modules() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_mutation/typescript/node_modules")
}

struct TempRepo(PathBuf);

impl TempRepo {
    /// The default repo: the upward-import package (`package.json` at the repo root, sources
    /// under `src/`, no Stryker config — the diff-scoped run supplies its own mutate ranges).
    /// The scan path handed to the rule is `<repo>/src`.
    fn new(slug: &str) -> Self {
        let repo = Self::init(slug);
        repo.write("package.json", PACKAGE_JSON);
        repo
    }

    /// The loose special case: flat scripts at the repo root with a `stryker.conf.json`, no
    /// manifest. The scan path is the repo root.
    fn loose(slug: &str) -> Self {
        let repo = Self::init(slug);
        repo.write("stryker.conf.json", STRYKER_CONF);
        repo
    }

    fn init(slug: &str) -> Self {
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
        std::os::unix::fs::symlink(toolchain_node_modules(), root.join("node_modules")).unwrap();
        TempRepo(root)
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
    let repo = TempRepo::new("survivor");
    repo.write("src/index.ts", BASELINE);
    repo.write("src/index.test.ts", BASELINE_TEST);
    repo.commit("baseline: fully-tested add reading ../package.json");
    let base = repo.head();
    repo.write("src/index.ts", WITH_SURVIVOR);
    repo.write("src/index.test.ts", WITH_SURVIVOR_TEST);
    repo.commit("add an assertion-light isPositive");

    let (count, survivors) = expect_tested(
        measure_typescript(
            &repo.0.join("src"),
            &[],
            &std::collections::BTreeMap::new(),
            Some(&base),
            &ts_adapter(),
        )
        .expect("stryker runs"),
    );
    assert!(
        count >= survivors.len(),
        "every survivor was judged, so the count covers them"
    );
    assert!(
        !survivors.is_empty(),
        "the added weak function should leave a survivor on the changed lines"
    );
    assert!(
        survivors
            .iter()
            .all(|s| s.file == "index.ts" && s.line >= 8),
        "survivors are scan-path-relative and only on the added lines; got {survivors:?}"
    );
}

#[test]
fn a_loose_tree_base_scopes_the_run_to_the_changed_lines() {
    let repo = TempRepo::loose("survivor");
    repo.write("index.ts", LOOSE_BASELINE);
    repo.write("index.test.ts", LOOSE_BASELINE_TEST);
    repo.commit("baseline: fully-tested add");
    let base = repo.head();
    repo.write("index.ts", LOOSE_WITH_SURVIVOR);
    repo.write("index.test.ts", LOOSE_WITH_SURVIVOR_TEST);
    repo.commit("add an assertion-light isPositive");

    let (count, survivors) = expect_tested(
        measure_typescript(
            &repo.0,
            &[],
            &std::collections::BTreeMap::new(),
            Some(&base),
            &ts_adapter(),
        )
        .expect("stryker runs"),
    );
    assert!(
        count >= survivors.len(),
        "every survivor was judged, so the count covers them"
    );
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
fn base_with_no_mutatable_changed_files_reports_the_engine_not_run() {
    let repo = TempRepo::new("notests");
    repo.write("src/index.ts", BASELINE);
    repo.write("src/index.test.ts", BASELINE_TEST);
    repo.commit("baseline");
    let base = repo.head();
    repo.write(
        "src/index.test.ts",
        &format!("{BASELINE_TEST}// touch the test file only\n"),
    );
    repo.commit("tweak only the test file");

    let measurement = measure_typescript(
        &repo.0.join("src"),
        &[],
        &std::collections::BTreeMap::new(),
        Some(&base),
        &ts_adapter(),
    )
    .expect("no run needed");
    assert_eq!(
        measurement,
        Measurement::EngineNotRun,
        "a test-file-only diff has nothing mutatable, so the engine never ran"
    );
}
