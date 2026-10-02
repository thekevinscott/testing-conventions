//! The `integration lint` rule's dispatcher: the suite-tier lints, per language.

use std::path::Path;

#[cfg(test)]
use crate::config;
use crate::waivers::{apply_waivers, ExemptSelect};
use crate::{colocated_test, isolation, lint, tiers, ts, violation, IntegrationLintLanguage};

/// Run the integration-test lints over the package root above `root`, printing each
/// violation and returning `1` when any are found. A tree with no manifest is scanned at `root`.
pub fn run(
    root: &Path,
    language: IntegrationLintLanguage,
    config_path: &Path,
) -> anyhow::Result<i32> {
    let package_root = tiers::package_root(root, manifest_name(language));
    let scan_root = package_root.as_deref().unwrap_or(root);
    let raw = find(root, package_root.as_deref(), scan_root, language)?;
    let violations = apply_waivers(raw, scan_root, config_path, select(language))?;
    if violations.is_empty() {
        return Ok(0);
    }
    for v in &violations {
        eprintln!("{}", violation::rendered(v));
    }
    eprintln!("error: {} lint violation(s)", violations.len());
    Ok(1)
}

/// The manifest file whose nearest ancestor is `language`'s package root.
fn manifest_name(language: IntegrationLintLanguage) -> &'static str {
    match language {
        IntegrationLintLanguage::Python => "pyproject.toml",
        IntegrationLintLanguage::TypeScript => "package.json",
        IntegrationLintLanguage::Rust => "Cargo.toml",
    }
}

/// The `[[<lang>.exempt]]` table `language`'s violations are waived against.
fn select(language: IntegrationLintLanguage) -> ExemptSelect {
    match language {
        IntegrationLintLanguage::Python => |c| c.exemptions(colocated_test::Language::Python),
        IntegrationLintLanguage::TypeScript => {
            |c| c.exemptions(colocated_test::Language::TypeScript)
        }
        IntegrationLintLanguage::Rust => |c| c.rust_exemptions(),
    }
}

/// `language`'s suite-tier violations. Python and TypeScript scan the package's suite
/// directories when a manifest names them, and fall back to the whole tree at `root`.
fn find(
    root: &Path,
    package_root: Option<&Path>,
    scan_root: &Path,
    language: IntegrationLintLanguage,
) -> anyhow::Result<Vec<lint::Violation>> {
    match (language, package_root) {
        (IntegrationLintLanguage::Python, Some(package_root)) => {
            lint::find_suite_violations(package_root)
        }
        (IntegrationLintLanguage::Python, None) => lint::find_violations(root),
        (IntegrationLintLanguage::TypeScript, Some(package_root)) => {
            ts::find_suite_violations(package_root)
        }
        (IntegrationLintLanguage::TypeScript, None) => ts::find_integration_violations(root),
        (IntegrationLintLanguage::Rust, _) => isolation::find_integration_violations(scan_root),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(slug: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("tc-integration-lint-{slug}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn each_language_looks_for_its_own_manifest() {
        assert_eq!(
            manifest_name(IntegrationLintLanguage::Python),
            "pyproject.toml"
        );
        assert_eq!(
            manifest_name(IntegrationLintLanguage::TypeScript),
            "package.json"
        );
        assert_eq!(manifest_name(IntegrationLintLanguage::Rust), "Cargo.toml");
    }

    #[test]
    fn each_language_waives_against_its_own_exempt_table() {
        let config = config::Config::default();
        for language in [
            IntegrationLintLanguage::Python,
            IntegrationLintLanguage::TypeScript,
            IntegrationLintLanguage::Rust,
        ] {
            assert!(select(language)(&config).is_empty());
        }
    }

    #[test]
    fn a_manifestless_python_or_typescript_tree_is_scanned_whole() {
        let dir = tree("manifestless");
        std::fs::write(dir.join("widget_test.py"), "def test_widget():\n    pass\n").unwrap();
        for language in [
            IntegrationLintLanguage::Python,
            IntegrationLintLanguage::TypeScript,
        ] {
            assert!(find(&dir, None, &dir, language).unwrap().is_empty());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_manifest_scopes_python_and_typescript_to_the_suite_directories() {
        let dir = tree("suite-scoped");
        std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"widget\"\n").unwrap();
        for language in [
            IntegrationLintLanguage::Python,
            IntegrationLintLanguage::TypeScript,
        ] {
            assert!(find(&dir, Some(&dir), &dir, language).unwrap().is_empty());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_clean_rust_suite_exits_zero() {
        let dir = tree("clean");
        std::fs::create_dir_all(dir.join("tests")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"widget\"\n").unwrap();
        std::fs::write(
            dir.join("tests/suite.rs"),
            "#[test]\nfn runs() {\n    assert!(true);\n}\n",
        )
        .unwrap();
        let exit = run(
            &dir,
            IntegrationLintLanguage::Rust,
            Path::new("no-such-integration-lint-config.toml"),
        );
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(exit.unwrap(), 0);
    }

    #[test]
    fn a_doubling_rust_suite_exits_one() {
        let dir = tree("doubling");
        std::fs::create_dir_all(dir.join("tests")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"widget\"\n").unwrap();
        std::fs::write(
            dir.join("tests/suite.rs"),
            "#[double]\nuse widget::loader;\n#[test]\nfn runs() {\n    assert!(true);\n}\n",
        )
        .unwrap();
        let exit = run(
            &dir,
            IntegrationLintLanguage::Rust,
            Path::new("no-such-integration-lint-config.toml"),
        );
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(exit.unwrap(), 1);
    }
}
