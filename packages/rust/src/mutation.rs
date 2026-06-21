//! Mutation testing for Rust (`unit mutation --language rust`, #201) — the rung
//! above coverage. A test that *runs* a line still passes if you delete its
//! assertions; a surviving mutant proves it. This module wraps
//! [cargo-mutants](https://github.com/sourcefrog/cargo-mutants): it runs the engine,
//! reads its `outcomes.json`, and reports the **surviving** mutants the suite failed
//! to catch.
//!
//! The gate is **binary, not a percentage** (equivalent mutants make a fixed score
//! unreachable, and a score isn't comparable across engines) and on by default: any
//! *un-exempted* surviving mutant is a finding. This module stays a pure measurement —
//! [`measure_rust`] returns the survivors and [`unexplained_survivors`] is the pure
//! core over a parsed report; the CLI layer turns a non-empty result into the failure.
//!
//! Diff-scoping (`--base`) is delegated to cargo-mutants' own `--in-diff`: the
//! `<base>...HEAD` diff is written out and passed through, so only mutants on changed
//! lines are tested ("no unexplained surviving mutant on the lines you touched").

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// A surviving mutant — a mutation the unit suite ran but failed to catch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Survivor {
    /// The mutated file, as cargo-mutants reports it (crate-root-relative, `/`-separated).
    pub file: String,
    /// The 1-based line the mutation starts on.
    pub line: u32,
    /// cargo-mutants' human description (e.g. `replace > with == in is_positive`).
    pub description: String,
}

/// A cargo-mutants `outcomes.json` export, pared to what the rule reads. Unmodeled
/// fields (`total_mutants`, `caught`, timings, …) are ignored.
#[derive(Debug, Clone, Deserialize)]
pub struct MutantsReport {
    pub outcomes: Vec<MutantOutcome>,
}

/// One scenario's outcome. `summary` is cargo-mutants' result word — `Success` for the
/// unmutated baseline, `CaughtMutant` / `MissedMutant` (and `Timeout` / `Unviable`)
/// for each mutant.
#[derive(Debug, Clone, Deserialize)]
pub struct MutantOutcome {
    pub summary: String,
    pub scenario: Scenario,
}

/// The scenario a result came from: the unmutated baseline, or one mutant. Matches
/// cargo-mutants' externally-tagged JSON (`"Baseline"` vs `{"Mutant": {…}}`).
#[derive(Debug, Clone, Deserialize)]
pub enum Scenario {
    Baseline,
    Mutant(MutantInfo),
}

/// The mutant a scenario describes, pared to the location + description the report
/// needs. cargo-mutants also carries `function`, `genre`, `package`, `replacement`;
/// those are ignored.
#[derive(Debug, Clone, Deserialize)]
pub struct MutantInfo {
    pub file: String,
    pub span: Span,
    pub name: String,
}

/// A source span; only the start line is read.
#[derive(Debug, Clone, Deserialize)]
pub struct Span {
    pub start: LineCol,
}

/// A line/column position; only the line is read.
#[derive(Debug, Clone, Deserialize)]
pub struct LineCol {
    pub line: u32,
}

/// Parse a cargo-mutants `outcomes.json` export.
pub fn parse_mutants_report(json: &str) -> Result<MutantsReport> {
    serde_json::from_str(json).context("parsing cargo-mutants outcomes.json")
}

/// The surviving mutants not lifted by a `mutation` exemption — the rule's findings.
///
/// A survivor is a `MissedMutant` outcome (the suite ran the mutated code but no test
/// failed). `exempt` is the resolved set of `mutation`-rule exempt paths (crate-root
/// relative); a survivor in an exempt file is dropped (an equivalent or deliberately
/// defensive mutation, lifted with a reason). `Timeout` / `Unviable` are *not*
/// survivors — a timeout is inconclusive, not a pass, and an unviable mutant never
/// compiled.
pub fn unexplained_survivors(report: &MutantsReport, exempt: &[String]) -> Vec<Survivor> {
    let survivors = report
        .outcomes
        .iter()
        .filter_map(|outcome| {
            if outcome.summary != "MissedMutant" {
                return None;
            }
            let Scenario::Mutant(mutant) = &outcome.scenario else {
                return None;
            };
            Some(Survivor {
                file: mutant.file.clone(),
                line: mutant.span.start.line,
                description: mutant.name.clone(),
            })
        })
        .collect();
    evaluate(survivors, exempt)
}

