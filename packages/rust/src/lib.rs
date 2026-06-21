pub mod co_change;
pub mod colocated_test;
pub mod config;
pub mod coverage;
pub mod e2e;
pub mod isolation;
pub mod lint;
pub mod mutation;
pub mod packaging;
pub mod patch_coverage;
pub mod ts;
pub mod violation;
pub mod workflow;

use std::path::{Path, PathBuf};

use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "testing-conventions",
    version,
    about = "Enforce testing conventions in libraries (Python, TypeScript, and Rust).",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Check the repository against its testing-conventions config.
    Check,
    /// Unit-test conventions.
    Unit {
        #[command(subcommand)]
        rule: UnitRule,
    },
    /// Integration-test conventions.
    Integration {
        #[command(subcommand)]
        rule: IntegrationRule,
    },
    /// Packaging conventions: test files must not ship in the built artifact.
    Packaging {
        /// Root of the built artifact to inspect (e.g. an unpacked wheel or `dist/`).
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: colocated_test::Language,
    },
    /// Workflow guard (private — hidden from `--help`, #191): every `testing-conventions`
    /// invocation in a CI workflow must name a subcommand this binary still exposes
    /// (guards the `@v0` path, #92). Run from our own CI, not a documented consumer command;
    /// it stays in the binary because the guard needs the in-process command tree.
    #[command(hide = true)]
    Workflow {
        /// Workflow file (or a directory of them) to scan.
        path: PathBuf,
    },
    /// End-to-end-test conventions.
    E2e {
        #[command(subcommand)]
        command: E2eCommand,
    },
}

/// Rules enforced on the unit-test suite (the README's "Unit" taxonomy).
#[derive(Subcommand, Debug)]
enum UnitRule {
    /// Check that every source file has a colocated, matching-named unit test
    /// (tree-wide presence). With `--base`, additionally run the commit-scoped
    /// `co-change` check over `<base>...HEAD` (#33): a modified or deleted source
    /// whose colocated test is not in the diff fails. Presence always runs;
    /// `--base` *adds* the diff-scoped check.
    ColocatedTest {
        /// Directory to scan recursively.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: colocated_test::Language,
        /// Opt-in commit-scoped co-change check (#33): diff `<base>...HEAD` and
        /// also flag a modified or deleted source whose colocated test didn't
        /// co-change. Absent means presence-only — there is no default. Python /
        /// TypeScript only: `--base --language rust` is rejected (inline
        /// `#[cfg(test)]` units have no sibling test to go stale).
        #[arg(long)]
        base: Option<String>,
        /// testing-conventions config file providing the `exempt` list. Optional:
        /// if the file is absent, no files are exempt.
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
    },
    /// Check that the unit suite meets the configured coverage floor. With
    /// `--base`, the same configured floor is measured over the `<base>...HEAD`
    /// diff (the changed lines) instead of the whole tree (#162) — a changed line
    /// below the floor fails, no matter how small the diff.
    Coverage {
        /// Directory whose unit suite is run and measured.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: colocated_test::Language,
        /// Opt-in diff-scoped coverage (#162): diff `<base>...HEAD` and measure the
        /// configured floor over only the changed lines, instead of the whole tree.
        /// Absent means whole-tree — there is no default. This is the patch-scoped
        /// check the old `unit patch-coverage` command did, re-homed onto the floor
        /// it shares.
        #[arg(long)]
        base: Option<String>,
        /// testing-conventions config file with the coverage thresholds and
        /// `exempt` list. Optional: if the file — or its `[<language>].coverage`
        /// table — is absent, the language's sane default floor is used and
        /// nothing is exempt.
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
    },
    /// Lint unit test files for isolation: mock every collaborator (Python, TypeScript, Rust).
    Lint {
        /// Crate root / source dir to scan recursively.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: isolation::Language,
        /// testing-conventions config file providing the `exempt` list (waivers).
        /// Optional: if the file is absent, nothing is waived.
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
    },
    /// Run mutation testing over the unit suite and fail on any surviving mutant not
    /// lifted by a `mutation` exemption — the rung above coverage (#201). The gate is
    /// on by default (no report-only mode). Rust only for now, and not yet wired into
    /// the reusable workflow (parity pending, #199).
    Mutation {
        /// Crate whose unit suite is mutated.
        path: PathBuf,
        /// Language convention to enforce (required). `rust` only for now.
        #[arg(long, value_enum)]
        language: colocated_test::Language,
        /// Opt-in diff-scoping (#201): restrict to mutants on lines a `<base>...HEAD`
        /// diff added or modified, via cargo-mutants' `--in-diff`. Absent means the
        /// whole crate (slower).
        #[arg(long)]
        base: Option<String>,
        /// testing-conventions config file providing the `exempt` list. Optional:
        /// absent means nothing is exempt (every survivor must be killed).
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
    },
}

