//! Coverage rule: the unit suite must clear the configured floor, with test files
//! and the config's exempt paths out of the denominator. Each language pairs a pure
//! `evaluate*` over a parsed report with a `measure*` that shells out to its tool.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// Omitted from the denominator: colocated unit tests are the suite, not a subject.
const TEST_OMIT: &str = "*_test.py";

/// Omitted too: `conftest.py` is pytest fixtures — test support, not a subject.
const SUPPORT_OMIT: &str = "*conftest.py";

/// The coverage floor to enforce, from a `[<language>].coverage` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thresholds {
    /// Minimum total coverage percent the unit suite must meet.
    pub fail_under: u8,
    /// Whether branch coverage must be measured (and folded into the total).
    pub branch: bool,
}

/// A coverage.py JSON report (`coverage json`), pared to the `totals` the floor reads
/// and the per-file `files` block the diff-scoped floor reads.
#[derive(Debug, Clone, Deserialize)]
pub struct CoverageReport {
    pub totals: Totals,
    /// Per-file line/branch detail, keyed by the path coverage.py reports (relative
    /// to the measured root).
    #[serde(default)]
    pub files: BTreeMap<String, FileCoverage>,
}

/// One `files` entry of a coverage.py report — what patch coverage reads to decide
/// whether a changed line is covered.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileCoverage {
    /// Executable lines the suite ran.
    #[serde(default)]
    pub executed_lines: Vec<u64>,
    /// Executable lines the suite never ran — an uncovered changed line is one of these.
    #[serde(default)]
    pub missing_lines: Vec<u64>,
    /// Lines excluded from coverage (e.g. `# pragma: no cover`); never a miss.
    #[serde(default)]
    pub excluded_lines: Vec<u64>,
    /// `[source, dest]` pairs for branches never taken; only `source` matters, and
    /// `dest` may be negative (a function / loop exit). Empty without `--branch`.
    #[serde(default)]
    pub missing_branches: Vec<Vec<i64>>,
    /// `[source, dest]` pairs for branches the suite took; with `missing_branches`
    /// they give branch coverage over the changed lines. Empty without `--branch`.
    #[serde(default)]
    pub executed_branches: Vec<Vec<i64>>,
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
    Pass,
    /// The message explains why (actual vs. required).
    Fail(String),
}

/// Parse a coverage.py JSON report (the output of `coverage json`).
pub fn parse_report(json: &str) -> Result<CoverageReport> {
    serde_json::from_str(json).context("parsing coverage.py JSON report")
}

/// Whether `report` meets `thresholds`. Branch coverage required but no branches
/// measured is a misconfigured run, and fails.
pub fn evaluate(report: &CoverageReport, thresholds: Thresholds) -> Outcome {
    if thresholds.branch && report.totals.num_branches == 0 {
        return Outcome::Fail(
            "branch coverage is required but the report measured no branches".to_string(),
        );
    }
    let actual = report.totals.percent_covered;
    let required = f64::from(thresholds.fail_under);
    // Tolerance so a report that rounds to the floor isn't failed by float noise.
    if actual + 1e-9 >= required {
        Outcome::Pass
    } else {
        Outcome::Fail(format!(
            "coverage {actual:.2}% is below the required {}%",
            thresholds.fail_under
        ))
    }
}

/// Run the unit suite under coverage.py in `root` and check it against `thresholds`.
/// `omit` is the `coverage`-rule exemptions as `root`-relative paths. The `coverage`
/// CLI, with `pytest` importable, must be on `PATH`.
pub fn measure(root: &Path, thresholds: Thresholds, omit: &[String]) -> Result<Outcome> {
    let report = run_coverage(root, omit, false)?;
    Ok(evaluate(&report, thresholds))
}

/// Run the Python unit suite with **every** source under `root` measured
/// (`--source=.`), so an untested source shows in `files` as wholly uncovered rather
/// than vanishing. `omit` is as in [`measure`].
pub fn measure_patch_report(root: &Path, omit: &[String]) -> Result<CoverageReport> {
    run_coverage(root, omit, true)
}

/// Like [`measure_patch_report`], but measuring only the files the suite imports,
/// exactly as [`measure`] does — the line-scoped exemption path recomputes the floor
/// over that same file set. `omit` is as in [`measure`].
pub fn measure_report(root: &Path, omit: &[String]) -> Result<CoverageReport> {
    run_coverage(root, omit, false)
}

/// A coverage.py data file under the temp dir — unique per call so parallel checks
/// don't collide, and removed on drop so nothing leaks into the scanned tree.
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
/// `include_all_sources` adds `--source=.`, so a source no test imports still appears
/// in `files` as wholly uncovered. The floor passes `false`; patch coverage `true`.
fn run_coverage(root: &Path, omit: &[String], include_all_sources: bool) -> Result<CoverageReport> {
    let data = DataFile::new();
    let omit = build_omit(omit);

    // Byte-code and the pytest cache are suppressed so the scanned tree stays pristine.
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

/// The single comma-joined `--omit` for the run: the test and support globs plus every
/// `coverage`-exempt path. coverage.py takes one `--omit` — repeated flags don't
/// accumulate, so the patterns must be joined.
fn build_omit(omit: &[String]) -> String {
    [TEST_OMIT.to_string(), SUPPORT_OMIT.to_string()]
        .into_iter()
        .chain(omit.iter().cloned())
        .collect::<Vec<_>>()
        .join(",")
}

/// What vitest measures: every TypeScript source under the scanned root. The
/// braces are a vitest (picomatch) glob, expanded by vitest, not the shell.
const TS_INCLUDE: &str = "**/*.{ts,tsx,mts,cts}";

/// The installed vitest's own default coverage excludes, resolved live via Node.
/// Passing *any* `--coverage.exclude` replaces vitest's built-in list rather than
/// extending it, so the defaults must be resolved and passed back explicitly.
fn vitest_default_excludes(root: &Path) -> Result<Vec<String>> {
    let run = Command::new("node")
        .current_dir(root)
        .args([
            "-e",
            "process.stdout.write(JSON.stringify(require('vitest/config').coverageConfigDefaults.exclude))",
        ])
        .output()
        .context("resolving vitest's default coverage excludes via node")?;
    if !run.status.success() {
        bail!(
            "could not resolve vitest's default coverage excludes in `{}`. The rule runs the \
             project's own vitest via `npx --no-install` and never downloads it, so `vitest` \
             must be installed in the project. node output:\n{}{}",
            root.display(),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr),
        );
    }
    parse_default_excludes(&run.stdout)
}