/// The shared evaluation core both engines feed: drop the survivors lifted by a
/// `mutation` exemption (a file-path match), leaving the rule's findings. Each engine
/// produces the raw survivor list its own way ([`unexplained_survivors`] from a
/// cargo-mutants report, [`stryker_survivors`] from a Stryker report); this applies
/// the reason-required exemptions identically across languages.
pub fn evaluate(survivors: Vec<Survivor>, exempt: &[String]) -> Vec<Survivor> {
    survivors
        .into_iter()
        .filter(|survivor| !exempt.iter().any(|path| path == &survivor.file))
        .collect()
}

/// Run cargo-mutants over the crate at `root` and return its un-exempted survivors.
///
/// With `base` set, only mutants on the `<base>...HEAD` changed lines are tested (via
/// cargo-mutants' `--in-diff`); without it, the whole crate. `exempt` is the resolved
/// `mutation`-rule exempt paths. `cargo-mutants` must be installed.
pub fn measure_rust(root: &Path, exempt: &[String], base: Option<&str>) -> Result<Vec<Survivor>> {
    let out = MutantsOut::new();
    let diff = match base {
        Some(base) => Some(write_base_diff(root, base, &out)?),
        None => None,
    };
    run_cargo_mutants(root, &out.0, diff.as_deref())?;
    let outcomes = out.0.join("mutants.out").join("outcomes.json");
    let json = std::fs::read_to_string(&outcomes).with_context(|| {
        format!(
            "reading cargo-mutants outcomes at `{}` — the run wrote none",
            outcomes.display()
        )
    })?;
    let report = parse_mutants_report(&json)?;
    Ok(unexplained_survivors(&report, exempt))
}

/// A Stryker `mutation.json` report (the mutation-testing-elements schema), pared to
/// the fields the rule reads. Unmodeled keys (`schemaVersion`, `thresholds`, `source`,
/// `testFiles`, `projectRoot`, `config`, …) are ignored.
#[derive(Debug, Clone, Deserialize)]
pub struct StrykerReport {
    /// Per-file mutants, keyed by project-relative, `/`-separated path.
    pub files: BTreeMap<String, StrykerFile>,
}

/// One file's mutants in a Stryker report.
#[derive(Debug, Clone, Deserialize)]
pub struct StrykerFile {
    #[serde(default)]
    pub mutants: Vec<StrykerMutant>,
}

/// One mutant, pared to the location + status + description the rule needs. Stryker
/// also carries `id`, `coveredBy`, `static`, `testsCompleted`, …; those are ignored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrykerMutant {
    pub mutator_name: String,
    #[serde(default)]
    pub replacement: Option<String>,
    pub status: String,
    pub location: StrykerLocation,
}

/// A mutant's source location; only the start line is read (reusing [`LineCol`], whose
/// extra `column` field Stryker also provides and serde ignores).
#[derive(Debug, Clone, Deserialize)]
pub struct StrykerLocation {
    pub start: LineCol,
}

/// Parse a Stryker `mutation.json` report.
pub fn parse_stryker_report(json: &str) -> Result<StrykerReport> {
    serde_json::from_str(json).context("parsing Stryker mutation.json")
}