/// Languages the integration-test lints support — its own set (Python,
/// TypeScript, Rust), distinct from the file-pairing `colocated_test::Language`,
/// so adding Rust here doesn't touch the colocated-test/coverage rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum IntegrationLintLanguage {
    /// Python test files (`*_test.py`, `test_*.py`, `conftest.py`).
    #[value(name = "python")]
    Python,
    /// TypeScript test files (`*.test.{ts,tsx,mts,cts}`).
    #[value(name = "typescript")]
    TypeScript,
    /// Rust integration crates under `tests/`.
    #[value(name = "rust")]
    Rust,
}

/// Lints enforced on integration tests (mocking mechanism & style, and more to
/// come). The README's "Integration" taxonomy.
#[derive(Subcommand, Debug)]
enum IntegrationRule {
    /// Lint integration test files for mocking mechanism & style (Python, TypeScript, Rust).
    Lint {
        /// Directory to scan recursively for test files.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: IntegrationLintLanguage,
        /// testing-conventions config file providing the `exempt` list (waivers).
        /// Optional: if the file is absent, nothing is waived.
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
    },
}

/// E2E attestation commands (#17): record a local e2e run and (later, #68)
/// verify in CI that the latest code commit is attested.
#[derive(Subcommand, Debug)]
enum E2eCommand {
    /// Run the e2e suite and write a committed attestation naming the current commit.
    Attest {
        /// The e2e command to run (e.g. `pnpm run e2e`), executed via the shell.
        command: String,
    },
    /// Verify the committed attestation names the latest code commit (the CI gate).
    Verify,
}

pub fn run<I, T>(args: I) -> anyhow::Result<i32>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(args)?;
    match cli.command {
        // The config-driven `check` umbrella isn't wired yet; the scaffold
        // proves the wiring while individual rules land under their test-kind
        // group (e.g. `unit colocated-test`).
        Some(Command::Check) | None => Ok(0),
        Some(Command::Unit { rule }) => match rule {
            UnitRule::ColocatedTest {
                path,
                language,
                base,
                config,
            } => run_unit_colocated_test(&path, language, base.as_deref(), &config),
            UnitRule::Coverage {
                path,
                language,
                base,
                config,
            } => run_unit_coverage(&path, language, base.as_deref(), &config),
            UnitRule::Lint {
                path,
                language,
                config,
            } => run_unit_lint(&path, language, &config),
            UnitRule::Mutation {
                path,
                language,
                base,
                config,
            } => run_unit_mutation(&path, language, base.as_deref(), &config),
        },
        Some(Command::Integration { rule }) => match rule {
            IntegrationRule::Lint {
                path,
                language,
                config,
            } => run_integration_lint(&path, language, &config),
        },
        Some(Command::Packaging { path, language }) => run_packaging(&path, language),
        Some(Command::Workflow { path }) => run_workflow(&path),
        Some(Command::E2e { command }) => match command {
            E2eCommand::Attest { command } => run_e2e_attest(&command),
            E2eCommand::Verify => run_e2e_verify(),
        },
    }
}

/// The binary's own clap command tree — the source of truth for which subcommands
/// it exposes. The `workflow` guard (#92) checks a workflow's invocations against
/// it, so a renamed or removed subcommand is caught the moment they diverge.
pub fn command() -> clap::Command {
    Cli::command()
}

