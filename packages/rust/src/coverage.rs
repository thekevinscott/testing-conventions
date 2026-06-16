//! Coverage rule (Python — issue #26; TypeScript — issue #31; exemptions — issue #32).
//!
//! Enforces the README's Coverage rule: a library's unit suite must meet the
//! configured floor, with test files excluded from the denominator. This module
//! is the deterministic core — given a parsed coverage report and the thresholds
//! from config, an `evaluate` function decides pass/fail. Producing the report
//! (shelling out to the language's coverage tool) is a thin layer on top, kept
//! separate so the guarantee is testable without that toolchain installed.
//!
//! Python (#26) uses coverage.py: a single total, branch coverage on. Given a
//! [`CoverageReport`] and [`Thresholds`], [`evaluate`] decides pass/fail, and
//! [`measure`] shells out to `coverage`. TypeScript (#31) is the twin: vitest
//! reports four independent metrics (lines / branches / functions / statements),
//! so it carries its own [`TypeScriptThresholds`], [`VitestReport`], and
//! [`evaluate_typescript`] / [`measure_typescript`] pair — sharing only the
//! [`Outcome`] type. Its subprocess layer shells out to `vitest`.
//!
//! Files exempted from coverage in config (issue #32) are omitted from the
//! denominator alongside the test files; the caller resolves them
//! ([`crate::config::resolve_exempt`]) and passes their paths to [`measure`] /
//! [`measure_typescript`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// Always omitted from the coverage denominator: colocated unit tests are the
/// suite, never a subject of it.
const TEST_OMIT: &str = "*_test.py";

/// Also always omitted: `conftest.py` holds pytest fixtures (test support), never
/// a coverage subject. `*conftest.py` matches it at any depth, mirroring the
/// `*_test.py` glob. (#112)
const SUPPORT_OMIT: &str = "*conftest.py";

/// The coverage floor to enforce, from a `[<language>].coverage` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thresholds {
    /// Minimum total coverage percent the unit suite must meet.
    pub fail_under: u8,
    /// Whether branch coverage must be measured (and folded into the total).
    pub branch: bool,
}

/// A coverage.py JSON report (`coverage json`), pared to what the checks need:
/// the `totals` (the floor and ratchet) and the per-file `files` block (patch
/// coverage, #132). Unmodeled fields (metadata, per-function/class data) are
/// ignored.
#[derive(Debug, Clone, Deserialize)]
pub struct CoverageReport {
    pub totals: Totals,
    /// Per-file line/branch detail, keyed by the path coverage.py reports
    /// (relative to the measured root). Additive: `#[serde(default)]`, so a report
    /// parsed for the floor alone (the inline tests) needs no `files`.
    #[serde(default)]
    pub files: BTreeMap<String, FileCoverage>,
}

/// Per-file coverage detail from a coverage.py report (one `files` entry) — what
/// patch coverage (#132) reads to decide whether a changed line is covered.
/// Unmodeled fields (the summary, per-function/class data) are ignored.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileCoverage {
    /// Executable lines the suite ran.
    #[serde(default)]
    pub executed_lines: Vec<u64>,
    /// Executable lines the suite never ran — an uncovered changed line is one of
    /// these.
    #[serde(default)]
    pub missing_lines: Vec<u64>,
    /// Lines excluded from coverage (e.g. `# pragma: no cover`); never a miss.
    #[serde(default)]
    pub excluded_lines: Vec<u64>,
    /// `[source_line, dest_line]` pairs for branches the suite never took; `dest`
    /// may be negative (a function / loop exit). Only the source line matters to
    /// patch coverage. Empty when branch coverage was off.
    #[serde(default)]
    pub missing_branches: Vec<Vec<i64>>,
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

// ---------------------------------------------------------------------------
// Non-regression ratchet (Python — #131, parent #46).
//
// Coverage can't regress: a committed `coverage-baseline.json` beside the
// measured tree records the last total per language, and a run that drops below
// the recorded baseline fails even when it still clears the configured floor.
// `read_baseline` loads the committed file (absent → no ratchet, backward
// compatible) and `evaluate_ratchet` is the pure comparison, mirroring
// `evaluate`'s float tolerance. The CLI runs both and fails if either does. The
// TypeScript/Rust arms and the explicit baseline-record step are later slices.
// ---------------------------------------------------------------------------

