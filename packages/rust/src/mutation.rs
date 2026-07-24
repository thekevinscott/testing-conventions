//! Mutation testing for Rust (`unit mutation --language rust`) — the rung
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
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// A surviving mutant — a mutation the unit suite ran but failed to catch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Survivor {
    /// The mutated file, scan-path-relative and `/`-separated — cargo-mutants reports
    /// workspace-root-relative paths, rebased onto the scan path before gating.
    pub file: String,
    /// The 1-based line the mutation starts on.
    pub line: u32,
    /// cargo-mutants' human description (e.g. `replace > with == in is_positive`).
    pub description: String,
}

/// One mutation measurement: whether the engine ran, and what it found. Telling
/// [`Measurement::EngineNotRun`] from an all-killed [`Measurement::Tested`] keeps a
/// vacuous pass visible — a diff-scoped run that never built a mutant reads differently
/// from one that tested mutants and caught every one, and a counted pass carries its
/// own evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Measurement {
    /// The `--base` diff carried no mutatable changed lines; the engine never ran.
    EngineNotRun,
    /// The engine ran: `count` viable, conclusive mutants judged (caught or missed),
    /// `survivors` the un-exempted surviving ones.
    Tested {
        count: usize,
        survivors: Vec<Survivor>,
    },
}

/// The `(file, line)` locations an engine produced a viable mutant for — the input the
/// line-scoped guard reads to tell an over-exemption (a listed line whose mutants
/// were all caught) from an out-of-scope line (no mutant there).
pub type MutatedLines = BTreeSet<(String, u32)>;

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
    evaluate(cargo_mutants_survivors(report), exempt)
}

/// The surviving mutants in a cargo-mutants report — the raw list before exemptions.
/// A survivor is a `MissedMutant` outcome (the suite ran the mutated code but no test
/// failed). `Timeout` / `Unviable` are not survivors.
fn cargo_mutants_survivors(report: &MutantsReport) -> Vec<Survivor> {
    report
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
        .collect()
}

/// The `(file, line)` locations cargo-mutants produced a **viable, conclusive** mutant
/// for — caught or missed (`CaughtMutant` / `MissedMutant`), not the inconclusive
/// `Timeout` / `Unviable`. The line-scoped guard reads this to tell an
/// over-exemption (a listed line whose mutants were all *caught*, no survivor) from an
/// out-of-scope line (no mutant there at all — e.g. outside a `--base` diff).
pub fn mutated_lines(report: &MutantsReport) -> MutatedLines {
    report
        .outcomes
        .iter()
        .filter_map(|outcome| {
            if outcome.summary != "CaughtMutant" && outcome.summary != "MissedMutant" {
                return None;
            }
            let Scenario::Mutant(mutant) = &outcome.scenario else {
                return None;
            };
            Some((mutant.file.clone(), mutant.span.start.line))
        })
        .collect()
}

/// The number of viable, conclusive mutants in a cargo-mutants report — `CaughtMutant`
/// plus `MissedMutant`, the same set [`mutated_lines`] reads. A passing run states this
/// count as its evidence.
fn conclusive_count(report: &MutantsReport) -> usize {
    report
        .outcomes
        .iter()
        .filter(|outcome| outcome.summary == "CaughtMutant" || outcome.summary == "MissedMutant")
        .count()
}

/// The shared whole-file evaluation core: drop the survivors lifted by a file-level
/// `mutation` exemption (a file-path match), leaving the rule's findings. The
/// line-scoped path ([`evaluate_scoped`]) generalizes this to per-line exemptions with
/// a determinism guard.
pub fn evaluate(survivors: Vec<Survivor>, exempt: &[String]) -> Vec<Survivor> {
    survivors
        .into_iter()
        .filter(|survivor| !exempt.iter().any(|path| path == &survivor.file))
        .collect()
}

/// Apply file- and line-scoped `mutation` exemptions to the raw `survivors`, with the
/// determinism guard. `mutated` is the set of `(file, line)` that produced a
/// viable mutant (caught or survived); `whole_file` is the file-level exemptions and
/// `line_scoped` the per-line ones.
///
/// Guard: a line-scoped exemption that names a line whose mutants were **all caught**
/// (in `mutated`, but with no survivor) is over-exemption — a hard error, the
/// counterpart to the stale-path rule. A listed line with **no** mutant at all is left
/// alone (it may simply be outside a `--base` diff), neither an error nor a drop. Then
/// every survivor whose file is whole-file-exempt, or whose `(file, line)` is
/// line-exempt, is dropped; an unlisted survivor still fails the gate.
pub fn evaluate_scoped(
    survivors: Vec<Survivor>,
    mutated: &MutatedLines,
    whole_file: &[String],
    line_scoped: &BTreeMap<String, BTreeSet<u32>>,
) -> Result<Vec<Survivor>> {
    let mut over: Vec<String> = Vec::new();
    for (file, lines) in line_scoped {
        for &line in lines {
            let has_survivor = survivors
                .iter()
                .any(|survivor| survivor.file == *file && survivor.line == line);
            if has_survivor {
                continue;
            }
            if mutated.contains(&(file.clone(), line)) {
                over.push(format!("\n  {file}:{line}"));
            }
        }
    }
    if !over.is_empty() {
        bail!(
            "a line-scoped mutation exemption may only list a line with a surviving mutant, but \
             these had mutants that were all caught:{}",
            over.concat()
        );
    }
    Ok(survivors
        .into_iter()
        .filter(|survivor| {
            let whole = whole_file.iter().any(|path| path == &survivor.file);
            let line = line_scoped
                .get(&survivor.file)
                .is_some_and(|lines| lines.contains(&survivor.line));
            !(whole || line)
        })
        .collect())
}

/// A mutant's outcome, normalized across the engines (Stryker / cosmic-ray / cargo-mutants)
/// — the union of their result vocabularies reduced to what the gate needs. Each
/// language adapter maps its native outcomes onto this so the Rust core gates on one
/// representation instead of three per-engine report formats. The serialized form is
/// `snake_case` (`no_coverage`, `compile_error`, …) — the wire contract adapters emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutantStatus {
    /// A test ran the mutated code but none failed — a survivor.
    Survived,
    /// A test failed on the mutant — caught.
    Killed,
    /// No test exercised the mutant at all — a survivor (worse than `Survived`).
    NoCoverage,
    /// The mutant ran but the suite timed out — inconclusive, not a survivor (but viable).
    Timeout,
    /// The mutant never compiled — not a viable mutant.
    CompileError,
    /// The mutant errored at runtime before a test could judge it — not viable.
    RuntimeError,
}

impl MutantStatus {
    /// Whether this outcome is a **survivor** — a mutant the suite failed to catch
    /// (`Survived` or `NoCoverage`). Mirrors the per-engine survivor rules.
    fn is_survivor(self) -> bool {
        matches!(self, MutantStatus::Survived | MutantStatus::NoCoverage)
    }

    /// Whether this came from a **viable, conclusive** mutant — one that actually ran
    /// (`Survived` / `Killed` / `NoCoverage` / `Timeout`), not one that never compiled or
    /// errored out. The determinism guard reads this to tell an over-exemption (a
    /// listed line whose mutants were all caught) from an out-of-scope line (no mutant there).
    fn is_viable(self) -> bool {
        matches!(
            self,
            MutantStatus::Survived
                | MutantStatus::Killed
                | MutantStatus::NoCoverage
                | MutantStatus::Timeout
        )
    }