/// Run the unit colocated-test check over `root` for `language`. Always runs the
/// tree-wide **presence** check (every source file has its colocated test; Rust:
/// an inline `#[cfg(test)]` module). When `base` is `Some`, *additionally* runs the
/// commit-scoped **co-change** check (#33) over `<base>...HEAD` — a modified or
/// deleted source whose colocated test didn't co-change — and the run fails if
/// either check does. Returns `0` only when both pass.
///
/// Presence loads the `colocated-test`-rule exemptions and co-change the
/// `co-change`-rule exemptions from the config at `config_path` (no config file →
/// no exemptions). `--base` rejects `--language rust`: Rust units are inline
/// `#[cfg(test)]` in the same file, so a sibling test can't go stale (presence,
/// without `--base`, still supports Rust).
fn run_unit_colocated_test(
    root: &Path,
    language: colocated_test::Language,
    base: Option<&str>,
    config_path: &Path,
) -> anyhow::Result<i32> {
    // `--base` carries the co-change check, which rejects Rust the same way the
    // standalone `unit co-change` did — before any work, so the message matches.
    if base.is_some() && language == colocated_test::Language::Rust {
        anyhow::bail!(
            "`unit colocated-test --base` supports `--language python` / `typescript`; Rust \
             units are inline `#[cfg(test)]` in the same file, so a sibling test can't go stale"
        );
    }
    let presence_clean = report_colocated_presence(root, language, config_path)?;
    let co_change_clean = match base {
        Some(base) => report_co_change(root, base, language, config_path)?,
        None => true,
    };
    Ok(if presence_clean && co_change_clean {
        0
    } else {
        1
    })
}

/// The tree-wide colocated-test **presence** check: every source file under `root`
/// has its colocated unit test (Rust: an inline `#[cfg(test)]` module). Prints each
/// orphan to stderr and returns `Ok(false)` when any are found, `Ok(true)` when the
/// tree is clean. The `colocated-test`-rule exemptions from the config at
/// `config_path` lift a file (no config file → nothing exempt).
fn report_colocated_presence(
    root: &Path,
    language: colocated_test::Language,
    config_path: &Path,
) -> anyhow::Result<bool> {
    let exempt = colocated_test_exemptions(root, language, config_path)?;
    let orphans = match language {
        // Rust units are inline `#[cfg(test)]` modules, so "colocated" means a test
        // module in the same file, not a sibling file (#40).
        colocated_test::Language::Rust => colocated_test::missing_inline_tests(root, &exempt)?,
        _ => colocated_test::missing_unit_tests(root, language, &exempt)?,
    };
    if orphans.is_empty() {
        return Ok(true);
    }
    let (label, summary) = match language {
        colocated_test::Language::Rust => (
            "missing inline `#[cfg(test)]` tests",
            "source file(s) with testable code but no inline `#[cfg(test)]` module \
             (add an inline test module, or an `exempt` entry with a reason)",
        ),
        _ => (
            "missing colocated unit test",
            "source file(s) missing a colocated unit test \
             (add a colocated test, or an `exempt` entry with a reason)",
        ),
    };
    for orphan in &orphans {
        eprintln!("{label}: {}", orphan.display());
    }
    eprintln!("error: {} {summary}", orphans.len());
    Ok(false)
}

/// The `colocated-test`-rule exempt paths for `language`, resolved (and validated)
/// from the config at `config_path`. A missing config file means no exemptions —
/// the check still runs, just with nothing exempted.
fn colocated_test_exemptions(
    root: &Path,
    language: colocated_test::Language,
    config_path: &Path,
) -> anyhow::Result<std::collections::BTreeSet<String>> {
    if !config_path.exists() {
        return Ok(std::collections::BTreeSet::new());
    }
    let config = config::load_config(config_path)?;
    config::resolve_exempt(
        root,
        config.exemptions(language),
        config::Rule::ColocatedTest,
    )
}

