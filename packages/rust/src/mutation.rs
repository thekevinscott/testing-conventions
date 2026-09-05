//! Mutation testing (`unit mutation`) — the rung above coverage: a test that *runs* a
//! line still passes if you delete its assertions, and a surviving mutant proves it. Each
//! language drives its engine through an adapter; this module measures, the CLI layer gates.

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
/// [`Measurement::EngineNotRun`] from an all-killed [`Measurement::Tested`] keeps a vacuous
/// pass visible, and a counted pass carries its own evidence.
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

/// A source span; the start and end lines are read.
#[derive(Debug, Clone, Deserialize)]
pub struct Span {
    pub start: LineCol,
    pub end: LineCol,
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

/// Parse a `cargo mutants --list --json` export: the crate's discoverable mutants, each
/// with its workspace-root-relative file and span.
fn parse_mutants_list(json: &str) -> Result<Vec<MutantInfo>> {
    serde_json::from_str(json).context("parsing the cargo-mutants mutant list")
}

/// The surviving mutants not lifted by a `mutation` exemption — the rule's findings.
/// `exempt` is the resolved set of crate-root-relative exempt paths; a survivor in an
/// exempt file is dropped.
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

/// The `(file, line)` locations cargo-mutants produced a **viable, conclusive** mutant for —
/// caught or missed, not the inconclusive `Timeout` / `Unviable`. The line-scoped guard reads
/// this to tell an over-exemption from a line that has no mutant at all.
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
/// `mutation` exemption. [`evaluate_scoped`] generalizes this to per-line exemptions.
pub fn evaluate(survivors: Vec<Survivor>, exempt: &[String]) -> Vec<Survivor> {
    survivors
        .into_iter()
        .filter(|survivor| !exempt.iter().any(|path| path == &survivor.file))
        .collect()
}

/// Apply file- and line-scoped `mutation` exemptions to the raw `survivors`, with the
/// determinism guard: a listed line whose mutants were all *caught* is over-exemption and a
/// hard error, while a listed line with no mutant is left alone (it may be off the diff).
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
/// so the Rust core gates on one representation instead of three report formats. The
/// serialized form is `snake_case` (`no_coverage`, `compile_error`, …) — the adapters' wire contract.
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

