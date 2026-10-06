use std::path::Path;

use crate::{colocated_test, config, mutation, split_scopes};

/// Run the per-language mutation engine over `root` and fail on any surviving mutant
/// not lifted by a `mutation` exemption. `base` scopes the run to the diff.
pub fn run(
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
            let scopes = config::resolve_exempt_scoped(root, &rust.exempt, config::Rule::Mutation)?;
            let (exempt, exempt_lines) = split_scopes(scopes);
            mutation::measure_rust(root, &exempt, &exempt_lines, base, &rust.features)?
        }
        colocated_test::Language::TypeScript => {
            let typescript = config.typescript.unwrap_or_default();
            let scopes =
                config::resolve_exempt_scoped(root, &typescript.exempt, config::Rule::Mutation)?;
            let (exempt, exempt_lines) = split_scopes(scopes);
            let adapter = ts_adapter.ok_or_else(|| {
                anyhow::anyhow!(
                    "the TypeScript mutation adapter path is required: pass \
                     `--ts-mutation-adapter <path>`. The npm `testing-conventions` CLI appends it \
                     automatically — run the check through that CLI, not the raw binary."
                )
            })?;
            mutation::measure_typescript(root, &exempt, &exempt_lines, base, adapter)?
        }
        colocated_test::Language::Python => {
            let python = config.python.unwrap_or_default();
            let scopes =
                config::resolve_exempt_scoped(root, &python.exempt, config::Rule::Mutation)?;
            let (exempt, exempt_lines) = split_scopes(scopes);
            mutation::measure_python(root, &exempt, &exempt_lines, base)?
        }
    };
    Ok(exit_code(measurement))
}

fn exit_code(measurement: mutation::Measurement) -> i32 {
    let (count, survivors) = match measurement {
        mutation::Measurement::EngineNotRun => {
            println!("unit mutation: no mutatable changed lines — engine not run");
            return 0;
        }
        mutation::Measurement::Tested { count, survivors } => (count, survivors),
    };
    if survivors.is_empty() {
        println!("{}", clean_message(count));
        return 0;
    }

    eprintln!("{}", survivor_report(&survivors));
    1
}

/// The report a surviving mutant set prints: the headline naming how to clear them, then one
/// `file:line: description` line per survivor.
fn survivor_report(survivors: &[mutation::Survivor]) -> String {
    let detail: Vec<String> = survivors
        .iter()
        .map(|s| format!("  {}:{}: {}", s.file, s.line, s.description))
        .collect();
    format!(
        "error: {} unexplained surviving mutant(s) — kill each with an assertion, or lift an \
         equivalent/defensive one with a reason-required \
         `[[<language>.exempt]] rules = [\"mutation\"]`:\n{}",
        survivors.len(),
        detail.join("\n")
    )
}

fn clean_message(count: usize) -> String {
    if count == 0 {
        "unit mutation: the engine found no mutants to test".to_string()
    } else {
        format!("unit mutation: no surviving mutants — every mutation was caught ({count} mutant(s) tested)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unrun_engine_and_all_clean_results_exit_zero() {
        assert_eq!(exit_code(mutation::Measurement::EngineNotRun), 0);
        for count in [0, 3] {
            assert_eq!(
                exit_code(mutation::Measurement::Tested {
                    count,
                    survivors: Vec::new(),
                }),
                0
            );
        }
    }

    #[test]
    fn clean_results_name_whether_mutants_were_tested() {
        assert_eq!(
            clean_message(0),
            "unit mutation: the engine found no mutants to test"
        );
        assert_eq!(
            clean_message(3),
            "unit mutation: no surviving mutants — every mutation was caught (3 mutant(s) tested)"
        );
    }

    #[test]
    fn a_survivor_exits_one() {
        assert_eq!(
            exit_code(mutation::Measurement::Tested {
                count: 1,
                survivors: vec![mutation::Survivor {
                    file: "src/lib.rs".to_string(),
                    line: 3,
                    description: "replace true with false".to_string(),
                }],
            }),
            1
        );
    }

    #[test]
    fn the_survivor_report_names_every_survivor_under_one_headline() {
        let survivor = |line, description: &str| mutation::Survivor {
            file: "src/lib.rs".to_string(),
            line,
            description: description.to_string(),
        };
        let report = survivor_report(&[survivor(3, "replace true with false"), survivor(9, "x")]);
        let mut lines = report.lines();
        assert_eq!(
            lines.next(),
            Some(
                "error: 2 unexplained surviving mutant(s) — kill each with an assertion, or lift \
                 an equivalent/defensive one with a reason-required `[[<language>.exempt]] \
                 rules = [\"mutation\"]`:"
            )
        );
        assert_eq!(
            lines.next(),
            Some("  src/lib.rs:3: replace true with false")
        );
        assert_eq!(lines.next(), Some("  src/lib.rs:9: x"));
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn an_invalid_config_fails_before_starting_an_engine() {
        let config_path = std::env::temp_dir().join(format!(
            "tc-unit-mutation-invalid-{}.toml",
            std::process::id()
        ));
        std::fs::write(&config_path, "[rust\n").unwrap();
        let result = run(
            Path::new(env!("CARGO_MANIFEST_DIR")),
            colocated_test::Language::Rust,
            None,
            &config_path,
            None,
        );
        let _ = std::fs::remove_file(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn typescript_requires_an_adapter() {
        let config_path = Path::new("no-such-config.toml");
        let result = run(
            Path::new(env!("CARGO_MANIFEST_DIR")),
            colocated_test::Language::TypeScript,
            None,
            config_path,
            None,
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("--ts-mutation-adapter"));
    }
}
