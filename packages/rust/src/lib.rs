pub mod agents;
pub mod co_change;
pub mod colocated_test;
pub mod config;
pub mod coverage;
pub mod e2e;
pub mod isolation;
pub mod lint;
pub mod mutation;
pub mod one_function;
pub mod packaging;
pub mod patch_coverage;
pub mod tiers;
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
    /// Write the testing contract into the repository's agent context file:
    /// a marker-delimited, hash-versioned block in `AGENTS.md` that a
    /// coding agent reads before writing code. Idempotent — re-running
    /// refreshes the owned region and touches nothing outside it.
    Install {
        /// The agent context file to manage.
        #[arg(default_value = "AGENTS.md")]
        path: PathBuf,
    },
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
    /// Workflow guard (private — hidden from `--help`): every `testing-conventions`
    /// invocation in a CI workflow must name a subcommand this binary still exposes
    /// (guards the `@v0` path). Run from our own CI, not a documented consumer command;
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

#[derive(Subcommand, Debug)]
enum UnitRule {
    /// Check that every source file has a colocated, matching-named unit test
    /// (tree-wide presence). With `--base`, additionally run the commit-scoped
    /// `co-change` check over `<base>...HEAD`: a modified or deleted source
    /// whose colocated test is not in the diff fails. Presence always runs;
    /// `--base` *adds* the diff-scoped check.
    ColocatedTest {
        /// Directory to scan recursively.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: colocated_test::Language,
        /// Opt-in commit-scoped co-change check: diff `<base>...HEAD` and
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
    /// diff (the changed lines) instead of the whole tree — a changed line
    /// below the floor fails, no matter how small the diff.
    Coverage {
        /// Directory whose unit suite is run and measured.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: colocated_test::Language,
        /// Opt-in diff-scoped coverage: diff `<base>...HEAD` and measure the
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
    /// Check that no source file holds more than one module-scope function whose body
    /// runs longer than the configured threshold. Trivial functions — at or under the
    /// threshold — share a file freely.
    OneFunctionPerFile {
        /// Directory to scan recursively.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: colocated_test::Language,
        /// testing-conventions config file providing the `max_lines` threshold and the
        /// `exempt` list. Optional: if the file — or its
        /// `[<language>].one_function_per_file` table — is absent, the default threshold
        /// of one line applies and nothing is exempt.
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
    /// lifted by a `mutation` exemption — the rung above coverage. The gate is
    /// on by default (no report-only mode). All three languages (Python, TypeScript,
    /// Rust) are at parity and wired into the reusable workflow as a diff-scoped,
    /// PR-only job.
    Mutation {
        /// Crate whose unit suite is mutated.
        path: PathBuf,
        /// Language convention to enforce (required): `python`, `typescript`, or `rust`.
        #[arg(long, value_enum)]
        language: colocated_test::Language,
        /// Opt-in diff-scoping: restrict to mutants on lines a `<base>...HEAD`
        /// diff added or modified, via cargo-mutants' `--in-diff`. Absent means the
        /// whole crate (slower).
        #[arg(long)]
        base: Option<String>,
        /// testing-conventions config file providing the `exempt` list. Optional:
        /// absent means nothing is exempt (every survivor must be killed).
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
        /// Path to the bundled TypeScript mutation adapter (`dist/mutation/main.js`), used
        /// only by `--language typescript`. The npm launcher appends it; hidden because a
        /// consumer never sets it by hand.
        #[arg(long = "ts-mutation-adapter", hide = true)]
        ts_adapter: Option<PathBuf>,
    },
}

/// Languages the integration-test lints support.
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

#[derive(Subcommand, Debug)]
enum E2eCommand {
    /// Run the e2e command of your choosing and, when it passes, commit the
    /// branch's receipt — the command (full suite, targeted subset, or a no-op)
    /// is the judgment the receipt records. Exits with the command's own code.
    Attest {
        /// The e2e command to run (e.g. `pnpm run e2e`), executed via the shell.
        command: String,
    },
    /// Verify a receipt answers this branch's e2e nudge (the CI gate).
    Verify {
        /// Directory whose committed receipts (`e2e-attestations/`) are read
        /// (default: current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Directory defining what counts as scoped source, if narrower than
        /// `path` (default: `path` itself). Must be `path` or a descendant of it.
        #[arg(long)]
        scope: Option<PathBuf>,
        /// Base ref for the branch's content diff (`<base>...HEAD`): a branch
        /// whose diff leaves the scoped source untouched owes no decision, and
        /// one that changed it passes when its diff adds or updates a receipt —
        /// the way the changed-line coverage/mutation gates read the diff, and
        /// indifferent to rebases and squash merges. Absent, presence of a
        /// committed receipt is the whole check.
        #[arg(long)]
        base: Option<String>,
        /// Extra scopes: repo-root-relative directories outside `path` that
        /// join the scoped diff — a shared source tree beside the package (a
        /// native core bound into several bindings) that no `--scope`
        /// at-or-below `path` can reach. Repeatable.
        #[arg(long = "extra-scope")]
        extra_scope: Vec<PathBuf>,
        /// Feature-gated subtrees carved back out of the `--extra-scope` union:
        /// repo-root-relative directories (a core `cli/` compiled out of the
        /// bindings) whose changes owe no decision. Repeatable.
        #[arg(long = "exclude")]
        exclude: Vec<PathBuf>,
    },
    /// Print the standardized receipt slug for a branch name — the receipt
    /// lives at `e2e-attestations/<slug>.json`.
    Slug {
        /// Branch name to standardize (default: the checked-out branch).
        branch: Option<String>,
    },
}

pub fn run<I, T>(args: I) -> anyhow::Result<i32>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    // Printed before parsing so a run that dies on an unrecognized flag still names its
    // version, and on stderr because `e2e slug`'s stdout is read by command substitution.
    eprintln!("testing-conventions {}", env!("CARGO_PKG_VERSION"));
    let cli = Cli::try_parse_from(args)?;
    match cli.command {
        None => Ok(0),
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
            UnitRule::OneFunctionPerFile {
                path,
                language,
                config,
            } => run_unit_one_function(&path, language, &config),
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
                ts_adapter,
            } => run_unit_mutation(
                &path,
                language,
                base.as_deref(),
                &config,
                ts_adapter.as_deref(),
            ),
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
            E2eCommand::Verify {
                path,
                scope,
                base,
                extra_scope,
                exclude,
            } => run_e2e_verify(
                &path,
                scope.as_deref(),
                base.as_deref(),
                &extra_scope,
                &exclude,
            ),
            E2eCommand::Slug { branch } => run_e2e_slug(branch.as_deref()),
        },
        Some(Command::Install { path }) => {
            agents::install(&path)?;
            Ok(0)
        }
    }
}