/// Where the committed coverage baseline lives, relative to the scanned root —
/// beside the measured tree, the way `--config` resolves alongside it.
pub const BASELINE_PATH: &str = "coverage-baseline.json";

/// The committed coverage baseline — the last recorded coverage per language.
/// Keyed by language so one file serves a multi-language repo; a language with
/// no entry has no ratchet (the floor still applies). The TypeScript and Rust
/// keys land with their slices.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    /// The recorded Python total, when present.
    #[serde(default)]
    pub python: Option<PythonBaseline>,
}

/// The recorded Python baseline: the last total percent the unit suite cleared.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonBaseline {
    /// The recorded total covered percent (line, plus branch when measured).
    pub percent_covered: f64,
}

/// Read the committed baseline at `root`/[`BASELINE_PATH`], or `None` when the
/// file is absent — an absent baseline means no ratchet, the same way a missing
/// config means nothing is exempt.
pub fn read_baseline(root: &Path) -> Result<Option<Baseline>> {
    let path = root.join(BASELINE_PATH);
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("reading coverage baseline `{}`", path.display()))?;
    let baseline = serde_json::from_str(&contents)
        .with_context(|| format!("parsing coverage baseline `{}`", path.display()))?;
    Ok(Some(baseline))
}

/// Decide whether `percent` regresses below `baseline`, the recorded total the
/// suite must not drop under. `None` (nothing recorded) is no ratchet →
/// [`Outcome::Pass`]. Carries the same hair of float tolerance as [`evaluate`] so
/// a percent that rounds to the baseline isn't failed by noise.
pub fn evaluate_ratchet(percent: f64, baseline: Option<f64>) -> Outcome {
    match baseline {
        Some(required) if percent + 1e-9 < required => Outcome::Fail(format!(
            "coverage {percent:.2}% regressed below the recorded baseline {required:.2}%"
        )),
        _ => Outcome::Pass,
    }
}

/// Run the unit suite under coverage.py in `root` and check it against
/// `thresholds`.
///
/// Shells out to `coverage run --branch` (omitting `*_test.py` and every path in
/// `omit` from the denominator) then `coverage json`, and evaluates the report.
/// `omit` holds the `coverage`-rule exemptions resolved from config, as
/// `root`-relative paths. The `coverage` CLI — with `pytest` importable — must be
/// on `PATH`.
pub fn measure(root: &Path, thresholds: Thresholds, omit: &[String]) -> Result<Outcome> {
    Ok(evaluate(&measure_report(root, omit)?, thresholds))
}

/// Run the Python unit suite under coverage.py in `root` and return the parsed
/// report — the totals the floor ([`evaluate`]) and the ratchet
/// ([`evaluate_ratchet`]) both read. `omit` is as in [`measure`].
pub fn measure_report(root: &Path, omit: &[String]) -> Result<CoverageReport> {
    run_coverage(root, omit, false)
}

