//! E2E tests for `unit coverage --update-baseline` (#143): drive the built CLI
//! binary against a temp copy of a fixture and assert it records the baseline.
//!
//! Opens at RED per AGENTS.md: `--update-baseline` doesn't exist yet, so the
//! built binary rejects the flag and writes nothing. Requires `coverage` +
//! `pytest` on PATH.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::coverage::read_baseline;

/// A temp copy of a fixture codebase (its `.py` files only), removed on drop.
struct Workspace(PathBuf);

impl Workspace {
    fn from_fixture(fixture: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "tc-update-baseline-e2e-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/unit_coverage/python")
            .join(fixture);
        for entry in std::fs::read_dir(&src).expect("fixture dir should exist") {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "py") {
                std::fs::copy(&path, dir.join(path.file_name().unwrap())).unwrap();
            }
        }
        Workspace(dir)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn records_the_measured_total() {
    // `full` is 100%; the built binary records a 100.0 baseline and exits zero.
    let ws = Workspace::from_fixture("full");
    let code = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args([
            "unit",
            "coverage",
            "--language",
            "python",
            "--update-baseline",
        ])
        .arg(&ws.0)
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code");
    assert_eq!(code, 0);
    assert_eq!(
        read_baseline(&ws.0)
            .unwrap()
            .and_then(|baseline| baseline.python)
            .map(|python| python.percent_covered),
        Some(100.0)
    );
}