    /// Whether this came from a **viable, conclusive** mutant — one that actually ran, not one
    /// that never compiled or errored out. The determinism guard reads this.
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
/// `mutation` exemption (with the determinism guard), leaving the rule's findings. This is
/// the engine-agnostic core each language arm feeds once its adapter has normalized.
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

/// Run cargo-mutants over the crate at `root` and return the [`Measurement`], or
/// [`Measurement::EngineNotRun`] for a `base` diff that changes no lines — or no Rust source
/// — under the crate. The tool provisions cargo-mutants itself ([`ensure_cargo_mutants`]).
pub fn measure_rust(
    root: &Path,
    exempt: &[String],
    exempt_lines: &BTreeMap<String, BTreeSet<u32>>,
    base: Option<&str>,
    features: &[String],
) -> Result<Measurement> {
    let out = MutantsOut::new();
    // cargo-mutants addresses files relative to the crate's cargo workspace root, so both the
    // `--in-diff` diff it consumes and the report paths it emits carry the scan path's
    // workspace-relative prefix. A standalone crate is its own workspace root: no prefix.
    let workspace_root = cargo_workspace_root(root)?;
    let prefix = canonical_scan_prefix(root, &workspace_root);
    let mut base_diff = None;
    let diff = match base {
        Some(base) => {
            match write_base_diff(root, &workspace_root, prefix.as_deref(), base, &out)? {
                None => return Ok(Measurement::EngineNotRun),
                Some(path) => {
                    let parsed =
                        parse_base_diff(&std::fs::read_to_string(&path).with_context(|| {
                            format!("reading the written base diff `{}`", path.display())
                        })?);
                    if !parsed.files.iter().any(|file| file.ends_with(".rs")) {
                        return Ok(Measurement::EngineNotRun);
                    }
                    base_diff = Some(parsed);
                    Some(path)
                }
            }
        }
        None => None,
    };
    let engine = ensure_cargo_mutants()?;
    let run = run_cargo_mutants(&engine, root, &out.0, diff.as_deref(), features)?;
    let outcomes = out.0.join("mutants.out").join("outcomes.json");
    // cargo-mutants writes no `outcomes.json` when a run produces no mutants, so a missing
    // report here is a run that judged zero — legitimate only if none of the crate's mutants
    // sits on the diff, which [`zero_mutant_verdict`] proves before the zero can stand.
    let json = match std::fs::read_to_string(&outcomes) {
        Ok(json) => json,
        Err(_) => {
            if let Some(diff) = &base_diff {
                let listed =
                    list_cargo_mutants(&engine, root, features, |command| command.output())?;
                zero_mutant_verdict(&listed, diff, &run)?;
            }
            return Ok(Measurement::Tested {
                count: 0,
                survivors: Vec::new(),
            });
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

/// Run the bundled TypeScript mutation adapter over the scan path at `root` and return the
/// [`Measurement`] — the TS arm, parity with [`measure_rust`]. The adapter runs at the package
/// root and its results are rebased scan-path-relative, so exemption paths match every check.
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
            if ranges.is_empty() {
                return Ok(Measurement::EngineNotRun);
            }
            Some(prefix_mutate_specs(ranges, prefix.as_deref()))
        }
        None => prefix.as_deref().map(scan_scoped_mutate_globs),
    };
    let test_files = prefix.as_deref().map(scan_scoped_test_file_globs);
    let json = run_ts_adapter(
        &package_root,
        adapter,
        mutate.as_deref(),
        test_files.as_deref(),
    )?;
    let mutants = to_scan_relative(parse_normalized_results(&json)?, prefix.as_deref());
    let survivors = evaluate_normalized(&mutants, exempt, exempt_lines)?;
    Ok(Measurement::Tested {
        count: normalized_conclusive_count(&mutants),
        survivors,
    })
}

/// The scan path relative to its package root, as a `/`-joined string. `None` when the scan
/// path *is* the package root, which also covers a loose tree with no manifest.
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

/// The test files under the scan path, addressed from the package root the adapter runs at.
/// Stryker matches these against the project's input files and hands the runner that subset,
/// so vitest stays rooted at the package root and its own `include` resolves unchanged.
fn scan_scoped_test_file_globs(prefix: &str) -> Vec<String> {
    vec![format!("{prefix}/**")]
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

/// The checked working directory for an adapter run rooted at `root`, for the named `engine`.
/// [`crate::tiers::package_root`] hands back `""` for a relative scan path like `src`, and
/// `Command::current_dir("")` fails with the same ENOENT a missing interpreter gives.
fn adapter_cwd<'a>(root: &'a Path, engine: &str) -> Result<&'a Path> {
    let cwd = if root.as_os_str().is_empty() {
        Path::new(".")
    } else {
        root
    };
    if !cwd.is_dir() {
        bail!(
            "the {engine} mutation adapter's working directory `{}` is not a directory",
            cwd.display()
        );
    }
    Ok(cwd)
}

/// The context a failed adapter spawn carries. `Command::output()` surfaces a bare ENOENT
/// that names nothing, so the message names every path the spawn used: the interpreter, the
/// entry point it was handed, and the directory it ran in.
fn spawn_context(interpreter: &str, entry: &str, cwd: &Path) -> String {
    format!(
        "spawning `{interpreter} {entry}` in `{}` (is `{interpreter}` on PATH?)",
        cwd.display()
    )
}