/// Run the Python unit suite under coverage.py in `root` with **every** source
/// under `root` measured (`coverage run --source=.`) and return the parsed report
/// — so an untested source shows in the `files` block as wholly uncovered rather
/// than vanishing. The per-file detail is what patch coverage (#132) reads; `omit`
/// is as in [`measure`] (an exempt file stays out of the run, so its changed
/// lines are lifted).
pub fn measure_patch_report(root: &Path, omit: &[String]) -> Result<CoverageReport> {
    run_coverage(root, omit, true)
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
///
/// `include_all_sources` adds `--source=.` so coverage measures every source
/// under `root` — even one no test imports, which then appears in the `files`
/// block as wholly uncovered. The floor passes `false` (measuring only imported
/// files, so its total is unchanged); patch coverage passes `true`.
fn run_coverage(root: &Path, omit: &[String], include_all_sources: bool) -> Result<CoverageReport> {
    let data = DataFile::new();
    let omit = build_omit(omit);

    // Branch coverage on; measure the sources in `root` with the test files —
    // and any `coverage`-waived files — omitted from the denominator. Byte-code
    // and the pytest cache are suppressed so the scanned tree stays pristine.
    let mut command = Command::new("coverage");
    command
        .current_dir(root)
        .args(["run", "--branch"])
        .arg(format!("--omit={omit}"));
    if include_all_sources {
        command.arg("--source=.");
    }
    let run = command
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

/// The single comma-joined `--omit` value for the coverage run: always the test
/// glob `*_test.py` and the support glob `*conftest.py`, plus every
/// `coverage`-exempt path from config. (coverage.py takes one `--omit` — repeated
/// flags don't accumulate, so the patterns must be joined.) An exempt file leaves
/// the denominator with its reason recorded in config — an auditable omission, not
/// a silent ignore-glob.
fn build_omit(omit: &[String]) -> String {
    [TEST_OMIT.to_string(), SUPPORT_OMIT.to_string()]
        .into_iter()
        .chain(omit.iter().cloned())
        .collect::<Vec<_>>()
        .join(",")
}

// ---------------------------------------------------------------------------
// TypeScript (vitest) — issue #31.
//
// The TypeScript twin of the Python rule above. vitest reports four independent
// metrics rather than Python's single total-plus-branch, so it carries its own
// thresholds, report shape, and evaluate/measure pair; only `Outcome` is shared.
// The split is the same: a pure `evaluate_typescript` over a parsed json-summary
// report, and a thin `measure_typescript` that shells out to vitest to produce
// one — so the enforcement core is testable without a Node toolchain.
// ---------------------------------------------------------------------------

/// What vitest measures: every TypeScript source under the scanned root. The
/// braces are a vitest (picomatch) glob, expanded by vitest, not the shell.
const TS_INCLUDE: &str = "**/*.{ts,tsx,mts,cts}";
/// Always excluded from the denominator: the colocated unit tests are the suite,
/// never a subject of it (`*.test.*`), and declaration files carry no runtime
/// code (`*.d.ts` / `*.d.mts` / `*.d.cts`).
const TS_TEST_EXCLUDE: &str = "**/*.test.*";
const TS_DECL_EXCLUDE: &str = "**/*.d.{ts,mts,cts}";

/// The four vitest coverage floors, from a `[typescript].coverage` table. Each
/// is an independent percent the unit suite must meet — vitest measures all four.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeScriptThresholds {
    pub lines: u8,
    pub branches: u8,
    pub functions: u8,
    pub statements: u8,
}

/// A vitest `coverage-summary.json` report, pared to the `total` block the check
/// needs. Per-file entries and unmodeled fields are ignored.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct VitestReport {
    pub total: VitestTotals,
}

/// The `total` block of a vitest json-summary report — the four metrics this
/// rule enforces. vitest also emits `branchesTrue`, which the check ignores.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct VitestTotals {
    pub lines: VitestMetric,
    pub branches: VitestMetric,
    pub functions: VitestMetric,
    pub statements: VitestMetric,
}

/// One metric's totals from a vitest json-summary block, pared to what the check
/// needs: the covered percent and the denominator size.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct VitestMetric {
    /// Percent covered — `None` when nothing was measured, which vitest writes as
    /// the string `"Unknown"` (and `total` is then `0`).
    #[serde(deserialize_with = "deserialize_pct")]
    pub pct: Option<f64>,
    /// Size of the denominator (statements/branches/functions/lines counted).
    pub total: u64,
}

