//! Coverage rule (Python — issue #26).
//!
//! Enforces the README's Coverage rule: a library's unit suite must meet the
//! configured floor, measured with branch coverage, with test files excluded
//! from the denominator. This module is the deterministic core — given a
//! coverage.py JSON report ([`CoverageReport`]) and the [`Thresholds`] from
//! config, [`evaluate`] decides pass/fail. Producing the report (shelling out
//! to `coverage`) is a thin layer on top, kept separate so the guarantee is
//! testable without a Python toolchain.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::waiver::{self, Scope};

/// Always omitted from the coverage denominator: colocated unit tests are the
/// suite, never a subject of it.
const TEST_OMIT: &str = "*_test.py";

/// The coverage floor to enforce, from a `[<language>].coverage` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thresholds {
    /// Minimum total coverage percent the unit suite must meet.
    pub fail_under: u8,
    /// Whether branch coverage must be measured (and folded into the total).
    pub branch: bool,
}

/// A coverage.py JSON report (`coverage json`), pared to the totals the check
/// needs. Unmodeled fields (per-file data, metadata) are ignored.
#[derive(Debug, Clone, Deserialize)]
pub struct CoverageReport {
    pub totals: Totals,
}

/// The `totals` block of a coverage.py report.
#[derive(Debug, Clone, Deserialize)]
pub struct Totals {
    /// Total covered percent — line coverage, plus branch when measured.
    pub percent_covered: f64,
    /// Branches measured; `0` when branch coverage was not enabled.
    #[serde(default)]
    pub num_branches: u64,
}

/// The result of checking a report against the thresholds.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// The floor is met.
    Pass,
    /// The floor is not met; the message explains why (actual vs. required).
    Fail(String),
}

/// Parse a coverage.py JSON report (the output of `coverage json`).
pub fn parse_report(json: &str) -> Result<CoverageReport> {
    serde_json::from_str(json).context("parsing coverage.py JSON report")
}

/// Decide whether `report` meets `thresholds`.
///
/// Fails when total coverage is below `fail_under`, or when branch coverage was
/// required but the report measured no branches (a misconfigured run).
pub fn evaluate(report: &CoverageReport, thresholds: Thresholds) -> Outcome {
    if thresholds.branch && report.totals.num_branches == 0 {
        return Outcome::Fail(
            "branch coverage is required but the report measured no branches".to_string(),
        );
    }
    let actual = report.totals.percent_covered;
    let required = f64::from(thresholds.fail_under);
    // A hair of tolerance so a report that rounds to the floor (e.g. 99.999…%
    // for a 100% target) isn't failed by float noise.
    if actual + 1e-9 >= required {
        Outcome::Pass
    } else {
        Outcome::Fail(format!(
            "coverage {actual:.2}% is below the required {}%",
            thresholds.fail_under
        ))
    }
}

/// Run the unit suite under coverage.py in `root` and check it against
/// `thresholds`.
///
/// Shells out to `coverage run --branch` (omitting `*_test.py` from the
/// denominator) then `coverage json`, and evaluates the report. The `coverage`
/// CLI — with `pytest` importable — must be on `PATH`.
pub fn measure(root: &Path, thresholds: Thresholds) -> Result<Outcome> {
    let report = run_coverage(root)?;
    Ok(evaluate(&report, thresholds))
}

/// A coverage.py data file under the temp dir — unique per call (so checks
/// running in parallel don't collide) and removed on drop (so nothing leaks
/// into the scanned tree).
struct DataFile(PathBuf);

impl DataFile {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "testing-conventions-{}-{}.coverage",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        );
        DataFile(std::env::temp_dir().join(name))
    }
}

impl Drop for DataFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Run coverage.py over the unit suite in `root` and return the parsed report.
fn run_coverage(root: &Path) -> Result<CoverageReport> {
    let data = DataFile::new();
    let omit = build_omit(root)?;

    // Branch coverage on; measure the sources in `root` with the test files —
    // and any `coverage`-waived files — omitted from the denominator. Byte-code
    // and the pytest cache are suppressed so the scanned tree stays pristine.
    let run = Command::new("coverage")
        .current_dir(root)
        .args(["run", "--branch"])
        .arg(format!("--omit={omit}"))
        .args(["-m", "pytest", "-q", "-p", "no:cacheprovider", "."])
        .env("COVERAGE_FILE", &data.0)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .output()
        .context("running `coverage run -m pytest` (is coverage.py installed?)")?;
    if !run.status.success() {
        bail!(
            "the unit suite did not run cleanly under coverage in `{}`:\n{}{}",
            root.display(),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr),
        );
    }

    // Emit the report to stdout and parse it.
    let json = Command::new("coverage")
        .current_dir(root)
        .args(["json", "-o", "-"])
        .env("COVERAGE_FILE", &data.0)
        .output()
        .context("running `coverage json`")?;
    if !json.status.success() {
        bail!(
            "`coverage json` failed:\n{}",
            String::from_utf8_lossy(&json.stderr),
        );
    }

    parse_report(&String::from_utf8_lossy(&json.stdout))
}

