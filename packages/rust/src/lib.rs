pub mod colocated_test;
pub mod config;
pub mod coverage;
pub mod lint;

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};

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
    /// Run every rule the config enables, over `<PATH>` (the config-driven umbrella).
    Check {
        /// Directory scanned recursively (sources + tests), passed to each rule.
        path: PathBuf,
        /// testing-conventions config file; its present `[python]` / `[typescript]` /
        /// `[rust]` tables decide which rules run.
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
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
        /// testing-conventions config file providing the coverage thresholds.
        #[arg(long, default_value = "testing-conventions.toml")]
        config: PathBuf,
    },
}

/// Lints enforced on integration tests (mocking mechanism & style, and more to
/// come). The README's "Integration" taxonomy.
#[derive(Subcommand, Debug)]
enum IntegrationRule {
    /// Lint integration test files for mocking mechanism & style (Python).
    Lint {
        /// Directory to scan recursively for Python test files.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: colocated_test::Language,
    },
}

pub fn run<I, T>(args: I) -> anyhow::Result<i32>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(args)?;
    match cli.command {
        Some(Command::Check { path, config }) => run_check(&path, &config),
        // A bare invocation (no subcommand) is a no-op success; `--help` lists the
        // available commands.
        None => Ok(0),
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
        },
        Some(Command::Integration { rule }) => match rule {
            IntegrationRule::Lint { path, language } => run_integration_lint(&path, language),
        },
    }
}

/// One rule the `check` umbrella runs, resolved from the config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlannedCheck {
    ColocatedTest(colocated_test::Language),
    Coverage(colocated_test::Language),
    IntegrationLint(colocated_test::Language),
}

impl PlannedCheck {
    /// A label matching the standalone subcommand this rule runs, e.g.
    /// `unit colocated-test (python)`.
    fn label(self) -> String {
        let (rule, language) = match self {
            PlannedCheck::ColocatedTest(language) => ("unit colocated-test", language),
            PlannedCheck::Coverage(language) => ("unit coverage", language),
            PlannedCheck::IntegrationLint(language) => ("integration lint", language),
        };
        format!("{rule} ({})", language_name(language))
    }

    /// Run this rule over `root`, delegating to the same helper its standalone
    /// subcommand uses — so `check` and the direct command behave identically.
    fn run(self, root: &Path, config_path: &Path) -> anyhow::Result<i32> {
        match self {
            PlannedCheck::ColocatedTest(language) => {
                run_unit_colocated_test(root, language, config_path)
            }
            PlannedCheck::Coverage(language) => run_unit_coverage(root, language, config_path),
            PlannedCheck::IntegrationLint(language) => run_integration_lint(root, language),
        }
    }
}

/// The rules `check` will run, plus notes for configured-but-unenforced items.
#[derive(Debug, Default, PartialEq, Eq)]
struct CheckPlan {
    /// Rules to run, in order.
    run: Vec<PlannedCheck>,
    /// Configured keys no implemented rule covers yet — surfaced to stderr so a
    /// forward-looking config entry stays visible, never a silent skip.
    notes: Vec<String>,
}