/// Deserialize a json-summary `pct`: a number for a measured metric (vitest
/// emits whole percents as JSON integers and fractional ones as floats), or the
/// string `"Unknown"` (→ `None`) when the denominator is empty.
fn deserialize_pct<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct PctVisitor;
    impl serde::de::Visitor<'_> for PctVisitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a coverage percent number or the string \"Unknown\"")
        }

        fn visit_f64<E>(self, value: f64) -> std::result::Result<Self::Value, E> {
            Ok(Some(value))
        }

        // serde_json hands a whole-number percent (e.g. `100`) to `visit_u64`;
        // percents are never negative, so `visit_i64` is not needed.
        fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E> {
            Ok(Some(value as f64))
        }

        // Any non-numeric percent (vitest writes the literal "Unknown") means the
        // metric had nothing to measure.
        fn visit_str<E>(self, _value: &str) -> std::result::Result<Self::Value, E> {
            Ok(None)
        }
    }
    deserializer.deserialize_any(PctVisitor)
}

/// Parse a vitest json-summary report (`coverage-summary.json`).
pub fn parse_vitest_report(json: &str) -> Result<VitestReport> {
    serde_json::from_str(json).context("parsing vitest coverage-summary JSON report")
}

/// Decide whether `report` meets every threshold in `thresholds`.
///
/// Fails when the run measured no code at all (an empty line denominator — a
/// wrong path, or a suite that touched nothing — is never a silent pass),
/// otherwise checks each of the four metrics and fails listing every one below
/// its floor. A metric whose denominator is empty *amid* a non-empty run (e.g.
/// branch-free code measured alongside real code) has nothing to miss and is
/// vacuously satisfied.
pub fn evaluate_typescript(report: &VitestReport, thresholds: TypeScriptThresholds) -> Outcome {
    let total = &report.total;
    // Vacuous-run guard: every source file has lines, so a zero line-denominator
    // means nothing was measured — a misconfigured run (wrong path, or every file
    // excluded), failed rather than passed on an empty measurement.
    if total.lines.total == 0 {
        return Outcome::Fail(
            "the unit suite measured no code — check the path and that the suite runs".to_string(),
        );
    }
    let checks = [
        ("lines", total.lines, thresholds.lines),
        ("branches", total.branches, thresholds.branches),
        ("functions", total.functions, thresholds.functions),
        ("statements", total.statements, thresholds.statements),
    ];
    let mut shortfalls = Vec::new();
    for (name, metric, required) in checks {
        // A metric with an empty denominator (e.g. branch-free code) has nothing
        // to cover and is vacuously full; a measured one compares its percent.
        let actual = metric.pct.unwrap_or(100.0);
        // A hair of tolerance so a percent that rounds to the floor isn't failed
        // by float noise (matches the Python path).
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

/// Run the unit suite under vitest coverage in `root` and check it against
/// `thresholds`.
///
/// Shells out to `npx vitest run` with v8 coverage and the json-summary reporter,
/// excluding `*.test.*`, declaration files, and every path in `exclude` from the
/// denominator, then evaluates the report. `exclude` holds the `coverage`-rule
/// exemptions resolved from config, as `root`-relative paths. `npx` resolves the
/// project-local `vitest`, so it and `@vitest/coverage-v8` must be installed
/// under `root`.
pub fn measure_typescript(
    root: &Path,
    thresholds: TypeScriptThresholds,
    exclude: &[String],
) -> Result<Outcome> {
    let report = run_vitest(root, exclude)?;
    Ok(evaluate_typescript(&report, thresholds))
}

/// A vitest reports directory under the temp dir — unique per call (so checks
/// running in parallel don't collide) and removed on drop (so the report never
/// leaks into the scanned tree). vitest writes `coverage-summary.json` here.
struct ReportDir(PathBuf);

impl ReportDir {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "testing-conventions-vitest-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        );
        ReportDir(std::env::temp_dir().join(name))
    }

    /// The json-summary file vitest writes under this directory.
    fn summary(&self) -> PathBuf {
        self.0.join("coverage-summary.json")
    }
}

impl Drop for ReportDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Run vitest over the unit suite in `root` and return the parsed report.
fn run_vitest(root: &Path, exclude: &[String]) -> Result<VitestReport> {
    let reports = ReportDir::new();

    // v8 coverage with the json-summary reporter, written to an out-of-tree temp
    // dir so the scanned tree stays pristine. `include` scopes measurement to the
    // sources under `root`; the test glob, declaration files, and the config
    // exemptions are excluded from the denominator. `all=true` counts source files
    // the suite never imported, so an untested file lowers coverage rather than
    // vanishing. `--no-cache` keeps vitest from writing a cache into the tree.
    let mut command = Command::new("npx");
    command
        .current_dir(root)
        .args(["--yes", "vitest", "run", "--no-cache"])
        .args([
            "--coverage.enabled",
            "--coverage.provider=v8",
            "--coverage.reporter=json-summary",
            "--coverage.all=true",
        ])
        .arg(format!(
            "--coverage.reportsDirectory={}",
            reports.0.display()
        ))
        .arg(format!("--coverage.include={TS_INCLUDE}"))
        .arg(format!("--coverage.exclude={TS_TEST_EXCLUDE}"))
        .arg(format!("--coverage.exclude={TS_DECL_EXCLUDE}"));
    for path in exclude {
        command.arg(format!("--coverage.exclude={path}"));
    }
    // CI=1 keeps vitest non-interactive (no watch prompt, plain output).
    let run = command.env("CI", "1").output().context(
        "running `npx vitest run --coverage` (are vitest and @vitest/coverage-v8 installed?)",
    )?;
    if !run.status.success() {
        bail!(
            "the unit suite did not run cleanly under vitest in `{}`:\n{}{}",
            root.display(),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr),
        );
    }

    let summary = reports.summary();
    let json = std::fs::read_to_string(&summary).with_context(|| {
        format!(
            "reading vitest coverage summary `{}` (did the run produce a json-summary report?)",
            summary.display()
        )
    })?;
    parse_vitest_report(&json)
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
            files: BTreeMap::new(),
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

    #[test]
    fn parses_the_per_file_block_for_patch_coverage() {
        // A realistic `coverage json` shape: a `files` map carrying the per-file
        // missing lines and `[src, dst]` branch pairs patch coverage (#132) reads.
        let json = r#"{
            "files": {
                "widget.py": {
                    "executed_lines": [1, 2, 3, 4, 6],
                    "summary": {"percent_covered": 85.0},
                    "missing_lines": [5],
                    "excluded_lines": [],
                    "missing_branches": [[4, 5]]
                }
            },
            "totals": {"percent_covered": 85.0, "num_branches": 4}
        }"#;
        let report = parse_report(json).expect("valid coverage.py json with files");
        let widget = report.files.get("widget.py").expect("widget.py is present");
        assert_eq!(widget.missing_lines, vec![5]);
        assert_eq!(widget.missing_branches, vec![vec![4, 5]]);
        // The floor still reads totals from the same report.
        assert_eq!(report.totals.percent_covered, 85.0);
    }

    #[test]
    fn a_report_without_a_files_block_parses_with_an_empty_map() {
        // The floor/ratchet path parses totals only; `files` defaults to empty.
        let report = parse_report(r#"{"totals":{"percent_covered":100.0,"num_branches":2}}"#)
            .expect("valid coverage.py json");
        assert!(report.files.is_empty());
    }

    #[test]
    fn omit_is_the_test_and_support_globs_when_nothing_is_exempt() {
        assert_eq!(build_omit(&[]), "*_test.py,*conftest.py");
    }

    #[test]
    fn omit_folds_in_the_exempt_paths_after_the_test_glob() {
        // The caller passes already-resolved, sorted, `root`-relative paths.
        let exempt = vec!["pkg/gen.py".to_string(), "shim.py".to_string()];
        assert_eq!(
            build_omit(&exempt),
            "*_test.py,*conftest.py,pkg/gen.py,shim.py"
        );
    }

    // --- Non-regression ratchet (#131) ---

    #[test]
    fn ratchet_passes_when_coverage_holds_at_the_baseline() {
        assert_eq!(evaluate_ratchet(100.0, Some(100.0)), Outcome::Pass);
    }

    #[test]
    fn ratchet_passes_when_coverage_improves_over_the_baseline() {
        assert_eq!(evaluate_ratchet(92.0, Some(85.0)), Outcome::Pass);
    }

    #[test]
    fn ratchet_fails_on_a_drop_below_the_baseline() {
        assert!(matches!(
            evaluate_ratchet(86.0, Some(90.0)),
            Outcome::Fail(message) if message.contains("regressed") && message.contains("90")
        ));
    }

    #[test]
    fn ratchet_is_vacuous_without_a_recorded_baseline() {
        assert_eq!(evaluate_ratchet(10.0, None), Outcome::Pass);
    }

    #[test]
    fn ratchet_tolerates_float_noise_at_the_baseline() {
        assert_eq!(evaluate_ratchet(99.999_999_999, Some(100.0)), Outcome::Pass);
    }

    static BASELINE_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// A throwaway directory under the temp dir, removed on drop — for the
    /// `read_baseline` file cases.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!(
                "tc-baseline-{}-{}",
                std::process::id(),
                BASELINE_COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn read_baseline_is_none_when_the_file_is_absent() {
        let dir = TempDir::new();
        assert!(read_baseline(&dir.0).unwrap().is_none());
    }

    #[test]
    fn read_baseline_parses_the_recorded_python_total() {
        let dir = TempDir::new();
        std::fs::write(
            dir.0.join(BASELINE_PATH),
            r#"{"python":{"percent_covered":91.5}}"#,
        )
        .unwrap();
        let baseline = read_baseline(&dir.0)
            .unwrap()
            .expect("a baseline file is present");
        assert_eq!(baseline.python.unwrap().percent_covered, 91.5);
    }

    #[test]
    fn read_baseline_errors_on_a_malformed_file() {
        let dir = TempDir::new();
        std::fs::write(dir.0.join(BASELINE_PATH), "{ not json").unwrap();
        assert!(read_baseline(&dir.0).is_err());
    }

    // --- TypeScript (vitest) — issue #31 ---

    fn metric(pct: f64) -> VitestMetric {
        VitestMetric {
            pct: Some(pct),
            total: 10,
        }
    }

    fn ts_report(lines: f64, branches: f64, functions: f64, statements: f64) -> VitestReport {
        VitestReport {
            total: VitestTotals {
                lines: metric(lines),
                branches: metric(branches),
                functions: metric(functions),
                statements: metric(statements),
            },
        }
    }

    const TS_FULL: TypeScriptThresholds = TypeScriptThresholds {
        lines: 100,
        branches: 100,
        functions: 100,
        statements: 100,
    };
    const TS_MID: TypeScriptThresholds = TypeScriptThresholds {
        lines: 80,
        branches: 75,
        functions: 80,
        statements: 80,
    };

    #[test]
    fn typescript_passes_when_every_metric_meets_its_floor() {
        assert_eq!(
            evaluate_typescript(&ts_report(100.0, 100.0, 100.0, 100.0), TS_FULL),
            Outcome::Pass
        );
    }

    #[test]
    fn typescript_fails_on_the_one_metric_below_its_floor() {
        // 100% lines but only 66.66% branches (the `below` fixture's shape): the
        // branch floor catches what line coverage misses — and only `branches` is
        // named as a shortfall, not the metrics that met their floor.
        let outcome = evaluate_typescript(&ts_report(100.0, 66.66, 100.0, 100.0), TS_MID);
        assert!(
            matches!(&outcome, Outcome::Fail(message) if message.contains("branches") && !message.contains("lines")),
            "got: {outcome:?}"
        );
    }

    #[test]
    fn typescript_fail_message_names_every_metric_below() {
        let outcome = evaluate_typescript(&ts_report(70.0, 70.0, 70.0, 70.0), TS_MID);
        assert!(
            matches!(&outcome, Outcome::Fail(message)
                if message.contains("lines")
                    && message.contains("branches")
                    && message.contains("functions")
                    && message.contains("statements")),
            "got: {outcome:?}"
        );
    }

    #[test]
    fn typescript_tolerates_float_noise_at_the_floor() {
        // A percent a hair under the floor from rounding still passes.
        assert_eq!(
            evaluate_typescript(&ts_report(99.999_999_999, 100.0, 100.0, 100.0), TS_FULL),
            Outcome::Pass
        );
    }

    #[test]
    fn typescript_empty_denominator_metric_is_vacuously_satisfied() {
        // Branch-free code measured alongside real code: branches has nothing to
        // cover (pct "Unknown") but lines/etc. are real and pass → overall pass.
        let report = VitestReport {
            total: VitestTotals {
                lines: metric(100.0),
                branches: VitestMetric {
                    pct: None,
                    total: 0,
                },
                functions: metric(100.0),
                statements: metric(100.0),
            },
        };
        assert_eq!(evaluate_typescript(&report, TS_FULL), Outcome::Pass);
    }

    #[test]
    fn typescript_fails_a_vacuous_run_that_measured_no_code() {
        // No lines in the denominator (everything excluded, or a wrong path): a
        // vacuous run is a failure, never a silent pass.
        let nothing = VitestMetric {
            pct: None,
            total: 0,
        };
        let report = VitestReport {
            total: VitestTotals {
                lines: nothing,
                branches: nothing,
                functions: nothing,
                statements: nothing,
            },
        };
        let outcome = evaluate_typescript(&report, TS_MID);
        assert!(
            matches!(&outcome, Outcome::Fail(message) if message.contains("measured no code")),
            "got: {outcome:?}"
        );
    }

    #[test]
    fn parses_a_vitest_summary_report() {
        // A realistic `coverage-summary.json`: the four metrics plus the
        // `branchesTrue` block and a per-file entry the check ignores.
        let json = r#"{
            "total": {
                "lines": {"total": 5, "covered": 4, "skipped": 0, "pct": 80},
                "statements": {"total": 5, "covered": 4, "skipped": 0, "pct": 80},
                "functions": {"total": 2, "covered": 2, "skipped": 0, "pct": 100},
                "branches": {"total": 3, "covered": 2, "skipped": 0, "pct": 66.66},
                "branchesTrue": {"total": 0, "covered": 0, "skipped": 0, "pct": "Unknown"}
            },
            "/abs/widget.ts": {
                "lines": {"total": 5, "covered": 4, "skipped": 0, "pct": 80}
            }
        }"#;
        let report = parse_vitest_report(json).expect("valid vitest json-summary");
        // A whole-number percent (`visit_u64`) and a fractional one (`visit_f64`).
        assert_eq!(report.total.lines.pct, Some(80.0));
        assert_eq!(report.total.branches.pct, Some(66.66));
        assert_eq!(report.total.functions.total, 2);
    }

    #[test]
    fn parses_an_unknown_pct_as_unmeasured() {
        let json = r#"{"total": {
            "lines": {"total": 0, "covered": 0, "skipped": 0, "pct": "Unknown"},
            "statements": {"total": 0, "covered": 0, "skipped": 0, "pct": "Unknown"},
            "functions": {"total": 0, "covered": 0, "skipped": 0, "pct": "Unknown"},
            "branches": {"total": 0, "covered": 0, "skipped": 0, "pct": "Unknown"}
        }}"#;
        let report = parse_vitest_report(json).expect("valid vitest json-summary");
        assert_eq!(report.total.lines.pct, None);
        assert_eq!(report.total.lines.total, 0);
    }

    #[test]
    fn a_pct_that_is_neither_number_nor_string_is_a_parse_error() {
        // vitest only ever writes a number or "Unknown"; anything else (here a
        // bool) is a malformed report, surfaced as an error rather than guessed.
        let json = r#"{"total":{
            "lines": {"total": 1, "covered": 1, "skipped": 0, "pct": true},
            "statements": {"total": 1, "covered": 1, "skipped": 0, "pct": 100},
            "functions": {"total": 1, "covered": 1, "skipped": 0, "pct": 100},
            "branches": {"total": 1, "covered": 1, "skipped": 0, "pct": 100}
        }}"#;
        assert!(parse_vitest_report(json).is_err());
    }
}
