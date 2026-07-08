//! Coverage rule.
//!
//! Enforces the README's Coverage rule: a library's unit suite must meet the
//! configured floor, with test files excluded from the denominator. This module
//! is the deterministic core — given a parsed coverage report and the thresholds
//! from config, an `evaluate` function decides pass/fail. Producing the report
//! (shelling out to the language's coverage tool) is a thin layer on top, kept
//! separate so the guarantee is testable without that toolchain installed.
//!
//! Python uses coverage.py: a single total, branch coverage on. Given a
//! [`CoverageReport`] and [`Thresholds`], [`evaluate`] decides pass/fail, and
//! [`measure`] shells out to `coverage`. TypeScript is the twin: vitest
//! reports four independent metrics (lines / branches / functions / statements),
//! so it carries its own [`TypeScriptThresholds`], [`VitestReport`], and
//! [`evaluate_typescript`] / [`measure_typescript`] pair — sharing only the
//! [`Outcome`] type. Its subprocess layer shells out to `vitest`. Rust is
//! the third twin: `cargo llvm-cov` reports regions/lines (branch coverage is
//! experimental), so it carries [`RustThresholds`], [`LlvmCovReport`], and
//! [`evaluate_rust`] / [`measure_rust`]; its subprocess layer shells out to
//! `cargo llvm-cov`.
//!
//! Files exempted from coverage in config are omitted from the
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
/// `*_test.py` glob.
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
/// coverage). Unmodeled fields (metadata, per-function/class data) are
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
/// patch coverage reads to decide whether a changed line is covered.
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
    /// floor counts an arc toward changed-line branch coverage when its source
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
/// than vanishing. The per-file detail is what patch coverage reads; `omit`
/// is as in [`measure`] (an exempt file stays out of the run, so its changed
/// lines are lifted).
pub fn measure_patch_report(root: &Path, omit: &[String]) -> Result<CoverageReport> {
    run_coverage(root, omit, true)
}