/// Resolve `config` into the rules to run: every implemented rule its present
/// language tables enable.
///
/// This mapping — which rules exist for which language — is the one place a new
/// rule wires into CI: add it here and the reusable workflow (which runs `check`)
/// enforces it with no workflow edit. A configured threshold with no rule yet is
/// recorded as a note rather than run, so it neither fails the build nor passes
/// silently.
fn plan_checks(config: &config::Config) -> CheckPlan {
    use colocated_test::Language::{Python, TypeScript};
    let mut plan = CheckPlan::default();

    if let Some(python) = &config.python {
        plan.run.push(PlannedCheck::ColocatedTest(Python));
        if python.coverage.is_some() {
            plan.run.push(PlannedCheck::Coverage(Python));
        }
        plan.run.push(PlannedCheck::IntegrationLint(Python));
    }

    if let Some(typescript) = &config.typescript {
        plan.run.push(PlannedCheck::ColocatedTest(TypeScript));
        if typescript.coverage.is_some() {
            // The schema models TypeScript coverage, but the rule isn't implemented.
            plan.notes.push(
                "[typescript].coverage is set but TypeScript coverage isn't implemented \
                 yet (#31) — skipping it"
                    .to_string(),
            );
        }
    }

    if config.rust.is_some() {
        // The colocated-test rule is file-based and doesn't cover Rust's inline
        // `#[cfg(test)]` (#40), and Rust coverage (#37) isn't implemented — a [rust]
        // table enables no checks yet, so say so rather than silently ignore it.
        plan.notes.push(
            "[rust] is set but no Rust rule is implemented yet (colocated-test #40, \
             coverage #37) — skipping it"
                .to_string(),
        );
    }

    plan
}

/// Run the config-driven umbrella: every rule the config at `config_path` enables,
/// over `root`.
///
/// Runs the whole plan (not fail-fast) so one pass surfaces every problem, and
/// returns `1` if any rule reports a violation or fails to run, `0` only when they
/// all pass. A missing/invalid config — or one that enables no checks at all — is
/// an error: the umbrella is config-driven, so with nothing configured there is
/// nothing to enforce.
fn run_check(root: &Path, config_path: &Path) -> anyhow::Result<i32> {
    let config = config::load_config(config_path)?;
    let plan = plan_checks(&config);

    for note in &plan.notes {
        eprintln!("note: {note}");
    }

    if plan.run.is_empty() {
        anyhow::bail!(
            "config `{}` enables no checks — add a [python] or [typescript] table so \
             `check` has a rule to run",
            config_path.display()
        );
    }

    let mut failures = 0usize;
    for check in &plan.run {
        let label = check.label();
        eprintln!("==> {label}");
        match check.run(root, config_path) {
            Ok(0) => {}
            // The rule already printed its own violations to stderr.
            Ok(_) => failures += 1,
            // Print the whole cause chain (as `main` does) so a rule that couldn't
            // run — bad config, missing toolchain — explains itself, then keep going
            // so the remaining rules still report.
            Err(err) => {
                eprintln!("error: {label} could not run: {err:#}");
                failures += 1;
            }
        }
    }

    if failures == 0 {
        Ok(0)
    } else {
        eprintln!("error: {failures} of {} check(s) failed", plan.run.len());
        Ok(1)
    }
}

/// The `--language` name for `language` (matches what the user types), for labels.
fn language_name(language: colocated_test::Language) -> &'static str {
    match language {
        colocated_test::Language::Python => "python",
        colocated_test::Language::TypeScript => "typescript",
    }
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
fn run_unit_coverage(
    root: &Path,
    language: colocated_test::Language,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let config = config::load_config(config_path)?;
    let (thresholds, exempt) = match language {
        colocated_test::Language::Python => {
            let python = config
                .python
                .as_ref()
                .context("config has no [python] table to read coverage thresholds from")?;
            let coverage = python
                .coverage
                .as_ref()
                .context("config [python] table has no `coverage` thresholds")?;
            let thresholds = coverage::Thresholds {
                fail_under: coverage.fail_under,
                branch: coverage.branch,
            };
            let exempt = config::resolve_exempt(root, &python.exempt, config::Rule::Coverage)?;
            (thresholds, exempt)
        }
        colocated_test::Language::TypeScript => anyhow::bail!(
            "`unit coverage` supports `--language python` only for now; \
             TypeScript coverage is a separate item"
        ),
    };
    let omit: Vec<String> = exempt.into_iter().collect();
    match coverage::measure(root, thresholds, &omit)? {
        coverage::Outcome::Pass => Ok(0),
        coverage::Outcome::Fail(reason) => {
            eprintln!("error: coverage check failed — {reason}");
            Ok(1)
        }
    }
}