/// The exclude patterns node printed, parsed and pared to the passable ones.
fn parse_default_excludes(stdout: &[u8]) -> Result<Vec<String>> {
    let excludes: Vec<String> = serde_json::from_slice(stdout).with_context(|| {
        format!(
            "vitest's default coverage excludes were not a JSON string array — got: {}",
            String::from_utf8_lossy(stdout)
        )
    })?;
    // A few of vitest's default patterns embed a literal NUL (its virtual-module
    // markers, e.g. `**/\0*`), which can't be passed as a process argument at all.
    Ok(excludes.into_iter().filter(|p| !p.contains('\0')).collect())
}

/// The four vitest coverage floors, from a `[typescript].coverage` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeScriptThresholds {
    pub lines: u8,
    pub branches: u8,
    pub functions: u8,
    pub statements: u8,
}

/// A vitest `coverage-summary.json` report, pared to the `total` block.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct VitestReport {
    pub total: VitestTotals,
}

/// The `total` block of a vitest json-summary report — the four metrics enforced.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct VitestTotals {
    pub lines: VitestMetric,
    pub branches: VitestMetric,
    pub functions: VitestMetric,
    pub statements: VitestMetric,
}

/// One metric's totals from a vitest json-summary block.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct VitestMetric {
    /// Percent covered — `None` when nothing was measured, which vitest writes as
    /// the string `"Unknown"` (and `total` is then `0`).
    #[serde(deserialize_with = "deserialize_pct")]
    pub pct: Option<f64>,
    /// Size of the denominator (statements/branches/functions/lines counted).
    pub total: u64,
}

/// A json-summary `pct`: a number for a measured metric, or the string `"Unknown"`
/// (→ `None`) when the denominator is empty.
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

        // serde_json routes a whole-number percent here; percents are never negative.
        fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E> {
            Ok(Some(value as f64))
        }

        // vitest writes the literal "Unknown" when the metric had nothing to measure.
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

/// Whether `report` meets every threshold. A run that measured no code at all fails
/// rather than passing vacuously; one metric with an empty denominator amid a
/// non-empty run has nothing to miss and is vacuously satisfied.
pub fn evaluate_typescript(report: &VitestReport, thresholds: TypeScriptThresholds) -> Outcome {
    let total = &report.total;
    // Every source file has lines, so a zero denominator means nothing was measured.
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
        // An empty denominator (branch-free code) has nothing to cover — vacuously full.
        let actual = metric.pct.unwrap_or(100.0);
        // Tolerance so a percent that rounds to the floor isn't failed by float noise.
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
/// `thresholds`. `exclude` is the `coverage`-rule exemptions as `root`-relative paths;
/// `npx` resolves the project-local `vitest` and `@vitest/coverage-v8`.
pub fn measure_typescript(
    root: &Path,
    thresholds: TypeScriptThresholds,
    exclude: &[String],
) -> Result<Outcome> {
    let report = run_vitest(root, exclude)?;
    Ok(evaluate_typescript(&report, thresholds))
}

/// A vitest reports directory under the temp dir — unique per call so parallel checks
/// don't collide, and removed on drop so nothing leaks into the scanned tree.
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
}

impl Drop for ReportDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Run vitest over the unit suite in `root` and return the parsed floor report.
fn run_vitest(root: &Path, exclude: &[String]) -> Result<VitestReport> {
    let json = run_vitest_coverage(root, exclude, "json-summary", "coverage-summary.json")?;
    parse_vitest_report(&json)
}

/// Run vitest coverage over the unit suite in `root` and return the contents of the
/// `report_file` the `reporter` wrote. `all=true` counts source files the suite never
/// imported, so an untested file is measured rather than vanishing.
fn run_vitest_coverage(
    root: &Path,
    exclude: &[String],
    reporter: &str,
    report_file: &str,
) -> Result<String> {
    let reports = ReportDir::new();

    let mut command = Command::new("npx");
    command
        .current_dir(root)
        // `--no-install`, never `--yes`: with `--yes` a missing vitest is silently
        // downloaded, where the other arms fail clean on a missing binary.
        .args(["--no-install", "vitest", "run", "--no-cache"])
        .args(["--coverage.enabled", "--coverage.provider=v8"])
        .arg(format!("--coverage.reporter={reporter}"))
        .arg("--coverage.all=true")
        .arg(format!(
            "--coverage.reportsDirectory={}",
            reports.0.display()
        ))
        .arg(format!("--coverage.include={TS_INCLUDE}"))
        // A consumer config's own `coverage.thresholds` neither decide the gate's exit
        // nor rewrite the config file — `autoUpdate` never writes during a gate run.
        .args([
            "--coverage.thresholds.lines=0",
            "--coverage.thresholds.branches=0",
            "--coverage.thresholds.functions=0",
            "--coverage.thresholds.statements=0",
            "--coverage.thresholds.autoUpdate=false",
        ]);
    for path in vitest_default_excludes(root)?.iter().chain(exclude) {
        command.arg(format!("--coverage.exclude={path}"));
    }
    // CI=1 keeps vitest non-interactive (no watch prompt, plain output).
    let run = command
        .env("CI", "1")
        .output()
        .context("running `npx --no-install vitest run --coverage`")?;
    if !run.status.success() {
        bail!(
            "the unit suite did not run cleanly under vitest in `{}`. The rule runs the \
             project's own vitest via `npx --no-install` and never downloads it, so `vitest` \
             and `@vitest/coverage-v8` must be installed in the project. vitest output:\n{}{}",
            root.display(),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr),
        );
    }

    read_vitest_report(&reports.0.join(report_file), reporter)
}

