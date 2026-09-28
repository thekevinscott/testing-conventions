//! The `unit coverage` subcommand's dispatch: read the config, pick the language's measurement
//! path, and turn its outcome into an exit code.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::{colocated_test, config, coverage, patch_coverage, split_scopes};

/// Which measurement a run selects, shared by all three languages.
#[derive(Debug, PartialEq, Eq)]
enum Scope<'a> {
    /// No `--base`, no line-scoped exemption: measure the whole tree.
    WholeTree,
    /// `--base` given: measure the `<base>...HEAD` diff.
    Diff(&'a str),
    /// No `--base`, but a line-scoped exemption to subtract from the whole tree.
    LineExempt,
}

/// Run the unit coverage check over `root`, measuring the configured floor over the
/// whole tree or, with `base` set, over the `<base>...HEAD` diff. `0` when the floor is met.
pub fn run(
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
            let scopes =
                config::resolve_exempt_scoped(root, &python.exempt, config::Rule::Coverage)?;
            let (omit, exempt_lines) = split_scopes(scopes);
            match scope(base, &exempt_lines) {
                Scope::Diff(base) => {
                    patch_coverage::measure(root, base, thresholds, &omit, &exempt_lines)?
                }
                Scope::WholeTree => coverage::measure(root, thresholds, &omit)?,
                Scope::LineExempt => {
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
            let scopes =
                config::resolve_exempt_scoped(root, &typescript.exempt, config::Rule::Coverage)?;
            let (exclude, exempt_lines) = split_scopes(scopes);
            match scope(base, &exempt_lines) {
                Scope::Diff(base) => patch_coverage::measure_typescript(
                    root,
                    base,
                    thresholds,
                    &exclude,
                    &exempt_lines,
                )?,
                Scope::WholeTree => coverage::measure_typescript(root, thresholds, &exclude)?,
                Scope::LineExempt => patch_coverage::measure_line_exempt_typescript(
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
            let scopes = config::resolve_exempt_scoped(root, &rust.exempt, config::Rule::Coverage)?;
            let (ignore, exempt_lines) = split_scopes(scopes);
            match scope(base, &exempt_lines) {
                Scope::Diff(base) => patch_coverage::measure_rust(
                    root,
                    base,
                    thresholds,
                    &ignore,
                    &exempt_lines,
                    &rust.features,
                )?,
                Scope::WholeTree => {
                    coverage::measure_rust(root, thresholds, &ignore, &rust.features)?
                }
                Scope::LineExempt => patch_coverage::measure_line_exempt_rust(
                    root,
                    thresholds,
                    &ignore,
                    &exempt_lines,
                    &rust.features,
                )?,
            }
        }
    };
    Ok(exit_code(outcome))
}

/// The measurement a `--base` flag and a resolved line-exemption set select together.
fn scope<'a>(base: Option<&'a str>, exempt_lines: &BTreeMap<String, BTreeSet<u32>>) -> Scope<'a> {
    match base {
        Some(base) => Scope::Diff(base),
        None if exempt_lines.is_empty() => Scope::WholeTree,
        None => Scope::LineExempt,
    }
}

/// The check's exit code: `0` for a met floor, `1` for a shortfall, which is named on stderr.
fn exit_code(outcome: coverage::Outcome) -> i32 {
    match outcome {
        coverage::Outcome::Pass => 0,
        coverage::Outcome::Fail(reason) => {
            eprintln!("error: coverage check failed — {reason}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_exemption() -> BTreeMap<String, BTreeSet<u32>> {
        BTreeMap::from([("widget.py".to_string(), BTreeSet::from([3]))])
    }

    #[test]
    fn a_base_selects_the_diff_no_matter_what_is_line_exempt() {
        assert_eq!(scope(Some("main"), &BTreeMap::new()), Scope::Diff("main"));
        assert_eq!(scope(Some("main"), &line_exemption()), Scope::Diff("main"));
    }

    #[test]
    fn no_base_and_no_line_exemption_selects_the_whole_tree() {
        assert_eq!(scope(None, &BTreeMap::new()), Scope::WholeTree);
    }

    #[test]
    fn no_base_with_a_line_exemption_selects_the_line_exempt_whole_tree() {
        assert_eq!(scope(None, &line_exemption()), Scope::LineExempt);
    }

    #[test]
    fn a_met_floor_exits_zero() {
        assert_eq!(exit_code(coverage::Outcome::Pass), 0);
    }

    #[test]
    fn a_shortfall_exits_one() {
        assert_eq!(
            exit_code(coverage::Outcome::Fail("lines 91% < 100%".to_string())),
            1
        );
    }

    #[test]
    fn an_unparseable_config_fails_before_any_suite_runs() {
        let dir = std::env::temp_dir().join(format!("tc-unit-coverage-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("testing-conventions.toml");
        std::fs::write(&config_path, "[python.coverage\n").unwrap();
        let result = run(&dir, colocated_test::Language::Python, None, &config_path);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.is_err());
    }
}
