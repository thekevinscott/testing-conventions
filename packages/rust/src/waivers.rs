//! Waiver application for the deterministic lints: a resolved `[[<lang>.exempt]]` table drops
//! the violations it covers.

use std::path::Path;

#[cfg(test)]
use crate::colocated_test;
use crate::{config, lint};

/// Selects a language's `[[<lang>.exempt]]` table from a loaded config.
pub type ExemptSelect = fn(&config::Config) -> &[config::Exemption];

/// Drop the violations whose `root`-relative path is exempt for their rule.
pub fn apply_waivers(
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

#[cfg(test)]
mod tests {
    use super::*;

    fn python_exemptions(config: &config::Config) -> &[config::Exemption] {
        config.exemptions(colocated_test::Language::Python)
    }

    #[test]
    fn a_violation_with_an_unwaivable_rule_id_is_kept() {
        let dir = std::env::temp_dir().join(format!("tc-waiver-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("testing-conventions.toml");
        std::fs::write(&config_path, "").unwrap();
        let violation = lint::Violation {
            file: dir.join("widget_test.py"),
            line: 1,
            rule: "not-a-waivable-rule",
            message: "synthetic".to_string(),
        };
        let kept = apply_waivers(
            vec![violation.clone()],
            &dir,
            &config_path,
            python_exemptions,
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(kept.unwrap(), vec![violation]);
    }

    #[test]
    fn a_missing_config_keeps_every_violation() {
        let violation = lint::Violation {
            file: std::path::PathBuf::from("/tree/widget_test.py"),
            line: 1,
            rule: "no-monkeypatch",
            message: "synthetic".to_string(),
        };
        let kept = apply_waivers(
            vec![violation.clone()],
            Path::new("/tree"),
            Path::new("/nonexistent-tc-waivers.toml"),
            python_exemptions,
        );
        assert_eq!(kept.unwrap(), vec![violation]);
    }

    #[test]
    fn waivers_resolve_each_rule_once_and_keep_out_of_root_files() {
        let dir = std::env::temp_dir().join(format!("tc-waiver-full-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("widget_test.py"), "def test_widget():\n    pass\n").unwrap();
        let config_path = dir.join("testing-conventions.toml");
        std::fs::write(
            &config_path,
            "[[python.exempt]]\n\
             path = \"widget_test.py\"\n\
             rules = [\"no-monkeypatch\"]\n\
             reason = \"synthetic waiver for the resolution paths\"\n",
        )
        .unwrap();
        let violation = |file: std::path::PathBuf| lint::Violation {
            file,
            line: 1,
            rule: "no-monkeypatch",
            message: "synthetic".to_string(),
        };
        let waived = violation(dir.join("widget_test.py"));
        let kept_in_root = violation(dir.join("other_test.py"));
        let outside_root = violation(std::path::PathBuf::from("/elsewhere/widget_test.py"));
        let kept = apply_waivers(
            vec![waived, kept_in_root.clone(), outside_root.clone()],
            &dir,
            &config_path,
            python_exemptions,
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(kept.unwrap(), vec![kept_in_root, outside_root]);
    }
}