/// Run the Python unit suite under coverage.py in `root` and return the parsed report
/// with its per-file `files` detail — measuring only the files the suite imports (no
/// `--source=.`), exactly as the whole-tree floor [`measure`] does. The line-scoped
/// exemption path reads this: it recomputes the floor over the measured lines
/// minus the exempt ones, so it must see the same file set [`measure`] does (an
/// untested-but-unimported file is out of scope for both), not the wider `--source=.`
/// set [`measure_patch_report`] uses for the diff. `omit` is as in [`measure`].
pub fn measure_report(root: &Path, omit: &[String]) -> Result<CoverageReport> {
    run_coverage(root, omit, false)
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

// TypeScript (vitest).
//
// The TypeScript twin of the Python rule above. vitest reports four independent
// metrics rather than Python's single total-plus-branch, so it carries its own
// thresholds, report shape, and evaluate/measure pair; only `Outcome` is shared.
// The split is the same: a pure `evaluate_typescript` over a parsed json-summary
// report, and a thin `measure_typescript` that shells out to vitest to produce
// one — so the enforcement core is testable without a Node toolchain.

/// What vitest measures: every TypeScript source under the scanned root. The
/// braces are a vitest (picomatch) glob, expanded by vitest, not the shell.
const TS_INCLUDE: &str = "**/*.{ts,tsx,mts,cts}";

/// The project's own installed vitest's default coverage excludes (test files,
/// declaration files, build-tool config files, `dist/`, `node_modules/`, …),
/// resolved live via Node rather than hand-maintained here.
///
/// Passing *any* `--coverage.exclude` value to vitest replaces its built-in
/// default list rather than extending it — so a rule-owned exclude flag (the
/// colocated test glob; a config-driven `coverage` exemption) would otherwise
/// silently un-exclude every default the provider ships, including its own
/// `**/{vite,vitest,eslint,...}.config.*` pattern. That default list is
/// exactly the ecosystem knowledge this tool has no business re-enumerating —
/// it's resolved from whatever vitest version `root` actually has installed,
/// so it can never go stale relative to it.
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
    let excludes: Vec<String> = serde_json::from_slice(&run.stdout).with_context(|| {
        format!(
            "vitest's default coverage excludes were not a JSON string array — got: {}",
            String::from_utf8_lossy(&run.stdout)
        )
    })?;
    // A couple of vitest's own default patterns embed a literal NUL byte (its
    // virtual-module boundary markers, e.g. `**/\0*`) — meaningless as a glob
    // against real files, and a NUL byte can't be passed as a process argument
    // at all, so those entries are dropped rather than sent to `Command::arg`.
    Ok(excludes.into_iter().filter(|p| !p.contains('\0')).collect())
}

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
/// of the `report_file` the `reporter` wrote. Shared by the floor (the
/// `json-summary` → `coverage-summary.json` pair) and patch coverage (the
/// detailed `json` → `coverage-final.json` Istanbul pair) — the two differ only in
/// the reporter and how they parse it.
///
/// v8 coverage is written to an out-of-tree temp dir so the scanned tree stays
/// pristine. `include` scopes measurement to the sources under `root`; vitest's
/// own default excludes (test files, declaration files, build-tool config
/// files, …, resolved live — see [`vitest_default_excludes`]) and the config
/// `exclude` paths are excluded from the denominator. `all=true` counts source
/// files the suite never imported, so an untested file is measured (lowering
/// the floor / showing as uncovered) rather than vanishing. `--no-cache` keeps
/// vitest from writing a cache into the tree.
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
        // `--no-install`, never `--yes`: run the project's own vitest (resolved via
        // Node's parent-dir lookup) and refuse to download anything. With `--yes` a
        // missing vitest would be silently fetched; the TS arm must fail clean like the
        // coverage.py / cargo-llvm-cov arms, which invoke their binary directly.
        .args(["--no-install", "vitest", "run", "--no-cache"])
        .args(["--coverage.enabled", "--coverage.provider=v8"])
        .arg(format!("--coverage.reporter={reporter}"))
        .arg("--coverage.all=true")
        .arg(format!(
            "--coverage.reportsDirectory={}",
            reports.0.display()
        ))
        .arg(format!("--coverage.include={TS_INCLUDE}"));
    // Passing any `--coverage.exclude` replaces vitest's own default exclude
    // list rather than extending it, so its defaults are resolved and passed
    // back explicitly, alongside the config-driven exemption paths.
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

    let path = reports.0.join(report_file);
    std::fs::read_to_string(&path).with_context(|| {
        format!(
            "reading vitest coverage report `{}` (did the run produce a {reporter} report?)",
            path.display()
        )
    })
}

// TypeScript diff-scoped coverage detail.
//
// What the diff-scoped floor (`crate::patch_coverage::measure_typescript`) reads:
// per-file coverage detail for the four vitest metrics. vitest's `json-summary`
// gives only per-file totals, so this measures with the detailed `json` (Istanbul
// `coverage-final.json`) reporter and reduces each file to the per-statement /
// per-branch-arm / per-function `(line, covered)` counts the floor's ratio needs.

/// One file's entry in a vitest v8 `coverage-final.json` (Istanbul) report, pared
/// to what patch coverage reads: the statement / branch / function maps and their
/// hit counts. Unmodeled fields (`path`, per-node metadata) are ignored.
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
    /// Branch id → per-arm execution counts (one count per branch arm).
    #[serde(default)]
    b: BTreeMap<String, Vec<u64>>,
    /// Function id → declaration location. A function whose hit count in `f` is `0`
    /// was never called. The diff-scoped floor reads this via
    /// [`istanbul_patch_detail`].
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

/// A branch entry — only its location (whose start line is the branch's source
/// line) matters; the `type` and per-path `locations` are ignored.
#[derive(Debug, Clone, Deserialize)]
struct IstanbulBranch {
    loc: IstanbulSpan,
}

/// A function entry — only its declaration's start line (the function's source
/// line) matters; the `name`, `loc`, and top-level `line` are ignored. vitest's
/// v8 export shapes this as `{"name":.., "decl":{"start":{"line":N,..},..}, ..}`.
#[derive(Debug, Clone, Deserialize)]
struct IstanbulFn {
    decl: IstanbulSpan,
}

