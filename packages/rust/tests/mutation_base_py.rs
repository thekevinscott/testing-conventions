//! Integration test for diff-scoped Python mutation — `unit mutation --language python
//! --base` (#203).
//!
//! cosmic-ray has no native git-diff mode, so the run is scoped to the changed `.py` files
//! (passed to the adapter as `--module`) and the survivors are filtered to the
//! `<base>...HEAD` changed lines in the core (line granularity, matching cargo-mutants'
//! `--in-diff` and the Stryker `--mutate` ranges of the other arms). Builds a throwaway
//! Python project in a git repo (the codebase is the fixture, per the #3 guardrail): a
//! fully-tested baseline, then a commit that adds an assertion-light function. The diff
//! scopes the run to the added lines, whose mutants survive — while the unchanged,
//! well-tested `add` isn't reported. Requires `git` and a `python3` with cosmic-ray + pytest
//! installed and the source package importable (`PYTHONPATH=packages/python/python`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::mutation::measure_python;

/// A baseline whose `add` is fully pinned by its test — no survivors.
const BASELINE: &str = "def add(a, b):\n    return a + b\n";

/// The change under test: a new `is_positive` whose test runs it but asserts nothing,
/// so every mutant on the added lines survives. `add` is untouched.
const WITH_SURVIVOR: &str =
    "def add(a, b):\n    return a + b\n\n\ndef is_positive(n):\n    return n > 0\n";

const BASELINE_TEST: &str =
    "from calc import add\n\n\ndef test_add():\n    assert add(2, 3) == 5\n    assert add(-1, 1) == 0\n";

const WITH_SURVIVOR_TEST: &str = "from calc import add, is_positive\n\n\ndef test_add():\n    assert add(2, 3) == 5\n    assert add(-1, 1) == 0\n\n\ndef test_is_positive_runs():\n    is_positive(1)\n";

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-mut-base-py-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        TempRepo(root)
    }

    fn write(&self, rel: &str, contents: &str) {
        std::fs::write(self.0.join(rel), contents).unwrap();
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
    repo.write("calc.py", BASELINE);
    repo.write("calc_test.py", BASELINE_TEST);
    repo.commit("baseline: fully-tested add");
    let base = repo.head();
    repo.write("calc.py", WITH_SURVIVOR);
    repo.write("calc_test.py", WITH_SURVIVOR_TEST);
    repo.commit("add an assertion-light is_positive");

    let survivors = measure_python(
        &repo.0,
        &[],
        &std::collections::BTreeMap::new(),
        Some(&base),
    )
    .expect("cosmic-ray runs");
    // The added `is_positive` (lines 5-6) is in the diff and assertion-light, so its
    // mutants survive; `add` (lines 1-2) is unchanged, so it's filtered out.
    assert!(
        !survivors.is_empty(),
        "the added weak function should leave a survivor on the changed lines"
    );
    assert!(
        survivors.iter().all(|s| s.file == "calc.py" && s.line >= 3),
        "only the added lines should be reported, not the well-tested `add`; got {survivors:?}"
    );
}