/// The binary's own clap command tree, which the `workflow` guard checks invocations against.
pub fn command() -> clap::Command {
    Cli::command()
}

/// Run the colocated-test presence check over `root`, plus the diff-scoped co-change
/// check when `base` is set. Returns `0` only when both pass.
fn run_unit_colocated_test(
    root: &Path,
    language: colocated_test::Language,
    base: Option<&str>,
    config_path: &Path,
) -> anyhow::Result<i32> {
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

/// Print every source file under `root` missing its colocated unit test; `Ok(false)`
/// when any were found.
fn report_colocated_presence(
    root: &Path,
    language: colocated_test::Language,
    config_path: &Path,
) -> anyhow::Result<bool> {
    let exempt = colocated_test_exemptions(root, language, config_path)?;
    let orphans = match language {
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

/// The `colocated-test`-rule exempt paths for `language`; empty when the config is absent.
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

/// Print every source under `root` that `<base>...HEAD` changed without its colocated
/// test; `Ok(false)` when any were found.
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

/// The `co-change`-rule exempt paths for `language`; empty when the config is absent.
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

/// Split a resolved exempt-scope map into whole-file paths and line-scoped sets.
fn split_scopes(
    scopes: std::collections::BTreeMap<String, config::LineScope>,
) -> (
    Vec<String>,
    std::collections::BTreeMap<String, std::collections::BTreeSet<u32>>,
) {
    let mut whole_file = Vec::new();
    let mut line_scoped = std::collections::BTreeMap::new();
    for (path, scope) in scopes {
        match scope {
            config::LineScope::WholeFile => whole_file.push(path),
            config::LineScope::Lines(lines) => {
                line_scoped.insert(path, lines);
            }
        }
    }
    (whole_file, line_scoped)
}

/// Run the unit coverage check over `root`, measuring the configured floor over the
/// whole tree or, with `base` set, over the `<base>...HEAD` diff. `0` when the floor is met.
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
            let (omit, exempt_lines) = split_scopes(config::resolve_exempt_scoped(
                root,
                &python.exempt,
                config::Rule::Coverage,
            )?);
            match base {
                Some(base) => {
                    patch_coverage::measure(root, base, thresholds, &omit, &exempt_lines)?
                }
                None if exempt_lines.is_empty() => coverage::measure(root, thresholds, &omit)?,
                None => {
                    patch_coverage::measure_line_exempt(root, thresholds, &omit, &exempt_lines)?
                }
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
            let (exclude, exempt_lines) = split_scopes(config::resolve_exempt_scoped(
                root,
                &typescript.exempt,
                config::Rule::Coverage,
            )?);
            match base {
                Some(base) => patch_coverage::measure_typescript(
                    root,
                    base,
                    thresholds,
                    &exclude,
                    &exempt_lines,
                )?,
                None if exempt_lines.is_empty() => {
                    coverage::measure_typescript(root, thresholds, &exclude)?
                }
                None => patch_coverage::measure_line_exempt_typescript(
                    root,
                    thresholds,
                    &exclude,
                    &exempt_lines,
                )?,
            }
        }
        colocated_test::Language::Rust => {
            let rust = config.rust.unwrap_or_default();
            let coverage = rust.coverage.unwrap_or_default();
            let thresholds = coverage::RustThresholds {
                regions: coverage.regions,
                lines: coverage.lines,
                functions: coverage.functions,
                branch: coverage.branch,
            };
            let (ignore, exempt_lines) = split_scopes(config::resolve_exempt_scoped(
                root,
                &rust.exempt,
                config::Rule::Coverage,
            )?);
            match base {
                Some(base) => patch_coverage::measure_rust(
                    root,
                    base,
                    thresholds,
                    &ignore,
                    &exempt_lines,
                    &rust.features,
                )?,
                None if exempt_lines.is_empty() => {
                    coverage::measure_rust(root, thresholds, &ignore, &rust.features)?
                }
                None => patch_coverage::measure_line_exempt_rust(
                    root,
                    thresholds,
                    &ignore,
                    &exempt_lines,
                    &rust.features,
                )?,
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

/// Run the per-language mutation engine over `root` and fail on any surviving mutant
/// not lifted by a `mutation` exemption. `base` scopes the run to the diff.
fn run_unit_mutation(
    root: &Path,
    language: colocated_test::Language,
    base: Option<&str>,
    config_path: &Path,
    ts_adapter: Option<&Path>,
) -> anyhow::Result<i32> {
    let config = if config_path.exists() {
        config::load_config(config_path)?
    } else {
        config::Config::default()
    };
    let measurement = match language {
        colocated_test::Language::Rust => {
            let rust = config.rust.unwrap_or_default();
            let (exempt, exempt_lines) = split_scopes(config::resolve_exempt_scoped(
                root,
                &rust.exempt,
                config::Rule::Mutation,
            )?);
            mutation::measure_rust(root, &exempt, &exempt_lines, base, &rust.features)?
        }
        colocated_test::Language::TypeScript => {
            let typescript = config.typescript.unwrap_or_default();
            let (exempt, exempt_lines) = split_scopes(config::resolve_exempt_scoped(
                root,
                &typescript.exempt,
                config::Rule::Mutation,
            )?);
            let adapter = ts_adapter.ok_or_else(|| {
                anyhow::anyhow!(
                    "the TypeScript mutation adapter path is required: pass \
                     `--ts-mutation-adapter <path>`. The npm `testing-conventions` CLI appends it \
                     automatically — run the rule through that CLI, not the raw binary."
                )
            })?;
            mutation::measure_typescript(root, &exempt, &exempt_lines, base, adapter)?
        }
        colocated_test::Language::Python => {
            let python = config.python.unwrap_or_default();
            let (exempt, exempt_lines) = split_scopes(config::resolve_exempt_scoped(
                root,
                &python.exempt,
                config::Rule::Mutation,
            )?);
            mutation::measure_python(root, &exempt, &exempt_lines, base)?
        }
    };
    let (count, survivors) = match measurement {
        mutation::Measurement::EngineNotRun => {
            println!("unit mutation: no mutatable changed lines — engine not run");
            return Ok(0);
        }
        mutation::Measurement::Tested { count, survivors } => (count, survivors),
    };
    if survivors.is_empty() {
        if count == 0 {
            println!("unit mutation: the engine found no mutants to test");
        } else {
            println!(
                "unit mutation: no surviving mutants — every mutation was caught \
                 ({count} mutant(s) tested)"
            );
        }
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

/// Run the one-function-per-file rule over `root`, printing each violation and returning
/// `1` when any are found. A language with no configured threshold reports that and exits `0`.
fn run_unit_one_function(
    root: &Path,
    language: colocated_test::Language,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let threshold = if config_path.exists() {
        config::load_config(config_path)?.one_function_threshold(language)
    } else {
        config::Config::default().one_function_threshold(language)
    };
    let Some(max_lines) = threshold else {
        let key = match language {
            colocated_test::Language::Python => "python",
            colocated_test::Language::TypeScript => "typescript",
            colocated_test::Language::Rust => "rust",
        };
        println!(
            "unit one-function-per-file: not enabled for {key} — \
             set `[{key}].one_function_per_file` to opt in"
        );
        return Ok(0);
    };
    let raw = one_function::find_violations(root, language, max_lines)?;
    let select: ExemptSelect = match language {
        colocated_test::Language::Python => |c| c.exemptions(colocated_test::Language::Python),
        colocated_test::Language::TypeScript => {
            |c| c.exemptions(colocated_test::Language::TypeScript)
        }
        colocated_test::Language::Rust => |c| c.rust_exemptions(),
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
    eprintln!(
        "error: {} function(s) sharing a file with another function over the \
         {max_lines}-line threshold (move each to its own module, or add an \
         `exempt` entry with a reason)",
        violations.len()
    );
    Ok(1)
}

/// Run the unit-suite isolation lints over `root`, printing each violation and returning
/// `1` when any are found.
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

/// Run the integration-test lints over the package root above `root`, printing each
/// violation and returning `1` when any are found. A tree with no manifest is scanned at `root`.
fn run_integration_lint(
    root: &Path,
    language: IntegrationLintLanguage,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let manifest = match language {
        IntegrationLintLanguage::Python => "pyproject.toml",
        IntegrationLintLanguage::TypeScript => "package.json",
        IntegrationLintLanguage::Rust => "Cargo.toml",
    };
    let package_root = tiers::package_root(root, manifest);
    let scan_root = package_root.as_deref().unwrap_or(root);
    let (raw, select): (Vec<lint::Violation>, ExemptSelect) = match language {
        IntegrationLintLanguage::Python => (
            match &package_root {
                Some(package_root) => lint::find_suite_violations(package_root)?,
                None => lint::find_violations(root)?,
            },
            |c| c.exemptions(colocated_test::Language::Python),
        ),
        IntegrationLintLanguage::TypeScript => (
            match &package_root {
                Some(package_root) => ts::find_suite_violations(package_root)?,
                None => ts::find_integration_violations(root)?,
            },
            |c| c.exemptions(colocated_test::Language::TypeScript),
        ),
        IntegrationLintLanguage::Rust => {
            (isolation::find_integration_violations(scan_root)?, |c| {
                c.rust_exemptions()
            })
        }
    };
    let violations = apply_waivers(raw, scan_root, config_path, select)?;
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

/// Selects a language's `[[<lang>.exempt]]` table from a loaded config.
type ExemptSelect = fn(&config::Config) -> &[config::Exemption];

/// Drop the violations whose `root`-relative path is exempt for their rule.
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

/// Inspect the built artifact at `artifact` — an unpacked directory or a packed archive —
/// for test files matching `language`'s globs. `1` when any are present.
fn run_packaging(artifact: &Path, language: colocated_test::Language) -> anyhow::Result<i32> {
    let globs = match language {
        colocated_test::Language::Python => vec!["*_test.py".to_string()],
        colocated_test::Language::TypeScript => vec!["*.test.*".to_string()],
        // `#[cfg(test)]` units compile out, so only the crate-root `tests/` dir can ship.
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

/// Flag every `testing-conventions` invocation under `path` naming a subcommand this
/// binary no longer exposes. `1` when any are found.
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

/// Run `command` as the branch's e2e decision and, when it passes, commit the receipt.
/// Returns `command`'s own exit code.
fn run_e2e_attest(command: &str) -> anyhow::Result<i32> {
    let repo = std::env::current_dir()?;
    let attestation = e2e::attest(&repo, command)?;
    if attestation.exit_code != 0 {
        eprintln!(
            "e2e command `{command}` exited {}; a receipt records a run that passed — \
             fix the failure and attest again",
            attestation.exit_code
        );
        return Ok(attestation.exit_code);
    }
    println!(
        "e2e receipt recorded for branch {} at {}/{}.json",
        attestation.branch,
        e2e::RECEIPTS_DIR,
        e2e::branch_slug(&attestation.branch),
    );
    Ok(0)
}

/// Verify a receipt under `path` answers this branch's e2e nudge. `0` when it does;
/// otherwise prints the hint and returns `1`. `scope` defaults to `path`; `base`, when set,
/// makes the check a `<base>...HEAD` content diff.
fn run_e2e_verify(
    path: &Path,
    scope: Option<&Path>,
    base: Option<&str>,
    extra_scopes: &[PathBuf],
    excludes: &[PathBuf],
) -> anyhow::Result<i32> {
    match e2e::verify_extra_scoped(path, scope.unwrap_or(path), base, extra_scopes, excludes)? {
        e2e::Verification::Fresh => Ok(0),
        e2e::Verification::Missing => {
            eprintln!(
                "no e2e receipt answers this change — run \
                 `testing-conventions e2e attest '<your e2e command>'`; the command is \
                 your judgment: the full suite, a targeted subset, or a no-op"
            );
            Ok(1)
        }
    }
}

/// Print the receipt slug for `branch`, defaulting to the checked-out branch.
fn run_e2e_slug(branch: Option<&str>) -> anyhow::Result<i32> {
    let slug = match branch {
        Some(name) => e2e::branch_slug(name),
        None => {
            let repo = std::env::current_dir()?;
            e2e::branch_slug(&e2e::current_branch(&repo)?)
        }
    };
    println!("{slug}");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_returns_ok_zero() {
        assert_eq!(run(["testing-conventions"]).unwrap(), 0);
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
}