/// The surviving mutants in a Stryker report — the raw list before exemptions.
///
/// A survivor is a `Survived` mutant (a test ran the mutated code but none failed) or a
/// `NoCoverage` one (no test exercised it at all — worse). `Killed` / `Timeout` are
/// caught; `CompileError` / `RuntimeError` never produced a viable mutant; `Ignored` /
/// `Pending` are out of scope. (Mirrors the cargo-mutants `MissedMutant`-only rule.)
pub fn stryker_survivors(report: &StrykerReport) -> Vec<Survivor> {
    let mut survivors = Vec::new();
    for (file, contents) in &report.files {
        for mutant in &contents.mutants {
            if mutant.status != "Survived" && mutant.status != "NoCoverage" {
                continue;
            }
            let description = match &mutant.replacement {
                Some(replacement) => {
                    format!("{} (-> {})", mutant.mutator_name, one_line(replacement))
                }
                None => mutant.mutator_name.clone(),
            };
            survivors.push(Survivor {
                file: file.clone(),
                line: mutant.location.start.line,
                description,
            });
        }
    }
    survivors
}

/// Collapse a (possibly multi-line) replacement to a single trimmed line, capped, so a
/// survivor's one-line description stays readable.
fn one_line(replacement: &str) -> String {
    let flat = replacement.split_whitespace().collect::<Vec<_>>().join(" ");
    const MAX: usize = 60;
    if flat.chars().count() > MAX {
        format!("{}…", flat.chars().take(MAX).collect::<String>())
    } else {
        flat
    }
}

/// Run Stryker over the TypeScript project at `root` and return its un-exempted
/// survivors — the TS arm of the mutation rule (#202), parity with [`measure_rust`].
///
/// With `base` set, only mutants on the `<base>...HEAD` changed lines are tested —
/// Stryker has no native git-diff mode, so the changed lines become `--mutate
/// <file>:<line>-<line>` ranges (line granularity, matching cargo-mutants' `--in-diff`).
/// Without it, the project's configured `mutate` set runs. `exempt` is the resolved
/// `mutation`-rule exempt paths. Stryker must be installed / resolvable.
pub fn measure_typescript(
    root: &Path,
    exempt: &[String],
    base: Option<&str>,
) -> Result<Vec<Survivor>> {
    let mutate = match base {
        Some(base) => {
            let ranges = mutate_ranges(root, base)?;
            // Nothing mutatable changed on the diff: no run, no survivors.
            if ranges.is_empty() {
                return Ok(Vec::new());
            }
            Some(ranges)
        }
        None => None,
    };
    let json = run_stryker(root, mutate.as_deref())?;
    let report = parse_stryker_report(&json)?;
    Ok(evaluate(stryker_survivors(&report), exempt))
}

/// Build the Stryker `--mutate` specs scoping a run to the `<base>...HEAD` changed
/// lines: each mutatable source file's contiguous runs of changed lines become a
/// `<file>:<start>-<end>` range (Stryker's line-range form). Reuses the patch-coverage
/// diff parser. Test and declaration files are filtered out — Stryker's configured
/// `mutate` set normally excludes them, but passing `--mutate` replaces that set.
fn mutate_ranges(root: &Path, base: &str) -> Result<Vec<String>> {
    let changed = crate::patch_coverage::changed_lines(root, base)?;
    let mut specs = Vec::new();
    for (file, lines) in changed {
        if !is_mutatable_ts(&file) {
            continue;
        }
        for (start, end) in contiguous_runs(&lines) {
            specs.push(format!("{file}:{start}-{end}"));
        }
    }
    Ok(specs)
}

/// Whether a changed file is a TypeScript/JavaScript *source* Stryker should mutate — a
/// `.ts`/`.tsx`/`.mts`/`.cts`/`.js`/`.jsx`/`.mjs`/`.cjs` file that is not a declaration
/// (`.d.ts`) or a test (`.test.` / `.spec.`).
fn is_mutatable_ts(file: &str) -> bool {
    let is_source = [".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"]
        .iter()
        .any(|ext| file.ends_with(ext));
    let is_decl = file.ends_with(".d.ts");
    let is_test = file.contains(".test.") || file.contains(".spec.");
    is_source && !is_decl && !is_test
}

