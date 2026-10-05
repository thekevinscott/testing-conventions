//! The `unit one-function-per-file` rule's dispatcher.

use std::path::Path;

use crate::waivers::{apply_waivers, ExemptSelect};
use crate::{colocated_test, config, one_function, violation};

/// Run the one-function-per-file rule over `root`, printing each violation and returning
/// `1` when any are found. A language with no configured threshold reports that and exits `0`.
pub fn run(
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
        println!("{}", opt_in_notice(language));
        return Ok(0);
    };
    let (raw, scanned) = one_function::find_violations(root, language, max_lines)?;
    let violations = apply_waivers(raw, root, config_path, select(language))?;
    if violations.is_empty() {
        eprintln!("one-function-per-file: scanned {scanned} file(s), 0 violations");
        return Ok(0);
    }
    for v in &violations {
        eprintln!("{}", violation::rendered(v));
    }
    eprintln!(
        "error: {} function(s) sharing a file with another function over the \
         {max_lines}-line threshold (move each to its own module, or add an \
         `exempt` entry with a reason)",
        violations.len()
    );
    Ok(1)
}

/// What an unconfigured language is told: the rule is off, and which table turns it on.
fn opt_in_notice(language: colocated_test::Language) -> String {
    let key = match language {
        colocated_test::Language::Python => "python",
        colocated_test::Language::TypeScript => "typescript",
        colocated_test::Language::Rust => "rust",
    };
    format!(
        "unit one-function-per-file: not enabled for {key} — \
         set `[{key}].one_function_per_file` to opt in"
    )
}

/// The `[[<lang>.exempt]]` table `language`'s violations are waived against.
fn select(language: colocated_test::Language) -> ExemptSelect {
    match language {
        colocated_test::Language::Python => |c| c.exemptions(colocated_test::Language::Python),
        colocated_test::Language::TypeScript => {
            |c| c.exemptions(colocated_test::Language::TypeScript)
        }
        colocated_test::Language::Rust => |c| c.rust_exemptions(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_language_names_its_own_opt_in_table() {
        assert_eq!(
            opt_in_notice(colocated_test::Language::Python),
            "unit one-function-per-file: not enabled for python — \
             set `[python].one_function_per_file` to opt in"
        );
        assert_eq!(
            opt_in_notice(colocated_test::Language::TypeScript),
            "unit one-function-per-file: not enabled for typescript — \
             set `[typescript].one_function_per_file` to opt in"
        );
        assert_eq!(
            opt_in_notice(colocated_test::Language::Rust),
            "unit one-function-per-file: not enabled for rust — \
             set `[rust].one_function_per_file` to opt in"
        );
    }

    #[test]
    fn each_language_waives_against_its_own_exempt_table() {
        let config = config::Config::default();
        for language in [
            colocated_test::Language::Python,
            colocated_test::Language::TypeScript,
            colocated_test::Language::Rust,
        ] {
            assert!(select(language)(&config).is_empty());
        }
    }

    #[test]
    fn an_unconfigured_language_reports_the_rule_is_off_and_exits_zero() {
        let exit = run(
            Path::new(env!("CARGO_MANIFEST_DIR")),
            colocated_test::Language::Rust,
            Path::new("no-such-one-function-config.toml"),
        );

        assert_eq!(exit.unwrap(), 0);
    }

    #[test]
    fn an_invalid_config_fails_before_scanning() {
        let config_path = std::env::temp_dir().join(format!(
            "tc-one-function-invalid-{}.toml",
            std::process::id()
        ));
        std::fs::write(&config_path, "[rust\n").unwrap();
        let result = run(
            Path::new(env!("CARGO_MANIFEST_DIR")),
            colocated_test::Language::Rust,
            &config_path,
        );
        let _ = std::fs::remove_file(&config_path);

        assert!(result.is_err());
    }
}