/// The commit-scoped **co-change** check (#33) over `root`, diffing `<base>...HEAD`:
/// every modified or deleted source whose colocated test didn't co-change. Prints
/// each stale source to stderr and returns `Ok(false)` when any are found,
/// `Ok(true)` when clean.
///
/// Loads the `co-change`-rule exemptions from the config at `config_path` (no
/// config file → no exemptions); an exempt source needn't co-change. The caller
/// rejects `--language rust` before this runs: Rust units are inline `#[cfg(test)]`
/// in the same file, so a sibling test can't go stale.
fn report_co_change(
    root: &Path,
    base: &str,
    language: colocated_test::Language,
    config_path: &Path,
) -> anyhow::Result<bool> {
    let exempt = co_change_exemptions(root, language, config_path)?;
    let stale = co_change::stale_sources(root, base, language, &exempt)?;
    if stale.is_empty() {
        return Ok(true);
    }
    for source in &stale {
        eprintln!(
            "source changed without its colocated test: {}",
            source.display()
        );
    }
    eprintln!(
        "error: {} source file(s) changed without their colocated test co-changing \
         (update the test, or add an `exempt` entry with a reason)",
        stale.len()
    );
    Ok(false)
}

/// The `co-change`-rule exempt paths for `language`, resolved (and validated)
/// from the config at `config_path`. A missing config file means no exemptions —
/// every changed source must co-change its test.
fn co_change_exemptions(
    root: &Path,
    language: colocated_test::Language,
    config_path: &Path,
) -> anyhow::Result<std::collections::BTreeSet<String>> {
    if !config_path.exists() {
        return Ok(std::collections::BTreeSet::new());
    }
    let config = config::load_config(config_path)?;
    config::resolve_exempt(root, config.exemptions(language), config::Rule::CoChange)
}

/// Run the unit-test coverage check over `root` for `language`, enforcing the
/// floor from the config at `config_path`. Returns `0` when the floor is met,
/// `1` otherwise.
///
/// With `base` set, the same configured floor is measured over the
/// `<base>...HEAD` diff (the changed lines) rather than the whole tree (#162),
/// via the diff-scoped [`patch_coverage::measure`] / `measure_typescript` /
/// `measure_rust`; without it, the whole-tree [`coverage::measure`] family runs.
///
/// Coverage is zero-config by default for Python and TypeScript (#80): a missing
/// config file — or a config with no `[<language>].coverage` table — falls back to
/// the language's sane default floor ([`config::PythonCoverage::default`] /
/// [`config::TypeScriptCoverage::default`]), the same way `unit colocated-test` and
/// `integration lint` treat an absent config as "nothing exempt". A present
/// `coverage` table overrides the default; `coverage`-rule exemptions still apply.
/// Rust (#206) is zero-config too: a missing `[rust].coverage` table falls back to
/// [`config::RustCoverage::default`] (`lines = 100`, `regions` opt-in, no branch).
fn run_unit_coverage(
    root: &Path,
    language: colocated_test::Language,
    base: Option<&str>,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let config = if config_path.exists() {
        config::load_config(config_path)?
    } else {
        config::Config::default()
    };
    let outcome = match language {
        colocated_test::Language::Python => {
            let python = config.python.unwrap_or_default();
            let coverage = python.coverage.unwrap_or_default();
            let thresholds = coverage::Thresholds {
                fail_under: coverage.fail_under,
                branch: coverage.branch,
            };
            let omit: Vec<String> =
                config::resolve_exempt(root, &python.exempt, config::Rule::Coverage)?
                    .into_iter()
                    .collect();
            match base {
                Some(base) => patch_coverage::measure(root, base, thresholds, &omit)?,
                None => coverage::measure(root, thresholds, &omit)?,
            }
        }
        colocated_test::Language::TypeScript => {
            let typescript = config.typescript.unwrap_or_default();
            let coverage = typescript.coverage.unwrap_or_default();
            let thresholds = coverage::TypeScriptThresholds {
                lines: coverage.lines,
                branches: coverage.branches,
                functions: coverage.functions,
                statements: coverage.statements,
            };
            let exclude: Vec<String> =
                config::resolve_exempt(root, &typescript.exempt, config::Rule::Coverage)?
                    .into_iter()
                    .collect();
            match base {
                Some(base) => patch_coverage::measure_typescript(root, base, thresholds, &exclude)?,
                None => coverage::measure_typescript(root, thresholds, &exclude)?,
            }
        }
        colocated_test::Language::Rust => {
            let rust = config.rust.unwrap_or_default();
            // Zero-config (#206): a missing `[rust].coverage` table falls back to the
            // default Rust floor (`lines = 100`, with `regions` opt-in and no branch
            // component) — matching Python/TypeScript — rather than erroring for a
            // required table. A present table overrides it.
            let coverage = rust.coverage.unwrap_or_default();
            let thresholds = coverage::RustThresholds {
                regions: coverage.regions,
                lines: coverage.lines,
            };
            let ignore: Vec<String> =
                config::resolve_exempt(root, &rust.exempt, config::Rule::Coverage)?
                    .into_iter()
                    .collect();
            match base {
                Some(base) => patch_coverage::measure_rust(root, base, thresholds, &ignore)?,
                None => coverage::measure_rust(root, thresholds, &ignore)?,
            }
        }
    };
    match outcome {
        coverage::Outcome::Pass => Ok(0),
        coverage::Outcome::Fail(reason) => {
            eprintln!("error: coverage check failed — {reason}");
            Ok(1)
        }
    }
}

