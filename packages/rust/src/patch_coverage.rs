//! Patch (changed-line) coverage (Python — #132; TypeScript — #135; Rust — #136;
//! parent #46).
//!
//! Enforces the README Coverage rule's changed-line guarantee: every line a diff
//! touches must be covered by the unit suite. Where [`crate::coverage`] measures
//! the *whole* suite against a floor (#26), this measures only the lines
//! `<base>...HEAD` added or modified — failing when any changed, executable line
//! is left uncovered.
//!
//! Two inputs are combined:
//!   - the **diff** — [`changed_lines`] runs `git diff --unified=0 <base>...HEAD`
//!     and returns the new-side line numbers each file gained. This diff machinery
//!     is language-agnostic, shared by all three arms.
//!   - the **coverage** — per the language. Python ([`check`]) reads coverage.py's
//!     per-file `missing_lines` / `missing_branches`
//!     ([`crate::coverage::measure_patch_report`]); a changed line is uncovered
//!     when it is a missing line or the source of a branch the suite never took
//!     ([`uncovered_changed_lines`]). TypeScript ([`check_typescript`]) and Rust
//!     ([`check_rust`]) reduce their per-file coverage (vitest's v8 export /
//!     `cargo llvm-cov`'s LCOV) to one uncovered-line set per file
//!     ([`crate::coverage::measure_patch_typescript`] /
//!     [`crate::coverage::measure_patch_rust`]) and intersect it directly with the
//!     set-based [`uncovered_changed_lines_ts`]. Either way, non-executable changed
//!     lines (comments, blanks) and `coverage`-exempt files have nothing to cover
//!     and are skipped.
//!
//! Relationship to the commit-scoped co-change rule ([`crate::co_change`], #33):
//! co-change enforces that a changed source and its colocated *test* move
//! together; patch coverage enforces that the changed *lines* are actually
//! exercised. They are complementary, not overlapping — co-change can pass (the
//! test file changed) while patch coverage fails (the change isn't covered), and
//! vice versa.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::coverage::{
    self, FileCoverage, Outcome, RustThresholds, Thresholds, TypeScriptThresholds,
};

/// A changed source line the unit suite doesn't cover — a `root`-relative path
/// and the 1-based new-side line number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uncovered {
    /// `root`-relative path of the changed file.
    pub file: String,
    /// The 1-based new-side line number that isn't covered.
    pub line: u64,
}

/// Every line added or modified in `root`'s `<base>...HEAD` diff that the unit
/// suite doesn't cover, sorted for deterministic output. `omit` is the
/// `coverage`-rule exemptions (as in [`crate::coverage::measure`]) — an exempt
/// file is omitted from the run, so its changed lines are lifted.
///
/// Scopes to `.py` sources (the Python arm this slice) and returns early — with
/// no coverage run — when the diff touches none, so a PR that changes only docs or
/// other languages doesn't pay for a measurement. Requires coverage.py + pytest +
/// git; an unresolvable `base` surfaces as an error rather than a silent pass.
pub fn check(root: &Path, base: &str, omit: &[String]) -> Result<Vec<Uncovered>> {
    let mut changed = changed_lines(root, base)?;
    changed.retain(|path, _| path.ends_with(".py"));
    if changed.is_empty() {
        return Ok(Vec::new());
    }
    let report = coverage::measure_patch_report(root, omit)?;
    let files = relative_keys(report.files, root);
    Ok(uncovered_changed_lines(&changed, &files))
}

/// TypeScript source extensions patch coverage scopes to — the set
/// `coverage`'s `TS_INCLUDE` measures. A `.d.ts` declaration ends in `.ts` but
/// carries no runtime code; vitest excludes it from the report, so its changed
/// lines find nothing to cover and are skipped without a special case here.
const TS_EXTENSIONS: [&str; 4] = [".ts", ".tsx", ".mts", ".cts"];