/// The single comma-joined `--omit` value for the coverage run: always
/// `*_test.py`, plus every Python source under `root` carrying a `coverage`
/// waiver. (coverage.py takes one `--omit` — repeated flags don't accumulate, so
/// the patterns must be joined.) A waived file leaves the denominator with its
/// reason recorded in the file itself — an auditable omission, not a silent
/// ignore-glob. A malformed waiver is an error.
fn build_omit(root: &Path) -> Result<String> {
    let mut waived = waived_coverage_files(root)?;
    waived.sort();
    Ok(std::iter::once(TEST_OMIT.to_string())
        .chain(waived)
        .collect::<Vec<_>>()
        .join(","))
}

/// Python source files under `root` carrying a `coverage` (or `all`) waiver, as
/// `root`-relative, `/`-separated paths (the form coverage.py records them in).
/// Malformed waivers are errors.
fn waived_coverage_files(root: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    collect_waived(root, root, &mut out)?;
    Ok(out)
}

/// Recursively gather `coverage`-waived `*.py` files under `dir` into `out`.
fn collect_waived(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("reading an entry under `{}`", dir.display()))?
            .path();
        if path.is_dir() {
            collect_waived(root, &path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("py") {
            let contents = std::fs::read_to_string(&path)
                .with_context(|| format!("reading `{}`", path.display()))?;
            let waived = waiver::waived_reason(&contents, Scope::Coverage)
                .with_context(|| format!("checking waivers in `{}`", path.display()))?;
            if waived.is_some() {
                let relative = path.strip_prefix(root).unwrap_or(&path);
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(percent_covered: f64, num_branches: u64) -> CoverageReport {
        CoverageReport {
            totals: Totals {
                percent_covered,
                num_branches,
            },
        }
    }

    #[test]
    fn passes_when_total_meets_the_floor() {
        assert_eq!(
            evaluate(
                &report(100.0, 12),
                Thresholds {
                    fail_under: 100,
                    branch: true
                }
            ),
            Outcome::Pass
        );
    }

    #[test]
    fn fails_when_total_is_below_the_floor() {
        assert!(matches!(
            evaluate(
                &report(80.0, 12),
                Thresholds {
                    fail_under: 100,
                    branch: true
                }
            ),
            Outcome::Fail(_)
        ));
    }

    #[test]
    fn fails_when_branch_required_but_unmeasured() {
        // branch=true but the report measured no branches → a misconfigured run.
        assert!(matches!(
            evaluate(
                &report(100.0, 0),
                Thresholds {
                    fail_under: 90,
                    branch: true
                }
            ),
            Outcome::Fail(_)
        ));
    }

    #[test]
    fn parses_a_coverage_py_report() {
        let json = r#"{"totals":{"percent_covered":91.5,"num_branches":8,"covered_lines":91}}"#;
        let report = parse_report(json).expect("valid coverage.py json");
        assert_eq!(report.totals.percent_covered, 91.5);
        assert_eq!(report.totals.num_branches, 8);
    }

    /// A throwaway directory tree, removed on drop, for the omit-scan tests.
    struct TempTree(PathBuf);

    impl TempTree {
        fn new(files: &[(&str, &str)]) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "tc-omit-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            for (rel, contents) in files {
                let path = root.join(rel);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, contents).unwrap();
            }
            TempTree(root)
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn omit_is_just_the_test_glob_when_nothing_is_waived() {
        let tree = TempTree::new(&[("widget.py", "x = 1\n"), ("widget_test.py", "")]);
        assert_eq!(build_omit(&tree.0).unwrap(), "*_test.py");
    }

    #[test]
    fn omit_folds_in_coverage_waived_files_sorted_and_relative() {
        let tree = TempTree::new(&[
            ("core.py", "x = 1\n"),
            (
                "shim.py",
                "# testing-conventions:waiver(coverage): launcher shim\n",
            ),
            (
                "pkg/gen.py",
                "# testing-conventions:waiver(all): generated code\n",
            ),
        ]);
        // *_test.py first, then waived files sorted; nested path is `/`-joined.
        assert_eq!(build_omit(&tree.0).unwrap(), "*_test.py,pkg/gen.py,shim.py");
    }

    #[test]
    fn a_location_only_waiver_does_not_omit_from_coverage() {
        let tree = TempTree::new(&[(
            "shim.py",
            "# testing-conventions:waiver(location): no colocated test\n",
        )]);
        assert_eq!(build_omit(&tree.0).unwrap(), "*_test.py");
    }

    #[test]
    fn a_malformed_waiver_makes_the_omit_scan_error() {
        let tree = TempTree::new(&[("shim.py", "# testing-conventions:waiver(coverage):\n")]);
        assert!(build_omit(&tree.0).is_err());
    }
}