/// Per-file coverage detail from a vitest v8 `coverage-final.json` (Istanbul)
/// report — the counts the diff-scoped floor needs. Each entry carries the
/// Istanbul maps reduced to `(line, …, covered)` tuples, so the pure
/// [`crate::patch_coverage::evaluate_patch_typescript`] can restrict each of the
/// four metrics to the changed lines.
#[derive(Debug, Clone, Default)]
pub struct TsPatchCoverage {
    /// One per `statementMap` entry: `(start_line, end_line, covered)` — `covered`
    /// is `s[id] > 0`. A statement counts toward the diff when any line it spans is
    /// a changed line.
    pub statements: Vec<(u64, u64, bool)>,
    /// One per branch **arm**: `(source_line, covered)` — `source_line` is the
    /// branch's `loc.start.line` (shared by every arm) and `covered` is that arm's
    /// count `> 0`. An arm counts toward the diff when its source line is changed.
    pub branch_arms: Vec<(u64, bool)>,
    /// One per `fnMap` entry: `(decl_line, covered)` — `decl_line` is `decl.start.line`
    /// and `covered` is `f[id] > 0`. A function counts toward the diff when its
    /// declaration line is changed.
    pub functions: Vec<(u64, bool)>,
}

/// Run the TypeScript unit suite under vitest in `root` and return the per-file
/// coverage detail for the four metrics — keyed by the absolute path vitest
/// reports, the caller re-keying to `root`-relative to match the diff. Reads the
/// Istanbul report for the diff-scoped floor: the per-statement /
/// per-branch-arm / per-function `(line, covered)` detail the floor's ratio needs.
/// `exclude` is the `coverage`-rule exemptions,
/// dropped from the run so an exempt file's changed lines are lifted. `npx`
/// resolves the project-local `vitest`, so it and `@vitest/coverage-v8` must be
/// installed under `root`.
pub fn measure_patch_typescript_detail(
    root: &Path,
    exclude: &[String],
) -> Result<BTreeMap<String, TsPatchCoverage>> {
    let json = run_vitest_coverage(root, exclude, "json", "coverage-final.json")?;
    istanbul_patch_detail(&json)
}

/// Pure: per-file [`TsPatchCoverage`] from a vitest v8 `coverage-final.json`
/// (Istanbul) report. Keyed by the path vitest reports (absolute). A file present
/// but with no statements/branches/functions maps to an empty `TsPatchCoverage`.
fn istanbul_patch_detail(json: &str) -> Result<BTreeMap<String, TsPatchCoverage>> {
    let files: BTreeMap<String, IstanbulFile> = serde_json::from_str(json)
        .context("parsing vitest coverage-final (Istanbul) JSON report")?;
    let mut out = BTreeMap::new();
    for (path, file) in files {
        let mut detail = TsPatchCoverage::default();
        // Each statement → (start, end, covered): covered when its count is > 0.
        for (id, span) in &file.statement_map {
            let covered = file.s.get(id).is_some_and(|&count| count > 0);
            detail
                .statements
                .push((span.start.line, span.end.line, covered));
        }
        // Each branch arm → (source_line, covered): the branch's location start line
        // (shared by every arm) with that arm's count > 0. v8 may model a branch as
        // a single arm (a `[count]` array) or several (`[arm0, arm1, …]`); one tuple
        // per arm either way.
        for (id, branch) in &file.branch_map {
            let line = branch.loc.start.line;
            if let Some(counts) = file.b.get(id) {
                for &count in counts {
                    detail.branch_arms.push((line, count > 0));
                }
            }
        }
        // Each function → (decl_line, covered): the declaration's start line with
        // its call count > 0.
        for (id, function) in &file.fn_map {
            let covered = file.f.get(id).is_some_and(|&count| count > 0);
            detail.functions.push((function.decl.start.line, covered));
        }
        out.insert(path, detail);
    }
    Ok(out)
}