    /// Whether the suite **judged** this mutant (`Survived` / `Killed` / `NoCoverage`) —
    /// the conclusive set a passing run counts as its evidence. A `Timeout` ran but
    /// judged nothing; `CompileError` / `RuntimeError` never produced a viable mutant.
    fn is_conclusive(self) -> bool {
        matches!(
            self,
            MutantStatus::Survived | MutantStatus::Killed | MutantStatus::NoCoverage
        )
    }
}

/// One mutant in the normalized result set: the engine-agnostic shape every language
/// adapter emits. Extra fields an adapter includes are ignored.
#[derive(Debug, Clone, Deserialize)]
pub struct NormalizedMutant {
    /// Project-relative, `/`-separated path of the mutated file.
    pub file: String,
    /// The 1-based line the mutant starts on.
    pub line: u32,
    /// The outcome, normalized across engines.
    pub status: MutantStatus,
    /// The engine's mutator/operator name (e.g. `ConditionalExpression`).
    pub mutator: String,
    /// The replacement text, when the engine reports one — used for a readable description.
    #[serde(default)]
    pub replacement: Option<String>,
}

/// Parse the normalized results an engine adapter emits — a flat JSON array of
/// [`NormalizedMutant`].
pub fn parse_normalized_results(json: &str) -> Result<Vec<NormalizedMutant>> {
    serde_json::from_str(json).context("parsing normalized mutation results")
}

/// Gate a normalized result set: drop the survivors lifted by a file- or line-scoped
/// `mutation` exemption (with the determinism guard), leaving the rule's findings.
///
/// This is the engine-agnostic core each language arm feeds once its adapter has produced
/// [`NormalizedMutant`]s — the replacement for the per-engine `*_survivors` /
/// `*_mutated_lines` + [`evaluate_scoped`] wiring. Survivors are `Survived` / `NoCoverage`
/// mutants; the guard reads every *viable* mutant's `(file, line)`.
pub fn evaluate_normalized(
    mutants: &[NormalizedMutant],
    whole_file: &[String],
    line_scoped: &BTreeMap<String, BTreeSet<u32>>,
) -> Result<Vec<Survivor>> {
    evaluate_scoped(
        normalized_survivors(mutants),
        &normalized_mutated_lines(mutants),
        whole_file,
        line_scoped,
    )
}

/// The surviving mutants in a normalized result set — the raw list before exemptions.
fn normalized_survivors(mutants: &[NormalizedMutant]) -> Vec<Survivor> {
    mutants
        .iter()
        .filter(|mutant| mutant.status.is_survivor())
        .map(|mutant| Survivor {
            file: mutant.file.clone(),
            line: mutant.line,
            description: describe_normalized(mutant),
        })
        .collect()
}

/// The `(file, line)` of every viable, conclusive mutant in a normalized result set — the
/// input the line-scoped guard in [`evaluate_scoped`] reads.
fn normalized_mutated_lines(mutants: &[NormalizedMutant]) -> MutatedLines {
    mutants
        .iter()
        .filter(|mutant| mutant.status.is_viable())
        .map(|mutant| (mutant.file.clone(), mutant.line))
        .collect()
}

/// The number of conclusive mutants in a normalized result set — the count a passing
/// run states as its evidence, parity with [`conclusive_count`].
fn normalized_conclusive_count(mutants: &[NormalizedMutant]) -> usize {
    mutants
        .iter()
        .filter(|mutant| mutant.status.is_conclusive())
        .count()
}

/// A one-line description for a normalized mutant: the mutator name, plus the replacement
/// (flattened + capped via [`one_line`]) when the engine reported one.
fn describe_normalized(mutant: &NormalizedMutant) -> String {
    match &mutant.replacement {
        Some(replacement) => format!("{} (-> {})", mutant.mutator, one_line(replacement)),
        None => mutant.mutator.clone(),
    }
}