/// Run the integration-test lints over `root` for `language`, printing each
/// violation to stderr as `path:line: rule — message` and returning `1` when any
/// are found, `0` otherwise.
fn run_integration_lint(root: &Path, language: colocated_test::Language) -> anyhow::Result<i32> {
    match language {
        colocated_test::Language::Python => {}
        colocated_test::Language::TypeScript => {
            anyhow::bail!("`integration lint` supports `--language python` only for now")
        }
    }
    let violations = lint::find_violations(root)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use colocated_test::Language;

    #[test]
    fn no_args_returns_ok_zero() {
        assert_eq!(run(["testing-conventions"]).unwrap(), 0);
    }

    #[test]
    fn check_requires_a_path() {
        // `check` now runs rules over a tree, so its `<PATH>` is required: a bare
        // `check` is a usage error, not the old reserved no-op.
        assert!(run(["testing-conventions", "check"]).is_err());
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

    // --- the config-driven `check` plan (#56) ---

    /// A config carrying only a `[python]` table, with the given coverage floor.
    fn python_config(coverage: Option<config::PythonCoverage>) -> config::Config {
        config::Config {
            python: Some(config::PythonConfig {
                coverage,
                exempt: vec![],
            }),
            typescript: None,
            rust: None,
        }
    }

    #[test]
    fn a_python_table_plans_colocated_test_and_lint_but_not_coverage_without_a_floor() {
        let plan = plan_checks(&python_config(None));
        assert_eq!(
            plan.run,
            vec![
                PlannedCheck::ColocatedTest(Language::Python),
                PlannedCheck::IntegrationLint(Language::Python),
            ]
        );
        assert!(plan.notes.is_empty(), "notes: {:?}", plan.notes);
    }

    #[test]
    fn a_python_coverage_floor_adds_the_coverage_rule() {
        let plan = plan_checks(&python_config(Some(config::PythonCoverage {
            branch: true,
            fail_under: 100,
        })));
        assert!(plan.run.contains(&PlannedCheck::Coverage(Language::Python)));
    }

    #[test]
    fn a_typescript_table_plans_colocated_test_only() {
        let config = config::Config {
            python: None,
            typescript: Some(config::TypeScriptConfig {
                coverage: None,
                exempt: vec![],
            }),
            rust: None,
        };
        assert_eq!(
            plan_checks(&config).run,
            vec![PlannedCheck::ColocatedTest(Language::TypeScript)]
        );
    }

    #[test]
    fn typescript_coverage_is_noted_not_run() {
        let config = config::Config {
            python: None,
            typescript: Some(config::TypeScriptConfig {
                coverage: Some(config::TypeScriptCoverage {
                    lines: 90,
                    branches: 90,
                    functions: 90,
                    statements: 90,
                }),
                exempt: vec![],
            }),
            rust: None,
        };
        let plan = plan_checks(&config);
        // colocated-test still runs; the unimplemented coverage rule is noted, not run.
        assert_eq!(
            plan.run,
            vec![PlannedCheck::ColocatedTest(Language::TypeScript)]
        );
        assert_eq!(plan.notes.len(), 1);
        assert!(
            plan.notes[0].contains("typescript"),
            "note: {}",
            plan.notes[0]
        );
    }

    #[test]
    fn a_rust_table_runs_nothing_and_is_noted() {
        let config = config::Config {
            python: None,
            typescript: None,
            rust: Some(config::RustConfig {
                coverage: None,
                exempt: vec![],
            }),
        };
        let plan = plan_checks(&config);
        assert!(plan.run.is_empty(), "run: {:?}", plan.run);
        assert_eq!(plan.notes.len(), 1);
    }

    #[test]
    fn an_empty_config_plans_nothing() {
        let config = config::Config {
            python: None,
            typescript: None,
            rust: None,
        };
        assert!(plan_checks(&config).run.is_empty());
    }
}
