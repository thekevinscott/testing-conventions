//! Coverage rule (Python — issue #26; TypeScript — issue #31; Rust — issue #37; exemptions — issue #32).
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
//! [`Outcome`] type. Its subprocess layer shells out to `vitest`. Rust (#37) is
//! the third twin: `cargo llvm-cov` reports regions/lines (branch coverage is
//! experimental), so it carries [`RustThresholds`], [`LlvmCovReport`], and
//! [`evaluate_rust`] / [`measure_rust`]; its subprocess layer shells out to
//! `cargo llvm-cov`.
//!
//! Files exempted from coverage in config (issue #32) are omitted from the
//! denominator alongside the test files; the caller resolves them
//! ([`crate::config::resolve_exempt`]) and passes their paths to [`measure`] /
//! [`measure_typescript`] / [`measure_rust`].

use std::collections::{BTreeMap, BTreeSet};
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
/// the `totals` (the floor) and the per-file `files` block (patch
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
    /// `[source_line, dest_line]` pairs for branches the suite DID take (coverage.py
    /// emits these alongside `missing_branches` under `--branch`). The diff-scoped
    /// floor (#162) counts an arc toward changed-line branch coverage when its source
    /// line is in the diff; with `missing_branches` it gives branch coverage over the
    /// changed lines. Empty when branch coverage was off.
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
/// Shells out to `coverage run --branch` (omitting `*_test.py` and every path in
/// `omit` from the denominator) then `coverage json`, and evaluates the report.
/// `omit` holds the `coverage`-rule exemptions resolved from config, as
/// `root`-relative paths. The `coverage` CLI — with `pytest` importable — must be
/// on `PATH`.
pub fn measure(root: &Path, thresholds: Thresholds, omit: &[String]) -> Result<Outcome> {
    let report = run_coverage(root, omit, false)?;
    Ok(evaluate(&report, thresholds))
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

/// Run vitest coverage over the unit suite in `root` and return the raw contents
/// of the `report_file` the `reporter` wrote. Shared by the floor (#31, the
/// `json-summary` → `coverage-summary.json` pair) and patch coverage (#135, the
/// detailed `json` → `coverage-final.json` Istanbul pair) — the two differ only in
/// the reporter and how they parse it.
///
/// v8 coverage is written to an out-of-tree temp dir so the scanned tree stays
/// pristine. `include` scopes measurement to the sources under `root`; the test
/// glob, declaration files, and the config `exclude` paths are excluded from the
/// denominator. `all=true` counts source files the suite never imported, so an
/// untested file is measured (lowering the floor / showing as uncovered) rather
/// than vanishing. `--no-cache` keeps vitest from writing a cache into the tree.
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
        .args(["--yes", "vitest", "run", "--no-cache"])
        .args(["--coverage.enabled", "--coverage.provider=v8"])
        .arg(format!("--coverage.reporter={reporter}"))
        .arg("--coverage.all=true")
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

    let path = reports.0.join(report_file);
    std::fs::read_to_string(&path).with_context(|| {
        format!(
            "reading vitest coverage report `{}` (did the run produce a {reporter} report?)",
            path.display()
        )
    })
}

// ---------------------------------------------------------------------------
// TypeScript patch (changed-line) coverage — issue #135.
//
// What patch coverage (`crate::patch_coverage::check_typescript`) reads: the set
// of uncovered lines per file. vitest's `json-summary` gives only per-file totals,
// so this measures with the detailed `json` (Istanbul `coverage-final.json`)
// reporter and reduces it to the lines a changed line must avoid — the v8 twin of
// coverage.py's `missing_lines` / `missing_branches`.
// ---------------------------------------------------------------------------

/// Run the TypeScript unit suite under vitest in `root` and return the uncovered
/// lines per file — keyed by the absolute path vitest reports, the caller
/// re-keying to `root`-relative to match the diff. A line is uncovered when it
/// carries a statement the suite never executed, or the source of a branch a path
/// of which the suite never took (the v8 analogue of the Python arm's missing line
/// / missing branch). `exclude` is the `coverage`-rule exemptions, dropped from the
/// run so an exempt file's changed lines are lifted. `npx` resolves the
/// project-local `vitest`, so it and `@vitest/coverage-v8` must be installed under
/// `root`.
pub fn measure_patch_typescript(
    root: &Path,
    exclude: &[String],
) -> Result<BTreeMap<String, BTreeSet<u64>>> {
    let json = run_vitest_coverage(root, exclude, "json", "coverage-final.json")?;
    uncovered_istanbul_lines(&json)
}