/// Run cargo-mutants over the crate at `root` and return the [`Measurement`]: the
/// un-exempted survivors plus the conclusive-mutant count, or
/// [`Measurement::EngineNotRun`] for a `base` diff with no changed lines under the crate.
///
/// With `base` set, only mutants on the `<base>...HEAD` changed lines are tested (via
/// cargo-mutants' `--in-diff`); without it, the whole crate. `exempt` is the file-level
/// `mutation` exempt paths and `exempt_lines` the line-scoped ones, applied with
/// the determinism guard in [`evaluate_scoped`]. The tool provisions cargo-mutants itself
/// on first use ([`ensure_cargo_mutants`]) — only a cargo toolchain need be present.
pub fn measure_rust(
    root: &Path,
    exempt: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
    base: Option<&str>,
    features: &[String],
) -> Result<Measurement> {
    let out = MutantsOut::new();
    // cargo-mutants addresses files relative to the crate's cargo workspace root, so
    // both the `--in-diff` diff it consumes and the report paths it emits carry the
    // scan path's workspace-relative prefix whenever the crate is a member of a
    // workspace rooted above it. A standalone crate is its own workspace root: no prefix.
    let workspace_root = cargo_workspace_root(root)?;
    let prefix = canonical_scan_prefix(root, &workspace_root);
    let diff = match base {
        // An empty diff (no changed lines under the crate — a PR that doesn't touch it)
        // means nothing to mutate: the engine is skipped, and the caller reports it.
        Some(base) => match write_base_diff(root, &workspace_root, prefix.as_deref(), base, &out)? {
            None => return Ok(Measurement::EngineNotRun),
            Some(path) => Some(path),
        },
        None => None,
    };
    let engine = ensure_cargo_mutants()?;
    run_cargo_mutants(&engine, root, &out.0, diff.as_deref(), features)?;
    let outcomes = out.0.join("mutants.out").join("outcomes.json");
    // cargo-mutants writes no `outcomes.json` when a run produces no mutants (e.g. an
    // `--in-diff` that matches none of the crate's lines). `run_cargo_mutants` already
    // bailed on a fatal exit, so a missing report here is an engine run that judged
    // zero mutants.
    let json = match std::fs::read_to_string(&outcomes) {
        Ok(json) => json,
        Err(_) => {
            return Ok(Measurement::Tested {
                count: 0,
                survivors: Vec::new(),
            })
        }
    };
    let report = rebase_report_paths(parse_mutants_report(&json)?, prefix.as_deref());
    let survivors = evaluate_scoped(
        cargo_mutants_survivors(&report),
        &mutated_lines(&report),
        exempt,
        exempt_lines,
    )?;
    Ok(Measurement::Tested {
        count: conclusive_count(&report),
        survivors,
    })
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

/// Run the bundled TypeScript mutation adapter over the scan path at `root` and return
/// the [`Measurement`] — the TS arm of the mutation rule, parity with [`measure_rust`].
///
/// The consumer installs **nothing** Stryker-related: the npm package ships a Node
/// adapter that drives Stryker through its own Node API and emits the engine-agnostic
/// [`NormalizedMutant`] schema, which this gates over via [`evaluate_normalized`]
/// — the same core the Rust and Python arms feed. Only the project's own test runner
/// (vitest) needs to be present, exactly as cargo-mutants needs a buildable crate and
/// cosmic-ray needs pytest.
///
/// The adapter runs at the **package root** — the nearest directory at or above `root`
/// holding a `package.json` ([`crate::tiers::package_root`]) — and Stryker runs **in
/// place** there: mutants are applied to the package's real tree (backed up under
/// `.stryker-tmp`, restored at run end), so the manifest, the package's `tsconfig.json`,
/// and every reference a source legitimately makes above the scan path
/// (`import pkg from '../package.json'`, a shared `../tsconfig`) resolve exactly as they
/// do in the tree — including for the engine's own tooling, which a sandbox copy would
/// resolve from the isolated install's tree instead. The run stays scan-path-scoped end
/// to end: mutate patterns address the
/// scan path within the package, the scan path's colocated suite judges the mutants (the
/// adapter's `--vitest-dir`), and the results are rebased scan-path-relative before
/// gating, so exemption paths match every other check. A scan path that is itself the
/// package root — or a loose tree with no manifest — runs at the scan path, unprefixed.
///
/// With `base` set, only mutants on the `<base>...HEAD` changed lines are tested —
/// Stryker has no native git-diff mode, so the changed lines become `--mutate
/// <file>:<line>-<line>` ranges (line granularity, matching cargo-mutants' `--in-diff`).
/// Without it, the scan path's sources run ([`scan_scoped_mutate_globs`]). `exempt` is the
/// file-level exempt paths and `exempt_lines` the line-scoped ones. `adapter` is the path
/// to the bundled Node adapter (`dist/mutation/main.js`) — the CLI receives it from the
/// npm launcher's `--ts-mutation-adapter` argument and hands it down explicitly.
pub fn measure_typescript(
    root: &Path,
    exempt: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
    base: Option<&str>,
    adapter: &Path,
) -> Result<Measurement> {
    let package_root =
        crate::tiers::package_root(root, "package.json").unwrap_or_else(|| root.to_path_buf());
    let prefix = scan_prefix(root, &package_root);
    let mutate = match base {
        Some(base) => {
            let ranges = mutate_ranges(root, base)?;
            // Nothing mutatable changed on the diff: the engine is skipped, and the
            // caller reports it.
            if ranges.is_empty() {
                return Ok(Measurement::EngineNotRun);
            }
            Some(prefix_mutate_specs(ranges, prefix.as_deref()))
        }
        None => prefix.as_deref().map(scan_scoped_mutate_globs),
    };
    let json = run_ts_adapter(&package_root, adapter, mutate.as_deref(), prefix.as_deref())?;
    let mutants = to_scan_relative(parse_normalized_results(&json)?, prefix.as_deref());
    let survivors = evaluate_normalized(&mutants, exempt, exempt_lines)?;
    Ok(Measurement::Tested {
        count: normalized_conclusive_count(&mutants),
        survivors,
    })
}

/// The scan path relative to its package root, as a `/`-joined string — the prefix every
/// package-root-relative path carries for a scan path below the root. `None` when the scan
/// path *is* the package root (nothing to prefix), which also covers a loose tree whose
/// "package root" fell back to the scan path itself.
fn scan_prefix(root: &Path, package_root: &Path) -> Option<String> {
    let rel = root.strip_prefix(package_root).ok()?;
    let parts: Vec<String> = rel
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

/// Prefix diff-scoped mutate specs (`<file>:<start>-<end>`, scan-path-relative) with the
/// scan prefix, so they address the same files from the package root the adapter runs at.
fn prefix_mutate_specs(specs: Vec<String>, prefix: Option<&str>) -> Vec<String> {
    match prefix {
        None => specs,
        Some(prefix) => specs
            .into_iter()
            .map(|spec| format!("{prefix}/{spec}"))
            .collect(),
    }
}

/// Stryker's default `mutate` set re-rooted at the scan path: every source under it except
/// test files and `__tests__` trees — the same shape Stryker itself defaults to for
/// `{src,lib}`, addressed from the package root the adapter runs at.
fn scan_scoped_mutate_globs(prefix: &str) -> Vec<String> {
    const EXTENSIONS: &str = "+(cjs|mjs|js|ts|mts|cts|jsx|tsx|html|vue|svelte)";
    vec![
        format!("{prefix}/**/!(*.+(s|S)pec|*.+(t|T)est).{EXTENSIONS}"),
        format!("!{prefix}/**/__tests__/**/*.{EXTENSIONS}"),
    ]
}

/// Rebase package-root-relative mutant paths onto the scan path: strip the scan prefix so
/// exemption matching and the reported survivors address scan-path-relative files, as every
/// other check does. A mutant outside the scan path is outside the gate's scope and dropped.
fn to_scan_relative(mutants: Vec<NormalizedMutant>, prefix: Option<&str>) -> Vec<NormalizedMutant> {
    let Some(prefix) = prefix else {
        return mutants;
    };
    let prefix = format!("{prefix}/");
    mutants
        .into_iter()
        .filter_map(|mut mutant| {
            mutant.file = mutant.file.strip_prefix(&prefix)?.to_string();
            Some(mutant)
        })
        .collect()
}

/// Run the bundled TS mutation `adapter` at `package_root` and return the
/// normalized-results JSON it writes. The adapter (a Node entry shipped with the npm
/// package) drives Stryker via its Node API and emits a [`NormalizedMutant`] array — so
/// the consumer drives the engine through this CLI alone; the npm package resolves
/// `@stryker-mutator/*` from the tool's own tree. The adapter's working directory is
/// Stryker's project root: the in-place run mutates and resolves against it.
///
/// `mutate`, when set, scopes the run to `--mutate` patterns; `vitest_dir`, when set,
/// scopes vitest's test discovery to that directory within the project (the scan path).
/// Results are written to a temp file the adapter names via `--out` (so Stryker's own
/// stdout logging can't corrupt them), then read back. `node` and the project's own test
/// runner must be available; a non-zero adapter exit surfaces its captured output.
fn run_ts_adapter(
    package_root: &Path,
    adapter: &Path,
    mutate: Option<&[String]>,
    vitest_dir: Option<&str>,
) -> Result<String> {
    let out = AdapterOut::new();
    std::fs::create_dir_all(&out.0).context("creating the mutation adapter output dir")?;
    let results = out.0.join("results.json");

    let mut command = Command::new("node");
    command
        .current_dir(package_root)
        .arg(adapter)
        .arg("--out")
        .arg(&results);
    if let Some(specs) = mutate {
        command.arg("--mutate").arg(specs.join(","));
    }
    if let Some(dir) = vitest_dir {
        command.arg("--vitest-dir").arg(dir);
    }
    let output = command
        .output()
        .context("running the TypeScript mutation adapter (is `node` installed?)")?;
    if !output.status.success() {
        bail!(
            "the TypeScript mutation adapter failed in `{}`:\n{}{}",
            package_root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    std::fs::read_to_string(&results).with_context(|| {
        format!(
            "reading the TypeScript mutation adapter's results from `{}`",
            results.display()
        )
    })
}

/// A unique temp dir for one TS mutation adapter run's `--out` JSON, removed on drop so
/// the scanned project stays pristine and parallel runs don't collide.
struct AdapterOut(PathBuf);

impl AdapterOut {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "testing-conventions-ts-adapter-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        );
        AdapterOut(std::env::temp_dir().join(name))
    }
}

impl Drop for AdapterOut {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
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

/// Run the bundled Python mutation adapter over the project at `root` and return the
/// [`Measurement`] — the Python arm of the mutation rule, parity with
/// [`measure_rust`] and [`measure_typescript`].
///
/// The tool drives the engine: the wheel ships a Python adapter that runs cosmic-ray through
/// its own library API (`WorkDB`) and emits the normalized [`NormalizedMutant`] schema
/// the gate consumes. maturin (`bindings = "bin"`) ships the rust binary directly as the wheel's
/// script — with no Python launcher to inject a path, unlike the TS arm — so the binary invokes
/// the adapter as an installed module (`python3 -m testing_conventions.mutation.main`), resolved
/// from the wheel's environment alongside cosmic-ray. The project supplies its own test runner
/// (pytest), exactly as cargo-mutants needs a buildable crate and Stryker needs vitest.
///
/// With `base` set, only mutants on the `<base>...HEAD` changed lines are reported: cosmic-ray
/// has no native git-diff mode, so the run is scoped to the changed `.py` files (passed as
/// `--module`) and the survivors are filtered to the changed lines in the core — line
/// granularity, matching the other arms. Without it, the whole project's sources run (tests
/// excluded). `exempt` is the file-level exempt paths and `exempt_lines` the line-scoped ones.
pub fn measure_python(
    root: &Path,
    exempt: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
    base: Option<&str>,
) -> Result<Measurement> {
    let changed = match base {
        Some(base) => Some(crate::patch_coverage::changed_lines(root, base)?),
        None => None,
    };
    let modules: Vec<String> = match &changed {
        None => Vec::new(),
        Some(changed) => {
            let modules: Vec<String> = changed
                .keys()
                .filter(|file| is_mutatable_py(file))
                .cloned()
                .collect();
            // Nothing mutatable changed on the diff: the engine is skipped, and the
            // caller reports it.
            if modules.is_empty() {
                return Ok(Measurement::EngineNotRun);
            }
            modules
        }
    };
    let json = run_py_adapter(root, &modules)?;
    let mut mutants = parse_normalized_results(&json)?;
    if let Some(changed) = &changed {
        // Diff-scoping v1: keep only mutants on the changed lines.
        mutants.retain(|mutant| {
            changed
                .get(&mutant.file)
                .is_some_and(|lines| lines.contains(&u64::from(mutant.line)))
        });
    }
    let survivors = evaluate_normalized(&mutants, exempt, exempt_lines)?;
    Ok(Measurement::Tested {
        count: normalized_conclusive_count(&mutants),
        survivors,
    })
}

/// Run the bundled Python mutation adapter over `root` and return the normalized-results JSON
/// it writes. The rust binary spawns `python3 -m testing_conventions.mutation.main --out <tmp>
/// [--module <path> ...]`; the adapter drives cosmic-ray in-process and emits a
/// [`NormalizedMutant`] array. `modules`, when non-empty, scopes the run to those source
/// files (the `<base>...HEAD` changed ones); empty runs the whole project. Results are written
/// to a temp file the adapter names via `--out`, then read back. `PYTHONDONTWRITEBYTECODE` keeps
/// `__pycache__` out of the scanned tree; a non-zero adapter exit surfaces its captured output.
fn run_py_adapter(root: &Path, modules: &[String]) -> Result<String> {
    let out = AdapterOut::new();
    std::fs::create_dir_all(&out.0).context("creating the mutation adapter output dir")?;
    let results = out.0.join("results.json");

    let mut command = Command::new("python3");
    command
        .current_dir(root)
        .args(["-m", "testing_conventions.mutation.main", "--out"])
        .arg(&results)
        .env("PYTHONDONTWRITEBYTECODE", "1");
    for module in modules {
        command.arg("--module").arg(module);
    }
    let output = command
        .output()
        .context("running the Python mutation adapter (is `python3` installed?)")?;
    if !output.status.success() {
        bail!(
            "the Python mutation adapter failed in `{}`:\n{}{}",
            root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    std::fs::read_to_string(&results).with_context(|| {
        format!(
            "reading the Python mutation adapter's results from `{}`",
            results.display()
        )
    })
}

/// Whether a changed file is a mutatable Python *source* — a `.py` that is not a test
/// (`*_test.py` / `test_*.py`) or `conftest.py`.
fn is_mutatable_py(file: &str) -> bool {
    if !file.ends_with(".py") {
        return false;
    }
    let base = file.rsplit('/').next().unwrap_or(file);
    !(base.ends_with("_test.py") || base.starts_with("test_") || base == "conftest.py")
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

/// The directory of the cargo workspace `root` belongs to — the source-tree root
/// cargo-mutants addresses its diff and report paths from. A standalone crate is its
/// own workspace root. `cargo locate-project --workspace` is the authoritative lookup:
/// membership involves member globs and `exclude` lists a manifest walk can't settle.
fn cargo_workspace_root(root: &Path) -> Result<PathBuf> {
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["locate-project", "--workspace", "--message-format", "plain"])
        .output()
        .context("running `cargo locate-project` (is cargo installed?)")?;
    if !output.status.success() {
        bail!(
            "cargo locate-project failed in `{}`: {}",
            root.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let manifest = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    manifest
        .parent()
        .map(Path::to_path_buf)
        .with_context(|| format!("no parent dir for the workspace manifest `{}`", manifest.display()))
}

/// The scan path's prefix relative to the workspace root ([`scan_prefix`]), over
/// canonicalized paths so a relative CLI scan path resolves against the absolute path
/// `cargo locate-project` reports. `None` when the scan path *is* the workspace root.
fn canonical_scan_prefix(root: &Path, workspace_root: &Path) -> Option<String> {
    let root = root.canonicalize().ok()?;
    let workspace_root = workspace_root.canonicalize().ok()?;
    scan_prefix(&root, &workspace_root)
}

/// Write the `<base>...HEAD` diff cargo-mutants' `--in-diff` scopes to, returning its
/// path — or `None` when the diff is empty (no changed lines under the crate).
///
/// cargo-mutants matches `--in-diff` paths relative to the crate's cargo workspace
/// root, so the diff is generated there: `--relative` makes the paths workspace-root-
/// relative, and for a workspace-member crate a pathspec (`prefix`) restricts the diff
/// to changes under the scan path. A standalone crate is its own workspace root, so
/// the same invocation runs in the crate dir with no pathspec. Scoping means a PR that
/// doesn't touch the crate yields an empty diff either way.
fn write_base_diff(
    root: &Path,
    workspace_root: &Path,
    prefix: Option<&str>,
    base: &str,
    out: &MutantsOut,
) -> Result<Option<PathBuf>> {
    let range = format!("{base}...HEAD");
    let (dir, args) = match prefix {
        None => (root, vec!["diff", "--relative", &range]),
        Some(prefix) => (
            workspace_root,
            vec!["diff", "--relative", &range, "--", prefix],
        ),
    };
    let output = Command::new("git")
        .current_dir(dir)
        .args(&args)
        .output()
        .context("running `git diff` for `--base` (is git installed?)")?;
    if !output.status.success() {
        bail!(
            "git diff {range} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    if output.stdout.is_empty() {
        return Ok(None);
    }
    std::fs::create_dir_all(&out.0).context("creating the mutants output dir")?;
    let path = out.0.join("base.diff");
    std::fs::write(&path, &output.stdout).context("writing the base diff")?;
    Ok(Some(path))
}

/// Rebase a cargo-mutants report's workspace-root-relative mutant paths onto the scan
/// path: strip the scan prefix so survivor reporting and `mutation` exemption matching
/// address scan-path-relative files, as every other check does. Baseline outcomes carry
/// no path and pass through; a mutant outside the scan path is outside the gate's scope
/// and dropped (parity with the TS arm's [`to_scan_relative`]). `None` is the standalone
/// crate: the report already addresses the scan path.
fn rebase_report_paths(report: MutantsReport, prefix: Option<&str>) -> MutantsReport {
    let Some(prefix) = prefix else {
        return report;
    };
    let prefix = format!("{prefix}/");
    MutantsReport {
        outcomes: report
            .outcomes
            .into_iter()
            .filter_map(|mut outcome| {
                if let Scenario::Mutant(mutant) = &mut outcome.scenario {
                    mutant.file = mutant.file.strip_prefix(&prefix)?.to_string();
                }
                Some(outcome)
            })
            .collect(),
    }
}

/// The cargo-mutants version the Rust arm provisions and pins to. Bumping this points the
/// cache at a fresh version-scoped directory, so the next run installs the new release.
const CARGO_MUTANTS_VERSION: &str = "27.1.0";

/// Ensure the pinned cargo-mutants is available and return the absolute path to its binary,
/// provisioning it on first use.
///
/// The consumer installs nothing and never names the engine:
/// cargo ships no library form of cargo-mutants, so — unlike the in-process TS/Python
/// adapters — the tool runs a pinned `cargo install cargo-mutants` into its own cache
/// directory and drives the installed binary from there. A cached binary is reused; only a
/// cargo toolchain need be present. This is the one deliberate asymmetry from the other
/// arms, called out per the cross-language-parity rule.
fn ensure_cargo_mutants() -> Result<PathBuf> {
    let root = cargo_mutants_cache_root();
    let bin = root.join("bin").join(cargo_mutants_bin_name());
    let lock_path = root.join(".install.lock");
    provision(&bin, &lock_path, || {
        run_install(&root, |command| command.output())
    })
}

/// The cargo-mutants binary's file name (`.exe` on Windows), as `cargo install --root`
/// lays it out under `<root>/bin/`.
fn cargo_mutants_bin_name() -> &'static str {
    if cfg!(windows) {
        "cargo-mutants.exe"
    } else {
        "cargo-mutants"
    }
}

/// The tool-owned, version-scoped cache directory cargo-mutants is installed under, so a
/// version bump provisions cleanly beside the old one and never clobbers a user's own
/// `~/.cargo/bin`.
fn cargo_mutants_cache_root() -> PathBuf {
    cache_base()
        .join("testing-conventions")
        .join(format!("cargo-mutants-{CARGO_MUTANTS_VERSION}"))
}

/// The base cache directory, read from OS-owned config. Split from [`resolve_cache_base`]
/// so the resolution logic is unit-tested without touching the process environment.
fn cache_base() -> PathBuf {
    resolve_cache_base(std::env::var_os("XDG_CACHE_HOME"), std::env::var_os("HOME"))
}

/// Resolve the base cache dir: `XDG_CACHE_HOME` when set and non-empty, else `$HOME/.cache`,
/// else the temp dir. Pure over its inputs.
fn resolve_cache_base(xdg: Option<OsString>, home: Option<OsString>) -> PathBuf {
    if let Some(dir) = xdg.filter(|value| !value.is_empty()) {
        return PathBuf::from(dir);
    }
    if let Some(dir) = home.filter(|value| !value.is_empty()) {
        return PathBuf::from(dir).join(".cache");
    }
    std::env::temp_dir()
}

/// Return `bin` if it already exists, otherwise take an exclusive advisory lock at
/// `lock_path`, re-check (another caller may have installed while this one waited for the
/// lock), and run `install` if still absent.
///
/// The lock closes a race: a bare check-then-install with no locking let N
/// concurrent callers that all observed an absent binary each launch a full `cargo install`
/// — correct (no corrupted output) but ruinously slow, since a from-source cargo-mutants
/// compile is duplicated N times instead of once. Concurrent callers now wait ~one install and
/// find the binary, instead of each running their own; a cold cache costs one serial install
/// regardless of how many callers race for it.
///
/// Pure over the filesystem plus the injected installer, so a test drives every branch with
/// a temp path and a fake installer (no from-source compile). An installer that reports
/// success but produces no binary is an error.
fn provision(
    bin: &Path,
    lock_path: &Path,
    install: impl FnOnce() -> Result<()>,
) -> Result<PathBuf> {
    if bin.exists() {
        return Ok(bin.to_path_buf());
    }
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).context("creating the provisioning lock's parent dir")?;
    }
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(lock_path)
        .context("opening the provisioning lock file")?;
    lock_file
        .lock()
        .context("acquiring the provisioning lock")?;
    // Re-check: another caller may have installed while this one waited for the lock.
    if bin.exists() {
        return Ok(bin.to_path_buf());
    }
    install()?;
    if !bin.exists() {
        bail!(
            "provisioning reported success but cargo-mutants is not at `{}`",
            bin.display()
        );
    }
    Ok(bin.to_path_buf())
}

/// The argv provisioning the pinned cargo-mutants into `root` (`cargo install cargo-mutants
/// --locked --version <X> --root <root>`). Split from execution so a test asserts the pin
/// and the isolated `--root` without a real install.
fn install_argv(root: &Path) -> Vec<OsString> {
    vec![
        OsString::from("install"),
        OsString::from("cargo-mutants"),
        OsString::from("--locked"),
        OsString::from("--version"),
        OsString::from(CARGO_MUTANTS_VERSION),
        OsString::from("--root"),
        root.as_os_str().to_os_string(),
    ]
}

/// Provision cargo-mutants into `root`, executing the built `cargo install` command with
/// `run`. `run` is injected so a test drives the success and failure branches with a fake
/// (no from-source compile). The coverage-instrumentation env is stripped so the compile
/// doesn't re-enter a `cargo llvm-cov` rustc wrapper.
fn run_install(
    root: &Path,
    run: impl FnOnce(&mut Command) -> std::io::Result<Output>,
) -> Result<()> {
    let mut command = Command::new("cargo");
    command.args(install_argv(root));
    strip_llvm_cov_env(&mut command);
    let output = run(&mut command)
        .context("provisioning cargo-mutants via `cargo install` (is cargo installed?)")?;
    if !output.status.success() {
        bail!(
            "failed to provision cargo-mutants {CARGO_MUTANTS_VERSION}:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    Ok(())
}

/// Strip the outer coverage-instrumentation env from a nested cargo invocation (the
/// cargo-mutants run, or the `cargo install` that provisions it) so it doesn't re-enter the
/// `cargo llvm-cov` rustc wrapper and hang, as when this rule's own tests run under coverage.
fn strip_llvm_cov_env(command: &mut Command) {
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
}

/// Run `<engine> mutants --output <out> [--in-diff <diff>] [-- --features <list>]` in `root`,
/// where `engine` is the provisioned cargo-mutants binary ([`ensure_cargo_mutants`]) invoked
/// by absolute path. The `[rust] features` list rides after cargo-mutants' `--` separator,
/// which forwards it to the cargo build/test runs — so `#[cfg(feature = ...)]` code is
/// compiled and its mutants exercised.
///
/// The exit code is classified by [`classify_mutants_exit`] (`0`/`2`/`3` normal, else fatal).
/// The outer instrumentation env is stripped so a nested run (this rule's own tests under
/// `cargo llvm-cov`) doesn't re-enter the rustc wrapper and hang.
fn run_cargo_mutants(
    engine: &Path,
    root: &Path,
    out: &Path,
    in_diff: Option<&Path>,
    features: &[String],
) -> Result<()> {
    let mut command = Command::new(engine);
    command
        .current_dir(root)
        .arg("mutants")
        .arg("--output")
        .arg(out);
    if let Some(diff) = in_diff {
        command.arg("--in-diff").arg(diff);
    }
    if !features.is_empty() {
        command.args(["--", "--features"]).arg(features.join(","));
    }
    strip_llvm_cov_env(&mut command);
    let output = command.output().context("running cargo-mutants")?;
    classify_mutants_exit(root, &output)
}

/// Classify a finished cargo-mutants run's exit code as a normal outcome or a fatal error.
/// Split from [`run_cargo_mutants`] so the exit-code handling is unit-tested with an injected
/// [`Output`] rather than a real (and, for a timeout, a genuinely slow) engine run.
///
/// cargo-mutants exits `0` when every mutant is caught, `2` when some are missed (survivors),
/// and `3` when some mutants **timed out** and none were missed — all three write an
/// `outcomes.json` the gate reads, and a timeout is inconclusive (this module's own `Timeout`
/// semantics), not a survivor. Any other code — a usage error, or a baseline that didn't
/// build/pass (exit 4) — is fatal.
fn classify_mutants_exit(root: &Path, output: &Output) -> Result<()> {
    match output.status.code() {
        // 0 = all caught, 2 = some missed (survivors), 3 = some timed out with none missed:
        // all three produce an outcomes.json to read, and a timeout is inconclusive.
        Some(0) | Some(2) | Some(3) => Ok(()),
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

    // A normalized result set covering every status: two survivors (Survived + NoCoverage),
    // a caught Killed, an inconclusive-but-viable Timeout, and two unviable mutants
    // (CompileError / RuntimeError). `snake_case` on the wire; an extra field is ignored.
    const NORMALIZED: &str = r#"[
        {"file": "src/a.ts", "line": 2, "status": "survived",
         "mutator": "ConditionalExpression", "replacement": "true", "id": "ignored"},
        {"file": "src/a.ts", "line": 5, "status": "no_coverage", "mutator": "ArithmeticOperator"},
        {"file": "src/a.ts", "line": 9, "status": "killed",
         "mutator": "BooleanLiteral", "replacement": "false"},
        {"file": "src/a.ts", "line": 12, "status": "timeout", "mutator": "BlockStatement"},
        {"file": "src/a.ts", "line": 15, "status": "compile_error", "mutator": "OptionalChaining"},
        {"file": "src/a.ts", "line": 18, "status": "runtime_error", "mutator": "StringLiteral"}
    ]"#;

    #[test]
    fn parses_the_normalized_schema() {
        let mutants = parse_normalized_results(NORMALIZED).expect("valid normalized results");
        assert_eq!(mutants.len(), 6);
        assert_eq!(mutants[0].status, MutantStatus::Survived);
        assert_eq!(mutants[1].status, MutantStatus::NoCoverage);
        assert_eq!(mutants[0].replacement.as_deref(), Some("true"));
        assert_eq!(mutants[1].replacement, None);
    }

    #[test]
    fn normalized_survivors_are_survived_and_nocoverage_only() {
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
        let survivors = normalized_survivors(&mutants);
        // Survived (2) + NoCoverage (5); not killed/timeout/compile/runtime.
        assert_eq!(survivors.len(), 2);
        assert_eq!((survivors[0].line, survivors[1].line), (2, 5));
        // Replacement is folded into the description when present, omitted otherwise.
        assert!(survivors[0].description.contains("ConditionalExpression"));
        assert!(survivors[0].description.contains("-> true"));
        assert_eq!(survivors[1].description, "ArithmeticOperator");
    }

    #[test]
    fn normalized_mutated_lines_collects_only_viable_mutants() {
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
        // Survived/Killed/NoCoverage/Timeout ran; CompileError/RuntimeError never produced
        // a viable mutant.
        assert_eq!(
            normalized_mutated_lines(&mutants),
            [2u32, 5, 9, 12]
                .into_iter()
                .map(|line| ("src/a.ts".to_string(), line))
                .collect()
        );
    }

    #[test]
    fn normalized_conclusive_count_is_survived_killed_and_nocoverage() {
        // Survived (2) + NoCoverage (5) + Killed (9) were judged; Timeout ran but judged
        // nothing, and CompileError / RuntimeError never produced a viable mutant.
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
        assert_eq!(normalized_conclusive_count(&mutants), 3);
        assert_eq!(normalized_conclusive_count(&[]), 0);
    }

    #[test]
    fn evaluate_normalized_reports_unexempted_survivors() {
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
        let kept = evaluate_normalized(&mutants, &[], &BTreeMap::new()).unwrap();
        assert_eq!(kept.len(), 2, "both survivors stand with no exemptions");
    }

    #[test]
    fn evaluate_normalized_drops_a_whole_file_exemption() {
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
        let kept =
            evaluate_normalized(&mutants, &["src/a.ts".to_string()], &BTreeMap::new()).unwrap();
        assert!(
            kept.is_empty(),
            "the whole-file exemption lifts both survivors"
        );
    }

    #[test]
    fn evaluate_normalized_drops_a_line_scoped_exemption() {
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
        let line_scoped = BTreeMap::from([("src/a.ts".to_string(), BTreeSet::from([2u32]))]);
        let kept = evaluate_normalized(&mutants, &[], &line_scoped).unwrap();
        // Line 2's survivor is lifted; line 5's still stands.
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].line, 5);
    }

    #[test]
    fn evaluate_normalized_rejects_exempting_a_caught_line() {
        // Line 9 had only a Killed mutant (viable, no survivor) — over-exemption is an error,
        // via the shared determinism guard.
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
        let line_scoped = BTreeMap::from([("src/a.ts".to_string(), BTreeSet::from([9u32]))]);
        let err = evaluate_normalized(&mutants, &[], &line_scoped).unwrap_err();
        assert!(
            err.to_string().contains("all caught") && err.to_string().contains("src/a.ts:9"),
            "got: {err}"
        );
    }

    #[test]
    fn evaluate_normalized_leaves_an_unviable_listed_line_alone() {
        // Line 15 had only a CompileError (no viable mutant) — neither an error nor a drop;
        // the real survivors still stand.
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
        let line_scoped = BTreeMap::from([("src/a.ts".to_string(), BTreeSet::from([15u32]))]);
        let kept = evaluate_normalized(&mutants, &[], &line_scoped).unwrap();
        assert_eq!(kept.len(), 2);
    }

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
    fn conclusive_count_is_caught_plus_missed() {
        // The MissedMutant and the CaughtMutant were judged; the Baseline is not a mutant.
        let report = parse_mutants_report(SAMPLE).unwrap();
        assert_eq!(conclusive_count(&report), 2);
        assert_eq!(conclusive_count(&MutantsReport { outcomes: vec![] }), 0);
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

    #[test]
    fn rebase_report_paths_strips_the_workspace_prefix() {
        // A workspace-member crate's report addresses files workspace-root-relative
        // (`member/src/lib.rs`); gating addresses them scan-path-relative (`src/lib.rs`).
        let report = parse_mutants_report(SAMPLE).unwrap();
        let prefixed = MutantsReport {
            outcomes: report
                .outcomes
                .iter()
                .cloned()
                .map(|mut outcome| {
                    if let Scenario::Mutant(mutant) = &mut outcome.scenario {
                        mutant.file = format!("member/{}", mutant.file);
                    }
                    outcome
                })
                .collect(),
        };
        let rebased = rebase_report_paths(prefixed, Some("member"));
        let survivors = unexplained_survivors(&rebased, &[]);
        assert_eq!(survivors.len(), 1);
        assert_eq!(survivors[0].file, "src/lib.rs");
        // The baseline outcome carries no path and passes through.
        assert_eq!(rebased.outcomes.len(), 3);
    }

    #[test]
    fn rebase_report_paths_drops_an_out_of_scope_mutant_and_keeps_none_identity() {
        let report = parse_mutants_report(SAMPLE).unwrap();
        // `src/lib.rs` / `src/other.rs` don't carry the `member/` prefix: out of scope.
        let rebased = rebase_report_paths(report.clone(), Some("member"));
        assert_eq!(
            rebased.outcomes.len(),
            1,
            "only the pathless baseline outcome remains"
        );
        // No prefix (a standalone crate): the report passes through untouched.
        let unchanged = rebase_report_paths(report, None);
        assert_eq!(unchanged.outcomes.len(), 3);
        assert_eq!(unexplained_survivors(&unchanged, &[])[0].file, "src/lib.rs");
    }

    #[test]
    fn scan_prefix_is_the_scan_path_relative_to_the_package_root() {
        assert_eq!(
            scan_prefix(Path::new("/repo/pkg/src"), Path::new("/repo/pkg")),
            Some("src".to_string())
        );
        assert_eq!(
            scan_prefix(Path::new("/repo/pkg/src/nested"), Path::new("/repo/pkg")),
            Some("src/nested".to_string())
        );
        // The scan path is the package root itself: nothing to prefix.
        assert_eq!(
            scan_prefix(Path::new("/repo/pkg"), Path::new("/repo/pkg")),
            None
        );
        // Relative scan paths resolve the same way.
        assert_eq!(
            scan_prefix(Path::new("pkg/src"), Path::new("pkg")),
            Some("src".to_string())
        );
    }

    #[test]
    fn prefix_mutate_specs_rebases_diff_ranges_onto_the_package_root() {
        let specs = vec!["index.ts:8-11".to_string(), "a/b.ts:2-2".to_string()];
        assert_eq!(
            prefix_mutate_specs(specs.clone(), Some("src")),
            vec![
                "src/index.ts:8-11".to_string(),
                "src/a/b.ts:2-2".to_string()
            ]
        );
        assert_eq!(prefix_mutate_specs(specs.clone(), None), specs);
    }

    #[test]
    fn scan_scoped_mutate_globs_mirror_strykers_default_under_the_scan_path() {
        assert_eq!(
            scan_scoped_mutate_globs("src"),
            vec![
                "src/**/!(*.+(s|S)pec|*.+(t|T)est).+(cjs|mjs|js|ts|mts|cts|jsx|tsx|html|vue|svelte)"
                    .to_string(),
                "!src/**/__tests__/**/*.+(cjs|mjs|js|ts|mts|cts|jsx|tsx|html|vue|svelte)"
                    .to_string(),
            ]
        );
    }

    #[test]
    fn to_scan_relative_strips_the_prefix_and_drops_out_of_scope_mutants() {
        let mutants = parse_normalized_results(
            r#"[
                {"file": "src/a.ts", "line": 2, "status": "survived", "mutator": "X"},
                {"file": "tests/e2e/t.ts", "line": 9, "status": "survived", "mutator": "X"}
            ]"#,
        )
        .unwrap();
        let rebased = to_scan_relative(mutants.clone(), Some("src"));
        assert_eq!(rebased.len(), 1, "the out-of-scan-path mutant is dropped");
        assert_eq!(rebased[0].file, "a.ts");
        // No prefix (the scan path is the package root): paths pass through untouched.
        let unchanged = to_scan_relative(mutants, None);
        assert_eq!(unchanged.len(), 2);
        assert_eq!(unchanged[0].file, "src/a.ts");
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

    #[test]
    fn is_mutatable_py_keeps_sources_and_drops_tests() {
        assert!(is_mutatable_py("calc.py"));
        assert!(is_mutatable_py("pkg/util.py"));
        assert!(!is_mutatable_py("calc_test.py"));
        assert!(!is_mutatable_py("test_calc.py"));
        assert!(!is_mutatable_py("pkg/conftest.py"));
        assert!(!is_mutatable_py("README.md"));
    }

    #[test]
    fn mutated_lines_collects_caught_and_missed() {
        // The MissedMutant (src/lib.rs:7) and the CaughtMutant (src/other.rs:3) are both
        // viable, conclusive mutants; the Baseline is not.
        let report = parse_mutants_report(SAMPLE).unwrap();
        assert_eq!(
            mutated_lines(&report),
            [
                ("src/lib.rs".to_string(), 7),
                ("src/other.rs".to_string(), 3)
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn evaluate_scoped_drops_a_survivor_on_an_exempt_line() {
        let report = parse_mutants_report(SAMPLE).unwrap();
        let line_scoped = BTreeMap::from([("src/lib.rs".to_string(), BTreeSet::from([7u32]))]);
        let kept = evaluate_scoped(
            cargo_mutants_survivors(&report),
            &mutated_lines(&report),
            &[],
            &line_scoped,
        )
        .unwrap();
        assert!(
            kept.is_empty(),
            "the src/lib.rs:7 survivor should be lifted"
        );
    }

    #[test]
    fn evaluate_scoped_rejects_exempting_a_caught_line() {
        // src/other.rs:3 had only a caught mutant (no survivor) — over-exemption.
        let report = parse_mutants_report(SAMPLE).unwrap();
        let line_scoped = BTreeMap::from([("src/other.rs".to_string(), BTreeSet::from([3u32]))]);
        let err = evaluate_scoped(
            cargo_mutants_survivors(&report),
            &mutated_lines(&report),
            &[],
            &line_scoped,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("all caught") && err.to_string().contains("src/other.rs:3"),
            "got: {err}"
        );
    }

    #[test]
    fn evaluate_scoped_leaves_an_unmutated_listed_line_alone() {
        // Line 99 has no mutant at all (e.g. outside a `--base` diff) — neither an error
        // nor a drop; the real survivor on line 7 still stands.
        let report = parse_mutants_report(SAMPLE).unwrap();
        let line_scoped = BTreeMap::from([("src/lib.rs".to_string(), BTreeSet::from([99u32]))]);
        let kept = evaluate_scoped(
            cargo_mutants_survivors(&report),
            &mutated_lines(&report),
            &[],
            &line_scoped,
        )
        .unwrap();
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].line, 7);
    }

    #[test]
    fn evaluate_scoped_still_honors_a_whole_file_exemption() {
        let report = parse_mutants_report(SAMPLE).unwrap();
        let kept = evaluate_scoped(
            cargo_mutants_survivors(&report),
            &mutated_lines(&report),
            &["src/lib.rs".to_string()],
            &BTreeMap::new(),
        )
        .unwrap();
        assert!(kept.is_empty());
    }

    fn unique_tmp() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "tc-provision-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn provision_returns_an_existing_binary_without_installing() {
        let tmp = unique_tmp();
        let bin = tmp.join("bin").join("cargo-mutants");
        let lock = tmp.join(".install.lock");
        std::fs::create_dir_all(bin.parent().unwrap()).unwrap();
        std::fs::write(&bin, b"binary").unwrap();
        let mut installed = false;
        let got = provision(&bin, &lock, || {
            installed = true;
            Ok(())
        })
        .unwrap();
        assert_eq!(got, bin);
        assert!(!installed, "a present binary must not be reinstalled");
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn provision_installs_when_the_binary_is_absent() {
        let tmp = unique_tmp();
        let bin = tmp.join("bin").join("cargo-mutants");
        let lock = tmp.join(".install.lock");
        let mut installed = false;
        let got = provision(&bin, &lock, || {
            installed = true;
            std::fs::create_dir_all(bin.parent().unwrap()).unwrap();
            std::fs::write(&bin, b"binary").unwrap();
            Ok(())
        })
        .unwrap();
        assert!(installed, "an absent binary must be installed");
        assert_eq!(got, bin);
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn provision_errors_when_install_produces_no_binary() {
        let tmp = unique_tmp();
        let bin = tmp.join("bin").join("cargo-mutants");
        let lock = tmp.join(".install.lock");
        let err = provision(&bin, &lock, || Ok(())).unwrap_err();
        assert!(
            err.to_string().contains("cargo-mutants is not at"),
            "got: {err}"
        );
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn provision_propagates_an_install_failure() {
        let tmp = unique_tmp();
        let bin = tmp.join("bin").join("cargo-mutants");
        let lock = tmp.join(".install.lock");
        let err = provision(&bin, &lock, || bail!("install blew up")).unwrap_err();
        assert!(err.to_string().contains("install blew up"), "got: {err}");
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn provision_does_not_duplicate_the_install_under_concurrent_callers() {
        // On a cold cache, N concurrent callers must share one install, not each run
        // their own — cargo-mutants' from-source compile duplicated N times (instead of once)
        // is what turned a ~1-minute cold-cache cost into ~7 minutes under nextest. A
        // barrier forces both threads to observe the absent binary together, and the install
        // closure sleeps briefly to widen the race window so this reproduces deterministically
        // rather than flakily.
        use std::sync::{Arc, Barrier};
        use std::thread;
        use std::time::Duration;

        let tmp = unique_tmp();
        let bin = tmp.join("bin").join("cargo-mutants");
        let lock = tmp.join(".install.lock");
        let install_count = Arc::new(AtomicU64::new(0));
        let barrier = Arc::new(Barrier::new(2));

        let handles: Vec<_> = (0..2)
            .map(|_| {
                let bin = bin.clone();
                let lock = lock.clone();
                let install_count = Arc::clone(&install_count);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    provision(&bin, &lock, || {
                        install_count.fetch_add(1, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(50));
                        std::fs::create_dir_all(bin.parent().unwrap()).unwrap();
                        std::fs::write(&bin, b"binary").unwrap();
                        Ok(())
                    })
                })
            })
            .collect();

        for h in handles {
            h.join()
                .expect("provisioning thread must not panic")
                .unwrap();
        }

        assert_eq!(
            install_count.load(Ordering::SeqCst),
            1,
            "two concurrent callers on a cold cache must share one install, not each run their own"
        );
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn resolve_cache_base_prefers_xdg_then_home_then_temp() {
        let xdg = |s: &str| Some(OsString::from(s));
        // XDG wins when set and non-empty.
        assert_eq!(
            resolve_cache_base(xdg("/xdg"), xdg("/home")),
            PathBuf::from("/xdg")
        );
        // An empty XDG falls through to $HOME/.cache.
        assert_eq!(
            resolve_cache_base(xdg(""), xdg("/home")),
            PathBuf::from("/home/.cache")
        );
        // A missing XDG likewise uses $HOME/.cache.
        assert_eq!(
            resolve_cache_base(None, xdg("/home")),
            PathBuf::from("/home/.cache")
        );
        // Neither set → the temp dir.
        assert_eq!(resolve_cache_base(None, None), std::env::temp_dir());
        assert_eq!(
            resolve_cache_base(xdg(""), Some(OsString::new())),
            std::env::temp_dir()
        );
    }

    #[test]
    fn cache_root_is_absolute_and_version_scoped() {
        let root = cargo_mutants_cache_root();
        assert!(
            root.ends_with(format!("cargo-mutants-{CARGO_MUTANTS_VERSION}")),
            "version-scoped; got {root:?}"
        );
        assert!(
            root.to_string_lossy().contains("testing-conventions"),
            "tool-namespaced; got {root:?}"
        );
        // A real base dir (HOME/XDG in the test env) makes it absolute — not an empty path.
        assert!(
            root.is_absolute(),
            "expected an absolute path; got {root:?}"
        );
    }

    #[test]
    fn install_argv_pins_the_version_and_isolates_the_root() {
        let argv: Vec<String> = install_argv(Path::new("/cache/cargo-mutants-27"))
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            argv,
            vec![
                "install",
                "cargo-mutants",
                "--locked",
                "--version",
                CARGO_MUTANTS_VERSION,
                "--root",
                "/cache/cargo-mutants-27",
            ]
        );
    }

    #[cfg(unix)]
    fn fake_output(code: i32, stderr: &str) -> Output {
        use std::os::unix::process::ExitStatusExt;
        Output {
            status: std::process::ExitStatus::from_raw(code << 8),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[cfg(unix)]
    #[test]
    fn run_install_succeeds_on_a_zero_exit() {
        let mut ran = false;
        run_install(Path::new("/cache/root"), |command| {
            ran = true;
            // The pinned argv reaches the runner.
            let argv: Vec<String> = command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            assert!(argv.contains(&CARGO_MUTANTS_VERSION.to_string()));
            Ok(fake_output(0, ""))
        })
        .unwrap();
        assert!(ran);
    }

    #[cfg(unix)]
    #[test]
    fn run_install_reports_a_nonzero_exit_with_the_engine_output() {
        let err = run_install(Path::new("/cache/root"), |_| {
            Ok(fake_output(1, "error: could not compile cargo-mutants"))
        })
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("failed to provision cargo-mutants")
                && err.to_string().contains("could not compile"),
            "got: {err}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_install_propagates_a_spawn_failure() {
        let err = run_install(Path::new("/cache/root"), |_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no cargo",
            ))
        })
        .unwrap_err();
        assert!(
            err.to_string().contains("is cargo installed?"),
            "got: {err}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn classify_mutants_exit_accepts_the_caught_and_survivor_exits() {
        // 0 (all caught) and 2 (some missed/survived) both leave an outcomes.json to read.
        classify_mutants_exit(Path::new("/crate"), &fake_output(0, "")).unwrap();
        classify_mutants_exit(Path::new("/crate"), &fake_output(2, "")).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn classify_mutants_exit_accepts_a_timeout_exit_3() {
        // cargo-mutants exits 3 when mutants timed out and none were missed — an
        // inconclusive-not-fatal outcome (this module's own `Timeout` semantics). It still
        // wrote an outcomes.json, so the run is a pass, not the "baseline failure" bail.
        classify_mutants_exit(Path::new("/crate"), &fake_output(3, ""))
            .expect("a timeout (exit 3) is inconclusive, not fatal");
    }

    #[cfg(unix)]
    #[test]
    fn classify_mutants_exit_is_fatal_on_a_baseline_failure() {
        // Exit 4 (the clean/baseline build or test failed) — and any other code — stays fatal.
        let err = classify_mutants_exit(Path::new("/crate"), &fake_output(4, "baseline broke"))
            .unwrap_err();
        assert!(
            err.to_string().contains("did not run cleanly")
                && err.to_string().contains("baseline broke"),
            "got: {err}"
        );
    }

    #[test]
    fn cargo_mutants_bin_name_matches_the_platform() {
        let name = cargo_mutants_bin_name();
        if cfg!(windows) {
            assert_eq!(name, "cargo-mutants.exe");
        } else {
            assert_eq!(name, "cargo-mutants");
        }
    }
}