/// Run `unit mutation` over `root` (#201, #202): run the per-language engine and fail
/// on any surviving mutant not lifted by a `mutation` exemption.
///
/// The gate is **on by default and binary** — "no *unexplained* surviving mutant":
/// every survivor must be killed with an assertion, or lifted by a reason-required
/// `[[<language>.exempt]] rules = ["mutation"]` for an equivalent / deliberately-defensive
/// mutation. There is no percentage floor (equivalent mutants make one unreachable)
/// and no report-only mode — the only loosening is a reasoned, per-file exemption.
/// Rust (cargo-mutants) and TypeScript (Stryker) are wired; Python lands with #203.
/// `--base` scopes the run to the diff.
fn run_unit_mutation(
    root: &Path,
    language: colocated_test::Language,
    base: Option<&str>,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let config = if config_path.exists() {
        config::load_config(config_path)?
    } else {
        config::Config::default()
    };
    let survivors = match language {
        colocated_test::Language::Rust => {
            let rust = config.rust.unwrap_or_default();
            let exempt: Vec<String> =
                config::resolve_exempt(root, &rust.exempt, config::Rule::Mutation)?
                    .into_iter()
                    .collect();
            mutation::measure_rust(root, &exempt, base)?
        }
        colocated_test::Language::TypeScript => {
            let typescript = config.typescript.unwrap_or_default();
            let exempt: Vec<String> =
                config::resolve_exempt(root, &typescript.exempt, config::Rule::Mutation)?
                    .into_iter()
                    .collect();
            mutation::measure_typescript(root, &exempt, base)?
        }
        colocated_test::Language::Python => anyhow::bail!(
            "`unit mutation` doesn't support Python yet — it lands with the mutation epic (#203)"
        ),
    };
    if survivors.is_empty() {
        println!("unit mutation: no surviving mutants — every mutation was caught");
        return Ok(0);
    }

    eprintln!(
        "error: {} unexplained surviving mutant(s) — kill each with an assertion, or lift an \
         equivalent/defensive one with a reason-required `[[<language>.exempt]] rules = [\"mutation\"]`:",
        survivors.len()
    );
    for survivor in &survivors {
        eprintln!(
            "  {}:{}: {}",
            survivor.file, survivor.line, survivor.description
        );
    }
    Ok(1)
}

/// Run the `unit lint` check over `root` for `language` — the unit-suite
/// isolation lints (`unmocked-collaborator`, `untyped-mock`, `no-out-of-module-call`,
/// `no-out-of-module-import`) — printing each violation to stderr as
/// `path:line: rule — message` and returning `1` when any are found, `0` otherwise.
fn run_unit_lint(
    root: &Path,
    language: isolation::Language,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let (raw, select): (Vec<lint::Violation>, ExemptSelect) = match language {
        isolation::Language::Rust => (isolation::find_violations(root)?, |c| c.rust_exemptions()),
        isolation::Language::TypeScript => (ts::find_unit_violations(root)?, |c| {
            c.exemptions(colocated_test::Language::TypeScript)
        }),
        isolation::Language::Python => (lint::find_unit_isolation_violations(root)?, |c| {
            c.exemptions(colocated_test::Language::Python)
        }),
    };
    let violations = apply_waivers(raw, root, config_path, select)?;
    if violations.is_empty() {
        return Ok(0);
    }
    for v in &violations {
        eprintln!(
            "{}:{}: {} — {}",
            v.file.display(),
            v.line,
            v.rule,
            v.message
        );
    }
    eprintln!("error: {} isolation violation(s)", violations.len());
    Ok(1)
}