// Rust (cargo llvm-cov).
//
// The Rust twin of the rules above. `cargo llvm-cov` reports LLVM source-based
// coverage as regions + lines (branch coverage is still experimental), so the
// Rust rule carries its own thresholds and `measure_rust` entry point; only the
// `Outcome` type is shared. Mirroring the Python/TypeScript split, a pure
// `evaluate_rust` over a parsed llvm-cov export and the thin subprocess layer
// that produces one land with the implementation.

/// The `cargo llvm-cov` coverage floors, from a `[rust].coverage` table (or the
/// zero-config default). `lines` is always enforced; the rest are opt-in — `None`
/// skips the check (the zero-config default floors lines only). A `branch`
/// floor adds `--branch` to the run, which instruments only on a nightly
/// toolchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RustThresholds {
    pub regions: Option<u8>,
    pub lines: u8,
    pub functions: Option<u8>,
    pub branch: Option<u8>,
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

/// The `totals` block of an llvm-cov export — the metrics this rule can enforce:
/// regions and lines always, `functions` and (under `--branch`) `branches` when
/// their opt-in floors are set. llvm-cov also reports `instantiations` and
/// `mcdc`, which the check ignores. `branches` is optional-with-default so an
/// export from a run without branch instrumentation still parses (it then reads
/// `count = 0`).
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LlvmCovTotals {
    pub regions: LlvmCovMetric,
    pub lines: LlvmCovMetric,
    pub functions: LlvmCovMetric,
    #[serde(default)]
    pub branches: Option<LlvmCovMetric>,
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
    // `regions`, `functions`, and `branch` are opt-in: the zero-config
    // default floors lines only, so each check is skipped unless a config set its
    // floor.
    let mut checks: Vec<(&str, f64, u8)> = Vec::new();
    if let Some(regions) = thresholds.regions {
        checks.push(("regions", totals.regions.percent, regions));
    }
    checks.push(("lines", totals.lines.percent, thresholds.lines));
    if let Some(functions) = thresholds.functions {
        checks.push(("functions", totals.functions.percent, functions));
    }
    if let Some(branch) = thresholds.branch {
        // The floor's run added `--branch` (a failed instrumentation is a run
        // error, surfaced before this point), so a zero branch denominator here
        // means the crate has no branch points — vacuously satisfied, mirroring
        // the diff-scoped floors' empty-denominator rule.
        if let Some(branches) = totals.branches.filter(|metric| metric.count > 0) {
            checks.push(("branches", branches.percent, branch));
        }
    }
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
/// Shells out to `cargo llvm-cov --lib --json --summary-only`, omitting every path in
/// `ignore` from the denominator (a single `--ignore-filename-regex`), then
/// evaluates the export. `ignore` holds the `coverage`-rule exemptions resolved
/// from config, as `root`-relative paths; `features` the `[rust] features` list to
/// enable on the run. `cargo-llvm-cov` must be installed.
pub fn measure_rust(
    root: &Path,
    thresholds: RustThresholds,
    ignore: &[String],
    features: &[String],
) -> Result<Outcome> {
    let report = run_llvm_cov(root, ignore, features, thresholds.branch.is_some())?;
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
/// `--summary-only` export — the totals the floor checks. `branch` adds
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
/// `format` args (`["--json", "--summary-only"]` for the whole-tree floor's totals,
/// `["--json"]` for the diff-scoped floor's per-region detail) and return its
/// stdout. Shared by the whole-tree floor and the diff-scoped floor,
/// so both measure the same unit-only slice.
///
/// The build goes to an out-of-tree target dir (via `CARGO_TARGET_DIR`) so the
/// scanned crate stays pristine; the `coverage`-rule exemptions become one
/// `--ignore-filename-regex`; the `[rust] features` list is enabled on the run so
/// `#[cfg(feature = ...)]` code is compiled and measured; and the outer
/// run's instrumentation env is stripped for nested-run hygiene (the loop below
/// explains why).
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
        // `--lib` scopes the run to the unit suite — the library target with its
        // inline `#[cfg(test)]` modules, the tool's definition of a Rust unit.
        // cargo-llvm-cov's default runs every test target, which lets the
        // integration tier under `tests/` pad the number.
        .arg("--lib")
        .args(format)
        .env("CARGO_TARGET_DIR", &target.0);
    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }
    if branch {
        // A configured branch floor measures branch outcomes; the flag
        // instruments only on a nightly toolchain — the error below names that.
        command.arg("--branch");
    }
    if let Some(regex) = ignore_filename_regex(root, ignore) {
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
        // Toolchain hygiene: when this tool is itself spawned by a cargo
        // process (a test harness, an xtask), cargo/rustup export the *spawning*
        // toolchain into the environment, and rustup gives those variables
        // precedence over the scanned crate's own `rust-toolchain.toml`. The
        // scanned crate's pin must decide — a branch-floor crate pins nightly
        // there — so the inherited selection is dropped and rustup resolves
        // fresh from the crate's directory.
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
        // A branch-floor run that fails is most often a stable toolchain (the
        // `--branch` instrumentation is nightly-only), so name the requirement
        // alongside the run's own output.
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

/// Per-file region detail from a `cargo llvm-cov --json` export — the per-region
/// counts the diff-scoped floor needs. Each entry carries one
/// `(start_line, end_line, covered)` tuple per code region, so the pure
/// [`crate::patch_coverage::evaluate_patch_rust`] can restrict both the regions and
/// lines metrics to the changed lines.
#[derive(Debug, Clone, Default)]
pub struct RustPatchCoverage {
    /// One per code region (a `kind == 0` region of the LLVM export):
    /// `(start_line, end_line, covered)` — `covered` is the region's
    /// `executionCount > 0`. A region counts toward the diff when any line it spans
    /// is a changed line.
    pub regions: Vec<(u64, u64, bool)>,
}

/// A full `cargo llvm-cov --json` export (LLVM's `llvm.coverage.json.export`),
/// modeling the per-function region detail the diff-scoped floor needs — separate
/// from [`LlvmCovReport`], which keeps only the `totals` the whole-tree floor
/// reads. A single run produces one `data` entry; unmodeled fields (`totals`,
/// `type`, `version`) are ignored.
#[derive(Debug, Clone, Deserialize)]
struct LlvmCovExport {
    data: Vec<LlvmCovExportData>,
}

/// One export entry — its per-function `functions` block carries the regions (the
/// `--summary-only` runs that feed [`LlvmCovReport`] omit it), and its `files` block
/// names the measured files. `--ignore-filename-regex` drops an exempt file from
/// `files` but *not* from `functions` (the regions array is unfiltered), so the
/// `files` list is the allowlist [`llvm_cov_patch_detail`] restricts the regions to.
#[derive(Debug, Clone, Deserialize)]
struct LlvmCovExportData {
    files: Vec<LlvmCovExportFile>,
    functions: Vec<LlvmCovFunction>,
}

/// One measured file in the export's `files` block — only its `filename` (the
/// absolute path) is needed, to build the not-ignored allowlist. The per-file
/// `segments` / `summary` detail is ignored (the regions come from `functions`).
#[derive(Debug, Clone, Deserialize)]
struct LlvmCovExportFile {
    filename: String,
}

/// One function's coverage in the export: the source files it spans (`filenames`,
/// indexed by a region's `fileID`) and its regions. Each region is a flat array
/// `[lineStart, colStart, lineEnd, colEnd, executionCount, fileID, expandedFileID,
/// kind]`; the fields are read positionally in [`llvm_cov_patch_detail`].
#[derive(Debug, Clone, Deserialize)]
struct LlvmCovFunction {
    filenames: Vec<String>,
    regions: Vec<Vec<i64>>,
}

/// Run the Rust unit suite under `cargo llvm-cov` in `root` and return the per-file
/// region detail — keyed by the absolute path llvm-cov reports, the caller re-keying
/// to `root`-relative to match the diff. Reads the full `--json` export for the
/// diff-scoped floor: the per-region `(line, covered)` detail the floor's
/// regions metric needs. `ignore` is the `coverage`-rule exemptions, dropped
/// from the run so an exempt file's changed lines are lifted. `cargo-llvm-cov` must
/// be installed.
pub fn measure_patch_rust_detail(
    root: &Path,
    ignore: &[String],
    features: &[String],
) -> Result<BTreeMap<String, RustPatchCoverage>> {
    // The diff-scoped floor judges regions + lines, so its run never adds
    // `--branch`.
    llvm_cov_patch_detail(&run_cargo_llvm_cov(
        root,
        ignore,
        &["--json"],
        features,
        false,
    )?)
}

/// Pure: per-file [`RustPatchCoverage`] from a `cargo llvm-cov --json` export.
/// Keyed by the path llvm-cov reports (absolute). Walks every function's regions;
/// for each region:
///   - **skips** any region whose file is not in the export's `files` allowlist —
///     `--ignore-filename-regex` drops an exempt file from `files` (and the totals)
///     but leaves it in the unfiltered `functions` regions, so honoring the
///     exemption means intersecting with `files`. A run with nothing exempt lists
///     every measured file, so this is a no-op there.
///   - **skips** any region whose `kind` (index 7) is not `0` — only `kind == 0`
///     code regions count toward coverage (gap / expansion / skipped / branch
///     regions carry no line-coverage signal). The kept count (with nothing
///     ignored) matches the `totals.regions.count` a `--summary-only` run reports.
///   - reads `start_line = region[0]`, `end_line = region[2]`,
///     `covered = region[4] > 0`, and the file `filenames[region[5]]` (the
///     `fileID`), pushing `(start_line, end_line, covered)` under that file.
///
/// A region array with fewer than 8 elements (malformed — never seen from
/// llvm-cov) is skipped rather than panicking on an index, as is one whose `fileID`
/// is out of range for its `filenames`.
fn llvm_cov_patch_detail(json: &str) -> Result<BTreeMap<String, RustPatchCoverage>> {
    let export: LlvmCovExport =
        serde_json::from_str(json).context("parsing cargo llvm-cov JSON export")?;
    let mut out: BTreeMap<String, RustPatchCoverage> = BTreeMap::new();
    for data in &export.data {
        // The `files` block honors `--ignore-filename-regex`; the `functions` regions
        // do not, so restrict to the measured (not-ignored) files.
        let measured: BTreeSet<&str> = data.files.iter().map(|f| f.filename.as_str()).collect();
        for function in &data.functions {
            for region in &function.regions {
                // A code region carries eight fields; anything shorter is malformed
                // (never emitted by llvm-cov) and skipped rather than indexed.
                if region.len() < 8 {
                    continue;
                }
                // Only `kind == 0` (a code region) contributes to line coverage;
                // gap (1) / expansion (2) / skipped / branch regions are ignored.
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
                // Skip a file the run ignored (absent from `files`) so a `coverage`
                // exemption drops its regions, lifting its changed lines.
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

/// The single `--ignore-filename-regex` value for the run, or `None` when nothing
/// is exempt. `cargo llvm-cov` takes one regex, so the `coverage`-exempt paths are
/// each regex-escaped (matched literally, not as a pattern) and joined with `|`. An
/// exempt file leaves the denominator with its reason recorded in config — an
/// auditable omission, not a silent ignore-glob.
///
/// llvm-cov reports absolute filenames and `--ignore-filename-regex` is a substring
/// search, so each `root`-relative exempt path is anchored to its full path under
/// `root` and terminated with `$`. Substring-matching an unanchored `src/a.rs` would
/// over-match a workspace member's `member/src/a.rs` (and a nested `src/xsrc/a.rs`);
/// the full-path anchor drops only the exempted file.
fn ignore_filename_regex(root: &Path, ignore: &[String]) -> Option<String> {
    if ignore.is_empty() {
        return None;
    }
    Some(
        ignore
            .iter()
            .map(|rel| {
                // Anchor to the file's absolute path: the exempt entry is validated
                // to exist under `root`, so `canonicalize` resolves it to the same
                // absolute form llvm-cov reports; the fallback keeps the anchor
                // deterministic when the path can't be resolved (e.g. in tests).
                let full = root.join(rel);
                let full = full.canonicalize().unwrap_or(full);
                format!("{}$", regex_escape(&full.to_string_lossy()))
            })
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
        // missing lines and `[src, dst]` branch pairs patch coverage reads.
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

    /// Like [`rust_report`] with explicit functions/branches metrics — the opt-in
    /// floors' tests. `branches: (count, percent)` so the vacuous
    /// zero-denominator case is constructible.
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
        // The opt-in functions floor is judged on the export's functions
        // total: 66.67% fails a 100 floor naming the metric, and clears 60.
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
        // The opt-in branch floor is judged on the branches total of a
        // `--branch` run: 50% fails a 100 floor naming the metric, and clears 50.
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
        // A successful `--branch` run over a crate with no branch points reports a
        // zero branch denominator — vacuously satisfied, mirroring the diff-scoped
        // floors' empty-denominator rule (a failed instrumentation is a run error,
        // never a zero count here).
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
    fn rust_skips_the_region_check_when_regions_is_opt_out() {
        // The zero-config default sets `regions: None`, so only lines are
        // enforced: a crate at 100% lines clears the floor even with low regions.
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
        // `regions: None` skips only the region check — the line floor still bites.
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
    fn llvm_cov_patch_detail_reads_code_regions_per_file() {
        // A realistic full `--json` export: one function spanning two regions on
        // `/abs/grade.rs` — line 6 covered (execCount 1), line 10 the uncovered
        // `else` arm (execCount 0). Both are `kind == 0` code regions, indexed back
        // to `filenames[0]`.
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
        // Only `kind == 0` counts: a gap region (kind 1) and an expansion region
        // (kind 2) on the same function are ignored, leaving just the one code region.
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
        // A region's `fileID` (index 5) selects its file from the function's
        // `filenames`; two regions under the same function land in different files.
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
        // A region array shorter than the eight fields (never seen from llvm-cov) is
        // skipped rather than panicking on an index; the well-formed one survives.
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
        // A region spanning lines 3–5 keeps both endpoints, so a changed line
        // anywhere in 3..=5 can count it.
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
        // `--ignore-filename-regex` drops an exempt file from `files` but leaves its
        // regions in `functions`; restricting to the `files` allowlist lifts them, so
        // the ignored file contributes nothing while the kept file still does.
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
    fn rust_ignore_regex_is_none_when_nothing_is_exempt() {
        assert_eq!(ignore_filename_regex(Path::new("/repo"), &[]), None);
    }

    #[test]
    fn rust_ignore_regex_anchors_each_exempt_path_to_its_full_path() {
        // The caller passes already-resolved, `root`-relative paths; each is joined
        // under `root`, regex-escaped (the `.` becomes `\.`), and `$`-anchored, then
        // joined into one alternation. (`/repo` doesn't exist, so `canonicalize`
        // falls back to the plain join — deterministic for the assertion.)
        let exempt = vec!["src/shim.rs".to_string(), "src/gen.rs".to_string()];
        assert_eq!(
            ignore_filename_regex(Path::new("/repo"), &exempt).as_deref(),
            Some(r"/repo/src/shim\.rs$|/repo/src/gen\.rs$")
        );
    }

    /// Model llvm-cov's substring `--ignore-filename-regex` for the fully-escaped,
    /// optionally end-anchored literals this tool emits: unescape the literal, honor a
    /// trailing `$` end-anchor, else substring-match. One matching alternative ignores
    /// the file.
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
    fn rust_ignore_regex_does_not_over_match_a_member_with_the_same_suffix() {
        // llvm-cov's `--ignore-filename-regex` is a substring search, so an entry for a
        // top-level `src/a.rs` must not also drop a workspace member's `member/src/a.rs`
        // — nor a nested `src/xsrc/a.rs` that merely shares the suffix.
        let regex = ignore_filename_regex(Path::new("/repo"), &["src/a.rs".to_string()]).unwrap();
        // The exempted file itself is still dropped.
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