/// Fold a sorted set of line numbers into inclusive `(start, end)` contiguous runs.
fn contiguous_runs(lines: &BTreeSet<u64>) -> Vec<(u64, u64)> {
    let mut runs: Vec<(u64, u64)> = Vec::new();
    for &line in lines {
        match runs.last_mut() {
            Some(run) if run.1 + 1 == line => run.1 = line,
            _ => runs.push((line, line)),
        }
    }
    runs
}

/// Run Stryker over `root` (resolving the project's own config) with the json reporter,
/// returning the contents of its `mutation.json`. `mutate`, when set, scopes the run to
/// `--mutate` line ranges. The report goes to Stryker's default
/// `reports/mutation/mutation.json`; it's read and then pruned (the file and any empty
/// parents) so the scanned tree stays pristine and a populated `reports/` is untouched.
///
/// Stryker's exit code is *not* trusted to mean "no survivors": a configured
/// `thresholds.break` makes it exit non-zero on a low score, which is exactly the
/// survivor case the rule reports on. So the report is read whenever it exists; only a
/// missing report (a real run failure) is fatal.
fn run_stryker(root: &Path, mutate: Option<&[String]>) -> Result<String> {
    let report_path = root.join("reports").join("mutation").join("mutation.json");
    let _cleanup = ReportCleanup(report_path.clone());
    // Drop any stale report so a previous run's output is never mistaken for this one's.
    let _ = std::fs::remove_file(&report_path);

    let mut command = Command::new("npx");
    command
        .current_dir(root)
        .args(["--yes", "stryker", "run", "--reporters", "json"]);
    if let Some(specs) = mutate {
        command.arg("--mutate").arg(specs.join(","));
    }
    let output = command
        .env("CI", "1")
        .output()
        .context("running `npx stryker run` (is @stryker-mutator/core installed?)")?;

    std::fs::read_to_string(&report_path).map_err(|_| {
        anyhow::anyhow!(
            "Stryker produced no report in `{}` (did it run cleanly?):\n{}{}",
            root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
    })
}

/// Removes the Stryker json report on drop, pruning the `mutation/` and `reports/`
/// parents only if they're left empty — so a user's own populated `reports/` survives.
struct ReportCleanup(PathBuf);

impl Drop for ReportCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        if let Some(mutation_dir) = self.0.parent() {
            // `remove_dir` only succeeds on an empty dir, so populated trees are safe.
            let _ = std::fs::remove_dir(mutation_dir);
            if let Some(reports_dir) = mutation_dir.parent() {
                let _ = std::fs::remove_dir(reports_dir);
            }
        }
    }
}

/// Run cosmic-ray over the Python project at `root` and return its un-exempted
/// survivors — the Python arm of the mutation rule (#203), parity with [`measure_rust`]
/// and [`measure_typescript`].
///
/// With `base` set, only mutants on the `<base>...HEAD` changed lines are tested;
/// without it, the whole project. `exempt` is the resolved `mutation`-rule exempt
/// paths. cosmic-ray + the test runner must be installed.
///
/// NOTE: stub — returns no survivors until the cosmic-ray wiring lands (the
/// implementation commit). It exists now so the red integration/e2e tests compile and
/// fail for the right reason before the engine is wired (#203).
pub fn measure_python(
    _root: &Path,
    _exempt: &[String],
    _base: Option<&str>,
) -> Result<Vec<Survivor>> {
    Ok(Vec::new())
}

/// A unique temp dir for one cargo-mutants run's `--output`, removed on drop so the
/// scanned crate stays pristine and parallel runs don't collide.
struct MutantsOut(PathBuf);

impl MutantsOut {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "testing-conventions-mutants-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        );
        MutantsOut(std::env::temp_dir().join(name))
    }
}

