//! Diff-scoped coverage floor (Python — #132; TypeScript — #135; Rust — #136;
//! folded into `unit coverage --base` — #162; parent #46).
//!
//! Enforces the README Coverage rule over the lines a diff touches: where
//! [`crate::coverage`] measures the *whole* suite against the configured floor
//! (#26), the `measure*` functions here measure that same floor over only the
//! lines `<base>...HEAD` added or modified — `covered ÷ total-changed-executable`,
//! against the thresholds `unit coverage` enforces whole-tree. `unit coverage
//! --base` routes here, so a diff that clears the configured floor passes even with
//! an uncovered changed line, and one below it fails no matter how small (#162).
//!
//! Two inputs are combined:
//!   - the **diff** — [`changed_lines`] runs `git diff --unified=0 <base>...HEAD`
//!     and returns the new-side line numbers each file gained. This diff machinery
//!     is language-agnostic, shared by all three arms.
//!   - the **coverage** — per the language. Python ([`measure`]) reads coverage.py's
//!     per-file lines and branch arcs ([`crate::coverage::measure_patch_report`]),
//!     restricting the `percent_covered` ratio to the changed lines
//!     ([`evaluate_patch`]). TypeScript ([`measure_typescript`]) reduces vitest's v8
//!     export to the four per-metric counts
//!     ([`crate::coverage::measure_patch_typescript_detail`]) and Rust
//!     ([`measure_rust`]) reduces `cargo llvm-cov`'s export to the per-region counts
//!     ([`crate::coverage::measure_patch_rust_detail`]); each metric's ratio is then
//!     restricted to the changed lines ([`evaluate_patch_typescript`] /
//!     [`evaluate_patch_rust`]). Either way, non-executable changed lines (comments,
//!     blanks) and `coverage`-exempt files have nothing to cover and drop out of the
//!     ratio.
//!
//! Relationship to the commit-scoped co-change rule ([`crate::co_change`], #33):
//! co-change enforces that a changed source and its colocated *test* move
//! together; the diff-scoped floor enforces that the changed *lines* are actually
//! exercised. They are complementary, not overlapping — co-change can pass (the
//! test file changed) while the floor fails (the change isn't covered), and
//! vice versa.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::coverage::{
    self, FileCoverage, Outcome, RustThresholds, Thresholds, TypeScriptThresholds,
};

/// TypeScript source extensions the diff-scoped floor scopes to — the set
/// `coverage`'s `TS_INCLUDE` measures. A `.d.ts` declaration ends in `.ts` but
/// carries no runtime code; vitest excludes it from the report, so its changed
/// lines find nothing to cover and are skipped without a special case here.
const TS_EXTENSIONS: [&str; 4] = [".ts", ".tsx", ".mts", ".cts"];

/// Diff-scoped Python coverage floor (#162): measure `thresholds` over the
/// `<base>...HEAD` changed `.py` lines instead of the whole tree. `omit` is the
/// `coverage`-rule exemptions, as in [`crate::coverage::measure`] — an exempt file
/// is omitted from the run, so its changed lines drop out of the ratio.
///
/// Scopes to `.py` sources and returns early — with no coverage run — when the diff
/// touches none, so a PR that changes only docs or other languages doesn't pay for a
/// measurement (and is vacuously covered). Requires coverage.py + pytest + git; an
/// unresolvable `base` surfaces as an error rather than a silent pass.
pub fn measure(
    root: &Path,
    base: &str,
    thresholds: Thresholds,
    omit: &[String],
) -> Result<Outcome> {
    let mut changed = changed_lines(root, base)?;
    changed.retain(|path, _| path.ends_with(".py"));
    if changed.is_empty() {
        return Ok(Outcome::Pass);
    }
    let report = coverage::measure_patch_report(root, omit)?;
    let files = relative_keys(report.files, root);
    Ok(evaluate_patch(&changed, &files, thresholds))
}