/// Every line added or modified in `root`'s `<base>...HEAD` diff that the
/// TypeScript unit suite (vitest) doesn't cover, sorted for deterministic output.
/// `exclude` is the `coverage`-rule exemptions (as in
/// [`crate::coverage::measure_typescript`]) — an excluded file is left out of the
/// run, so its changed lines are lifted.
///
/// The TypeScript twin of [`check`] (#135): reuses the same `<base>...HEAD` diff
/// machinery ([`changed_lines`]), scoped to `.ts` / `.tsx` / `.mts` / `.cts`
/// sources, and maps the changed lines against vitest's per-file v8 coverage
/// ([`crate::coverage::measure_patch_typescript`]). Returns early — with no
/// coverage run — when the diff touches no TypeScript source, so a PR that changes
/// only docs or other languages doesn't pay for a measurement. Requires vitest +
/// git; an unresolvable `base` surfaces as an error rather than a silent pass.
pub fn check_typescript(root: &Path, base: &str, exclude: &[String]) -> Result<Vec<Uncovered>> {
    let mut changed = changed_lines(root, base)?;
    changed.retain(|path, _| TS_EXTENSIONS.iter().any(|ext| path.ends_with(ext)));
    if changed.is_empty() {
        return Ok(Vec::new());
    }
    let uncovered = relative_keys(coverage::measure_patch_typescript(root, exclude)?, root);
    Ok(uncovered_changed_lines_ts(&changed, &uncovered))
}

/// Every line added or modified in `root`'s `<base>...HEAD` diff that the Rust
/// unit suite (`cargo llvm-cov`) doesn't cover, sorted for deterministic output.
/// `exclude` is the `coverage`-rule exemptions (as in
/// [`crate::coverage::measure_rust`]) — an excluded file is dropped from the run,
/// so its changed lines are lifted.
///
/// The Rust twin of [`check`] (#136), built on the Rust coverage rule (#37):
/// reuses the same `<base>...HEAD` diff machinery ([`changed_lines`]), scoped to
/// `.rs` sources, and maps the changed lines against `cargo llvm-cov`'s per-line
/// coverage ([`crate::coverage::measure_patch_rust`]). Returns early — with no
/// coverage run — when the diff touches no Rust source, so a PR that changes only
/// docs or other languages doesn't pay for a measurement. Requires `cargo-llvm-cov`
/// + git; an unresolvable `base` surfaces as an error rather than a silent pass.
pub fn check_rust(root: &Path, base: &str, exclude: &[String]) -> Result<Vec<Uncovered>> {
    let mut changed = changed_lines(root, base)?;
    changed.retain(|path, _| path.ends_with(".rs"));
    if changed.is_empty() {
        return Ok(Vec::new());
    }
    // `cargo llvm-cov`'s per-line coverage reduces to one uncovered-line set per
    // file (an LCOV `DA:<line>,0`), the same shape vitest's does — so the
    // intersection is the set-based [`uncovered_changed_lines_ts`].
    let uncovered = relative_keys(coverage::measure_patch_rust(root, exclude)?, root);
    Ok(uncovered_changed_lines_ts(&changed, &uncovered))
}

// ---------------------------------------------------------------------------
// Diff-scoped coverage floor — issue #162.
//
// Where `check` / `check_typescript` / `check_rust` above *list* the uncovered
// changed lines (the implicit-100% `unit patch-coverage`), these measure the
// configured floor over the diff: covered ÷ total changed-executable, against the
// same thresholds `unit coverage` enforces whole-tree. `unit coverage --base`
// routes here, so a diff that clears the configured floor passes even with an
// uncovered changed line — and one below it fails, no matter how small.
//
// Opens at RED per AGENTS.md: the ratio isn't computed yet, so each reports
// `Pass` regardless (an unresolvable base still errors). The real reduction —
// Python lines+branches, the four TypeScript metrics, Rust regions+lines, each
// restricted to the changed lines — lands once CI witnesses these red.
// ---------------------------------------------------------------------------

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
/// over the `<base>...HEAD` changed `.ts`/`.tsx`/`.mts`/`.cts` lines. `exclude` is
/// the `coverage`-rule exemptions, as in [`check_typescript`]. **Stub — opens red.**
pub fn measure_typescript(
    root: &Path,
    base: &str,
    thresholds: TypeScriptThresholds,
    exclude: &[String],
) -> Result<Outcome> {
    changed_lines(root, base)?;
    let _ = (thresholds, exclude);
    Ok(Outcome::Pass)
}

