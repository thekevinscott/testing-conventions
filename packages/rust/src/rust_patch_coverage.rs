use std::collections::{BTreeMap, BTreeSet};

use crate::coverage::{self, Outcome, RustThresholds};

/// Pure: the two `cargo llvm-cov` floors (regions, lines) over the changed lines. A region
/// counts when the diff touches any line it spans, a line when a region covers it; an empty
/// denominator is vacuously full, not the whole-tree "measured no code" failure.
pub(crate) fn evaluate_patch_rust(
    changed: &BTreeMap<String, BTreeSet<u64>>,
    detail: &BTreeMap<String, coverage::RustPatchCoverage>,
    thresholds: RustThresholds,
) -> Outcome {
    let (mut r_cov, mut r_tot) = (0u64, 0u64);
    let (mut l_cov, mut l_tot) = (0u64, 0u64);

    for (file, lines) in changed {
        let Some(cov) = detail.get(file) else {
            continue;
        };

        for &(start, end, covered) in &cov.regions {
            if (start..=end).any(|line| lines.contains(&line)) {
                r_tot += 1;
                if covered {
                    r_cov += 1;
                }
            }
        }

        for &line in lines {
            let mut measured = false;
            let mut covered_here = false;
            for &(start, end, covered) in &cov.regions {
                if start <= line && line <= end {
                    measured = true;
                    covered_here |= covered;
                }
            }
            if measured {
                l_tot += 1;
                if covered_here {
                    l_cov += 1;
                }
            }
        }
    }

    let pct = |covered: u64, total: u64| {
        if total == 0 {
            100.0
        } else {
            100.0 * covered as f64 / total as f64
        }
    };
    // `regions` is opt-in: skip the region check unless a config set a floor,
    // matching the whole-tree `coverage::evaluate_rust`.
    let mut checks: Vec<(&str, f64, u8)> = Vec::new();
    if let Some(regions) = thresholds.regions {
        checks.push(("regions", pct(r_cov, r_tot), regions));
    }
    checks.push(("lines", pct(l_cov, l_tot), thresholds.lines));
    let mut shortfalls = Vec::new();
    for (name, actual, required) in checks {
        // A hair of tolerance so a percent that rounds to the floor isn't failed by
        // float noise (matches the whole-tree `coverage::evaluate_rust`).
        if actual + 1e-9 < f64::from(required) {
            shortfalls.push(format!("{name} {actual:.2}% < {required}%"));
        }
    }
    if shortfalls.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail(format!(
            "coverage below thresholds: {}",
            shortfalls.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn changed(entries: &[(&str, &[u64])]) -> BTreeMap<String, BTreeSet<u64>> {
        entries
            .iter()
            .map(|(path, lines)| (path.to_string(), lines.iter().copied().collect()))
            .collect()
    }

    use coverage::RustPatchCoverage;

    fn rust_detail(entries: &[(&str, RustPatchCoverage)]) -> BTreeMap<String, RustPatchCoverage> {
        entries
            .iter()
            .map(|(path, cov)| (path.to_string(), cov.clone()))
            .collect()
    }

    const RUST_FLOOR_80: RustThresholds = RustThresholds {
        regions: Some(80),
        lines: 80,
        functions: None,
        branch: None,
    };

    #[test]
    fn rust_patch_a_fully_covered_diff_passes() {
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(1, 1, true), (2, 2, true)],
            },
        )]);
        assert_eq!(
            evaluate_patch_rust(&changed(&[("w.rs", &[1, 2])]), &detail, RUST_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_patch_below_floor_fails_and_names_the_metrics() {
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(1, 1, true), (2, 2, true), (3, 3, true), (4, 4, false)],
            },
        )]);
        let out = evaluate_patch_rust(&changed(&[("w.rs", &[1, 2, 3, 4])]), &detail, RUST_FLOOR_80);
        assert!(
            matches!(&out, Outcome::Fail(m)
                if m.contains("regions 75.00% < 80%")
                    && m.contains("lines 75.00% < 80%")),
            "got: {out:?}"
        );
    }

    #[test]
    fn rust_patch_the_same_diff_clears_a_lower_floor() {
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(1, 1, true), (2, 2, true), (3, 3, true), (4, 4, false)],
            },
        )]);
        let floor_70 = RustThresholds {
            regions: Some(70),
            lines: 70,
            functions: None,
            branch: None,
        };
        assert_eq!(
            evaluate_patch_rust(&changed(&[("w.rs", &[1, 2, 3, 4])]), &detail, floor_70),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_patch_skips_the_region_check_when_regions_is_opt_out() {
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(1, 4, true), (4, 4, false)],
            },
        )]);
        let lines_only = RustThresholds {
            regions: None,
            lines: 100,
            functions: None,
            branch: None,
        };
        assert_eq!(
            evaluate_patch_rust(&changed(&[("w.rs", &[1, 2, 3, 4])]), &detail, lines_only),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_patch_an_uncovered_region_on_a_changed_line_fails_both_metrics() {
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(5, 5, false)],
            },
        )]);
        let out = evaluate_patch_rust(&changed(&[("w.rs", &[5])]), &detail, RUST_FLOOR_80);
        assert!(
            matches!(&out, Outcome::Fail(m)
                if m.contains("regions 0.00% < 80%") && m.contains("lines 0.00% < 80%")),
            "got: {out:?}"
        );
    }

    #[test]
    fn rust_patch_a_changed_file_absent_from_coverage_is_skipped() {
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(1, 1, true)],
            },
        )]);
        assert_eq!(
            evaluate_patch_rust(&changed(&[("other.rs", &[1, 2])]), &detail, RUST_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_patch_a_comment_only_diff_passes() {
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(1, 1, true), (2, 2, true)],
            },
        )]);
        assert_eq!(
            evaluate_patch_rust(&changed(&[("w.rs", &[9, 10])]), &detail, RUST_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_patch_an_empty_diff_passes() {
        assert_eq!(
            evaluate_patch_rust(&changed(&[]), &BTreeMap::new(), RUST_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_patch_a_multiline_region_counts_when_any_of_its_lines_changed() {
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(3, 5, false)],
            },
        )]);
        let out = evaluate_patch_rust(&changed(&[("w.rs", &[4])]), &detail, RUST_FLOOR_80);
        assert!(
            matches!(&out, Outcome::Fail(m)
                if m.contains("regions 0.00% < 80%") && m.contains("lines 0.00% < 80%")),
            "got: {out:?}"
        );
    }

    #[test]
    fn rust_patch_a_line_covered_by_any_region_is_covered() {
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(4, 4, false), (4, 6, true)],
            },
        )]);
        let out = evaluate_patch_rust(&changed(&[("w.rs", &[4])]), &detail, RUST_FLOOR_80);
        assert!(
            matches!(&out, Outcome::Fail(m)
                if m.contains("regions 50.00% < 80%") && !m.contains("lines")),
            "got: {out:?}"
        );
    }
}