/// Pure: the configured floor measured over the changed lines. Reproduces
/// coverage.py's `percent_covered` — (executed lines + taken branch arcs) ÷
/// (executable lines + all branch arcs) — restricted to the lines the diff touched,
/// so the same number `unit coverage` enforces whole-tree is judged on the diff.
///
/// A changed line absent from `files` (a comment or blank, a test file, or a
/// `coverage`-exempt file omitted from the run) has nothing to cover and is skipped;
/// when nothing executable changed, the diff is vacuously covered (`Pass`). With
/// `branch`, a branch arc counts toward the ratio when its source line is in the diff
/// — taken arcs as covered, untaken as missed — exactly as the whole-tree total folds
/// branches in. No small-diff carve-out: a tiny diff below the floor fails like any
/// other (#162).
fn evaluate_patch(
    changed: &BTreeMap<String, BTreeSet<u64>>,
    files: &BTreeMap<String, FileCoverage>,
    thresholds: Thresholds,
) -> Outcome {
    let mut covered: u64 = 0;
    let mut total: u64 = 0;
    for (file, lines) in changed {
        let Some(cov) = files.get(file) else {
            continue;
        };
        let executed: BTreeSet<u64> = cov.executed_lines.iter().copied().collect();
        let missing: BTreeSet<u64> = cov.missing_lines.iter().copied().collect();
        for &line in lines {
            if executed.contains(&line) {
                covered += 1;
                total += 1;
            } else if missing.contains(&line) {
                total += 1;
            }
        }
        if thresholds.branch {
            for arc in &cov.executed_branches {
                if arc_source_in(arc, lines) {
                    covered += 1;
                    total += 1;
                }
            }
            for arc in &cov.missing_branches {
                if arc_source_in(arc, lines) {
                    total += 1;
                }
            }
        }
    }
    if total == 0 {
        return Outcome::Pass;
    }
    let actual = 100.0 * covered as f64 / total as f64;
    // A hair of tolerance so a percent that rounds to the floor isn't failed by float
    // noise (matches the whole-tree `coverage::evaluate`).
    if actual + 1e-9 >= f64::from(thresholds.fail_under) {
        Outcome::Pass
    } else {
        Outcome::Fail(format!(
            "changed-line coverage {actual:.2}% is below the required {}%",
            thresholds.fail_under
        ))
    }
}

/// Whether a branch arc's source line (the first of its `[src, dst]` pair; `dst` may
/// be a negative exit, which is irrelevant) falls in `lines`.
fn arc_source_in(arc: &[i64], lines: &BTreeSet<u64>) -> bool {
    arc.first()
        .and_then(|&src| u64::try_from(src).ok())
        .is_some_and(|src| lines.contains(&src))
}

/// Diff-scoped TypeScript coverage floor (#162): the four vitest metrics measured
/// over the `<base>...HEAD` changed `.ts`/`.tsx`/`.mts`/`.cts` lines instead of the
/// whole tree. `exclude` is the `coverage`-rule exemptions, as in
/// [`crate::coverage::measure_typescript`] — an excluded file is left out of the
/// run, so its changed lines drop out of the ratios.
///
/// Scopes to TypeScript sources and returns early — with no coverage run — when the
/// diff touches none, so a PR that changes only docs or other languages doesn't pay
/// for a measurement (and is vacuously covered). Requires vitest + git; an
/// unresolvable `base` surfaces as an error rather than a silent pass.
pub fn measure_typescript(
    root: &Path,
    base: &str,
    thresholds: TypeScriptThresholds,
    exclude: &[String],
) -> Result<Outcome> {
    let mut changed = changed_lines(root, base)?;
    changed.retain(|path, _| TS_EXTENSIONS.iter().any(|ext| path.ends_with(ext)));
    if changed.is_empty() {
        return Ok(Outcome::Pass);
    }
    let detail = relative_keys(
        coverage::measure_patch_typescript_detail(root, exclude)?,
        root,
    );
    Ok(evaluate_patch_typescript(&changed, &detail, thresholds))
}