/// One file's entry in a vitest v8 `coverage-final.json` (Istanbul) report, pared
/// to what patch coverage reads: the statement / branch maps and their hit counts.
/// Unmodeled fields (`path`, `fnMap`/`f`, per-node metadata) are ignored.
#[derive(Debug, Clone, Deserialize)]
struct IstanbulFile {
    /// Statement id → source span. A statement whose hit count in `s` is `0` was
    /// never executed, so its lines are uncovered.
    #[serde(rename = "statementMap", default)]
    statement_map: BTreeMap<String, IstanbulSpan>,
    /// Statement id → execution count.
    #[serde(default)]
    s: BTreeMap<String, u64>,
    /// Branch id → branch location. A branch with a `0` among its `b` counts had a
    /// path the suite never took, so its source line is uncovered.
    #[serde(rename = "branchMap", default)]
    branch_map: BTreeMap<String, IstanbulBranch>,
    /// Branch id → per-path execution counts.
    #[serde(default)]
    b: BTreeMap<String, Vec<u64>>,
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

/// A branch entry — only its location (whose start line is the branch's source
/// line) matters; the `type` and per-path `locations` are ignored.
#[derive(Debug, Clone, Deserialize)]
struct IstanbulBranch {
    loc: IstanbulSpan,
}

/// Pure: every uncovered line per file from a vitest v8 `coverage-final.json`
/// (Istanbul) report — a statement the suite never ran (every line it spans) and
/// the source line of a branch a path of which it never took. Keyed by the path
/// vitest reports (absolute). A file present but fully covered maps to an empty
/// set. Mirrors [`crate::patch_coverage::uncovered_changed_lines`]'s Python rule
/// (missing line ∪ missing-branch source) for the v8 shape.
fn uncovered_istanbul_lines(json: &str) -> Result<BTreeMap<String, BTreeSet<u64>>> {
    let files: BTreeMap<String, IstanbulFile> = serde_json::from_str(json)
        .context("parsing vitest coverage-final (Istanbul) JSON report")?;
    let mut out = BTreeMap::new();
    for (path, file) in files {
        let mut lines = BTreeSet::new();
        // A statement never executed (`s[id] == 0`) — every line it spans is
        // uncovered (a single-line statement spans one line).
        for (id, span) in &file.statement_map {
            if file.s.get(id) == Some(&0) {
                lines.extend(span.start.line..=span.end.line);
            }
        }
        // A branch with an untaken path (a `0` among its counts) — its source line
        // (the location's start) is uncovered, even when the line itself ran.
        for (id, branch) in &file.branch_map {
            if file.b.get(id).is_some_and(|counts| counts.contains(&0)) {
                lines.insert(branch.loc.start.line);
            }
        }
        out.insert(path, lines);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Rust (cargo llvm-cov) — issue #37.
//
// The Rust twin of the rules above. `cargo llvm-cov` reports LLVM source-based
// coverage as regions + lines (branch coverage is still experimental), so the
// Rust rule carries its own thresholds and `measure_rust` entry point; only the
// `Outcome` type is shared. Mirroring the Python/TypeScript split, a pure
// `evaluate_rust` over a parsed llvm-cov export and the thin subprocess layer
// that produces one land with the implementation (#37).
// ---------------------------------------------------------------------------

/// The two `cargo llvm-cov` coverage floors, from a `[rust].coverage` table.
/// Branch coverage is still experimental, so only regions and lines are enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RustThresholds {
    pub regions: u8,
    pub lines: u8,
}

/// A `cargo llvm-cov --json` export (LLVM's `llvm.coverage.json.export`), pared to
/// the totals the check needs. A single run produces one `data` entry; unmodeled
/// fields (per-file/per-function detail, `type`, `version`) are ignored.
#[derive(Debug, Clone, Deserialize)]
pub struct LlvmCovReport {
    pub data: Vec<LlvmCovData>,
}

/// One export entry — only its `totals` are needed (`--summary-only` omits the
/// per-file and per-function detail).
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LlvmCovData {
    pub totals: LlvmCovTotals,
}

/// The `totals` block of an llvm-cov export — the two metrics this rule enforces.
/// llvm-cov also reports `functions`, `instantiations`, and (experimental)
/// `branches`, which the check ignores.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LlvmCovTotals {
    pub regions: LlvmCovMetric,
    pub lines: LlvmCovMetric,
}

/// One metric's totals from an llvm-cov export, pared to what the check needs: the
/// denominator size and the covered percent.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LlvmCovMetric {
    /// Size of the denominator (regions or lines counted).
    pub count: u64,
    /// How many were covered.
    pub covered: u64,
    /// Covered percent — llvm-cov computes `100 * covered / count`.
    pub percent: f64,
}

