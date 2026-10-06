//! Diff-scoped coverage floor: the thresholds `unit coverage` enforces whole-tree,
//! measured over only the lines `<base>...HEAD` added or modified. Each language pairs
//! a `measure*` that shells out to its tool with a pure `evaluate_patch*` over the diff.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::coverage::{
    self, FileCoverage, Outcome, RustThresholds, Thresholds, TypeScriptThresholds,
};

/// TypeScript source extensions the diff-scoped floor scopes to — the set
/// `coverage`'s `TS_INCLUDE` measures.
const TS_EXTENSIONS: [&str; 4] = [".ts", ".tsx", ".mts", ".cts"];

/// Diff-scoped Python coverage floor: measure `thresholds` over the `<base>...HEAD`
/// changed `.py` lines instead of the whole tree. `omit` is the `coverage`-rule
/// exemptions; an exempt file's changed lines drop out of the ratio with it.
pub fn measure(
    root: &Path,
    base: &str,
    thresholds: Thresholds,
    omit: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
) -> Result<Outcome> {
    let mut changed = changed_lines(root, base)?;
    changed.retain(|path, _| path.ends_with(".py"));
    lift_exempt_lines(&mut changed, exempt_lines);
    if changed.is_empty() {
        return Ok(Outcome::Pass);
    }
    let report = coverage::measure_report(root, omit)?;
    let files = relative_keys(report.files, root);
    Ok(evaluate_patch(&changed, &files, thresholds))
}

/// Drop the line-scoped `coverage` exemptions from a `--base` diff's changed-line set.
/// The over-exemption guard belongs to the whole-tree floor ([`measure_line_exempt`]),
/// since the diff job can't classify a line its own diff didn't touch.
fn lift_exempt_lines(
    changed: &mut BTreeMap<String, BTreeSet<u64>>,
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
) {
    for (file, exempt) in exempt_lines {
        if let Some(lines) = changed.get_mut(file) {
            lines.retain(|&line| !u32::try_from(line).is_ok_and(|line| exempt.contains(&line)));
        }
    }
}