/// The report the vitest run wrote, read back for parsing.
fn read_vitest_report(path: &Path, reporter: &str) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| {
        format!(
            "reading vitest coverage report `{}` (did the run produce a {reporter} report?)",
            path.display()
        )
    })
}

/// One file's entry in a vitest v8 `coverage-final.json` (Istanbul) report, pared to
/// the statement / branch / function maps and their hit counts.
#[derive(Debug, Clone, Deserialize)]
struct IstanbulFile {
    /// Statement id → source span; a `0` count in `s` means its lines are uncovered.
    #[serde(rename = "statementMap", default)]
    statement_map: BTreeMap<String, IstanbulSpan>,
    /// Statement id → execution count.
    #[serde(default)]
    s: BTreeMap<String, u64>,
    /// Branch id → location; a `0` among its `b` counts means a path never taken.
    #[serde(rename = "branchMap", default)]
    branch_map: BTreeMap<String, IstanbulBranch>,
    /// Branch id → per-arm execution counts.
    #[serde(default)]
    b: BTreeMap<String, Vec<u64>>,
    /// Function id → declaration location; a `0` count in `f` means never called.
    #[serde(rename = "fnMap", default)]
    fn_map: BTreeMap<String, IstanbulFn>,
    /// Function id → execution count.
    #[serde(default)]
    f: BTreeMap<String, u64>,
}

/// A source span — only the 1-based line numbers matter to patch coverage.
#[derive(Debug, Clone, Deserialize)]
struct IstanbulSpan {
    start: IstanbulPos,
    end: IstanbulPos,
}

/// A position in a source span; the `column` is ignored.
#[derive(Debug, Clone, Deserialize)]
struct IstanbulPos {
    line: u64,
}

/// A branch entry — only `loc.start.line`, the branch's source line, matters.
#[derive(Debug, Clone, Deserialize)]
struct IstanbulBranch {
    loc: IstanbulSpan,
}

/// A function entry — only `decl.start.line` matters. vitest's v8 export shapes this
/// as `{"name":.., "decl":{"start":{"line":N,..},..}, ..}`.
#[derive(Debug, Clone, Deserialize)]
struct IstanbulFn {
    decl: IstanbulSpan,
}

/// Per-file detail from a vitest Istanbul report — the Istanbul maps reduced to the
/// tuples [`crate::patch_coverage::evaluate_patch_typescript`] restricts to the diff.
#[derive(Debug, Clone, Default)]
pub struct TsPatchCoverage {
    /// One per `statementMap` entry: `(start_line, end_line, covered)`. A statement
    /// counts toward the diff when any line it spans is changed.
    pub statements: Vec<(u64, u64, bool)>,
    /// One per branch **arm**: `(source_line, covered)`, the source line shared by
    /// every arm of a branch.
    pub branch_arms: Vec<(u64, bool)>,
    /// One per `fnMap` entry: `(decl_line, covered)`. A function counts toward the
    /// diff when its declaration line is changed.
    pub functions: Vec<(u64, bool)>,
}

/// Run the TypeScript unit suite under vitest and return the per-file detail for the
/// four metrics, keyed by the absolute path vitest reports. `exclude` is the
/// `coverage`-rule exemptions, dropped so an exempt file's changed lines are lifted.
pub fn measure_patch_typescript_detail(
    root: &Path,
    exclude: &[String],
) -> Result<BTreeMap<String, TsPatchCoverage>> {
    let json = run_vitest_coverage(root, exclude, "json", "coverage-final.json")?;
    istanbul_patch_detail(&json)
}

/// Pure: per-file [`TsPatchCoverage`] from a vitest v8 Istanbul report, keyed by the
/// absolute path vitest reports.
fn istanbul_patch_detail(json: &str) -> Result<BTreeMap<String, TsPatchCoverage>> {
    let files: BTreeMap<String, IstanbulFile> = serde_json::from_str(json)
        .context("parsing vitest coverage-final (Istanbul) JSON report")?;
    let mut out = BTreeMap::new();
    for (path, file) in files {
        let mut detail = TsPatchCoverage::default();
        for (id, span) in &file.statement_map {
            let covered = file.s.get(id).is_some_and(|&count| count > 0);
            detail
                .statements
                .push((span.start.line, span.end.line, covered));
        }
        // v8 models a branch as one arm (a `[count]` array) or several; one tuple per
        // arm either way.
        for (id, branch) in &file.branch_map {
            let line = branch.loc.start.line;
            if let Some(counts) = file.b.get(id) {
                for &count in counts {
                    detail.branch_arms.push((line, count > 0));
                }
            }
        }
        for (id, function) in &file.fn_map {
            let covered = file.f.get(id).is_some_and(|&count| count > 0);
            detail.functions.push((function.decl.start.line, covered));
        }
        out.insert(path, detail);
    }
    Ok(out)
}