/// Run the integration-test lints over `root` for `language`, printing each
/// violation to stderr as `path:line: rule — message` and returning `1` when any
/// are found, `0` otherwise.
fn run_integration_lint(
    root: &Path,
    language: IntegrationLintLanguage,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let (raw, select): (Vec<lint::Violation>, ExemptSelect) = match language {
        IntegrationLintLanguage::Python => (lint::find_violations(root)?, |c| {
            c.exemptions(colocated_test::Language::Python)
        }),
        IntegrationLintLanguage::TypeScript => (ts::find_integration_violations(root)?, |c| {
            c.exemptions(colocated_test::Language::TypeScript)
        }),
        IntegrationLintLanguage::Rust => (isolation::find_integration_violations(root)?, |c| {
            c.rust_exemptions()
        }),
    };
    let violations = apply_waivers(raw, root, config_path, select)?;
    if violations.is_empty() {
        return Ok(0);
    }
    for v in &violations {
        eprintln!(
            "{}:{}: {} — {}",
            v.file.display(),
            v.line,
            v.rule,
            v.message
        );
    }
    eprintln!("error: {} lint violation(s)", violations.len());
    Ok(1)
}

/// Selects a language's `[[<lang>.exempt]]` table from a loaded config — the one
/// varying piece between the `unit lint` and `integration lint` waiver paths.
type ExemptSelect = fn(&config::Config) -> &[config::Exemption];

/// Drop the violations waived by the config's `exempt` list (#32/#102). A
/// violation is waived when its `rule` is a known [`config::Rule`] and its
/// `root`-relative path is exempt for that rule. `exemptions` selects the
/// language's `[[<lang>.exempt]]` table from the loaded config. A missing config
/// file waives nothing; a reason-less or stale entry errors (via `load_config` /
/// `resolve_exempt`), so the escape hatch can't silently rot.
fn apply_waivers(
    violations: Vec<lint::Violation>,
    root: &Path,
    config_path: &Path,
    exemptions: ExemptSelect,
) -> anyhow::Result<Vec<lint::Violation>> {
    use std::collections::hash_map::Entry;

    if !config_path.exists() {
        return Ok(violations);
    }
    let config = config::load_config(config_path)?;
    let exempt = exemptions(&config);
    // Resolve each rule's exempt set once (and surface a stale entry as an error).
    let mut resolved: std::collections::HashMap<config::Rule, std::collections::BTreeSet<String>> =
        std::collections::HashMap::new();
    let mut kept = Vec::new();
    for violation in violations {
        let waived = match config::Rule::from_id(violation.rule) {
            Some(rule) => {
                let exempt_paths = match resolved.entry(rule) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => {
                        entry.insert(config::resolve_exempt(root, exempt, rule)?)
                    }
                };
                violation
                    .file
                    .strip_prefix(root)
                    .ok()
                    .map(|rel| rel.to_string_lossy().replace('\\', "/"))
                    .is_some_and(|rel| exempt_paths.contains(&rel))
            }
            None => false,
        };
        if !waived {
            kept.push(violation);
        }
    }
    Ok(kept)
}

/// Run the packaging check: inspect the built artifact at `artifact` for test
/// files that must not ship (README "Packaging"), per `language`'s test-file
/// globs.
///
/// `artifact` is either an already-unpacked directory or a packed artifact the
/// rule unpacks itself — a Python wheel (`.whl`) today; the TypeScript (#73) and
/// Rust (#74) archives follow. Returns `0` when no test file is present, `1`
/// otherwise (after printing each offending path, relative to the artifact root).
fn run_packaging(artifact: &Path, language: colocated_test::Language) -> anyhow::Result<i32> {
    let globs = match language {
        colocated_test::Language::Python => vec!["*_test.py".to_string()],
        colocated_test::Language::TypeScript => vec!["*.test.*".to_string()],
        // `#[cfg(test)]` units compile out for free; the only thing to keep out of
        // the `.crate` source tarball is the crate-root integration `tests/` dir.
        colocated_test::Language::Rust => vec!["tests/".to_string()],
    };
    let offenders = packaging::inspect(artifact, &globs)?;
    if offenders.is_empty() {
        return Ok(0);
    }
    for offender in &offenders {
        eprintln!("test file in built artifact: {}", offender.display());
    }
    eprintln!(
        "error: {} test file(s) present in the built artifact \
         (they must be excluded from packaging)",
        offenders.len()
    );
    Ok(1)
}