/// Pure: the configured floor over the changed lines. Reproduces coverage.py's
/// `percent_covered` — (executed lines + taken arcs) ÷ (executable lines + all arcs) —
/// restricted to the diff; a diff in which nothing executable changed is vacuously covered.
fn evaluate_patch(
    changed: &BTreeMap<String, BTreeSet<u64>>,
    files: &BTreeMap<String, FileCoverage>,
    thresholds: Thresholds,
) -> Outcome {
    let (covered, total) = python_ratio(changed, files, thresholds.branch);
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

/// Pure: coverage.py's `percent_covered` numerator/denominator restricted to `selected`.
/// Shared by the diff-scoped floor ([`evaluate_patch`], selected = the changed lines) and
/// the line-scoped one ([`measure_line_exempt`], selected = measured minus exempt).
fn python_ratio(
    selected: &BTreeMap<String, BTreeSet<u64>>,
    files: &BTreeMap<String, FileCoverage>,
    branch: bool,
) -> (u64, u64) {
    let mut covered: u64 = 0;
    let mut total: u64 = 0;
    for (file, lines) in selected {
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
        if branch {
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
    (covered, total)
}

/// Whether a branch arc's source line (the first of its `[src, dst]` pair) is in `lines`.
fn arc_source_in(arc: &[i64], lines: &BTreeSet<u64>) -> bool {
    arc.first()
        .and_then(|&src| u64::try_from(src).ok())
        .is_some_and(|src| lines.contains(&src))
}

/// Diff-scoped TypeScript coverage floor: the four vitest metrics measured over the
/// `<base>...HEAD` changed `.ts`/`.tsx`/`.mts`/`.cts` lines instead of the whole tree.
/// `exclude` is the `coverage`-rule exemptions; an excluded file's lines drop out with it.
pub fn measure_typescript(
    root: &Path,
    base: &str,
    thresholds: TypeScriptThresholds,
    exclude: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
) -> Result<Outcome> {
    let mut changed = changed_lines(root, base)?;
    changed.retain(|path, _| TS_EXTENSIONS.iter().any(|ext| path.ends_with(ext)));
    lift_exempt_lines(&mut changed, exempt_lines);
    if changed.is_empty() {
        return Ok(Outcome::Pass);
    }
    let detail = relative_keys(
        coverage::measure_patch_typescript_detail(root, exclude)?,
        root,
    );
    Ok(evaluate_patch_typescript(&changed, &detail, thresholds))
}

/// A covered-of-total tally over the changed lines.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Tally {
    covered: u64,
    total: u64,
}

impl Tally {
    /// Count one site. An uncovered site raises only the denominator.
    fn count(&mut self, covered: bool) {
        self.total += 1;
        self.covered += u64::from(covered);
    }

    fn add(&mut self, other: Tally) {
        self.covered += other.covered;
        self.total += other.total;
    }

    /// The tally as a percent. An empty denominator is vacuously full, not a failure.
    fn percent(self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            100.0 * self.covered as f64 / self.total as f64
        }
    }
}

/// Pure: the four vitest floors over the changed lines.
fn evaluate_patch_typescript(
    changed: &BTreeMap<String, BTreeSet<u64>>,
    detail: &BTreeMap<String, coverage::TsPatchCoverage>,
    thresholds: TypeScriptThresholds,
) -> Outcome {
    let tallies = typescript_tallies(changed, detail);
    let checks = [
        ("lines", tallies.lines.percent(), thresholds.lines),
        ("branches", tallies.branches.percent(), thresholds.branches),
        (
            "functions",
            tallies.functions.percent(),
            thresholds.functions,
        ),
        (
            "statements",
            tallies.statements.percent(),
            thresholds.statements,
        ),
    ];
    coverage::verdict(
        checks
            .into_iter()
            .filter_map(|(name, actual, required)| coverage::shortfall(name, actual, required))
            .collect(),
    )
}

/// The four vitest tallies the diff-scoped floor enforces, accumulated together.
#[derive(Default)]
struct TsTallies {
    statements: Tally,
    lines: Tally,
    branches: Tally,
    functions: Tally,
}

/// Accumulate the four tallies over only the changed lines of each file vitest measured. A
/// changed file absent from `detail` contributes nothing — vitest never measured it, so it
/// has no coverage to judge rather than zero coverage.
fn typescript_tallies(
    changed: &BTreeMap<String, BTreeSet<u64>>,
    detail: &BTreeMap<String, coverage::TsPatchCoverage>,
) -> TsTallies {
    let mut tallies = TsTallies::default();
    for (file, touched) in changed {
        let Some(cov) = detail.get(file) else {
            continue;
        };
        tallies
            .statements
            .add(statements_tally(&cov.statements, touched));
        tallies.lines.add(lines_tally(&cov.statements, touched));
        tallies
            .branches
            .add(on_line_tally(&cov.branch_arms, touched));
        tallies
            .functions
            .add(on_line_tally(&cov.functions, touched));
    }
    tallies
}

/// A statement counts when the diff touches any line it spans.
fn statements_tally(statements: &[(u64, u64, bool)], touched: &BTreeSet<u64>) -> Tally {
    let mut tally = Tally::default();
    for &(start, end, covered) in statements {
        if (start..=end).any(|line| touched.contains(&line)) {
            tally.count(covered);
        }
    }
    tally
}

/// A changed line counts when a statement *starts* on it, and is covered when any statement
/// starting there ran — one line can hold several.
fn lines_tally(statements: &[(u64, u64, bool)], touched: &BTreeSet<u64>) -> Tally {
    let mut tally = Tally::default();
    for &line in touched {
        let mut starts_here = false;
        let mut covered_here = false;
        for &(start, _end, covered) in statements {
            if start == line {
                starts_here = true;
                covered_here |= covered;
            }
        }
        if starts_here {
            tally.count(covered_here);
        }
    }
    tally
}

/// A branch arm or a function declaration counts when the diff touches its own line.
fn on_line_tally(sites: &[(u64, bool)], touched: &BTreeSet<u64>) -> Tally {
    let mut tally = Tally::default();
    for &(line, covered) in sites {
        if touched.contains(&line) {
            tally.count(covered);
        }
    }
    tally
}

/// Diff-scoped Rust coverage floor: the `cargo llvm-cov` regions/lines metrics measured
/// over the `<base>...HEAD` changed `.rs` lines instead of the whole tree. `ignore` is the
/// `coverage`-rule exemptions; an exempt file's lines drop out of the ratios with it.
pub fn measure_rust(
    root: &Path,
    base: &str,
    thresholds: RustThresholds,
    ignore: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
    features: &[String],
) -> Result<Outcome> {
    let mut changed = changed_lines(root, base)?;
    changed.retain(|path, _| path.ends_with(".rs"));
    lift_exempt_lines(&mut changed, exempt_lines);
    if changed.is_empty() {
        return Ok(Outcome::Pass);
    }
    let detail = relative_keys(
        coverage::measure_patch_rust_detail(root, ignore, features)?,
        root,
    );
    Ok(crate::rust_patch_coverage::evaluate_patch_rust(
        &changed, &detail, thresholds,
    ))
}

/// The new-side lines each file gained in `repo`'s `<base>...HEAD` merge-base diff, keyed
/// by `repo`-relative path. Pinned against the caller's git config — `core.quotepath=off`,
/// `--no-ext-diff`, forced `a/`/`b/` prefixes — so [`new_side_path`]'s `b/` strip holds.
pub fn changed_lines(repo: &Path, base: &str) -> Result<BTreeMap<String, BTreeSet<u64>>> {
    let range = format!("{base}...HEAD");
    let output = Command::new("git")
        .current_dir(repo)
        .args(unified_diff_argv(&range))
        .output()
        .with_context(|| format!("running `git diff` in `{}`", repo.display()))?;
    diff_exit(&output, repo, &range)?;
    Ok(parse_unified_diff(&String::from_utf8_lossy(&output.stdout)))
}

/// The `git diff` argv `changed_lines` runs. Every flag is load-bearing: the config
/// overrides and prefix flags pin the output against the caller's git config so
/// [`new_side_path`]'s `b/` strip holds, and `--unified=0` keeps context lines out of the
/// changed set.
fn unified_diff_argv(range: &str) -> [&str; 11] {
    [
        "-c",
        "core.quotepath=off",
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--no-renames",
        "--unified=0",
        "--relative",
        "--src-prefix=a/",
        "--dst-prefix=b/",
        range,
    ]
}

/// The error a failed `git diff` becomes, naming the range and the repo. Kept out of
/// [`changed_lines`] so that seam stays branch-free.
fn diff_exit(output: &std::process::Output, repo: &Path, range: &str) -> Result<()> {
    if !output.status.success() {
        bail!(
            "`git diff {range}` failed in `{}`: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// Pure: parse `git diff --unified=0` output into the new-side lines each file gained.
/// Hunk state guards header detection: a `+++ ` line is a file header only before the
/// first `@@`; inside a hunk it is body — an added line whose content began `++ `.
fn parse_unified_diff(diff: &str) -> BTreeMap<String, BTreeSet<u64>> {
    let mut scan = DiffScan::default();
    for line in diff.lines() {
        scan.feed(line);
    }
    scan.changed
}

/// The walk's position in a `git diff --unified=0` stream: which file's header was last
/// seen, which new-side line the next `+` lands on, and whether a hunk is open. The four
/// move together, so they are one value rather than four locals.
#[derive(Default)]
struct DiffScan {
    changed: BTreeMap<String, BTreeSet<u64>>,
    current: Option<String>,
    next_line: u64,
    in_hunk: bool,
}

impl DiffScan {
    /// Advance the scan by one line of diff output.
    fn feed(&mut self, line: &str) {
        if line.starts_with("diff --git ") {
            self.in_hunk = false;
            self.current = None;
        } else if line.starts_with("@@") {
            self.in_hunk = true;
            if let Some(start) = hunk_new_start(line) {
                self.next_line = start;
            }
        } else if !self.in_hunk {
            if let Some(header) = line.strip_prefix("+++ ") {
                self.current = new_side_path(header);
            }
        } else if line.starts_with('+') {
            if let Some(file) = &self.current {
                self.changed
                    .entry(file.clone())
                    .or_default()
                    .insert(self.next_line);
            }
            self.next_line += 1;
        }
    }
}

/// The `repo`-relative new-side path from a `+++` diff header, or `None` for a deletion
/// (`+++ /dev/null`). Decodes a C-quoted path before stripping git's `b/` prefix.
fn new_side_path(header: &str) -> Option<String> {
    let raw = header
        .split('\t')
        .next()
        .unwrap_or(header)
        .trim_end_matches('\r');
    if raw == "/dev/null" {
        return None;
    }
    // Git quotes the whole thing including the prefix (`"b/föö.py"`), so decode first.
    let unquoted = crate::git_path::unquote_c_path(raw);
    let path = unquoted.strip_prefix("b/").unwrap_or(&unquoted);
    Some(path.replace('\\', "/"))
}

/// The new-side start line from a hunk header `@@ -a,b +c,d @@ …` — the `c`. With
/// `--unified=0` the added lines that follow are numbered consecutively from it.
fn hunk_new_start(header: &str) -> Option<u64> {
    let plus = header.split_whitespace().find(|t| t.starts_with('+'))?;
    let digits = plus.trim_start_matches('+');
    digits.split(',').next().unwrap_or(digits).parse().ok()
}

/// Re-key a report's per-file map to `root`-relative `/`-joined paths so they match the
/// diff's. coverage.py reports relative to where it ran (here `root`) and vitest reports
/// absolute; an absolute path is stripped to `root`, a relative one left as-is.
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

/// Diff-free Python coverage floor with line-scoped exemptions: measure `thresholds` over
/// every measured line except the `exempt_lines`. `omit` is the whole-file `coverage`
/// exemptions. Runs only when `exempt_lines` is non-empty.
pub fn measure_line_exempt(
    root: &Path,
    thresholds: Thresholds,
    omit: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
) -> Result<Outcome> {
    let report = coverage::measure_report(root, omit)?;
    let files = relative_keys(report.files, root);
    let detail: BTreeMap<String, (BTreeSet<u64>, BTreeSet<u64>)> = files
        .iter()
        .map(|(file, cov)| (file.clone(), python_measured_missed(cov, thresholds.branch)))
        .collect();
    let line_set = apply_line_exemptions(&detail, exempt_lines)?;
    let (covered, total) = python_ratio(&line_set, &files, thresholds.branch);
    Ok(floor_outcome(covered, total, thresholds.fail_under))
}

/// TypeScript twin of [`measure_line_exempt`]: the four vitest metrics over every measured
/// line except the `exempt_lines`. `exclude` is the whole-file `coverage` exemptions.
pub fn measure_line_exempt_typescript(
    root: &Path,
    thresholds: TypeScriptThresholds,
    exclude: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
) -> Result<Outcome> {
    let detail = relative_keys(
        coverage::measure_patch_typescript_detail(root, exclude)?,
        root,
    );
    let measured_missed: BTreeMap<String, (BTreeSet<u64>, BTreeSet<u64>)> = detail
        .iter()
        .map(|(file, cov)| (file.clone(), ts_measured_missed(cov)))
        .collect();
    let line_set = apply_line_exemptions(&measured_missed, exempt_lines)?;
    Ok(evaluate_patch_typescript(&line_set, &detail, thresholds))
}

/// Rust twin of [`measure_line_exempt`]: the `cargo llvm-cov` regions/lines metrics over
/// every measured line except the `exempt_lines`. `ignore` is the whole-file exemptions.
pub fn measure_line_exempt_rust(
    root: &Path,
    thresholds: RustThresholds,
    ignore: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
    features: &[String],
) -> Result<Outcome> {
    let detail = relative_keys(
        coverage::measure_patch_rust_detail(root, ignore, features)?,
        root,
    );
    let measured_missed: BTreeMap<String, (BTreeSet<u64>, BTreeSet<u64>)> = detail
        .iter()
        .map(|(file, cov)| (file.clone(), rust_measured_missed(cov, thresholds)))
        .collect();
    let line_set = apply_line_exemptions(&measured_missed, exempt_lines)?;
    Ok(crate::rust_patch_coverage::evaluate_patch_rust(
        &line_set, &detail, thresholds,
    ))
}

/// The whole-tree floor verdict for a recomputed `covered`/`total`, with the same message
/// and float tolerance as the tool-total [`crate::coverage::evaluate`].
fn floor_outcome(covered: u64, total: u64, fail_under: u8) -> Outcome {
    if total == 0 {
        return Outcome::Pass;
    }
    let actual = 100.0 * covered as f64 / total as f64;
    if actual + 1e-9 >= f64::from(fail_under) {
        Outcome::Pass
    } else {
        Outcome::Fail(format!(
            "coverage {actual:.2}% is below the required {fail_under}%"
        ))
    }
}

/// The `(measured, missed)` lines for one Python file. `missed` is the uncovered lines,
/// plus (under branch coverage) any line that is the source of an untaken branch arc.
fn python_measured_missed(cov: &FileCoverage, branch: bool) -> (BTreeSet<u64>, BTreeSet<u64>) {
    let executed: BTreeSet<u64> = cov.executed_lines.iter().copied().collect();
    let missing: BTreeSet<u64> = cov.missing_lines.iter().copied().collect();
    let measured: BTreeSet<u64> = executed.union(&missing).copied().collect();
    let mut missed = missing;
    if branch {
        for arc in &cov.missing_branches {
            if let Some(src) = arc.first().and_then(|&s| u64::try_from(s).ok()) {
                if measured.contains(&src) {
                    missed.insert(src);
                }
            }
        }
    }
    (measured, missed)
}

/// The `(measured, missed)` lines for one TypeScript file. A unit is anchored on the lines
/// it spans, so a line carrying any uncovered unit is exemptable.
fn ts_measured_missed(cov: &coverage::TsPatchCoverage) -> (BTreeSet<u64>, BTreeSet<u64>) {
    let mut measured = BTreeSet::new();
    let mut missed = BTreeSet::new();
    let units = cov
        .statements
        .iter()
        .flat_map(|&(start, end, covered)| (start..=end).map(move |line| (line, covered)))
        .chain(cov.branch_arms.iter().copied())
        .chain(cov.functions.iter().copied());
    for (line, covered) in units {
        measured.insert(line);
        if !covered {
            missed.insert(line);
        }
    }
    (measured, missed)
}

/// The `(measured, missed)` lines for one Rust file. `missed` honors the enforced metrics
/// — with `regions` on, any line in an uncovered region; with lines-only, a line covered
/// by regions but by no *covered* one.
fn rust_measured_missed(
    cov: &coverage::RustPatchCoverage,
    thresholds: RustThresholds,
) -> (BTreeSet<u64>, BTreeSet<u64>) {
    let measured = measured_lines(cov);
    let missed = measured
        .iter()
        .copied()
        .filter(|&line| is_missed(cov, line, thresholds))
        .collect();
    (measured, missed)
}

/// Every line any region of `cov` spans — the lines llvm-cov actually measured for the file.
fn measured_lines(cov: &coverage::RustPatchCoverage) -> BTreeSet<u64> {
    let mut measured = BTreeSet::new();
    for &(start, end, _covered) in &cov.regions {
        for line in start..=end {
            measured.insert(line);
        }
    }
    measured
}

/// Whether `line` counts as missed, which the enforced metrics decide. With a region floor
/// on, a line is missed when *any* region over it is uncovered — region coverage is the
/// stricter reading. With lines only, a line is missed when *no* region over it is covered,
/// so one covered region redeems it.
fn is_missed(cov: &coverage::RustPatchCoverage, line: u64, thresholds: RustThresholds) -> bool {
    let mut covered_here = false;
    let mut uncovered_region = false;
    for &(start, end, covered) in &cov.regions {
        if start <= line && line <= end {
            if covered {
                covered_here = true;
            } else {
                uncovered_region = true;
            }
        }
    }
    if thresholds.regions.is_some() {
        uncovered_region
    } else {
        !covered_here
    }
}

/// The per-file line set the floor is measured over — every measured line minus the exempt
/// ones, after the determinism guard.
fn apply_line_exemptions(
    detail: &BTreeMap<String, (BTreeSet<u64>, BTreeSet<u64>)>,
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
) -> Result<BTreeMap<String, BTreeSet<u64>>> {
    guard_exempt_lines_are_failing(detail, exempt_lines)?;
    Ok(detail
        .iter()
        .map(|(file, (measured, _))| (file.clone(), kept_lines(measured, exempt_lines.get(file))))
        .collect())
}

/// The determinism guard: each exempt line must be genuinely failing, so an exemption can't
/// excuse working code.
fn guard_exempt_lines_are_failing(
    detail: &BTreeMap<String, (BTreeSet<u64>, BTreeSet<u64>)>,
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
) -> Result<()> {
    let mut over: Vec<String> = Vec::new();
    for (file, lines) in exempt_lines {
        let missed = detail.get(file).map(|(_, missed)| missed);
        for &line in lines {
            let failing = missed.is_some_and(|missed| missed.contains(&u64::from(line)));
            if !failing {
                over.push(format!("\n  {file}:{line}"));
            }
        }
    }
    if !over.is_empty() {
        bail!(
            "a line-scoped coverage exemption may only list uncovered lines, but these are \
             covered or carry no measured code:{}",
            over.concat()
        );
    }
    Ok(())
}

/// `measured` less the `exempt` lines. A measured line too large for a `u32` can carry no
/// exemption, since an exemption is written as one.
fn kept_lines(measured: &BTreeSet<u64>, exempt: Option<&BTreeSet<u32>>) -> BTreeSet<u64> {
    measured
        .iter()
        .copied()
        .filter(|&line| {
            !exempt
                .is_some_and(|exempt| u32::try_from(line).is_ok_and(|line| exempt.contains(&line)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_diff_argv_pins_every_flag_its_doc_comment_calls_load_bearing() {
        let argv = unified_diff_argv("main...HEAD");
        // The config override and prefix flags pin the output against the caller's git
        // config, which is what lets `new_side_path` strip a fixed `b/`.
        assert!(argv.starts_with(&["-c", "core.quotepath=off", "diff"]));
        assert!(argv.contains(&"--src-prefix=a/"));
        assert!(argv.contains(&"--dst-prefix=b/"));
        assert!(argv.contains(&"--no-ext-diff"));
        assert!(argv.contains(&"--no-color"));
        assert!(argv.contains(&"--no-renames"));
        // Context lines are not changed lines.
        assert!(argv.contains(&"--unified=0"));
        assert_eq!(argv.last(), Some(&"main...HEAD"));
    }

    #[test]
    fn a_failed_diff_names_the_range_and_the_stderr() {
        let output = std::process::Output {
            status: failed_status(),
            stdout: Vec::new(),
            stderr: b"fatal: bad revision".to_vec(),
        };
        let err = diff_exit(&output, Path::new("/repo"), "nope...HEAD").unwrap_err();
        let text = err.to_string();
        assert!(text.contains("nope...HEAD"), "got: {text}");
        assert!(text.contains("fatal: bad revision"), "got: {text}");
    }

    #[test]
    fn a_successful_diff_is_not_an_error() {
        let output = std::process::Output {
            status: ok_status(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        assert!(diff_exit(&output, Path::new("/repo"), "main...HEAD").is_ok());
    }

    #[test]
    fn measured_lines_spans_every_region_inclusive_of_both_ends() {
        let cov = coverage::RustPatchCoverage {
            regions: vec![(3, 5, true), (9, 9, false)],
        };
        assert_eq!(
            measured_lines(&cov),
            [3, 4, 5, 9].into_iter().collect::<BTreeSet<u64>>()
        );
    }

    #[test]
    fn with_a_region_floor_any_uncovered_region_misses_the_line() {
        // Line 4 is spanned by one covered and one uncovered region.
        let cov = coverage::RustPatchCoverage {
            regions: vec![(4, 4, true), (4, 4, false)],
        };
        assert!(is_missed(&cov, 4, region_thresholds()));
    }

    #[test]
    fn with_lines_only_one_covered_region_redeems_the_line() {
        let cov = coverage::RustPatchCoverage {
            regions: vec![(4, 4, true), (4, 4, false)],
        };
        assert!(!is_missed(&cov, 4, lines_only_thresholds()));
    }

    #[test]
    fn a_line_no_region_covers_is_missed_under_either_reading() {
        let cov = coverage::RustPatchCoverage {
            regions: vec![(4, 4, false)],
        };
        assert!(is_missed(&cov, 4, region_thresholds()));
        assert!(is_missed(&cov, 4, lines_only_thresholds()));
    }

    fn region_thresholds() -> RustThresholds {
        RustThresholds {
            regions: Some(100),
            lines: 100,
            functions: None,
            branch: None,
        }
    }

    fn lines_only_thresholds() -> RustThresholds {
        RustThresholds {
            regions: None,
            lines: 100,
            functions: None,
            branch: None,
        }
    }

    /// An exit status built directly rather than spawned: `/bin/true` and `/bin/false` would
    /// be two real processes to manufacture a value `ExitStatusExt` hands over for free.
    fn status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code << 8)
    }

    fn ok_status() -> std::process::ExitStatus {
        status(0)
    }

    fn failed_status() -> std::process::ExitStatus {
        status(1)
    }

    fn changed(entries: &[(&str, &[u64])]) -> BTreeMap<String, BTreeSet<u64>> {
        entries
            .iter()
            .map(|(path, lines)| (path.to_string(), lines.iter().copied().collect()))
            .collect()
    }

    #[test]
    fn changed_lines_reports_a_spawn_failure() {
        let err = changed_lines(Path::new("/nonexistent-tc-patch-coverage"), "main").unwrap_err();
        assert!(err.to_string().contains("running `git diff`"), "got: {err}");
    }

    #[test]
    fn parses_added_lines_from_a_hunk() {
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

    #[test]
    fn a_plus_plus_body_line_is_not_a_file_header() {
        // An added line whose content begins `++ ` renders as `+++ …` — hunk *body*, not a
        // `+++` file header. Read as a header it would divert the file's later added lines
        // to a bogus key, dropping them from scoping: a false green.
        let diff = "diff --git a/w.py b/w.py\n\
                    index abc..def 100644\n\
                    --- a/w.py\n\
                    +++ b/w.py\n\
                    @@ -1,0 +1,3 @@\n\
                    +++ 1\n\
                    +y = 1\n\
                    +z = 2\n";
        assert_eq!(parse_unified_diff(diff), changed(&[("w.py", &[1, 2, 3])]));
    }

    #[test]
    fn new_side_path_decodes_a_c_quoted_non_ascii_path() {
        // With `core.quotepath` on (git's default) a non-ASCII header path is C-quoted:
        // `src/föö.py` → `"b/src/f\303\266\303\266.py"`. Left quoted it matches no
        // coverage-report key, so the changed lines are silently skipped — a vacuous pass.
        assert_eq!(
            new_side_path("\"b/src/f\\303\\266\\303\\266.py\"").as_deref(),
            Some("src/föö.py")
        );
        assert_eq!(new_side_path("b/src/föö.py").as_deref(), Some("src/föö.py"));
    }

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
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1, 2, 3], &[4], &[], &[]))]);
        let out = evaluate_patch(&changed(&[("w.py", &[1, 2, 3, 4])]), &files, FLOOR_85);
        assert!(
            matches!(&out, Outcome::Fail(m) if m.contains("75.00%")),
            "got: {out:?}"
        );
    }

    #[test]
    fn patch_the_same_diff_clears_a_lower_floor() {
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
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1, 2], &[], &[[2, 3]], &[[2, 4]]))]);
        let out = evaluate_patch(&changed(&[("w.py", &[1, 2])]), &files, FLOOR_85);
        assert!(
            matches!(&out, Outcome::Fail(m) if m.contains("75.00%")),
            "got: {out:?}"
        );
    }

    #[test]
    fn patch_branches_off_ignores_arcs() {
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
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1], &[], &[], &[]))]);
        assert_eq!(
            evaluate_patch(&changed(&[("w_test.py", &[1, 2])]), &files, FLOOR_85),
            Outcome::Pass
        );
    }

    #[test]
    fn patch_a_diff_with_no_executable_changed_lines_passes() {
        let files = BTreeMap::from([("w.py".to_string(), cov(&[1, 2], &[], &[], &[]))]);
        assert_eq!(
            evaluate_patch(&changed(&[("w.py", &[9, 10])]), &files, FLOOR_85),
            Outcome::Pass
        );
    }

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
        assert_eq!(
            evaluate_patch_typescript(&changed(&[]), &BTreeMap::new(), TS_FLOOR_80),
            Outcome::Pass
        );
    }

    #[test]
    fn ts_patch_a_multiline_statement_counts_when_any_of_its_lines_changed() {
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

    fn exempt(entries: &[(&str, &[u32])]) -> BTreeMap<String, BTreeSet<u32>> {
        entries
            .iter()
            .map(|(path, lines)| (path.to_string(), lines.iter().copied().collect()))
            .collect()
    }

    #[test]
    fn python_measured_missed_reads_lines_and_branch_sources() {
        let full = cov(&[1], &[2, 3, 4], &[], &[[2, 3], [2, 4]]);
        let (measured, missed) = python_measured_missed(&full, true);
        assert_eq!(measured, [1, 2, 3, 4].into_iter().collect());
        assert_eq!(missed, [2, 3, 4].into_iter().collect());
        let partial = cov(&[5], &[], &[], &[[5, 6]]);
        let (_, missed_no_branch) = python_measured_missed(&partial, false);
        assert!(missed_no_branch.is_empty());
        let (_, missed_branch) = python_measured_missed(&partial, true);
        assert_eq!(missed_branch, [5].into_iter().collect());
    }

    #[test]
    fn python_measured_missed_skips_an_arc_from_an_unmeasured_line() {
        let partial = cov(&[1], &[], &[], &[[99, 1]]);
        let (measured, missed) = python_measured_missed(&partial, true);
        assert_eq!(measured, [1].into_iter().collect());
        assert!(missed.is_empty(), "got: {missed:?}");
    }

    #[test]
    fn python_measured_missed_skips_an_arc_with_a_negative_source() {
        let partial = cov(&[1], &[], &[], &[[-1, 1]]);
        let (measured, missed) = python_measured_missed(&partial, true);
        assert_eq!(measured, [1].into_iter().collect());
        assert!(missed.is_empty(), "got: {missed:?}");
    }

    #[test]
    fn ts_measured_missed_anchors_units_on_their_lines() {
        let cov = coverage::TsPatchCoverage {
            statements: vec![(1, 1, true), (3, 4, false)],
            branch_arms: vec![(1, false)],
            functions: vec![(6, false)],
        };
        let (measured, missed) = ts_measured_missed(&cov);
        assert_eq!(measured, [1, 3, 4, 6].into_iter().collect());
        assert_eq!(missed, [1, 3, 4, 6].into_iter().collect());
    }

    #[test]
    fn rust_measured_missed_honors_the_enforced_metrics() {
        let cov = coverage::RustPatchCoverage {
            regions: vec![(1, 1, true), (5, 6, false)],
        };
        let with_regions = RustThresholds {
            regions: Some(100),
            lines: 100,
            functions: None,
            branch: None,
        };
        let (measured, missed) = rust_measured_missed(&cov, with_regions);
        assert_eq!(measured, [1, 5, 6].into_iter().collect());
        assert_eq!(missed, [5, 6].into_iter().collect());
        let lines_only = RustThresholds {
            regions: None,
            lines: 100,
            functions: None,
            branch: None,
        };
        let (_, missed_lines) = rust_measured_missed(&cov, lines_only);
        assert_eq!(missed_lines, [5, 6].into_iter().collect());
    }

    #[test]
    fn apply_line_exemptions_drops_listed_misses_from_the_line_set() {
        let detail = BTreeMap::from([(
            "shim.py".to_string(),
            (
                [1u64, 2, 3, 4].into_iter().collect::<BTreeSet<u64>>(),
                [2u64, 3, 4].into_iter().collect::<BTreeSet<u64>>(),
            ),
        )]);
        let line_set = apply_line_exemptions(&detail, &exempt(&[("shim.py", &[2, 3, 4])])).unwrap();
        assert_eq!(line_set["shim.py"], [1].into_iter().collect());
    }

    #[test]
    fn apply_line_exemptions_rejects_a_covered_listed_line() {
        let detail = BTreeMap::from([(
            "shim.py".to_string(),
            (
                [1u64, 2].into_iter().collect::<BTreeSet<u64>>(),
                [2u64].into_iter().collect::<BTreeSet<u64>>(),
            ),
        )]);
        let err = apply_line_exemptions(&detail, &exempt(&[("shim.py", &[1, 2])])).unwrap_err();
        assert!(
            err.to_string().contains("uncovered lines") && err.to_string().contains("shim.py:1"),
            "got: {err}"
        );
    }

    #[test]
    fn apply_line_exemptions_rejects_an_unmeasured_listed_line() {
        let detail = BTreeMap::from([(
            "w.py".to_string(),
            (
                [2u64].into_iter().collect::<BTreeSet<u64>>(),
                [2u64].into_iter().collect::<BTreeSet<u64>>(),
            ),
        )]);
        let err = apply_line_exemptions(&detail, &exempt(&[("w.py", &[9])])).unwrap_err();
        assert!(err.to_string().contains("w.py:9"), "got: {err}");
    }

    #[test]
    fn floor_outcome_matches_the_whole_tree_message() {
        assert_eq!(floor_outcome(7, 7, 100), Outcome::Pass);
        let out = floor_outcome(7, 8, 100);
        assert!(
            matches!(&out, Outcome::Fail(m) if m == "coverage 87.50% is below the required 100%"),
            "got: {out:?}"
        );
        assert_eq!(floor_outcome(0, 0, 100), Outcome::Pass);
    }

    #[test]
    fn lift_exempt_lines_removes_exempt_lines_from_the_diff() {
        let mut changed = changed(&[("shim.py", &[1, 2, 3, 4]), ("core.py", &[5])]);
        lift_exempt_lines(
            &mut changed,
            &exempt(&[("shim.py", &[2, 3]), ("gone.py", &[9])]),
        );
        assert_eq!(changed["shim.py"], [1, 4].into_iter().collect());
        assert_eq!(changed["core.py"], [5].into_iter().collect());
    }

    #[test]
    fn kept_lines_drops_only_the_exempt_lines() {
        let measured = BTreeSet::from([3u64, 4, 5]);
        assert_eq!(
            kept_lines(&measured, Some(&BTreeSet::from([4u32]))),
            BTreeSet::from([3, 5])
        );
        assert_eq!(kept_lines(&measured, None), measured);
    }

    #[test]
    fn a_measured_line_too_large_for_a_u32_can_carry_no_exemption() {
        // An exemption is written as a u32, so a line number that cannot be one is kept
        // however the exemption set reads.
        let measured = BTreeSet::from([u64::from(u32::MAX) + 1]);
        assert_eq!(
            kept_lines(&measured, Some(&BTreeSet::from([u32::MAX]))),
            measured
        );
    }
}