/// The `cargo llvm-cov` coverage floors, from a `[rust].coverage` table. `lines` is
/// always enforced; the rest are opt-in, `None` skipping the check. A `branch` floor
/// adds `--branch`, which instruments only on a nightly toolchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RustThresholds {
    pub regions: Option<u8>,
    pub lines: u8,
    pub functions: Option<u8>,
    pub branch: Option<u8>,
}

/// A `cargo llvm-cov --json` export, pared to the totals the floor reads. A single
/// run produces one `data` entry.
#[derive(Debug, Clone, Deserialize)]
pub struct LlvmCovReport {
    pub data: Vec<LlvmCovData>,
}

/// One export entry — `--summary-only` omits everything but its `totals`.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LlvmCovData {
    pub totals: LlvmCovTotals,
}

/// The `totals` block of an llvm-cov export. `branches` is optional so an export from
/// a run without branch instrumentation still parses.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LlvmCovTotals {
    pub regions: LlvmCovMetric,
    pub lines: LlvmCovMetric,
    pub functions: LlvmCovMetric,
    #[serde(default)]
    pub branches: Option<LlvmCovMetric>,
}

/// One metric's totals from an llvm-cov export.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LlvmCovMetric {
    /// Size of the denominator (regions or lines counted).
    pub count: u64,
    pub covered: u64,
    pub percent: f64,
}

/// Parse a `cargo llvm-cov --json` export.
pub fn parse_llvm_cov_report(json: &str) -> Result<LlvmCovReport> {
    serde_json::from_str(json).context("parsing cargo llvm-cov JSON report")
}