impl Drop for MutantsOut {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Write the `<base>...HEAD` diff cargo-mutants' `--in-diff` scopes to, returning its path.
fn write_base_diff(root: &Path, base: &str, out: &MutantsOut) -> Result<PathBuf> {
    let range = format!("{base}...HEAD");
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", &range])
        .output()
        .context("running `git diff` for `--base` (is git installed?)")?;
    if !output.status.success() {
        bail!(
            "git diff {range} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::create_dir_all(&out.0).context("creating the mutants output dir")?;
    let path = out.0.join("base.diff");
    std::fs::write(&path, &output.stdout).context("writing the base diff")?;
    Ok(path)
}

/// Run `cargo mutants --output <out> [--in-diff <diff>]` in `root`.
///
/// cargo-mutants exits `0` when every mutant is caught and `2` when some survive (or
/// time out / are unviable) — both are normal here, since survivors are the rule's
/// *output*, not an error. Any other code (usage error, or a baseline that didn't
/// build/pass) is fatal. As with the coverage run, the outer instrumentation env is
/// stripped so a nested run (this rule's own tests under `cargo llvm-cov`) doesn't
/// re-enter the rustc wrapper and hang.
fn run_cargo_mutants(root: &Path, out: &Path, in_diff: Option<&Path>) -> Result<()> {
    let mut command = Command::new("cargo");
    command
        .current_dir(root)
        .arg("mutants")
        .arg("--output")
        .arg(out);
    if let Some(diff) = in_diff {
        command.arg("--in-diff").arg(diff);
    }
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
        .context("running `cargo mutants` (is cargo-mutants installed?)")?;
    match output.status.code() {
        // 0 = all caught, 2 = some survived/timed out: both produce a report to read.
        Some(0) | Some(2) => Ok(()),
        _ => bail!(
            "cargo-mutants did not run cleanly in `{}` (baseline build/test failure?):\n{}{}",
            root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A pared `outcomes.json`: the baseline, one missed mutant, and one caught — the
    // real shape (externally-tagged `scenario`, extra fields the rule ignores).
    const SAMPLE: &str = r#"{
        "outcomes": [
            {"scenario": "Baseline", "summary": "Success",
             "phase_results": []},
            {"scenario": {"Mutant": {"file": "src/lib.rs", "package": "p", "genre": "FnValue",
                "replacement": "true", "name": "src/lib.rs:7:7: replace > with == in is_positive",
                "function": {"function_name": "is_positive"},
                "span": {"start": {"line": 7, "column": 7}, "end": {"line": 7, "column": 8}}}},
             "summary": "MissedMutant"},
            {"scenario": {"Mutant": {"file": "src/other.rs", "package": "p", "genre": "FnValue",
                "replacement": "0", "name": "src/other.rs:3:5: replace add -> i32 with 0",
                "span": {"start": {"line": 3, "column": 5}, "end": {"line": 3, "column": 9}}}},
             "summary": "CaughtMutant"}
        ],
        "total_mutants": 2
    }"#;

    #[test]
    fn parses_the_outcomes_export() {
        let report = parse_mutants_report(SAMPLE).expect("valid outcomes.json");
        assert_eq!(report.outcomes.len(), 3);
        assert!(matches!(report.outcomes[0].scenario, Scenario::Baseline));
    }

    #[test]
    fn collects_only_missed_mutants_as_survivors() {
        let report = parse_mutants_report(SAMPLE).unwrap();
        let survivors = unexplained_survivors(&report, &[]);
        // Only the MissedMutant — the baseline and the CaughtMutant are not survivors.
        assert_eq!(survivors.len(), 1);
        assert_eq!(survivors[0].file, "src/lib.rs");
        assert_eq!(survivors[0].line, 7);
        assert!(survivors[0].description.contains("replace > with =="));
    }

    #[test]
    fn an_exemption_drops_a_survivor_in_that_file() {
        let report = parse_mutants_report(SAMPLE).unwrap();
        let exempt = vec!["src/lib.rs".to_string()];
        assert!(unexplained_survivors(&report, &exempt).is_empty());
    }

    #[test]
    fn an_exemption_on_another_file_leaves_the_survivor() {
        let report = parse_mutants_report(SAMPLE).unwrap();
        let exempt = vec!["src/elsewhere.rs".to_string()];
        assert_eq!(unexplained_survivors(&report, &exempt).len(), 1);
    }

    // A pared Stryker `mutation.json`: one Survived, one NoCoverage, one Killed — the
    // real shape (per-file `files` map, extra fields the rule ignores).
    const STRYKER_SAMPLE: &str = r#"{
        "schemaVersion": "1.0",
        "files": {
            "src/index.ts": {
                "language": "typescript",
                "source": "...",
                "mutants": [
                    {"id": "0", "mutatorName": "ConditionalExpression", "replacement": "true",
                     "status": "Survived", "coveredBy": ["t0"],
                     "location": {"start": {"line": 2, "column": 10}, "end": {"line": 2, "column": 15}}},
                    {"id": "1", "mutatorName": "ArithmeticOperator", "replacement": "a - b",
                     "status": "NoCoverage",
                     "location": {"start": {"line": 5, "column": 3}, "end": {"line": 5, "column": 8}}},
                    {"id": "2", "mutatorName": "BooleanLiteral", "replacement": "false",
                     "status": "Killed",
                     "location": {"start": {"line": 9, "column": 1}, "end": {"line": 9, "column": 6}}}
                ]
            }
        }
    }"#;

    #[test]
    fn parses_a_stryker_report() {
        let report = parse_stryker_report(STRYKER_SAMPLE).expect("valid mutation.json");
        assert_eq!(report.files["src/index.ts"].mutants.len(), 3);
    }

    #[test]
    fn collects_survived_and_nocoverage_as_survivors() {
        let report = parse_stryker_report(STRYKER_SAMPLE).unwrap();
        let survivors = stryker_survivors(&report);
        // Survived + NoCoverage are survivors; the Killed mutant is not.
        assert_eq!(survivors.len(), 2);
        assert!(survivors.iter().all(|s| s.file == "src/index.ts"));
        assert_eq!(survivors[0].line, 2);
        assert!(survivors[0].description.contains("ConditionalExpression"));
        assert!(survivors[0].description.contains("true"));
        assert_eq!(survivors[1].line, 5);
    }

    #[test]
    fn evaluate_drops_exempt_files_for_either_engine() {
        let report = parse_stryker_report(STRYKER_SAMPLE).unwrap();
        let survivors = stryker_survivors(&report);
        let exempt = vec!["src/index.ts".to_string()];
        assert!(evaluate(survivors, &exempt).is_empty());
    }

    #[test]
    fn is_mutatable_ts_keeps_sources_and_drops_tests_and_decls() {
        assert!(is_mutatable_ts("src/index.ts"));
        assert!(is_mutatable_ts("src/util.tsx"));
        assert!(is_mutatable_ts("src/util.js"));
        assert!(!is_mutatable_ts("src/index.test.ts"));
        assert!(!is_mutatable_ts("src/index.spec.ts"));
        assert!(!is_mutatable_ts("src/types.d.ts"));
        assert!(!is_mutatable_ts("README.md"));
    }

    #[test]
    fn contiguous_runs_collapses_adjacent_lines() {
        let lines: BTreeSet<u64> = [2u64, 3, 4, 7, 9, 10].into_iter().collect();
        assert_eq!(contiguous_runs(&lines), vec![(2, 4), (7, 7), (9, 10)]);
        assert!(contiguous_runs(&BTreeSet::new()).is_empty());
    }

    #[test]
    fn one_line_flattens_and_caps() {
        assert_eq!(one_line("a -\n  b"), "a - b");
        let long = "x".repeat(80);
        let capped = one_line(&long);
        assert!(capped.chars().count() <= 61 && capped.ends_with('…'));
    }
}