/// Diff-scoped Rust coverage floor (#162): the `cargo llvm-cov` regions/lines
/// metrics measured over the `<base>...HEAD` changed `.rs` lines. `ignore` is the
/// `coverage`-rule exemptions, as in [`check_rust`]. **Stub — opens red.**
pub fn measure_rust(
    root: &Path,
    base: &str,
    thresholds: RustThresholds,
    ignore: &[String],
) -> Result<Outcome> {
    changed_lines(root, base)?;
    let _ = (thresholds, ignore);
    Ok(Outcome::Pass)
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

/// Pure: every changed line the coverage report marks uncovered — a `missing_line`,
/// or the source of a `missing_branch` (a branch out of the line the suite never
/// took). A changed file absent from `files` was not measured (a test file, or a
/// `coverage`-exempt file omitted from the run) and contributes nothing; a changed
/// line that is neither missing nor a branch source (a comment or blank) has
/// nothing to cover. `files` is keyed by `root`-relative path, as `changed` is.
pub fn uncovered_changed_lines(
    changed: &BTreeMap<String, BTreeSet<u64>>,
    files: &BTreeMap<String, FileCoverage>,
) -> Vec<Uncovered> {
    let mut uncovered = Vec::new();
    for (file, lines) in changed {
        let Some(coverage) = files.get(file) else {
            continue;
        };
        let missing: BTreeSet<u64> = coverage.missing_lines.iter().copied().collect();
        // The source line of each branch never taken (the first of the
        // `[src, dst]` pair; `dst` may be negative — an exit — but `src` is a real
        // line, so a negative drops out via `try_from`).
        let branch_sources: BTreeSet<u64> = coverage
            .missing_branches
            .iter()
            .filter_map(|pair| pair.first().copied())
            .filter_map(|src| u64::try_from(src).ok())
            .collect();
        for &line in lines {
            if missing.contains(&line) || branch_sources.contains(&line) {
                uncovered.push(Uncovered {
                    file: file.clone(),
                    line,
                });
            }
        }
    }
    uncovered.sort();
    uncovered
}

/// Pure: every changed line a TypeScript coverage report marks uncovered.
/// `uncovered` is the per-file set of uncovered lines
/// ([`crate::coverage::measure_patch_typescript`]) — statements the suite never
/// ran and the source lines of branches a path of which it never took — keyed by
/// `root`-relative path, as `changed` is. A changed file absent from `uncovered`
/// was not measured (a test file, a declaration file, or a `coverage`-exempt file
/// excluded from the run) and contributes nothing; a changed line not in its set
/// (a comment or blank) has nothing to cover.
///
/// The TypeScript counterpart to [`uncovered_changed_lines`]: where coverage.py
/// splits missing lines from missing branches, vitest's report is reduced to one
/// uncovered-line set per file upstream, so this is the plain intersection.
pub fn uncovered_changed_lines_ts(
    changed: &BTreeMap<String, BTreeSet<u64>>,
    uncovered: &BTreeMap<String, BTreeSet<u64>>,
) -> Vec<Uncovered> {
    let mut out = Vec::new();
    for (file, lines) in changed {
        let Some(uncovered_lines) = uncovered.get(file) else {
            continue;
        };
        for &line in lines {
            if uncovered_lines.contains(&line) {
                out.push(Uncovered {
                    file: file.clone(),
                    line,
                });
            }
        }
    }
    out.sort();
    out
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

    fn file_coverage(missing_lines: &[u64], missing_branches: &[[i64; 2]]) -> FileCoverage {
        FileCoverage {
            executed_lines: Vec::new(),
            missing_lines: missing_lines.to_vec(),
            excluded_lines: Vec::new(),
            missing_branches: missing_branches.iter().map(|b| b.to_vec()).collect(),
            executed_branches: Vec::new(),
        }
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

    // ---- uncovered_changed_lines --------------------------------------------

    #[test]
    fn a_missing_changed_line_is_uncovered() {
        let out = uncovered_changed_lines(
            &changed(&[("widget.py", &[5])]),
            &BTreeMap::from([("widget.py".to_string(), file_coverage(&[5], &[]))]),
        );
        assert_eq!(
            out,
            vec![Uncovered {
                file: "widget.py".to_string(),
                line: 5
            }]
        );
    }

    #[test]
    fn a_covered_changed_line_is_not_reported() {
        // Line 3 changed but it's neither missing nor a branch source → covered.
        let out = uncovered_changed_lines(
            &changed(&[("widget.py", &[3])]),
            &BTreeMap::from([("widget.py".to_string(), file_coverage(&[5], &[[4, 5]]))]),
        );
        assert!(out.is_empty());
    }

    #[test]
    fn a_changed_branch_source_is_uncovered() {
        // Line 4 is executed (not a missing line) but a branch out of it was never
        // taken (`[4, 5]`), so a change to line 4 is still uncovered.
        let out = uncovered_changed_lines(
            &changed(&[("widget.py", &[4])]),
            &BTreeMap::from([("widget.py".to_string(), file_coverage(&[5], &[[4, 5]]))]),
        );
        assert_eq!(
            out,
            vec![Uncovered {
                file: "widget.py".to_string(),
                line: 4
            }]
        );
    }

    #[test]
    fn a_negative_branch_dest_is_ignored() {
        // `[6, -1]` is a branch to a function exit; the source line 6 is what
        // matters, and a change to line 6 is uncovered.
        let out = uncovered_changed_lines(
            &changed(&[("widget.py", &[6])]),
            &BTreeMap::from([("widget.py".to_string(), file_coverage(&[], &[[6, -1]]))]),
        );
        assert_eq!(
            out,
            vec![Uncovered {
                file: "widget.py".to_string(),
                line: 6
            }]
        );
    }

    #[test]
    fn a_changed_file_absent_from_coverage_is_skipped() {
        // A test file (omitted from the run) never appears in the report, so its
        // changed lines contribute nothing rather than panicking on a lookup.
        let out = uncovered_changed_lines(
            &changed(&[("widget_test.py", &[1, 2])]),
            &BTreeMap::from([("widget.py".to_string(), file_coverage(&[5], &[]))]),
        );
        assert!(out.is_empty());
    }

    #[test]
    fn reports_are_sorted_across_files_and_lines() {
        let out = uncovered_changed_lines(
            &changed(&[("z.py", &[2, 1]), ("a.py", &[9])]),
            &BTreeMap::from([
                ("z.py".to_string(), file_coverage(&[1, 2], &[])),
                ("a.py".to_string(), file_coverage(&[9], &[])),
            ]),
        );
        assert_eq!(
            out,
            vec![
                Uncovered {
                    file: "a.py".to_string(),
                    line: 9
                },
                Uncovered {
                    file: "z.py".to_string(),
                    line: 1
                },
                Uncovered {
                    file: "z.py".to_string(),
                    line: 2
                },
            ]
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

    // ---- uncovered_changed_lines_ts (TypeScript, #135) -----------------------

    #[test]
    fn ts_a_changed_uncovered_line_is_reported() {
        // Line 4 changed and the vitest report marks it uncovered → reported.
        let out = uncovered_changed_lines_ts(
            &changed(&[("widget.ts", &[4])]),
            &changed(&[("widget.ts", &[3, 4, 5])]),
        );
        assert_eq!(
            out,
            vec![Uncovered {
                file: "widget.ts".to_string(),
                line: 4
            }]
        );
    }

    #[test]
    fn ts_a_covered_changed_line_is_not_reported() {
        // Line 2 changed but it isn't in the uncovered set → covered, not reported.
        let out = uncovered_changed_lines_ts(
            &changed(&[("widget.ts", &[2])]),
            &changed(&[("widget.ts", &[3, 4, 5])]),
        );
        assert!(out.is_empty());
    }

    #[test]
    fn ts_a_changed_file_absent_from_coverage_is_skipped() {
        // A test file never appears in the report (it's excluded from the run), so
        // its changed lines contribute nothing rather than panicking on a lookup.
        let out = uncovered_changed_lines_ts(
            &changed(&[("widget.test.ts", &[1, 2])]),
            &changed(&[("widget.ts", &[5])]),
        );
        assert!(out.is_empty());
    }

    #[test]
    fn ts_reports_are_sorted_across_files_and_lines() {
        let out = uncovered_changed_lines_ts(
            &changed(&[("z.ts", &[2, 1]), ("a.ts", &[9])]),
            &changed(&[("z.ts", &[1, 2]), ("a.ts", &[9])]),
        );
        assert_eq!(
            out,
            vec![
                Uncovered {
                    file: "a.ts".to_string(),
                    line: 9
                },
                Uncovered {
                    file: "z.ts".to_string(),
                    line: 1
                },
                Uncovered {
                    file: "z.ts".to_string(),
                    line: 2
                },
            ]
        );
    }
}