/// Pure: the four vitest floors measured over the changed lines. Each metric's
/// ratio is restricted to the lines the diff touched, so the same numbers
/// `unit coverage` enforces whole-tree are judged on the diff:
///   - **statements**: a `statementMap` entry counts when any line in its
///     `start..=end` is a changed line; covered when its flag is set.
///   - **lines**: a changed line counts when ≥1 statement *starts* on it; covered
///     when ≥1 statement starting on it is covered.
///   - **branches**: a branch arm counts when its `source_line` is a changed line;
///     covered when its flag is set.
///   - **functions**: a function counts when its `decl_line` is a changed line;
///     covered when its flag is set.
///
/// A changed file absent from `detail` (a test file, a declaration file, or a
/// `coverage`-exempt file left out of the run) has nothing to cover and is skipped.
/// Each metric's percent is `100 * covered / total`, or `100` when its denominator
/// is empty — a diff-scoped empty denominator is **vacuously satisfied**, not the
/// "measured no code" failure the whole-tree [`coverage::evaluate_typescript`]
/// returns (a diff may legitimately touch no branches or functions). The fail
/// message lists every metric below its floor, matching
/// [`coverage::evaluate_typescript`]'s. No small-diff carve-out: a tiny diff below
/// the floor fails like any other (#162).
fn evaluate_patch_typescript(
    changed: &BTreeMap<String, BTreeSet<u64>>,
    detail: &BTreeMap<String, coverage::TsPatchCoverage>,
    thresholds: TypeScriptThresholds,
) -> Outcome {
    let (mut s_cov, mut s_tot) = (0u64, 0u64);
    let (mut l_cov, mut l_tot) = (0u64, 0u64);
    let (mut b_cov, mut b_tot) = (0u64, 0u64);
    let (mut f_cov, mut f_tot) = (0u64, 0u64);

    for (file, lines) in changed {
        let Some(cov) = detail.get(file) else {
            continue;
        };

        // Statements: count one whenever any line it spans was changed.
        for &(start, end, covered) in &cov.statements {
            if (start..=end).any(|line| lines.contains(&line)) {
                s_tot += 1;
                if covered {
                    s_cov += 1;
                }
            }
        }

        // Lines: a changed line on which ≥1 statement *starts* counts; covered when
        // ≥1 statement starting on it is covered.
        for &line in lines {
            let mut starts_here = false;
            let mut covered_here = false;
            for &(start, _end, covered) in &cov.statements {
                if start == line {
                    starts_here = true;
                    covered_here |= covered;
                }
            }
            if starts_here {
                l_tot += 1;
                if covered_here {
                    l_cov += 1;
                }
            }
        }

        // Branch arms: count one whenever its source line was changed.
        for &(source_line, covered) in &cov.branch_arms {
            if lines.contains(&source_line) {
                b_tot += 1;
                if covered {
                    b_cov += 1;
                }
            }
        }

        // Functions: count one whenever its declaration line was changed.
        for &(decl_line, covered) in &cov.functions {
            if lines.contains(&decl_line) {
                f_tot += 1;
                if covered {
                    f_cov += 1;
                }
            }
        }
    }

    // An empty denominator is vacuously full (100%) — a diff may touch no branch or
    // function, which is satisfied, not the whole-tree "measured no code" failure.
    let pct = |covered: u64, total: u64| {
        if total == 0 {
            100.0
        } else {
            100.0 * covered as f64 / total as f64
        }
    };
    let checks = [
        ("lines", pct(l_cov, l_tot), thresholds.lines),
        ("branches", pct(b_cov, b_tot), thresholds.branches),
        ("functions", pct(f_cov, f_tot), thresholds.functions),
        ("statements", pct(s_cov, s_tot), thresholds.statements),
    ];
    let mut shortfalls = Vec::new();
    for (name, actual, required) in checks {
        // A hair of tolerance so a percent that rounds to the floor isn't failed by
        // float noise (matches the whole-tree `coverage::evaluate_typescript`).
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

/// Diff-scoped Rust coverage floor (#162): the `cargo llvm-cov` regions/lines
/// metrics measured over the `<base>...HEAD` changed `.rs` lines instead of the
/// whole tree. `ignore` is the `coverage`-rule exemptions, as in
/// [`crate::coverage::measure_rust`] — an exempt file is dropped from the run, so
/// its changed lines drop out of the ratios.
///
/// Scopes to `.rs` sources and returns early — with no coverage run — when the diff
/// touches none, so a PR that changes only docs or other languages doesn't pay for a
/// measurement (and is vacuously covered). Requires `cargo-llvm-cov` + git; an
/// unresolvable `base` surfaces as an error rather than a silent pass.
pub fn measure_rust(
    root: &Path,
    base: &str,
    thresholds: RustThresholds,
    ignore: &[String],
) -> Result<Outcome> {
    let mut changed = changed_lines(root, base)?;
    changed.retain(|path, _| path.ends_with(".rs"));
    if changed.is_empty() {
        return Ok(Outcome::Pass);
    }
    let detail = relative_keys(coverage::measure_patch_rust_detail(root, ignore)?, root);
    Ok(evaluate_patch_rust(&changed, &detail, thresholds))
}

/// Pure: the two `cargo llvm-cov` floors (regions, lines) measured over the changed
/// lines. Each metric's ratio is restricted to the lines the diff touched, so the
/// same numbers `unit coverage` enforces whole-tree are judged on the diff:
///   - **regions**: a code region counts when any line in its `start..=end` is a
///     changed line; covered when its flag is set.
///   - **lines**: a changed line counts when ≥1 region covers it (`start <= line <=
///     end`); covered when ≥1 covering region has its flag set.
///
/// A changed file absent from `detail` (a test-only file or a `coverage`-exempt file
/// dropped from the run) has nothing to cover and is skipped. Each metric's percent
/// is `100 * covered / total`, or `100` when its denominator is empty — a
/// diff-scoped empty denominator is **vacuously satisfied**, not the "measured no
/// code" failure the whole-tree [`coverage::evaluate_rust`] returns (a diff may
/// legitimately touch no measured region). The fail message lists every metric below
/// its floor, matching [`coverage::evaluate_rust`]'s. No small-diff carve-out: a tiny
/// diff below the floor fails like any other (#162).
fn evaluate_patch_rust(
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

        // Regions: count one whenever any line it spans was changed.
        for &(start, end, covered) in &cov.regions {
            if (start..=end).any(|line| lines.contains(&line)) {
                r_tot += 1;
                if covered {
                    r_cov += 1;
                }
            }
        }

        // Lines: a changed line covered by ≥1 region counts; covered when ≥1 region
        // covering it has its flag set.
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

    // An empty denominator is vacuously full (100%) — a diff may touch no measured
    // region, which is satisfied, not the whole-tree "measured no code" failure.
    let pct = |covered: u64, total: u64| {
        if total == 0 {
            100.0
        } else {
            100.0 * covered as f64 / total as f64
        }
    };
    // `regions` is opt-in (#206): skip the region check unless a config set a floor,
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

/// The new-side lines each file gained in `repo`'s `<base>...HEAD` diff, keyed by
/// `repo`-relative path. The diff machinery shared by the TS / Rust twins.
///
/// `<base>...HEAD` is the merge-base diff — the changes this branch introduced
/// (what a PR shows). `--unified=0` drops context lines so every `+` line is a
/// real addition; `--no-renames` keeps a rename a delete + an add (the added side
/// is held to coverage); `--relative` reports paths relative to `repo`. Returns an
/// error if `git diff` fails (e.g. `base` names no resolvable ref).
pub fn changed_lines(repo: &Path, base: &str) -> Result<BTreeMap<String, BTreeSet<u64>>> {
    let range = format!("{base}...HEAD");
    let output = Command::new("git")
        .current_dir(repo)
        .args([
            "diff",
            "--no-color",
            "--no-renames",
            "--unified=0",
            "--relative",
            &range,
        ])
        .output()
        .with_context(|| format!("running `git diff` in `{}`", repo.display()))?;
    if !output.status.success() {
        bail!(
            "`git diff {range}` failed in `{}`: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(parse_unified_diff(&String::from_utf8_lossy(&output.stdout)))
}

/// Pure: parse `git diff --unified=0` output into the new-side lines each file
/// gained. Tracks the current file from each `+++` header and the new-side line
/// counter from each `@@ … +c,d @@` hunk header, then records every following `+`
/// line (a deletion `-` consumes no new-side number). A deleted file
/// (`+++ /dev/null`) yields no entry.
fn parse_unified_diff(diff: &str) -> BTreeMap<String, BTreeSet<u64>> {
    let mut changed: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut next_line: u64 = 0;
    for line in diff.lines() {
        if let Some(header) = line.strip_prefix("+++ ") {
            current = new_side_path(header);
        } else if line.starts_with("@@") {
            if let Some(start) = hunk_new_start(line) {
                next_line = start;
            }
        } else if line.starts_with('+') {
            // An added new-side line — the `+++` header is handled above, so this
            // is diff body. Record it against the current file and advance.
            if let Some(file) = &current {
                changed.entry(file.clone()).or_default().insert(next_line);
            }
            next_line += 1;
        }
        // `-` (deleted) and metadata lines consume no new-side line and are skipped.
    }
    changed
}

/// The `repo`-relative new-side path from a `+++` diff header, or `None` for a
/// deletion (`+++ /dev/null`). Strips git's `b/` prefix and a trailing tab.
fn new_side_path(header: &str) -> Option<String> {
    let path = header
        .split('\t')
        .next()
        .unwrap_or(header)
        .trim_end_matches('\r');
    if path == "/dev/null" {
        return None;
    }
    let path = path.strip_prefix("b/").unwrap_or(path);
    Some(path.replace('\\', "/"))
}

/// The new-side start line from a hunk header `@@ -a,b +c,d @@ …` — the `c`. With
/// `--unified=0` the added lines that follow are numbered consecutively from it.
fn hunk_new_start(header: &str) -> Option<u64> {
    let plus = header.split_whitespace().find(|t| t.starts_with('+'))?;
    let digits = plus.trim_start_matches('+');
    digits.split(',').next().unwrap_or(digits).parse().ok()
}

/// Re-key a report's per-file map to `root`-relative `/`-joined paths so they match
/// the diff's paths. coverage.py reports paths relative to where it ran (here
/// `root`) and vitest reports absolute paths; an absolute path is stripped to
/// `root`, a relative one left as-is.
fn relative_keys<V>(files: BTreeMap<String, V>, root: &Path) -> BTreeMap<String, V> {
    files
        .into_iter()
        .map(|(key, value)| {
            let path = Path::new(&key);
            let rel = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            (rel, value)
        })
        .collect()
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

    // ---- parse_unified_diff --------------------------------------------------

    #[test]
    fn parses_added_lines_from_a_hunk() {
        // `+4,2` → two added lines numbered from 4; the function context after the
        // second `@@` is ignored.
        let diff = "diff --git a/widget.py b/widget.py\n\
                    index abc..def 100644\n\
                    --- a/widget.py\n\
                    +++ b/widget.py\n\
                    @@ -3,0 +4,2 @@ def f(x):\n\
                    +    if x == 99:\n\
                    +        return 7\n";
        assert_eq!(parse_unified_diff(diff), changed(&[("widget.py", &[4, 5])]));
    }

    #[test]
    fn parses_a_new_file_as_added_from_line_one() {
        let diff = "diff --git a/lonely.py b/lonely.py\n\
                    new file mode 100644\n\
                    index 0000000..bbb\n\
                    --- /dev/null\n\
                    +++ b/lonely.py\n\
                    @@ -0,0 +1,2 @@\n\
                    +def lonely():\n\
                    +    return 41\n";
        assert_eq!(parse_unified_diff(diff), changed(&[("lonely.py", &[1, 2])]));
    }

    #[test]
    fn a_deletion_only_hunk_records_no_added_lines() {
        // `+3,0` adds nothing; the `-` lines consume no new-side number.
        let diff = "diff --git a/widget.py b/widget.py\n\
                    index abc..def 100644\n\
                    --- a/widget.py\n\
                    +++ b/widget.py\n\
                    @@ -4,2 +3,0 @@ def f(x):\n\
                    -    dead = 1\n\
                    -    return dead\n";
        assert!(parse_unified_diff(diff).is_empty());
    }

    #[test]
    fn a_deleted_file_yields_no_entry() {
        let diff = "diff --git a/gone.py b/gone.py\n\
                    deleted file mode 100644\n\
                    index abc..0000000\n\
                    --- a/gone.py\n\
                    +++ /dev/null\n\
                    @@ -1,2 +0,0 @@\n\
                    -def gone():\n\
                    -    return 0\n";
        assert!(parse_unified_diff(diff).is_empty());
    }

    #[test]
    fn parses_multiple_files_and_a_single_line_hunk() {
        // `+2` (no count) is one line at line 2; a nested path is kept verbatim.
        let diff = "diff --git a/a.py b/a.py\n\
                    --- a/a.py\n\
                    +++ b/a.py\n\
                    @@ -1,0 +2 @@ def a():\n\
                    +    x = 1\n\
                    diff --git a/pkg/b.py b/pkg/b.py\n\
                    --- a/pkg/b.py\n\
                    +++ b/pkg/b.py\n\
                    @@ -10,0 +11,1 @@\n\
                    +    y = 2\n";
        assert_eq!(
            parse_unified_diff(diff),
            changed(&[("a.py", &[2]), ("pkg/b.py", &[11])])
        );
    }

    // ---- evaluate_patch (diff-scoped floor, #162) ---------------------------

    fn cov(
        executed: &[u64],
        missing: &[u64],
        executed_branches: &[[i64; 2]],
        missing_branches: &[[i64; 2]],
    ) -> FileCoverage {
        FileCoverage {
            executed_lines: executed.to_vec(),
            missing_lines: missing.to_vec(),
            excluded_lines: Vec::new(),
            executed_branches: executed_branches.iter().map(|b| b.to_vec()).collect(),
            missing_branches: missing_branches.iter().map(|b| b.to_vec()).collect(),
        }
    }

    const FLOOR_85: Thresholds = Thresholds {
        fail_under: 85,
        branch: true,
    };

    #[test]
    fn patch_a_fully_covered_diff_passes() {
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1, 2, 3], &[], &[], &[]))]);
        assert_eq!(
            evaluate_patch(&changed(&[("w.py", &[1, 2, 3])]), &files, FLOOR_85),
            Outcome::Pass
        );
    }

    #[test]
    fn patch_below_floor_fails_and_names_the_percent() {
        // 3 of 4 changed executable lines covered → 75% < 85.
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1, 2, 3], &[4], &[], &[]))]);
        let out = evaluate_patch(&changed(&[("w.py", &[1, 2, 3, 4])]), &files, FLOOR_85);
        assert!(
            matches!(&out, Outcome::Fail(m) if m.contains("75.00%")),
            "got: {out:?}"
        );
    }

    #[test]
    fn patch_the_same_diff_clears_a_lower_floor() {
        // The #162 behavior: 75% passes a 70 floor despite the uncovered line.
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1, 2, 3], &[4], &[], &[]))]);
        let floor_70 = Thresholds {
            fail_under: 70,
            branch: true,
        };
        assert_eq!(
            evaluate_patch(&changed(&[("w.py", &[1, 2, 3, 4])]), &files, floor_70),
            Outcome::Pass
        );
    }

    #[test]
    fn patch_counts_branch_arcs_whose_source_is_a_changed_line() {
        // Lines 1,2 executed (2 covered) + a taken arc out of line 2 (covered) and an
        // untaken arc out of line 2 (missed): 3 covered of 4 → 75% < 85.
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1, 2], &[], &[[2, 3]], &[[2, 4]]))]);
        let out = evaluate_patch(&changed(&[("w.py", &[1, 2])]), &files, FLOOR_85);
        assert!(
            matches!(&out, Outcome::Fail(m) if m.contains("75.00%")),
            "got: {out:?}"
        );
    }

    #[test]
    fn patch_branches_off_ignores_arcs() {
        // Same data, branch disabled: only the two executed lines count → 100%.
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1, 2], &[], &[[2, 3]], &[[2, 4]]))]);
        let no_branch = Thresholds {
            fail_under: 85,
            branch: false,
        };
        assert_eq!(
            evaluate_patch(&changed(&[("w.py", &[1, 2])]), &files, no_branch),
            Outcome::Pass
        );
    }

    #[test]
    fn patch_a_changed_file_absent_from_coverage_is_skipped() {
        // A test file (never measured) contributes nothing; with no other executable
        // changed line the diff is vacuously covered.
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1], &[], &[], &[]))]);
        assert_eq!(
            evaluate_patch(&changed(&[("w_test.py", &[1, 2])]), &files, FLOOR_85),
            Outcome::Pass
        );
    }

    #[test]
    fn patch_a_diff_with_no_executable_changed_lines_passes() {
        // Changed lines are comments/blanks (in neither executed nor missing) → vacuous.
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1, 2], &[], &[], &[]))]);
        assert_eq!(
            evaluate_patch(&changed(&[("w.py", &[9, 10])]), &files, FLOOR_85),
            Outcome::Pass
        );
    }

    // ---- evaluate_patch_typescript (diff-scoped TS floor, #162) -------------

    use coverage::TsPatchCoverage;

    fn ts_detail(entries: &[(&str, TsPatchCoverage)]) -> BTreeMap<String, TsPatchCoverage> {
        entries
            .iter()
            .map(|(path, cov)| (path.to_string(), cov.clone()))
            .collect()
    }

    const TS_FLOOR_80: TypeScriptThresholds = TypeScriptThresholds {
        lines: 80,
        branches: 80,
        functions: 80,
        statements: 80,
    };

    #[test]
    fn ts_patch_a_fully_covered_diff_passes() {
        // Two statements on lines 1-2, both starting on their line and both covered;
        // a covered function on line 1; a taken branch arm off line 2 → 100% all four.
        let detail = ts_detail(&[(
            "w.ts",
            TsPatchCoverage {
                statements: vec![(1, 1, true), (2, 2, true)],
                branch_arms: vec![(2, true)],
                functions: vec![(1, true)],
            },
        )]);
        assert_eq!(
            evaluate_patch_typescript(&changed(&[("w.ts", &[1, 2])]), &detail, TS_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn ts_patch_below_floor_fails_and_names_the_metric() {
        // Four changed lines each carry one statement; three covered, one not →
        // statements (and lines) 75% < 80, named; branches/functions are empty
        // (vacuously 100) and not named.
        let detail = ts_detail(&[(
            "w.ts",
            TsPatchCoverage {
                statements: vec![(1, 1, true), (2, 2, true), (3, 3, true), (4, 4, false)],
                branch_arms: vec![],
                functions: vec![],
            },
        )]);
        let out =
            evaluate_patch_typescript(&changed(&[("w.ts", &[1, 2, 3, 4])]), &detail, TS_FLOOR_80);
        assert!(
            matches!(&out, Outcome::Fail(m)
                if m.contains("statements 75.00% < 80%")
                    && m.contains("lines 75.00% < 80%")
                    && !m.contains("branches")
                    && !m.contains("functions")),
            "got: {out:?}"
        );
    }

    #[test]
    fn ts_patch_the_same_diff_clears_a_lower_floor() {
        // The #162 behavior: the 75% diff passes a 70 floor despite the uncovered line.
        let detail = ts_detail(&[(
            "w.ts",
            TsPatchCoverage {
                statements: vec![(1, 1, true), (2, 2, true), (3, 3, true), (4, 4, false)],
                branch_arms: vec![],
                functions: vec![],
            },
        )]);
        let floor_70 = TypeScriptThresholds {
            lines: 70,
            branches: 70,
            functions: 70,
            statements: 70,
        };
        assert_eq!(
            evaluate_patch_typescript(&changed(&[("w.ts", &[1, 2, 3, 4])]), &detail, floor_70),
            Outcome::Pass
        );
    }

    #[test]
    fn ts_patch_an_untaken_branch_arm_on_a_changed_line_fails_branches() {
        // Line 3's statement ran (covered) but one of its two branch arms never did:
        // branches 50% < 80, named; lines/statements are 100 (the statement is covered).
        let detail = ts_detail(&[(
            "w.ts",
            TsPatchCoverage {
                statements: vec![(3, 3, true)],
                branch_arms: vec![(3, true), (3, false)],
                functions: vec![],
            },
        )]);
        let out = evaluate_patch_typescript(&changed(&[("w.ts", &[3])]), &detail, TS_FLOOR_80);
        assert!(
            matches!(&out, Outcome::Fail(m)
                if m.contains("branches 50.00% < 80%")
                    && !m.contains("lines")
                    && !m.contains("statements")),
            "got: {out:?}"
        );
    }

    #[test]
    fn ts_patch_an_uncovered_function_decl_on_a_changed_line_fails_functions() {
        // A function declared on changed line 9 was never called → functions 0% < 80.
        let detail = ts_detail(&[(
            "w.ts",
            TsPatchCoverage {
                statements: vec![],
                branch_arms: vec![],
                functions: vec![(9, false)],
            },
        )]);
        let out = evaluate_patch_typescript(&changed(&[("w.ts", &[9])]), &detail, TS_FLOOR_80);
        assert!(
            matches!(&out, Outcome::Fail(m) if m.contains("functions 0.00% < 80%")),
            "got: {out:?}"
        );
    }

    #[test]
    fn ts_patch_a_changed_file_absent_from_coverage_is_skipped() {
        // A test file (never measured) contributes nothing; with no other changed
        // executable line the diff is vacuously covered.
        let detail = ts_detail(&[(
            "w.ts",
            TsPatchCoverage {
                statements: vec![(1, 1, true)],
                branch_arms: vec![],
                functions: vec![],
            },
        )]);
        assert_eq!(
            evaluate_patch_typescript(&changed(&[("w.test.ts", &[1, 2])]), &detail, TS_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn ts_patch_a_comment_only_diff_passes() {
        // The changed lines carry no statement/branch/function (a comment or blank) →
        // every denominator empty → vacuously covered.
        let detail = ts_detail(&[(
            "w.ts",
            TsPatchCoverage {
                statements: vec![(1, 1, true), (2, 2, true)],
                branch_arms: vec![(2, true)],
                functions: vec![(1, true)],
            },
        )]);
        assert_eq!(
            evaluate_patch_typescript(&changed(&[("w.ts", &[9, 10])]), &detail, TS_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn ts_patch_an_empty_diff_passes() {
        // No changed lines at all → vacuously covered at any floor.
        assert_eq!(
            evaluate_patch_typescript(&changed(&[]), &BTreeMap::new(), TS_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn ts_patch_a_multiline_statement_counts_when_any_of_its_lines_changed() {
        // A statement spanning lines 3-5 that never ran; only line 4 is in the diff →
        // it still counts (and is uncovered) → statements 0% < 80. No statement
        // *starts* on line 4, so lines has an empty denominator (vacuously 100).
        let detail = ts_detail(&[(
            "w.ts",
            TsPatchCoverage {
                statements: vec![(3, 5, false)],
                branch_arms: vec![],
                functions: vec![],
            },
        )]);
        let out = evaluate_patch_typescript(&changed(&[("w.ts", &[4])]), &detail, TS_FLOOR_80);
        assert!(
            matches!(&out, Outcome::Fail(m)
                if m.contains("statements 0.00% < 80%") && !m.contains("lines")),
            "got: {out:?}"
        );
    }

    // ---- evaluate_patch_rust (diff-scoped Rust floor, #162) -----------------

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
    };

    #[test]
    fn rust_patch_a_fully_covered_diff_passes() {
        // Two single-line code regions on lines 1-2, both covered → regions and lines
        // both 100%.
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
        // Four single-line regions on lines 1-4; three covered, one not → regions (and
        // lines) 75% < 80, both named.
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
        // The #162 behavior: the 75% diff passes a 70 floor despite the uncovered region.
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(1, 1, true), (2, 2, true), (3, 3, true), (4, 4, false)],
            },
        )]);
        let floor_70 = RustThresholds {
            regions: Some(70),
            lines: 70,
        };
        assert_eq!(
            evaluate_patch_rust(&changed(&[("w.rs", &[1, 2, 3, 4])]), &detail, floor_70),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_patch_skips_the_region_check_when_regions_is_opt_out() {
        // The zero-config default (#206) sets `regions: None`, so the diff-scoped floor
        // enforces lines only: a diff whose changed lines are all covered passes even
        // though one of its regions is uncovered (lines 1-4 are each covered by ≥1
        // region, but region 4 is not).
        let detail = rust_detail(&[(
            "w.rs",
            RustPatchCoverage {
                regions: vec![(1, 4, true), (4, 4, false)],
            },
        )]);
        let lines_only = RustThresholds {
            regions: None,
            lines: 100,
        };
        assert_eq!(
            evaluate_patch_rust(&changed(&[("w.rs", &[1, 2, 3, 4])]), &detail, lines_only),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_patch_an_uncovered_region_on_a_changed_line_fails_both_metrics() {
        // A single uncovered region on changed line 5 → regions 0% and lines 0%, both
        // below the floor.
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
        // A test-only file (never measured) contributes nothing; with no other changed
        // measured line the diff is vacuously covered.
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
        // The changed lines (9-10) carry no region (a comment or blank) → both
        // denominators empty → vacuously covered.
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
        // No changed lines at all → vacuously covered at any floor.
        assert_eq!(
            evaluate_patch_rust(&changed(&[]), &BTreeMap::new(), RUST_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_patch_a_multiline_region_counts_when_any_of_its_lines_changed() {
        // A region spanning lines 3-5 that never ran; only line 4 is in the diff → it
        // still counts for both metrics (the region spans line 4, so line 4 is a
        // measured-but-uncovered line) → regions 0% and lines 0% < 80.
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
        // Two overlapping regions span changed line 4 — one uncovered, one covered.
        // For the lines metric the line is covered (≥1 covering region's flag is set);
        // for regions, one of the two counts as covered → regions 50% (< 80, fails) but
        // lines 100% (≥ 80, not named).
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
