pub mod config;
pub mod coverage;
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
        language: location::Language,
        /// testing-conventions config file providing the coverage thresholds.
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
        // group (e.g. `unit location`).
        Some(Command::Check) | None => Ok(0),
        Some(Command::Unit { rule }) => match rule {
            UnitRule::Location {
                path,
                language,
                config,
            } => run_unit_location(&path, language, &config),
            UnitRule::Coverage {
                path,
                language,
                config,
            } => run_unit_coverage(&path, language, &config),
        },
    }
}

/// Run the unit-test location check over `root` for `language`, reporting orphans.
///
/// Loads the `location`-rule exemptions from the config at `config_path` (no
/// config file → no exemptions). Returns `0` when every source file has its
/// colocated unit test; otherwise prints each orphan to stderr and returns `1`.
fn run_unit_location(
    root: &Path,
    language: location::Language,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let exempt = location_exemptions(root, language, config_path)?;
    let orphans = location::missing_unit_tests(root, language, &exempt)?;
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

/// The `location`-rule exempt paths for `language`, resolved (and validated) from
/// the config at `config_path`. A missing config file means no exemptions — the
/// check still runs, just with nothing exempted.
fn location_exemptions(
    root: &Path,
    language: location::Language,
    config_path: &Path,
) -> anyhow::Result<std::collections::BTreeSet<String>> {
    if !config_path.exists() {
        return Ok(std::collections::BTreeSet::new());
    }
    let config = config::load_config(config_path)?;
    config::resolve_exempt(root, config.exemptions(language), config::Rule::Location)
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
    let (thresholds, exempt) = match language {
        location::Language::Python => {
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
        location::Language::TypeScript => anyhow::bail!(
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
