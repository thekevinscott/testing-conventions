pub mod colocated_test;
pub mod config;
pub mod coverage;
pub mod isolation;
pub mod lint;
pub mod packaging;
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
    /// Workflow guard: every `testing-conventions` invocation in a CI workflow must
    /// name a subcommand this binary still exposes (guards the `@v0` path, #92).
    Workflow {
        /// Workflow file (or a directory of them) to scan.
        path: PathBuf,
    },
}

/// Rules enforced on the unit-test suite (the README's "Unit" taxonomy).
#[derive(Subcommand, Debug)]
enum UnitRule {
    /// Check that every source file has a colocated, matching-named unit test.
    ColocatedTest {
        /// Directory to scan recursively.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: colocated_test::Language,
        /// testing-conventions config file providing the `exempt` list. Optional:
        /// if the file is absent, no files are exempt.
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
    },
    /// Check that the unit suite meets the configured coverage floor.
    Coverage {
        /// Directory whose unit suite is run and measured.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: colocated_test::Language,
        /// testing-conventions config file with the coverage thresholds and
        /// `exempt` list. Optional: if the file — or its `[<language>].coverage`
        /// table — is absent, the language's sane default floor is used and
        /// nothing is exempt.
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
    },
    /// Check that inline unit tests call nothing out of their own module (Rust).
    Isolation {
        /// Crate root to scan recursively (its `Cargo.toml` names external crates).
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: isolation::Language,
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
                config,
            } => run_unit_colocated_test(&path, language, &config),
            UnitRule::Coverage {
                path,
                language,
                config,
            } => run_unit_coverage(&path, language, &config),
            UnitRule::Isolation { path, language } => run_unit_isolation(&path, language),
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
    }
}

/// The binary's own clap command tree — the source of truth for which subcommands
/// it exposes. The `workflow` guard (#92) checks a workflow's invocations against
/// it, so a renamed or removed subcommand is caught the moment they diverge.
pub fn command() -> clap::Command {
    Cli::command()
}

/// Run the unit-test colocated-test check over `root` for `language`, reporting orphans.
///
/// Loads the `colocated-test`-rule exemptions from the config at `config_path` (no
/// config file → no exemptions). Returns `0` when every source file has its
/// colocated unit test; otherwise prints each orphan to stderr and returns `1`.
fn run_unit_colocated_test(
    root: &Path,
    language: colocated_test::Language,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let exempt = colocated_test_exemptions(root, language, config_path)?;
    let orphans = colocated_test::missing_unit_tests(root, language, &exempt)?;
    if orphans.is_empty() {
        return Ok(0);
    }
    for orphan in &orphans {
        eprintln!("missing colocated unit test: {}", orphan.display());
    }
    eprintln!(
        "error: {} source file(s) missing a colocated unit test \
         (add a colocated test, or an `exempt` entry with a reason)",
        orphans.len()
    );
    Ok(1)
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

/// Run the unit-test coverage check over `root` for `language`, enforcing the
/// floor from the config at `config_path`. Returns `0` when the floor is met,
/// `1` otherwise.
///
/// Coverage is zero-config by default (#80): a missing config file — or a config
/// with no `[<language>].coverage` table — falls back to the language's sane
/// default floor ([`config::PythonCoverage::default`] /
/// [`config::TypeScriptCoverage::default`]), the same way `unit colocated-test`
/// and `integration lint` treat an absent config as "nothing exempt". A present
/// `coverage` table overrides the default; `coverage`-rule exemptions still apply.
fn run_unit_coverage(
    root: &Path,
    language: colocated_test::Language,
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
            coverage::measure(root, thresholds, &omit)?
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
            coverage::measure_typescript(root, thresholds, &exclude)?
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

/// Run the unit-isolation check over `root` for `language`, printing each
/// violation to stderr as `path:line: rule — message` and returning `1` when any
/// are found, `0` otherwise.
fn run_unit_isolation(root: &Path, language: isolation::Language) -> anyhow::Result<i32> {
    let violations = match language {
        isolation::Language::Rust => isolation::find_violations(root)?,
        isolation::Language::TypeScript => ts::find_unit_violations(root)?,
    };
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
    let (raw, waived) = match language {
        IntegrationLintLanguage::Python => (
            lint::find_violations(root)?,
            lint_waivers(root, colocated_test::Language::Python, config_path)?,
        ),
        IntegrationLintLanguage::TypeScript => (
            ts::find_integration_violations(root)?,
            lint_waivers(root, colocated_test::Language::TypeScript, config_path)?,
        ),
        // The Rust `no-first-party-double` lint is bright-line; the inline
        // `waiver:` hatch is a separate slice, so nothing is waived here yet.
        IntegrationLintLanguage::Rust => (
            isolation::find_integration_violations(root)?,
            std::collections::BTreeSet::new(),
        ),
    };
    let violations: Vec<lint::Violation> = raw
        .into_iter()
        .filter(|v| !is_waived(v, root, &waived))
        .collect();
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

/// The `no-constant-patch` waivers (root-relative paths) from the config at
/// `config_path` — the only waivable lint (#52). A missing config file means
/// nothing is waived.
fn lint_waivers(
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
        config::Rule::NoConstantPatch,
    )
}

/// `true` when `violation` is a `no-constant-patch` finding in a waived file.
fn is_waived(
    violation: &lint::Violation,
    root: &Path,
    waived: &std::collections::BTreeSet<String>,
) -> bool {
    violation.rule == "no-constant-patch"
        && violation
            .file
            .strip_prefix(root)
            .ok()
            .map(|rel| rel.to_string_lossy().replace('\\', "/"))
            .is_some_and(|rel| waived.contains(&rel))
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
}
