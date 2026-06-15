pub mod config;
pub mod coverage;
pub mod lint;
pub mod location;

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
}

/// Rules enforced on the unit-test suite (the README's "Unit" taxonomy).
#[derive(Subcommand, Debug)]
enum UnitRule {
    /// Check that every source file has a colocated unit test.
    Location {
        /// Directory to scan recursively.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: location::Language,
    },
    /// Check that the unit suite meets the configured coverage floor.
    Coverage {
        /// Directory whose unit suite is run and measured.
        path: PathBuf,
        /// Language convention to enforce (required).
        #[arg(long, value_enum)]
        language: location::Language,
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
        language: location::Language,
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
        // group (e.g. `unit location`).
        Some(Command::Check) | None => Ok(0),
        Some(Command::Unit { rule }) => match rule {
            UnitRule::Location { path, language } => run_unit_location(&path, language),
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

/// Run the unit-test location check over `root` for `language`, reporting orphans.
///
/// Returns `0` when every source file has its colocated unit test; otherwise
/// prints each orphan to stderr and returns `1`.
fn run_unit_location(root: &Path, language: location::Language) -> anyhow::Result<i32> {
    let orphans = location::missing_unit_tests(root, language)?;
    if orphans.is_empty() {
        return Ok(0);
    }
    for orphan in &orphans {
        eprintln!("missing colocated unit test: {}", orphan.display());
    }
    eprintln!(
        "error: {} source file(s) missing a colocated unit test",
        orphans.len()
    );
    Ok(1)
}

/// Run the unit-test coverage check over `root` for `language`, enforcing the
/// floor from the config at `config_path`. Returns `0` when the floor is met,
/// `1` otherwise.
fn run_unit_coverage(
    root: &Path,
    language: location::Language,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let config = config::load_config(config_path)?;
    let thresholds = match language {
        location::Language::Python => {
            let python = config
                .python
                .context("config has no [python] table to read coverage thresholds from")?;
            coverage::Thresholds {
                fail_under: python.coverage.fail_under,
                branch: python.coverage.branch,
            }
        }
        location::Language::TypeScript => anyhow::bail!(
            "`unit coverage` supports `--language python` only for now; \
             TypeScript coverage is a separate item"
        ),
    };
    match coverage::measure(root, thresholds)? {
        coverage::Outcome::Pass => Ok(0),
        coverage::Outcome::Fail(reason) => {
            eprintln!("error: coverage check failed — {reason}");
            Ok(1)
        }
    }
}

/// Run the integration-test lints over `root` for `language`.
///
/// Skeleton (#48): the lint set is empty, so this reports nothing and returns
/// `0`. The lints (#49–#52) turn real violations into a non-zero exit, printing
/// each to stderr as `path:line: rule — message`.
fn run_integration_lint(root: &Path, language: location::Language) -> anyhow::Result<i32> {
    match language {
        location::Language::Python => {}
        location::Language::TypeScript => {
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
