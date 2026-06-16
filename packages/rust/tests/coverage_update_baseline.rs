//! Integration tests for `unit coverage --update-baseline` — the Python coverage
//! baseline writer (#143, parent #46). It is the *record* half of the
//! non-regression ratchet (#131 shipped the *gate*): measure the suite and
//! write/ratchet the committed `coverage-baseline.json` beside the tree.
//!
//! Ratchets **up only** — recording never lowers an existing baseline (lowering
//! stays a deliberate hand-edit). These drive the CLI through `run()` against a
//! temp COPY of a fixture (so the committed fixtures stay pristine and the
//! baseline is writable) and read the result back with `coverage::read_baseline`.
//!
//! Opens at RED per AGENTS.md: `--update-baseline` doesn't exist yet, so the flag
//! is rejected and no baseline is written. Requires `coverage` + `pytest` on PATH.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::coverage::read_baseline;
use testing_conventions::run;

/// A temp copy of a fixture codebase (its `.py` files only), removed on drop —
/// so recording a baseline never mutates the committed fixtures.
struct Workspace(PathBuf);

impl Workspace {
    fn from_fixture(fixture: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "tc-update-baseline-{}-{}",
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

    fn seed_baseline(&self, percent: f64) {
        std::fs::write(
            self.0.join("coverage-baseline.json"),
            format!("{{\"python\":{{\"percent_covered\":{percent}}}}}\n"),
        )
        .unwrap();
    }

    fn recorded_percent(&self) -> Option<f64> {
        read_baseline(&self.0)
            .unwrap()
            .and_then(|baseline| baseline.python)
            .map(|python| python.percent_covered)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Run `unit coverage --language python --update-baseline <dir>` (zero-config —
/// recording doesn't enforce the floor) and return the exit code.
fn update_baseline(dir: &Path) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "coverage".into(),
        "--language".into(),
        "python".into(),
        "--update-baseline".into(),
        dir.as_os_str().to_owned(),
    ];
    run(argv).expect("`unit coverage --update-baseline` should run to an exit code")
}

#[test]
fn records_the_measured_total_when_no_baseline_exists() {
    // `full` is 100%; with no committed baseline, recording writes 100.0.
    let ws = Workspace::from_fixture("full");
    assert_eq!(update_baseline(&ws.0), 0);
    assert_eq!(ws.recorded_percent(), Some(100.0));
}

#[test]
fn raises_a_lower_existing_baseline() {
    // A stale-low baseline (50%) is bumped up to the measured 100%.
    let ws = Workspace::from_fixture("full");
    ws.seed_baseline(50.0);
    assert_eq!(update_baseline(&ws.0), 0);
    assert_eq!(ws.recorded_percent(), Some(100.0));
}

#[test]
fn never_lowers_an_existing_baseline() {
    // `above_85` is ~85.71%; recording against a higher committed baseline (100%)
    // must not lower it — the ratchet only moves up.
    let ws = Workspace::from_fixture("above_85");
    ws.seed_baseline(100.0);
    assert_eq!(update_baseline(&ws.0), 0);
    assert_eq!(ws.recorded_percent(), Some(100.0));
}
