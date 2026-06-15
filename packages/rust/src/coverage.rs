//! Coverage rule (Python — issue #26).
//!
//! Enforces the README's Coverage rule: a library's unit suite must meet the
//! configured floor, measured with branch coverage, with test files excluded
//! from the denominator. This module is the deterministic core — given a
//! coverage.py JSON report ([`CoverageReport`]) and the [`Thresholds`] from
//! config, [`evaluate`] decides pass/fail. Producing the report (shelling out
//! to `coverage`) is a thin layer on top, kept separate so the guarantee is
//! testable without a Python toolchain.

use anyhow::{Context, Result};
use serde::Deserialize;

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
    todo!("#26: enforce the coverage floor")
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
}