/// Whether `report` meets its thresholds. A run that measured no regions at all — a
/// wrong path, or a crate that compiled nothing — fails rather than passing vacuously.
pub fn evaluate_rust(report: &LlvmCovReport, thresholds: RustThresholds) -> Outcome {
    let Some(totals) = report.data.first().map(|entry| &entry.totals) else {
        return Outcome::Fail("the cargo llvm-cov report contained no data".to_string());
    };
    // Every compiled crate has regions, so a zero denominator measured nothing.
    if totals.regions.count == 0 {
        return Outcome::Fail(
            "the unit suite measured no code — check the path and that the suite runs".to_string(),
        );
    }
    // The zero-config default floors lines only; the rest are opt-in.
    let mut checks: Vec<(&str, f64, u8)> = Vec::new();
    if let Some(regions) = thresholds.regions {
        checks.push(("regions", totals.regions.percent, regions));
    }
    checks.push(("lines", totals.lines.percent, thresholds.lines));
    if let Some(functions) = thresholds.functions {
        checks.push(("functions", totals.functions.percent, functions));
    }
    if let Some(branch) = thresholds.branch {
        // A failed instrumentation is a run error surfaced before this point, so a zero
        // branch denominator means the crate has no branch points — vacuously satisfied.
        if let Some(branches) = totals.branches.filter(|metric| metric.count > 0) {
            checks.push(("branches", branches.percent, branch));
        }
    }
    let mut shortfalls = Vec::new();
    for (name, actual, required) in checks {
        // Tolerance so a percent that rounds to the floor isn't failed by float noise.
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

/// Run the unit suite under `cargo llvm-cov` in `root` and check it against
/// `thresholds`. `ignore` is the `coverage`-rule exemptions as `root`-relative paths;
/// `features` the `[rust] features` list to enable. `cargo-llvm-cov` must be installed.
pub fn measure_rust(
    root: &Path,
    thresholds: RustThresholds,
    ignore: &[String],
    features: &[String],
) -> Result<Outcome> {
    let report = run_llvm_cov(root, ignore, features, thresholds.branch.is_some())?;
    Ok(evaluate_rust(&report, thresholds))
}

/// A `CARGO_TARGET_DIR` under the temp dir — unique per call so parallel checks don't
/// collide, and removed on drop so the build never leaks into the scanned tree.
struct TargetDir(PathBuf);

impl TargetDir {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "testing-conventions-llvm-cov-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        );
        TargetDir(std::env::temp_dir().join(name))
    }
}

impl Drop for TargetDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The parsed `--summary-only` export — the totals the floor checks. `branch` adds
/// `--branch` for a configured branch floor.
fn run_llvm_cov(
    root: &Path,
    ignore: &[String],
    features: &[String],
    branch: bool,
) -> Result<LlvmCovReport> {
    parse_llvm_cov_report(&run_cargo_llvm_cov(
        root,
        ignore,
        &["--json", "--summary-only"],
        features,
        branch,
    )?)
}

/// Run `cargo llvm-cov --lib` over the unit suite in `root` with the given coverage
/// `format` args and return its stdout. Shared by the whole-tree floor and the
/// diff-scoped floor, so both measure the same unit-only slice.
fn run_cargo_llvm_cov(
    root: &Path,
    ignore: &[String],
    format: &[&str],
    features: &[String],
    branch: bool,
) -> Result<String> {
    let target = TargetDir::new();

    let mut command = Command::new("cargo");
    command
        .current_dir(root)
        .arg("llvm-cov")
        // cargo-llvm-cov's default runs every test target, which lets the integration
        // tier under `tests/` pad the number.
        .arg("--lib")
        .args(format)
        .env("CARGO_TARGET_DIR", &target.0);
    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }
    if branch {
        // Instruments only on a nightly toolchain — the error below names that.
        command.arg("--branch");
    }
    if let Some(regex) = ignore_filename_regex(root, ignore) {
        command.arg("--ignore-filename-regex").arg(regex);
    }
    // When this check runs under an outer `cargo llvm-cov`, an inherited
    // `RUSTC_WRAPPER` makes the inner run re-enter cargo-llvm-cov on every rustc
    // invocation and hang until the runner is OOM-killed. Strip the outer state.
    for var in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTDOCFLAGS",
        "CARGO_ENCODED_RUSTDOCFLAGS",
        "LLVM_PROFILE_FILE",
        "CARGO_LLVM_COV",
        "CARGO_LLVM_COV_SHOW_ENV",
        "CARGO_LLVM_COV_TARGET_DIR",
        "CARGO_LLVM_COV_BUILD_DIR",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "__CARGO_LLVM_COV_RUSTC_WRAPPER",
        "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS",
        "__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES",
        // rustup gives an inherited toolchain selection precedence over the scanned
        // crate's own `rust-toolchain.toml`, so a spawning cargo would override the
        // nightly a branch-floor crate pins there.
        "RUSTUP_TOOLCHAIN",
        "CARGO",
        "RUSTC",
    ] {
        command.env_remove(var);
    }
    let output = command
        .output()
        .context("running `cargo llvm-cov` (is cargo-llvm-cov installed?)")?;
    if !output.status.success() {
        let hint = if branch {
            "\n(the [rust].coverage `branch` floor runs with --branch, which requires a \
             nightly toolchain — pin one in the crate's rust-toolchain.toml with \
             llvm-tools-preview, or set a rustup directory override)"
        } else {
            ""
        };
        bail!(
            "the unit suite did not run cleanly under cargo llvm-cov in `{}`:{hint}\n{}{}",
            root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Per-file region detail from a `cargo llvm-cov --json` export — what
/// [`crate::patch_coverage::evaluate_patch_rust`] restricts to the changed lines.
#[derive(Debug, Clone, Default)]
pub struct RustPatchCoverage {
    /// One per `kind == 0` code region: `(start_line, end_line, covered)`. A region
    /// counts toward the diff when any line it spans is changed.
    pub regions: Vec<(u64, u64, bool)>,
}

/// A full `cargo llvm-cov --json` export, modeling the per-function region detail the
/// diff-scoped floor needs — separate from [`LlvmCovReport`], which keeps the totals.
#[derive(Debug, Clone, Deserialize)]
struct LlvmCovExport {
    data: Vec<LlvmCovExportData>,
}

/// One export entry. `--ignore-filename-regex` drops an exempt file from `files` but
/// *not* from `functions` (the regions array is unfiltered), so `files` is the
/// allowlist [`llvm_cov_patch_detail`] restricts the regions to.
#[derive(Debug, Clone, Deserialize)]
struct LlvmCovExportData {
    files: Vec<LlvmCovExportFile>,
    functions: Vec<LlvmCovFunction>,
}

/// One measured file in the export's `files` block — only its absolute `filename` is
/// needed, to build the not-ignored allowlist.
#[derive(Debug, Clone, Deserialize)]
struct LlvmCovExportFile {
    filename: String,
}

/// One function's coverage: the files it spans (`filenames`, indexed by a region's
/// `fileID`) and its regions. Each region is a flat array `[lineStart, colStart,
/// lineEnd, colEnd, executionCount, fileID, expandedFileID, kind]`, read positionally.
#[derive(Debug, Clone, Deserialize)]
struct LlvmCovFunction {
    filenames: Vec<String>,
    regions: Vec<Vec<i64>>,
}

/// Run the Rust unit suite under `cargo llvm-cov` and return the per-file region
/// detail, keyed by the absolute path llvm-cov reports. `ignore` is the
/// `coverage`-rule exemptions, dropped so an exempt file's changed lines are lifted.
pub fn measure_patch_rust_detail(
    root: &Path,
    ignore: &[String],
    features: &[String],
) -> Result<BTreeMap<String, RustPatchCoverage>> {
    // The diff-scoped floor judges regions + lines, so its run never adds `--branch`.
    let json = run_cargo_llvm_cov(root, ignore, &["--json"], features, false)?;
    llvm_cov_patch_detail(&json)
}

/// Pure: per-file [`RustPatchCoverage`] from a `cargo llvm-cov --json` export, keyed
/// by the absolute path llvm-cov reports. Only `kind == 0` code regions in the `files`
/// allowlist count; a malformed short region is skipped rather than indexed.
fn llvm_cov_patch_detail(json: &str) -> Result<BTreeMap<String, RustPatchCoverage>> {
    let export: LlvmCovExport =
        serde_json::from_str(json).context("parsing cargo llvm-cov JSON export")?;
    let mut out: BTreeMap<String, RustPatchCoverage> = BTreeMap::new();
    for data in &export.data {
        let measured: BTreeSet<&str> = data.files.iter().map(|f| f.filename.as_str()).collect();
        for function in &data.functions {
            for region in &function.regions {
                if region.len() < 8 {
                    continue;
                }
                // gap (1) / expansion (2) / branch regions carry no line-coverage signal.
                if region[7] != 0 {
                    continue;
                }
                let file_id = region[5];
                let Ok(file_id) = usize::try_from(file_id) else {
                    continue;
                };
                let Some(file) = function.filenames.get(file_id) else {
                    continue;
                };
                // A `coverage` exemption drops the file's regions, lifting its lines.
                if !measured.contains(file.as_str()) {
                    continue;
                }
                let start = region[0].max(0) as u64;
                let end = region[2].max(0) as u64;
                let covered = region[4] > 0;
                out.entry(file.clone())
                    .or_default()
                    .regions
                    .push((start, end, covered));
            }
        }
    }
    Ok(out)
}

/// The single `--ignore-filename-regex` for the run, or `None` when nothing is exempt.
/// It is a substring search over absolute filenames, so each exempt path is escaped,
/// joined under `root`, and `$`-anchored — else it over-matches `member/src/a.rs`.
fn ignore_filename_regex(root: &Path, ignore: &[String]) -> Option<String> {
    if ignore.is_empty() {
        return None;
    }
    Some(
        ignore
            .iter()
            .map(|rel| {
                // The fallback keeps the anchor deterministic when the path can't be
                // resolved (e.g. in tests).
                let full = root.join(rel);
                let full = full.canonicalize().unwrap_or(full);
                format!("{}$", regex_escape(&full.to_string_lossy()))
            })
            .collect::<Vec<_>>()
            .join("|"),
    )
}

/// Escape `s`'s regex metacharacters so an exempt path matches literally.
fn regex_escape(s: &str) -> String {
    const META: &str = r"\.+*?()|[]{}^$";
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if META.contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
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
        assert_eq!(report.totals.percent_covered, 85.0);
    }

    #[test]
    fn a_report_without_a_files_block_parses_with_an_empty_map() {
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
        let exempt = vec!["pkg/gen.py".to_string(), "shim.py".to_string()];
        assert_eq!(
            build_omit(&exempt),
            "*_test.py,*conftest.py,pkg/gen.py,shim.py"
        );
    }

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
        assert_eq!(
            evaluate_typescript(&ts_report(99.999_999_999, 100.0, 100.0, 100.0), TS_FULL),
            Outcome::Pass
        );
    }

    #[test]
    fn typescript_empty_denominator_metric_is_vacuously_satisfied() {
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
        let json = r#"{"total":{
            "lines": {"total": 1, "covered": 1, "skipped": 0, "pct": true},
            "statements": {"total": 1, "covered": 1, "skipped": 0, "pct": 100},
            "functions": {"total": 1, "covered": 1, "skipped": 0, "pct": 100},
            "branches": {"total": 1, "covered": 1, "skipped": 0, "pct": 100}
        }}"#;
        assert!(parse_vitest_report(json).is_err());
    }

    fn rust_metric(percent: f64) -> LlvmCovMetric {
        LlvmCovMetric {
            count: 10,
            covered: 10,
            percent,
        }
    }

    fn rust_report(regions: f64, lines: f64) -> LlvmCovReport {
        LlvmCovReport {
            data: vec![LlvmCovData {
                totals: LlvmCovTotals {
                    regions: rust_metric(regions),
                    lines: rust_metric(lines),
                    functions: rust_metric(lines),
                    branches: None,
                },
            }],
        }
    }

    /// Like [`rust_report`] with explicit functions/branches; `branches: (count,
    /// percent)` so the vacuous zero-denominator case is constructible.
    fn rust_report_full(
        regions: f64,
        lines: f64,
        functions: f64,
        branches: (u64, f64),
    ) -> LlvmCovReport {
        let (count, percent) = branches;
        LlvmCovReport {
            data: vec![LlvmCovData {
                totals: LlvmCovTotals {
                    regions: rust_metric(regions),
                    lines: rust_metric(lines),
                    functions: rust_metric(functions),
                    branches: Some(LlvmCovMetric {
                        count,
                        covered: count,
                        percent,
                    }),
                },
            }],
        }
    }

    const RUST_FULL: RustThresholds = RustThresholds {
        regions: Some(100),
        lines: 100,
        functions: None,
        branch: None,
    };
    const RUST_MID: RustThresholds = RustThresholds {
        regions: Some(80),
        lines: 85,
        functions: None,
        branch: None,
    };

    #[test]
    fn rust_functions_floor_fails_below_and_passes_at_its_bar() {
        let report = rust_report_full(100.0, 100.0, 66.67, (0, 0.0));
        let floor = |functions| RustThresholds {
            regions: None,
            lines: 50,
            functions: Some(functions),
            branch: None,
        };
        assert!(matches!(
            evaluate_rust(&report, floor(100)),
            Outcome::Fail(message) if message.contains("functions")
        ));
        assert_eq!(evaluate_rust(&report, floor(60)), Outcome::Pass);
    }

    #[test]
    fn rust_branch_floor_fails_below_and_passes_at_its_bar() {
        let report = rust_report_full(100.0, 100.0, 100.0, (2, 50.0));
        let floor = |branch| RustThresholds {
            regions: None,
            lines: 50,
            functions: None,
            branch: Some(branch),
        };
        assert!(matches!(
            evaluate_rust(&report, floor(100)),
            Outcome::Fail(message) if message.contains("branches")
        ));
        assert_eq!(evaluate_rust(&report, floor(50)), Outcome::Pass);
    }

    #[test]
    fn rust_a_branchless_crate_clears_any_branch_floor_vacuously() {
        let report = rust_report_full(100.0, 100.0, 100.0, (0, 0.0));
        let floor = RustThresholds {
            regions: None,
            lines: 50,
            functions: None,
            branch: Some(100),
        };
        assert_eq!(evaluate_rust(&report, floor), Outcome::Pass);
    }

    #[test]
    fn rust_passes_when_both_metrics_meet_their_floor() {
        assert_eq!(
            evaluate_rust(&rust_report(100.0, 100.0), RUST_FULL),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_fails_on_the_one_metric_below_its_floor() {
        let outcome = evaluate_rust(&rust_report(70.0, 100.0), RUST_MID);
        assert!(
            matches!(&outcome, Outcome::Fail(message) if message.contains("regions") && !message.contains("lines")),
            "got: {outcome:?}"
        );
    }

    #[test]
    fn rust_fail_message_names_every_metric_below() {
        let outcome = evaluate_rust(&rust_report(50.0, 50.0), RUST_MID);
        assert!(
            matches!(&outcome, Outcome::Fail(message)
                if message.contains("regions") && message.contains("lines")),
            "got: {outcome:?}"
        );
    }

    #[test]
    fn rust_skips_the_region_check_when_regions_is_opt_out() {
        let thresholds = RustThresholds {
            regions: None,
            lines: 100,
            functions: None,
            branch: None,
        };
        assert_eq!(
            evaluate_rust(&rust_report(40.0, 100.0), thresholds),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_still_fails_lines_with_regions_opt_out() {
        let thresholds = RustThresholds {
            regions: None,
            lines: 100,
            functions: None,
            branch: None,
        };
        let outcome = evaluate_rust(&rust_report(100.0, 80.0), thresholds);
        assert!(
            matches!(&outcome, Outcome::Fail(message)
                if message.contains("lines") && !message.contains("regions")),
            "got: {outcome:?}"
        );
    }

    #[test]
    fn rust_tolerates_float_noise_at_the_floor() {
        assert_eq!(
            evaluate_rust(&rust_report(99.999_999_999, 100.0), RUST_FULL),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_fails_a_vacuous_run_that_measured_no_code() {
        let nothing = LlvmCovMetric {
            count: 0,
            covered: 0,
            percent: 0.0,
        };
        let report = LlvmCovReport {
            data: vec![LlvmCovData {
                totals: LlvmCovTotals {
                    regions: nothing,
                    lines: nothing,
                    functions: nothing,
                    branches: None,
                },
            }],
        };
        let outcome = evaluate_rust(&report, RUST_MID);
        assert!(
            matches!(&outcome, Outcome::Fail(message) if message.contains("measured no code")),
            "got: {outcome:?}"
        );
    }

    #[test]
    fn rust_fails_an_export_with_no_data() {
        let report = LlvmCovReport { data: vec![] };
        assert!(matches!(evaluate_rust(&report, RUST_MID), Outcome::Fail(_)));
    }

    #[test]
    fn parses_a_cargo_llvm_cov_report() {
        let json = r#"{
            "data": [{"totals": {
                "regions": {"count": 12, "covered": 9, "notcovered": 3, "percent": 75.0},
                "lines": {"count": 20, "covered": 18, "percent": 90.0},
                "functions": {"count": 3, "covered": 3, "percent": 100.0}
            }}],
            "type": "llvm.coverage.json.export",
            "version": "2.0.1"
        }"#;
        let report = parse_llvm_cov_report(json).expect("valid llvm-cov json");
        assert_eq!(report.data[0].totals.regions.percent, 75.0);
        assert_eq!(report.data[0].totals.lines.count, 20);
    }

    #[test]
    fn llvm_cov_patch_detail_reads_code_regions_per_file() {
        let json = r#"{
            "data": [{
                "files": [{"filename": "/abs/grade.rs"}],
                "functions": [{
                    "filenames": ["/abs/grade.rs"],
                    "regions": [
                        [6, 5, 6, 26, 1, 0, 0, 0],
                        [10, 9, 10, 17, 0, 0, 0, 0]
                    ]
                }],
                "totals": {}
            }],
            "type": "llvm.coverage.json.export",
            "version": "3.0.1"
        }"#;
        let out = llvm_cov_patch_detail(json).expect("valid llvm-cov export");
        assert_eq!(
            out["/abs/grade.rs"].regions,
            vec![(6, 6, true), (10, 10, false)]
        );
    }

    #[test]
    fn llvm_cov_patch_detail_skips_non_code_regions() {
        let json = r#"{
            "data": [{
                "files": [{"filename": "/abs/a.rs"}],
                "functions": [{
                    "filenames": ["/abs/a.rs"],
                    "regions": [
                        [1, 1, 1, 10, 2, 0, 0, 0],
                        [2, 1, 2, 10, 0, 0, 0, 1],
                        [3, 1, 3, 10, 0, 0, 0, 2]
                    ]
                }]
            }]
        }"#;
        let out = llvm_cov_patch_detail(json).expect("valid llvm-cov export");
        assert_eq!(out["/abs/a.rs"].regions, vec![(1, 1, true)]);
    }

    #[test]
    fn llvm_cov_patch_detail_groups_regions_by_filename_id() {
        let json = r#"{
            "data": [{
                "files": [{"filename": "/abs/a.rs"}, {"filename": "/abs/b.rs"}],
                "functions": [{
                    "filenames": ["/abs/a.rs", "/abs/b.rs"],
                    "regions": [
                        [1, 1, 1, 5, 1, 0, 0, 0],
                        [9, 1, 9, 5, 0, 1, 1, 0]
                    ]
                }]
            }]
        }"#;
        let out = llvm_cov_patch_detail(json).expect("valid llvm-cov export");
        assert_eq!(out["/abs/a.rs"].regions, vec![(1, 1, true)]);
        assert_eq!(out["/abs/b.rs"].regions, vec![(9, 9, false)]);
    }

    #[test]
    fn llvm_cov_patch_detail_skips_a_malformed_short_region() {
        let json = r#"{
            "data": [{
                "files": [{"filename": "/abs/a.rs"}],
                "functions": [{
                    "filenames": ["/abs/a.rs"],
                    "regions": [
                        [4, 1, 4],
                        [5, 1, 5, 9, 1, 0, 0, 0]
                    ]
                }]
            }]
        }"#;
        let out = llvm_cov_patch_detail(json).expect("valid llvm-cov export");
        assert_eq!(out["/abs/a.rs"].regions, vec![(5, 5, true)]);
    }

    #[test]
    fn llvm_cov_patch_detail_spans_a_multiline_region() {
        let json = r#"{
            "data": [{
                "files": [{"filename": "/abs/a.rs"}],
                "functions": [{
                    "filenames": ["/abs/a.rs"],
                    "regions": [[3, 5, 5, 6, 0, 0, 0, 0]]
                }]
            }]
        }"#;
        let out = llvm_cov_patch_detail(json).expect("valid llvm-cov export");
        assert_eq!(out["/abs/a.rs"].regions, vec![(3, 5, false)]);
    }

    #[test]
    fn llvm_cov_patch_detail_drops_a_file_absent_from_the_files_allowlist() {
        let json = r#"{
            "data": [{
                "files": [{"filename": "/abs/kept.rs"}],
                "functions": [{
                    "filenames": ["/abs/kept.rs", "/abs/ignored.rs"],
                    "regions": [
                        [1, 1, 1, 9, 1, 0, 0, 0],
                        [2, 1, 2, 9, 0, 1, 0, 0]
                    ]
                }]
            }]
        }"#;
        let out = llvm_cov_patch_detail(json).expect("valid llvm-cov export");
        assert_eq!(out["/abs/kept.rs"].regions, vec![(1, 1, true)]);
        assert!(!out.contains_key("/abs/ignored.rs"));
    }

    #[test]
    fn llvm_cov_patch_detail_malformed_json_is_an_error() {
        assert!(llvm_cov_patch_detail("{ not json").is_err());
    }

    #[test]
    fn llvm_cov_patch_detail_skips_a_negative_file_id() {
        let json = r#"{
            "data": [{
                "files": [{"filename": "/abs/a.rs"}],
                "functions": [{
                    "filenames": ["/abs/a.rs"],
                    "regions": [[1, 1, 1, 5, 1, -1, 0, 0]]
                }]
            }]
        }"#;
        let out = llvm_cov_patch_detail(json).expect("valid llvm-cov export");
        assert!(out.is_empty(), "got: {out:?}");
    }

    #[test]
    fn llvm_cov_patch_detail_skips_an_out_of_range_file_id() {
        let json = r#"{
            "data": [{
                "files": [{"filename": "/abs/a.rs"}],
                "functions": [{
                    "filenames": ["/abs/a.rs"],
                    "regions": [[1, 1, 1, 5, 1, 7, 0, 0]]
                }]
            }]
        }"#;
        let out = llvm_cov_patch_detail(json).expect("valid llvm-cov export");
        assert!(out.is_empty(), "got: {out:?}");
    }

    #[test]
    fn istanbul_patch_detail_keeps_a_branch_without_counts() {
        let json = r#"{
            "/abs/a.ts": {
                "statementMap": {},
                "s": {},
                "branchMap": {"0": {"loc": {"start": {"line": 3}, "end": {"line": 3}}}},
                "b": {},
                "fnMap": {},
                "f": {}
            }
        }"#;
        let out = istanbul_patch_detail(json).expect("valid Istanbul report");
        assert!(out["/abs/a.ts"].branch_arms.is_empty(), "got: {out:?}");
    }

    #[test]
    fn default_excludes_that_are_not_json_name_the_output() {
        let err = parse_default_excludes(b"vitest warmed up first").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("not a JSON string array"), "got: {msg}");
        assert!(msg.contains("vitest warmed up first"), "got: {msg}");
    }

    #[test]
    fn default_excludes_drop_a_nul_bearing_pattern() {
        let parsed = parse_default_excludes(br#"["**/dist/**", "**/\u0000*"]"#).unwrap();
        assert_eq!(parsed, vec!["**/dist/**".to_string()]);
    }

    #[test]
    fn a_missing_vitest_report_names_the_reporter() {
        let path = std::env::temp_dir().join("tc-no-such-report/coverage-final.json");
        let err = read_vitest_report(&path, "json").unwrap_err();
        assert!(format!("{err:#}").contains("json report"), "got: {err:#}");
    }

    #[test]
    fn rust_ignore_regex_is_none_when_nothing_is_exempt() {
        assert_eq!(ignore_filename_regex(Path::new("/repo"), &[]), None);
    }

    #[test]
    fn rust_ignore_regex_anchors_each_exempt_path_to_its_full_path() {
        // `/repo` doesn't exist, so `canonicalize` falls back to the plain join.
        let exempt = vec!["src/shim.rs".to_string(), "src/gen.rs".to_string()];
        assert_eq!(
            ignore_filename_regex(Path::new("/repo"), &exempt).as_deref(),
            Some(r"/repo/src/shim\.rs$|/repo/src/gen\.rs$")
        );
    }

    /// Model llvm-cov's substring `--ignore-filename-regex` for the escaped, optionally
    /// `$`-anchored literals this tool emits. One matching alternative ignores the file.
    fn llvm_would_ignore(regex: &str, filename: &str) -> bool {
        regex.split('|').any(|alt| {
            let (lit, anchored) = match alt.strip_suffix('$') {
                Some(head) => (head, true),
                None => (alt, false),
            };
            let lit = lit.replace('\\', "");
            if anchored {
                filename.ends_with(&lit)
            } else {
                filename.contains(&lit)
            }
        })
    }

    #[test]
    fn llvm_would_ignore_matches_an_unanchored_literal_anywhere() {
        assert!(llvm_would_ignore("/repo/src", "/repo/src/a.rs"));
        assert!(!llvm_would_ignore("/elsewhere", "/repo/src/a.rs"));
    }

    #[test]
    fn rust_ignore_regex_does_not_over_match_a_member_with_the_same_suffix() {
        let regex = ignore_filename_regex(Path::new("/repo"), &["src/a.rs".to_string()]).unwrap();
        assert!(
            llvm_would_ignore(&regex, "/repo/src/a.rs"),
            "the exempted file must still be ignored: {regex}"
        );
        assert!(
            !llvm_would_ignore(&regex, "/repo/member/src/a.rs"),
            "`src/a.rs` over-matched `member/src/a.rs`: {regex}"
        );
        assert!(
            !llvm_would_ignore(&regex, "/repo/src/xsrc/a.rs"),
            "`src/a.rs` over-matched `src/xsrc/a.rs`: {regex}"
        );
    }
}