/// Parse a `cargo llvm-cov --json` export.
pub fn parse_llvm_cov_report(json: &str) -> Result<LlvmCovReport> {
    serde_json::from_str(json).context("parsing cargo llvm-cov JSON report")
}

/// Decide whether `report` meets both thresholds.
///
/// Fails when the run measured no regions at all (an empty denominator — a wrong
/// path, or a crate that compiled nothing — is never a silent pass), otherwise
/// checks regions and lines and fails listing each below its floor.
pub fn evaluate_rust(report: &LlvmCovReport, thresholds: RustThresholds) -> Outcome {
    let Some(totals) = report.data.first().map(|entry| &entry.totals) else {
        return Outcome::Fail("the cargo llvm-cov report contained no data".to_string());
    };
    // Vacuous-run guard: every compiled crate has regions, so a zero region
    // denominator means nothing was measured — failed rather than passed on an
    // empty measurement (mirrors the TypeScript path).
    if totals.regions.count == 0 {
        return Outcome::Fail(
            "the unit suite measured no code — check the path and that the suite runs".to_string(),
        );
    }
    let checks = [
        ("regions", totals.regions.percent, thresholds.regions),
        ("lines", totals.lines.percent, thresholds.lines),
    ];
    let mut shortfalls = Vec::new();
    for (name, actual, required) in checks {
        // A hair of tolerance so a percent that rounds to the floor isn't failed by
        // float noise (matches the Python / TypeScript paths).
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
/// `thresholds`.
///
/// Shells out to `cargo llvm-cov --json --summary-only`, omitting every path in
/// `ignore` from the denominator (a single `--ignore-filename-regex`), then
/// evaluates the export. `ignore` holds the `coverage`-rule exemptions resolved
/// from config, as `root`-relative paths. `cargo-llvm-cov` must be installed.
pub fn measure_rust(root: &Path, thresholds: RustThresholds, ignore: &[String]) -> Result<Outcome> {
    let report = run_llvm_cov(root, ignore)?;
    Ok(evaluate_rust(&report, thresholds))
}

/// A `cargo llvm-cov` target directory under the temp dir — unique per call (so
/// checks running in parallel don't collide) and removed on drop (so the build
/// never leaks into the scanned tree). Passed to the run as `CARGO_TARGET_DIR`.
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

/// Run cargo llvm-cov over the unit suite in `root` and return the parsed
/// `--summary-only` export — the totals the floor checks.
fn run_llvm_cov(root: &Path, ignore: &[String]) -> Result<LlvmCovReport> {
    parse_llvm_cov_report(&run_cargo_llvm_cov(
        root,
        ignore,
        &["--json", "--summary-only"],
    )?)
}

/// Run `cargo llvm-cov` over the unit suite in `root` with the given coverage
/// `format` args (`["--json", "--summary-only"]` for the floor's totals,
/// `["--lcov"]` for patch coverage's per-line detail) and return its stdout.
/// Shared by the floor (#37) and patch coverage (#136).
///
/// The build goes to an out-of-tree target dir (via `CARGO_TARGET_DIR`) so the
/// scanned crate stays pristine; the `coverage`-rule exemptions become one
/// `--ignore-filename-regex`; and the outer run's instrumentation env is stripped
/// for nested-run hygiene (the loop below explains why).
fn run_cargo_llvm_cov(root: &Path, ignore: &[String], format: &[&str]) -> Result<String> {
    let target = TargetDir::new();

    let mut command = Command::new("cargo");
    command
        .current_dir(root)
        .arg("llvm-cov")
        .args(format)
        .env("CARGO_TARGET_DIR", &target.0);
    if let Some(regex) = ignore_filename_regex(ignore) {
        command.arg("--ignore-filename-regex").arg(regex);
    }
    // Nested-run hygiene: when this check itself runs under `cargo llvm-cov` (the
    // package's own coverage job), the outer run exports its instrumentation state
    // into our environment — the coverage flags and profile path, and (because
    // cargo-llvm-cov drives instrumentation through a rustc wrapper) a
    // `RUSTC_WRAPPER` pointing back at `cargo-llvm-cov`. Inherited, that wrapper
    // makes the inner run re-enter cargo-llvm-cov on every rustc invocation and
    // never finish — it hangs compiling the scanned crate until the runner is
    // OOM-killed. Strip the lot so the inner run instruments from a clean slate.
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
    ] {
        command.env_remove(var);
    }
    let output = command
        .output()
        .context("running `cargo llvm-cov` (is cargo-llvm-cov installed?)")?;
    if !output.status.success() {
        bail!(
            "the unit suite did not run cleanly under cargo llvm-cov in `{}`:\n{}{}",
            root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run the Rust unit suite under `cargo llvm-cov` in `root` and return the
/// uncovered lines per file — keyed by the absolute path llvm-cov reports, the
/// caller re-keying to `root`-relative to match the diff. A line is uncovered when
/// llvm-cov records no execution for it (an LCOV `DA:<line>,0`). What patch
/// coverage (#136, [`crate::patch_coverage::check_rust`]) reads; `ignore` is the
/// `coverage`-rule exemptions, dropped from the run so an exempt file's changed
/// lines are lifted. `cargo-llvm-cov` must be installed.
pub fn measure_patch_rust(
    root: &Path,
    ignore: &[String],
) -> Result<BTreeMap<String, BTreeSet<u64>>> {
    Ok(uncovered_lcov_lines(&run_cargo_llvm_cov(
        root,
        ignore,
        &["--lcov"],
    )?))
}

/// Pure: every uncovered line per file from a `cargo llvm-cov --lcov` report — a
/// `DA:<line>,<count>` record with a zero count, grouped under the `SF:<path>` it
/// falls within (an `end_of_record` closes the file). Keyed by the path llvm-cov
/// reports (absolute). A measured file with no zero-count line maps to an empty
/// set. Lines a file's records don't mention (a comment, a blank) aren't executable
/// and so are never uncovered.
fn uncovered_lcov_lines(lcov: &str) -> BTreeMap<String, BTreeSet<u64>> {
    let mut out: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in lcov.lines() {
        if let Some(path) = line.strip_prefix("SF:") {
            let path = path.trim().to_string();
            out.entry(path.clone()).or_default();
            current = Some(path);
        } else if let Some(rest) = line.strip_prefix("DA:") {
            // `DA:<line>,<count>[,<checksum>]` — a zero count is an uncovered line.
            if let Some(file) = &current {
                let mut fields = rest.split(',');
                if let (Some(line_no), Some(count)) = (fields.next(), fields.next()) {
                    if let (Ok(line_no), Ok(0)) =
                        (line_no.trim().parse::<u64>(), count.trim().parse::<u64>())
                    {
                        out.entry(file.clone()).or_default().insert(line_no);
                    }
                }
            }
        } else if line.trim() == "end_of_record" {
            current = None;
        }
    }
    out
}

/// The single `--ignore-filename-regex` value for the run, or `None` when nothing
/// is exempt. `cargo llvm-cov` takes one regex, so the `coverage`-exempt paths are
/// each regex-escaped (matched literally, not as a pattern) and joined with `|`. An
/// exempt file leaves the denominator with its reason recorded in config — an
/// auditable omission, not a silent ignore-glob.
fn ignore_filename_regex(ignore: &[String]) -> Option<String> {
    if ignore.is_empty() {
        return None;
    }
    Some(
        ignore
            .iter()
            .map(|path| regex_escape(path))
            .collect::<Vec<_>>()
            .join("|"),
    )
}

/// Escape the regex metacharacters in `s` so it matches literally — an exempt path
/// carries `.` (and may carry other metacharacters) that must not read as regex.
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
        // The floor path parses totals only; `files` defaults to empty.
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

    // --- TypeScript patch coverage (Istanbul `coverage-final.json`) — issue #135 ---

    #[test]
    fn istanbul_flags_an_unexecuted_statement() {
        // s1 (line 2) never ran → line 2 is uncovered; s0 (line 1) ran → not.
        let json = r#"{"/abs/widget.ts":{
            "statementMap":{"0":{"start":{"line":1,"column":0},"end":{"line":1,"column":40}},
                            "1":{"start":{"line":2,"column":2},"end":{"line":2,"column":20}}},
            "s":{"0":1,"1":0},
            "branchMap":{},"b":{}
        }}"#;
        let out = uncovered_istanbul_lines(json).unwrap();
        assert_eq!(out["/abs/widget.ts"], BTreeSet::from([2]));
    }

    #[test]
    fn istanbul_flags_an_untaken_branch_source() {
        // The branch out of line 3 has an untaken path (`[4, 0]`) → line 3 is
        // uncovered, even though its statement ran.
        let json = r#"{"/abs/widget.ts":{
            "statementMap":{"0":{"start":{"line":3,"column":2},"end":{"line":3,"column":20}}},
            "s":{"0":5},
            "branchMap":{"0":{"loc":{"start":{"line":3,"column":2},"end":{"line":3,"column":40}}}},
            "b":{"0":[4,0]}
        }}"#;
        let out = uncovered_istanbul_lines(json).unwrap();
        assert_eq!(out["/abs/widget.ts"], BTreeSet::from([3]));
    }

    #[test]
    fn istanbul_v8_single_arm_branch_counts_as_uncovered() {
        // vitest's v8 provider models each branch arm as its own entry with a
        // single-element count array; `[0]` is an arm the suite never took.
        let json = r#"{"/abs/widget.ts":{
            "statementMap":{},"s":{},
            "branchMap":{"0":{"loc":{"start":{"line":7,"column":0},"end":{"line":7,"column":3}}}},
            "b":{"0":[0]}
        }}"#;
        let out = uncovered_istanbul_lines(json).unwrap();
        assert_eq!(out["/abs/widget.ts"], BTreeSet::from([7]));
    }

    #[test]
    fn istanbul_spans_every_line_of_an_unexecuted_multiline_statement() {
        // A statement that never ran and spans lines 4-6 marks all three.
        let json = r#"{"/abs/widget.ts":{
            "statementMap":{"0":{"start":{"line":4,"column":2},"end":{"line":6,"column":3}}},
            "s":{"0":0},
            "branchMap":{},"b":{}
        }}"#;
        let out = uncovered_istanbul_lines(json).unwrap();
        assert_eq!(out["/abs/widget.ts"], BTreeSet::from([4, 5, 6]));
    }

    #[test]
    fn istanbul_fully_covered_file_has_no_uncovered_lines() {
        let json = r#"{"/abs/widget.ts":{
            "statementMap":{"0":{"start":{"line":1,"column":0},"end":{"line":1,"column":40}}},
            "s":{"0":3},
            "branchMap":{"0":{"loc":{"start":{"line":1,"column":0},"end":{"line":1,"column":40}}}},
            "b":{"0":[2,1]}
        }}"#;
        let out = uncovered_istanbul_lines(json).unwrap();
        assert!(out["/abs/widget.ts"].is_empty());
    }

    #[test]
    fn istanbul_widget_report_flags_statement_and_branch_lines() {
        // The realistic shape vitest emits for the `if (n === 42) { return 'answer';
        // }` red fixture: lines 4-5 are unexecuted statements and line 3 is an
        // untaken branch source → {3, 4, 5}.
        let json = r#"{"/abs/widget.ts":{
            "statementMap":{
                "0":{"start":{"line":1,"column":0},"end":{"line":1,"column":43}},
                "1":{"start":{"line":2,"column":2},"end":{"line":2,"column":25}},
                "2":{"start":{"line":3,"column":2},"end":{"line":3,"column":16}},
                "3":{"start":{"line":4,"column":4},"end":{"line":4,"column":20}},
                "4":{"start":{"line":5,"column":2},"end":{"line":5,"column":3}},
                "5":{"start":{"line":6,"column":2},"end":{"line":6,"column":15}}
            },
            "s":{"0":1,"1":2,"2":2,"3":0,"4":0,"5":1},
            "branchMap":{
                "0":{"loc":{"start":{"line":2,"column":2},"end":{"line":2,"column":25}}},
                "1":{"loc":{"start":{"line":3,"column":2},"end":{"line":3,"column":16}}}
            },
            "b":{"0":[2],"1":[0]}
        }}"#;
        let out = uncovered_istanbul_lines(json).unwrap();
        assert_eq!(out["/abs/widget.ts"], BTreeSet::from([3, 4, 5]));
    }

    #[test]
    fn istanbul_malformed_json_is_an_error() {
        assert!(uncovered_istanbul_lines("{ not json").is_err());
    }

    // --- Rust (cargo llvm-cov) — issue #37 ---

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
                },
            }],
        }
    }

    const RUST_FULL: RustThresholds = RustThresholds {
        regions: 100,
        lines: 100,
    };
    const RUST_MID: RustThresholds = RustThresholds {
        regions: 80,
        lines: 85,
    };

    #[test]
    fn rust_passes_when_both_metrics_meet_their_floor() {
        assert_eq!(
            evaluate_rust(&rust_report(100.0, 100.0), RUST_FULL),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_fails_on_the_one_metric_below_its_floor() {
        // 100% lines but only 70% regions: the regions floor catches what line
        // coverage misses — and only `regions` is named, not the metric that met
        // its floor.
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
    fn rust_tolerates_float_noise_at_the_floor() {
        // A percent a hair under the floor from rounding still passes.
        assert_eq!(
            evaluate_rust(&rust_report(99.999_999_999, 100.0), RUST_FULL),
            Outcome::Pass
        );
    }

    #[test]
    fn rust_fails_a_vacuous_run_that_measured_no_code() {
        // No regions in the denominator (a wrong path, or a crate that compiled
        // nothing): a vacuous run is a failure, never a silent pass.
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
        // `cargo llvm-cov` always emits one `data` entry; an empty array is a
        // malformed run, failed rather than treated as a pass.
        let report = LlvmCovReport { data: vec![] };
        assert!(matches!(evaluate_rust(&report, RUST_MID), Outcome::Fail(_)));
    }

    #[test]
    fn parses_a_cargo_llvm_cov_report() {
        // A realistic `--json --summary-only` export: regions/lines (enforced) plus
        // the functions block and the `type`/`version` the check ignores.
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
    fn rust_ignore_regex_is_none_when_nothing_is_exempt() {
        assert_eq!(ignore_filename_regex(&[]), None);
    }

    #[test]
    fn rust_ignore_regex_escapes_and_joins_exempt_paths() {
        // The caller passes already-resolved, `root`-relative paths; each is
        // regex-escaped (the `.` becomes `\.`) and joined into one alternation.
        let exempt = vec!["src/shim.rs".to_string(), "src/gen.rs".to_string()];
        assert_eq!(
            ignore_filename_regex(&exempt).as_deref(),
            Some(r"src/shim\.rs|src/gen\.rs")
        );
    }

    // --- Rust patch coverage (`cargo llvm-cov --lcov`) — issue #136 ---

    #[test]
    fn lcov_flags_an_unexecuted_line() {
        // The `below` fixture's shape: line 10 is the uncovered `else` arm.
        let lcov = "SF:/abs/grade.rs\nDA:6,1\nDA:7,1\nDA:8,1\nDA:10,0\nDA:12,1\nend_of_record\n";
        let out = uncovered_lcov_lines(lcov);
        assert_eq!(out["/abs/grade.rs"], BTreeSet::from([10]));
    }

    #[test]
    fn lcov_a_fully_covered_file_maps_to_an_empty_set() {
        let lcov = "SF:/abs/widget.rs\nDA:1,2\nDA:2,1\nend_of_record\n";
        let out = uncovered_lcov_lines(lcov);
        assert!(out["/abs/widget.rs"].is_empty());
    }

    #[test]
    fn lcov_groups_uncovered_lines_by_source_file() {
        let lcov =
            "SF:/abs/a.rs\nDA:3,0\nend_of_record\nSF:/abs/b.rs\nDA:5,1\nDA:6,0\nend_of_record\n";
        let out = uncovered_lcov_lines(lcov);
        assert_eq!(out["/abs/a.rs"], BTreeSet::from([3]));
        assert_eq!(out["/abs/b.rs"], BTreeSet::from([6]));
    }

    #[test]
    fn lcov_a_da_record_outside_a_file_is_ignored() {
        // A stray `DA` before any `SF` (shouldn't happen) contributes nothing
        // rather than panicking; the `end_of_record` closes the file.
        let lcov = "DA:9,0\nSF:/abs/a.rs\nDA:1,1\nend_of_record\nDA:2,0\n";
        let out = uncovered_lcov_lines(lcov);
        assert_eq!(out.len(), 1);
        assert!(out["/abs/a.rs"].is_empty());
    }

    #[test]
    fn lcov_a_checksummed_da_record_parses() {
        // LCOV may append a line checksum: `DA:<line>,<count>,<checksum>`.
        let lcov = "SF:/abs/a.rs\nDA:4,0,abc123\nend_of_record\n";
        let out = uncovered_lcov_lines(lcov);
        assert_eq!(out["/abs/a.rs"], BTreeSet::from([4]));
    }
}