/// Run the workflow guard over `path` (a workflow file or directory): flag every
/// `testing-conventions` invocation that names a subcommand this binary no longer
/// exposes, printing each as `path:line: rule — message` and returning `1` when any
/// are found, `0` otherwise.
fn run_workflow(path: &Path) -> anyhow::Result<i32> {
    let violations = workflow::check(path, &command())?;
    if violations.is_empty() {
        return Ok(0);
    }
    for v in &violations {
        eprintln!(
            "{}:{}: {} — {}",
            v.file.display(),
            v.line,
            v.rule,
            v.message
        );
    }
    eprintln!(
        "error: {} workflow invocation(s) name a subcommand this binary no longer exposes",
        violations.len()
    );
    Ok(1)
}

/// Run `command` as an e2e suite and write a committed attestation naming the
/// current commit (#67). Force-runs: the attestation is written regardless of
/// the command's exit code, so this exits `0` once the attestation is recorded.
fn run_e2e_attest(command: &str) -> anyhow::Result<i32> {
    let repo = std::env::current_dir()?;
    let attestation = e2e::attest(&repo, command)?;
    println!(
        "e2e attestation recorded for commit {} (command exited {})",
        attestation.commit, attestation.exit_code
    );
    Ok(0)
}

/// Verify the committed e2e attestation names the latest code commit (#68) — the
/// CI side of the nudge. Exits `0` when fresh; otherwise prints the actionable
/// hint and exits `1`. Never runs e2e, never judges the recorded run.
fn run_e2e_verify() -> anyhow::Result<i32> {
    let repo = std::env::current_dir()?;
    match e2e::verify(&repo)? {
        e2e::Verification::Fresh => Ok(0),
        e2e::Verification::Missing => {
            eprintln!(
                "e2e attestation missing — run `testing-conventions e2e attest '<your e2e command>'`"
            );
            Ok(1)
        }
        e2e::Verification::Stale { attested, latest } => {
            eprintln!(
                "e2e attestation out of date: attested {}, latest code commit {} — \
                 run `testing-conventions e2e attest '<your e2e command>'`",
                &attested[..attested.len().min(7)],
                &latest[..latest.len().min(7)]
            );
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_returns_ok_zero() {
        assert_eq!(run(["testing-conventions"]).unwrap(), 0);
    }

    #[test]
    fn check_returns_ok_zero() {
        assert_eq!(run(["testing-conventions", "check"]).unwrap(), 0);
    }

    #[test]
    fn unknown_flag_errors() {
        assert!(run(["testing-conventions", "--bogus"]).is_err());
    }

    #[test]
    fn help_flag_returns_clap_display_help() {
        let err = run(["testing-conventions", "--help"]).expect_err("--help should bubble");
        let clap_err = err
            .downcast_ref::<clap::Error>()
            .expect("error should be a clap::Error");
        assert_eq!(clap_err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn version_flag_returns_clap_display_version() {
        let err = run(["testing-conventions", "--version"]).expect_err("--version should bubble");
        let clap_err = err
            .downcast_ref::<clap::Error>()
            .expect("error should be a clap::Error");
        assert_eq!(clap_err.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn unit_mutation_rejects_a_non_rust_language() {
        // Least parity (#199): `unit mutation` is Rust-only for now, so a
        // python/typescript request errors before running any engine — no silent
        // pass, and no fixture/toolchain needed.
        let err = run([
            "testing-conventions",
            "unit",
            "mutation",
            "pkg",
            "--language",
            "python",
        ])
        .unwrap_err();
        assert!(
            err.to_string().contains("supports `--language rust` only"),
            "got: {err}"
        );
    }
}