/// Run the bundled TS mutation `adapter` at `package_root` and return the normalized-results
/// JSON it writes. Results go to a temp file the adapter names via `--out`, so Stryker's own
/// stdout logging can't corrupt them; a non-zero adapter exit surfaces its captured output.
fn run_ts_adapter(
    package_root: &Path,
    adapter: &Path,
    mutate: Option<&[String]>,
    test_files: Option<&[String]>,
) -> Result<String> {
    let out = AdapterOut::new();
    std::fs::create_dir_all(&out.0).context("creating the mutation adapter output dir")?;
    let results = out.0.join("results.json");

    let cwd = adapter_cwd(package_root, "TypeScript")?;

    let mut command = Command::new("node");
    command
        .current_dir(cwd)
        .arg(adapter)
        .arg("--out")
        .arg(&results);
    if let Some(specs) = mutate {
        command.arg("--mutate").arg(specs.join(","));
    }
    if let Some(globs) = test_files {
        command.arg("--test-files").arg(globs.join(","));
    }
    let output = command
        .output()
        .with_context(|| spawn_context("node", &adapter.display().to_string(), cwd))?;
    if !output.status.success() {
        bail!(
            "the TypeScript mutation adapter failed in `{}`:\n{}{}",
            cwd.display(),
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

/// Build the Stryker `--mutate` specs scoping a run to the `<base>...HEAD` changed lines, as
/// `<file>:<start>-<end>` ranges. Test and declaration files are filtered out here because
/// passing `--mutate` replaces Stryker's configured set rather than narrowing it.
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
/// [`Measurement`] — the Python arm, parity with [`measure_rust`]. maturin ships the binary
/// directly, so it invokes the adapter as a module resolved from the wheel's own environment.
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
            if modules.is_empty() {
                return Ok(Measurement::EngineNotRun);
            }
            modules
        }
    };
    let json = run_py_adapter(root, &modules)?;
    let mut mutants = parse_normalized_results(&json)?;
    if let Some(changed) = &changed {
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

/// Run the bundled Python mutation adapter over `root` and return the normalized-results
/// JSON it writes. `modules`, when non-empty, scopes the run to those source files; empty
/// runs the whole project. `PYTHONDONTWRITEBYTECODE` keeps `__pycache__` out of the tree.
fn run_py_adapter(root: &Path, modules: &[String]) -> Result<String> {
    let out = AdapterOut::new();
    std::fs::create_dir_all(&out.0).context("creating the mutation adapter output dir")?;
    let results = out.0.join("results.json");

    let cwd = adapter_cwd(root, "Python")?;

    const ENTRY: &str = "-m testing_conventions.mutation.main";
    let mut command = Command::new("python3");
    command
        .current_dir(cwd)
        .args(["-m", "testing_conventions.mutation.main", "--out"])
        .arg(&results)
        .env("PYTHONDONTWRITEBYTECODE", "1");
    for module in modules {
        command.arg("--module").arg(module);
    }
    let output = command
        .output()
        .with_context(|| spawn_context("python3", ENTRY, cwd))?;
    if !output.status.success() {
        bail!(
            "the Python mutation adapter failed in `{}`:\n{}{}",
            cwd.display(),
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

/// The directory of the cargo workspace `root` belongs to. `cargo locate-project --workspace`
/// is the authoritative lookup: membership involves member globs and `exclude` lists a
/// manifest walk can't settle.
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
    manifest.parent().map(Path::to_path_buf).with_context(|| {
        format!(
            "no parent dir for the workspace manifest `{}`",
            manifest.display()
        )
    })
}

/// The scan path's prefix relative to the workspace root ([`scan_prefix`]), over
/// canonicalized paths so a relative CLI scan path resolves against the absolute path
/// `cargo locate-project` reports. `None` when the scan path *is* the workspace root.
fn canonical_scan_prefix(root: &Path, workspace_root: &Path) -> Option<String> {
    let root = root.canonicalize().ok()?;
    let workspace_root = workspace_root.canonicalize().ok()?;
    scan_prefix(&root, &workspace_root)
}

/// Write the `<base>...HEAD` diff cargo-mutants' `--in-diff` scopes to, returning its path —
/// or `None` when the diff is empty. cargo-mutants matches `--in-diff` paths relative to the
/// cargo workspace root, so the diff is generated there, `--relative`, with `prefix` as a pathspec.
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

/// The tool's own reading of a base diff: the changed files (new-side paths, `b/` stripped)
/// and the inserted line numbers per file. Paths stay workspace-root-relative, the basis
/// cargo-mutants addresses mutants on.
struct BaseDiff {
    files: Vec<String>,
    inserted: BTreeMap<String, BTreeSet<u32>>,
}

/// Parse a unified diff into a [`BaseDiff`]. Each hunk body is consumed by the counts its `@@`
/// header declares, so a content line beginning `+++` or `---` never reads as a file header.
/// A deleted file (`+++ /dev/null`) carries neither a changed file nor inserted lines.
fn parse_base_diff(diff: &str) -> BaseDiff {
    let mut files = Vec::new();
    let mut inserted: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut lines = diff.lines();
    while let Some(line) = lines.next() {
        if let Some(path) = line.strip_prefix("+++ ") {
            current = (path != "/dev/null").then(|| {
                let path = path.strip_prefix("b/").unwrap_or(path).to_string();
                files.push(path.clone());
                path
            });
        } else if let Some(header) = line.strip_prefix("@@ ") {
            let Some((new_start, old_count, new_count)) = parse_hunk_header(header) else {
                continue;
            };
            let mut new_line = new_start;
            let (mut old_left, mut new_left) = (old_count, new_count);
            while old_left > 0 || new_left > 0 {
                let Some(line) = lines.next() else { break };
                if line.starts_with('\\') {
                    // "\ No newline at end of file" annotates the previous line and
                    // counts against neither side.
                } else if line.starts_with('+') {
                    if let Some(file) = &current {
                        inserted.entry(file.clone()).or_default().insert(new_line);
                    }
                    new_line += 1;
                    new_left = new_left.saturating_sub(1);
                } else if line.starts_with('-') {
                    old_left = old_left.saturating_sub(1);
                } else {
                    new_line += 1;
                    old_left = old_left.saturating_sub(1);
                    new_left = new_left.saturating_sub(1);
                }
            }
        }
    }
    BaseDiff { files, inserted }
}

/// The `(new_start, old_count, new_count)` of a hunk header's `-a[,b] +c[,d]` part.
fn parse_hunk_header(header: &str) -> Option<(u32, u32, u32)> {
    let mut parts = header.split(' ');
    let (_, old_count) = parse_range(parts.next()?.strip_prefix('-')?)?;
    let (new_start, new_count) = parse_range(parts.next()?.strip_prefix('+')?)?;
    Some((new_start, old_count, new_count))
}

/// A hunk range `start[,count]`; the count defaults to 1.
fn parse_range(range: &str) -> Option<(u32, u32)> {
    match range.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((range.parse().ok()?, 1)),
    }
}

/// Rebase a cargo-mutants report's workspace-root-relative mutant paths onto the scan path, so
/// exemption matching and survivor reporting address scan-path-relative files. A baseline
/// outcome carries no path and passes through; a mutant outside the scan path is dropped.
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
/// provisioning it on first use. cargo ships no library form, so — unlike the in-process
/// TS/Python adapters — a pinned `cargo install` runs into the tool's own cache directory.
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
/// `lock_path`, re-check, and run `install` if still absent. The lock keeps N concurrent
/// callers to one from-source compile instead of N. An install producing no binary is an error.
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

/// Provision cargo-mutants into `root`, executing the built `cargo install` with `run`, which
/// is injected so a test drives both branches with a fake. The coverage-instrumentation env is
/// stripped so the compile doesn't re-enter a `cargo llvm-cov` rustc wrapper.
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

/// Run the cargo-mutants argv ([`mutants_argv`]) in `root`, where `engine` is the provisioned
/// binary invoked by absolute path, returning its [`Output`]. The outer instrumentation env is
/// stripped so a nested run (this rule's own tests under coverage) can't re-enter the wrapper.
fn run_cargo_mutants(
    engine: &Path,
    root: &Path,
    out: &Path,
    in_diff: Option<&Path>,
    features: &[String],
) -> Result<Output> {
    let mut command = Command::new(engine);
    command
        .current_dir(root)
        .args(mutants_argv(out, in_diff, features));
    strip_llvm_cov_env(&mut command);
    let output = command.output().context("running cargo-mutants")?;
    classify_mutants_exit(root, &output)?;
    Ok(output)
}

/// Decide whether an engine run that judged zero mutants is legitimate: `listed` is the crate's
/// full mutant list and `diff` the tool's own reading of the diff the engine filtered by. A
/// listed mutant whose span touches an inserted line proves the filter dropped real mutants.
fn zero_mutant_verdict(listed: &[MutantInfo], diff: &BaseDiff, run: &Output) -> Result<()> {
    let dropped: Vec<&MutantInfo> = listed
        .iter()
        .filter(|mutant| {
            diff.inserted.get(&mutant.file).is_some_and(|lines| {
                lines
                    .range(mutant.span.start.line..=mutant.span.end.line)
                    .next()
                    .is_some()
            })
        })
        .collect();
    if dropped.is_empty() {
        return Ok(());
    }
    let sites: Vec<String> = dropped
        .iter()
        .map(|mutant| {
            format!(
                "  {}:{}: {}",
                mutant.file, mutant.span.start.line, mutant.name
            )
        })
        .collect();
    bail!(
        "cargo-mutants tested no mutants, but {} of the crate's {} mutant site(s) sit on the diff's inserted lines — the changed-line filter dropped real mutants:\n{}\nengine output:\n{}{}",
        dropped.len(),
        listed.len(),
        sites.join("\n"),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    )
}

/// The argv for one cargo-mutants mutant listing: `mutants --list --json
/// [--features <list>]`, mirroring the run's own feature selection so both see the same
/// mutant set.
fn list_argv(features: &[String]) -> Vec<OsString> {
    let mut argv = vec![
        OsString::from("mutants"),
        OsString::from("--list"),
        OsString::from("--json"),
    ];
    if !features.is_empty() {
        argv.push(OsString::from("--features"));
        argv.push(OsString::from(features.join(",")));
    }
    argv
}

/// List the crate's discoverable mutants via `cargo mutants --list --json`, executing the
/// built command with `run`. `run` is injected so a test drives the success and failure
/// branches with a fake (no real engine).
fn list_cargo_mutants(
    engine: &Path,
    root: &Path,
    features: &[String],
    run: impl FnOnce(&mut Command) -> std::io::Result<Output>,
) -> Result<Vec<MutantInfo>> {
    let mut command = Command::new(engine);
    command.current_dir(root).args(list_argv(features));
    strip_llvm_cov_env(&mut command);
    let output = run(&mut command).context("listing the crate's mutants with cargo-mutants")?;
    if !output.status.success() {
        bail!(
            "cargo-mutants --list failed in `{}`:\n{}{}",
            root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    parse_mutants_list(&String::from_utf8_lossy(&output.stdout))
}

/// The argv for one cargo-mutants run: `mutants --output <out> [--in-diff <diff>] [--features
/// <list>]`. `features` rides on the engine's own `--features` so it reaches every cargo
/// invocation; after a `--` it would reach `cargo test` alone and the baseline build would fail.
fn mutants_argv(out: &Path, in_diff: Option<&Path>, features: &[String]) -> Vec<OsString> {
    let mut argv = vec![
        OsString::from("mutants"),
        OsString::from("--output"),
        out.as_os_str().to_os_string(),
    ];
    if let Some(diff) = in_diff {
        argv.push(OsString::from("--in-diff"));
        argv.push(diff.as_os_str().to_os_string());
    }
    if !features.is_empty() {
        argv.push(OsString::from("--features"));
        argv.push(OsString::from(features.join(",")));
    }
    argv
}

/// Classify a finished cargo-mutants run's exit code as a normal outcome or a fatal error.
/// `0` (all caught), `2` (some missed) and `3` (some timed out, none missed) each write an
/// `outcomes.json` the gate reads. Any other code — a baseline that didn't build (4) — is fatal.
fn classify_mutants_exit(root: &Path, output: &Output) -> Result<()> {
    match output.status.code() {
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
        assert_eq!(survivors.len(), 2);
        assert_eq!((survivors[0].line, survivors[1].line), (2, 5));
        assert!(survivors[0].description.contains("ConditionalExpression"));
        assert!(survivors[0].description.contains("-> true"));
        assert_eq!(survivors[1].description, "ArithmeticOperator");
    }

    #[test]
    fn normalized_mutated_lines_collects_only_viable_mutants() {
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
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
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].line, 5);
    }

    #[test]
    fn evaluate_normalized_rejects_exempting_a_caught_line() {
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
        let mutants = parse_normalized_results(NORMALIZED).unwrap();
        let line_scoped = BTreeMap::from([("src/a.ts".to_string(), BTreeSet::from([15u32]))]);
        let kept = evaluate_normalized(&mutants, &[], &line_scoped).unwrap();
        assert_eq!(kept.len(), 2);
    }

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
        assert_eq!(survivors.len(), 1);
        assert_eq!(survivors[0].file, "src/lib.rs");
        assert_eq!(survivors[0].line, 7);
        assert!(survivors[0].description.contains("replace > with =="));
    }

    #[test]
    fn a_survivor_description_carries_no_location_prefix() {
        let report = parse_mutants_report(SAMPLE).unwrap();
        let survivors = unexplained_survivors(&report, &[]);
        assert_eq!(
            survivors[0].description, "replace > with == in is_positive",
            "the name's embedded `file:line:col:` prefix is stripped"
        );
    }

    #[test]
    fn conclusive_count_is_caught_plus_missed() {
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
        assert_eq!(rebased.outcomes.len(), 3);
    }

    #[test]
    fn rebase_report_paths_drops_an_out_of_scope_mutant_and_keeps_none_identity() {
        let report = parse_mutants_report(SAMPLE).unwrap();
        let rebased = rebase_report_paths(report.clone(), Some("member"));
        assert_eq!(
            rebased.outcomes.len(),
            1,
            "only the pathless baseline outcome remains"
        );
        let unchanged = rebase_report_paths(report, None);
        assert_eq!(unchanged.outcomes.len(), 3);
        assert_eq!(unexplained_survivors(&unchanged, &[])[0].file, "src/lib.rs");
    }

    #[test]
    fn adapter_cwd_normalises_the_empty_package_root_to_the_current_dir() {
        // `tiers::package_root` yields `""` for a relative scan path such as `src`, and
        // `Command::current_dir("")` fails with ENOENT — which the adapter's error context
        // mislabelled as a missing `node`, hitting every TypeScript consumer of the gate.
        assert_eq!(
            adapter_cwd(Path::new(""), "TypeScript").unwrap(),
            Path::new(".")
        );
        assert_eq!(
            adapter_cwd(Path::new("src"), "TypeScript").unwrap(),
            Path::new("src")
        );
    }

    #[test]
    fn adapter_cwd_rejects_a_directory_that_is_not_there() {
        // `Command::output()` reports a missing working directory with the same ENOENT as a
        // missing interpreter, so an unchecked spawn blames the interpreter for a wrong path.
        let err = adapter_cwd(Path::new("no/such/dir"), "Python")
            .expect_err("a directory that is not there is an error");
        assert_eq!(
            err.to_string(),
            "the Python mutation adapter's working directory `no/such/dir` is not a directory"
        );
    }

    #[test]
    fn spawn_context_names_the_interpreter_the_entry_and_the_working_directory() {
        assert_eq!(
            spawn_context("node", "/pkg/dist/mutation/main.js", Path::new("/pkg")),
            "spawning `node /pkg/dist/mutation/main.js` in `/pkg` (is `node` on PATH?)"
        );
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
        assert_eq!(
            scan_prefix(Path::new("/repo/pkg"), Path::new("/repo/pkg")),
            None
        );
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
    fn scan_scoped_test_file_globs_narrow_the_run_without_moving_the_runner_root() {
        assert_eq!(
            scan_scoped_test_file_globs("src"),
            vec!["src/**".to_string()]
        );
        assert_eq!(
            scan_scoped_test_file_globs("packages/core/src"),
            vec!["packages/core/src/**".to_string()]
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
        // On a cold cache, N concurrent callers must share one install: cargo-mutants' compile
        // duplicated N times turned a ~1-minute cold-cache cost into ~7 minutes. The barrier and
        // the sleeping installer widen the race window so this reproduces deterministically.
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
        assert_eq!(
            resolve_cache_base(xdg("/xdg"), xdg("/home")),
            PathBuf::from("/xdg")
        );
        assert_eq!(
            resolve_cache_base(xdg(""), xdg("/home")),
            PathBuf::from("/home/.cache")
        );
        assert_eq!(
            resolve_cache_base(None, xdg("/home")),
            PathBuf::from("/home/.cache")
        );
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

    #[test]
    fn mutants_argv_enables_features_on_the_engine_itself() {
        let argv = |diff, features: &[&str]| -> Vec<String> {
            mutants_argv(
                Path::new("/out"),
                diff,
                &features.iter().map(|f| f.to_string()).collect::<Vec<_>>(),
            )
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
        };
        assert_eq!(
            argv(None, &["cli", "boost"]),
            vec!["mutants", "--output", "/out", "--features", "cli,boost"]
        );
        assert_eq!(
            argv(Some(Path::new("/out/base.diff")), &["cli"]),
            vec![
                "mutants",
                "--output",
                "/out",
                "--in-diff",
                "/out/base.diff",
                "--features",
                "cli",
            ]
        );
        assert_eq!(argv(None, &[]), vec!["mutants", "--output", "/out"]);
    }

    #[test]
    fn list_argv_mirrors_the_run_feature_selection() {
        let argv = |features: &[&str]| -> Vec<String> {
            list_argv(&features.iter().map(|f| f.to_string()).collect::<Vec<_>>())
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect()
        };
        assert_eq!(argv(&[]), vec!["mutants", "--list", "--json"]);
        assert_eq!(
            argv(&["cli", "boost"]),
            vec!["mutants", "--list", "--json", "--features", "cli,boost"]
        );
    }

    #[test]
    fn parse_base_diff_maps_inserted_lines_per_hunk() {
        let diff = "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,4 +1,5 @@
 fn a() {}
+fn b() {}
 fn c() {}
-fn d() {}
+fn e() {}
 fn f() {}
@@ -10,2 +11,4 @@
 tail
+one
+two
 more
";
        let parsed = parse_base_diff(diff);
        assert_eq!(parsed.files, vec!["src/lib.rs"]);
        assert_eq!(
            parsed.inserted.get("src/lib.rs"),
            Some(&BTreeSet::from([2, 4, 12, 13]))
        );
    }

    #[test]
    fn parse_base_diff_leaves_a_deletion_only_file_without_inserted_lines() {
        let diff = "\
--- a/src/gone.rs
+++ b/src/gone.rs
@@ -5,2 +4,0 @@
-x
-y
";
        let parsed = parse_base_diff(diff);
        assert_eq!(parsed.files, vec!["src/gone.rs"]);
        assert!(parsed.inserted.is_empty());
    }

    #[test]
    fn parse_base_diff_skips_a_deleted_file() {
        let diff = "\
--- a/src/dead.rs
+++ /dev/null
@@ -1,2 +0,0 @@
-a
-b
";
        let parsed = parse_base_diff(diff);
        assert!(parsed.files.is_empty());
        assert!(parsed.inserted.is_empty());
    }

    #[test]
    fn parse_base_diff_consumes_hunk_bodies_by_count_so_content_never_reads_as_a_header() {
        // The inserted content line begins with `+++`; consuming the hunk by its declared
        // counts keeps it a body line, not a second file header.
        let diff = "\
+++ b/notes.txt
@@ -1,1 +1,2 @@
 keep
++++ not a header
";
        let parsed = parse_base_diff(diff);
        assert_eq!(parsed.files, vec!["notes.txt"]);
        assert_eq!(parsed.inserted.get("notes.txt"), Some(&BTreeSet::from([2])));
    }

    #[test]
    fn parse_base_diff_defaults_an_elided_hunk_count_to_one() {
        let diff = "\
+++ b/one.txt
@@ -1 +1 @@
-old
+new
";
        let parsed = parse_base_diff(diff);
        assert_eq!(parsed.inserted.get("one.txt"), Some(&BTreeSet::from([1])));
    }

    #[test]
    fn parse_base_diff_skips_no_newline_annotations_mid_hunk() {
        let diff = "\
+++ b/n.txt
@@ -1 +1 @@
-old
\\ No newline at end of file
+new
\\ No newline at end of file
";
        let parsed = parse_base_diff(diff);
        assert_eq!(parsed.inserted.get("n.txt"), Some(&BTreeSet::from([1])));
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
    fn fake_stdout(code: i32, stdout: &str) -> Output {
        use std::os::unix::process::ExitStatusExt;
        Output {
            status: std::process::ExitStatus::from_raw(code << 8),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    #[cfg(unix)]
    #[test]
    fn list_cargo_mutants_parses_the_listing_from_a_clean_run() {
        let json = r#"[{"file": "src/lib.rs", "name": "replace add -> 0",
            "span": {"start": {"line": 3, "column": 1}, "end": {"line": 5, "column": 2}}}]"#;
        let listed = list_cargo_mutants(
            Path::new("/cache/bin/cargo-mutants"),
            Path::new("/crate"),
            &["cli".to_string()],
            |command| {
                let argv: Vec<String> = command
                    .get_args()
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect();
                assert_eq!(
                    argv,
                    vec!["mutants", "--list", "--json", "--features", "cli"]
                );
                assert_eq!(command.get_current_dir(), Some(Path::new("/crate")));
                Ok(fake_stdout(0, json))
            },
        )
        .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].file, "src/lib.rs");
        assert_eq!(listed[0].span.start.line, 3);
        assert_eq!(listed[0].span.end.line, 5);
        assert_eq!(listed[0].name, "replace add -> 0");
    }

    #[cfg(unix)]
    #[test]
    fn list_cargo_mutants_reports_a_nonzero_exit_with_the_engine_output() {
        let err = list_cargo_mutants(
            Path::new("/cache/bin/cargo-mutants"),
            Path::new("/crate"),
            &[],
            |_| Ok(fake_output(1, "error: no such option")),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("cargo-mutants --list failed")
                && err.to_string().contains("no such option"),
            "got: {err}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn list_cargo_mutants_propagates_a_spawn_failure() {
        let err = list_cargo_mutants(
            Path::new("/cache/bin/cargo-mutants"),
            Path::new("/crate"),
            &[],
            |_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "no engine",
                ))
            },
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("listing the crate's mutants with cargo-mutants"),
            "got: {err}"
        );
    }

    #[cfg(unix)]
    fn listed_mutant(file: &str, start: u32, end: u32, name: &str) -> MutantInfo {
        MutantInfo {
            file: file.to_string(),
            span: Span {
                start: LineCol { line: start },
                end: LineCol { line: end },
            },
            name: name.to_string(),
        }
    }

    #[cfg(unix)]
    fn diff_with_inserted(file: &str, lines: &[u32]) -> BaseDiff {
        BaseDiff {
            files: vec![file.to_string()],
            inserted: BTreeMap::from([(file.to_string(), lines.iter().copied().collect())]),
        }
    }

    #[cfg(unix)]
    #[test]
    fn zero_mutant_verdict_accepts_a_zero_with_no_mutant_on_the_inserted_lines() {
        let run = fake_output(0, "");
        zero_mutant_verdict(&[], &diff_with_inserted("src/lib.rs", &[5]), &run).unwrap();
        let listed = [listed_mutant("src/lib.rs", 5, 8, "replace add -> 0")];
        zero_mutant_verdict(&listed, &diff_with_inserted("src/lib.rs", &[4, 9]), &run).unwrap();
        zero_mutant_verdict(&listed, &diff_with_inserted("src/other.rs", &[6]), &run).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn zero_mutant_verdict_is_fatal_on_a_mutant_at_either_span_boundary() {
        let listed = [listed_mutant("src/lib.rs", 5, 8, "replace add -> 0")];
        let run = fake_stdout(0, "0 mutants tested");
        for line in [5, 8] {
            let err =
                zero_mutant_verdict(&listed, &diff_with_inserted("src/lib.rs", &[line]), &run)
                    .unwrap_err();
            let message = err.to_string();
            assert!(
                message.contains("1 of the crate's 1 mutant site(s)")
                    && message.contains("src/lib.rs:5: replace add -> 0")
                    && message.contains("0 mutants tested"),
                "got: {message}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn classify_mutants_exit_accepts_the_caught_and_survivor_exits() {
        classify_mutants_exit(Path::new("/crate"), &fake_output(0, "")).unwrap();
        classify_mutants_exit(Path::new("/crate"), &fake_output(2, "")).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn classify_mutants_exit_accepts_a_timeout_exit_3() {
        classify_mutants_exit(Path::new("/crate"), &fake_output(3, ""))
            .expect("a timeout (exit 3) is inconclusive, not fatal");
    }

    #[cfg(unix)]
    #[test]
    fn classify_mutants_exit_is_fatal_on_a_baseline_failure() {
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
