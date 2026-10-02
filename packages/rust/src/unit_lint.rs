//! The `unit lint` rule's dispatcher: the unit-suite isolation lints, per language.

use std::path::Path;

use crate::waivers::{apply_waivers, ExemptSelect};
use crate::{colocated_test, isolation, lint, tiers, ts, violation};

/// What one language's unit-isolation scan found, and the root its waivers resolve against.
struct Scan<'a> {
    violations: Vec<lint::Violation>,
    select: ExemptSelect,
    waiver_root: &'a Path,
}

/// Run the unit-suite isolation lints over `root`, printing each violation and returning
/// `1` when any are found.
pub fn run(root: &Path, language: isolation::Language, config_path: &Path) -> anyhow::Result<i32> {
    let crate_root = tiers::package_root(root, "Cargo.toml");
    let scan = scan(root, crate_root.as_deref(), language)?;
    let violations = apply_waivers(scan.violations, scan.waiver_root, config_path, scan.select)?;
    if violations.is_empty() {
        return Ok(0);
    }
    for v in &violations {
        eprintln!("{}", violation::rendered(v));
    }
    eprintln!("error: {} isolation violation(s)", violations.len());
    Ok(1)
}

/// Scan `root` for `language`'s unit-isolation violations. The Rust arm resolves waivers
/// against `crate_root` so a scan pointed at `src/` keeps exempt paths crate-root-relative.
fn scan<'a>(
    root: &'a Path,
    crate_root: Option<&'a Path>,
    language: isolation::Language,
) -> anyhow::Result<Scan<'a>> {
    Ok(match language {
        isolation::Language::Rust => {
            let crate_root = crate_root.unwrap_or(root);
            Scan {
                violations: isolation::find_violations(root, crate_root)?,
                select: |c| c.rust_exemptions(),
                waiver_root: crate_root,
            }
        }
        isolation::Language::TypeScript => Scan {
            violations: ts::find_unit_violations(root)?,
            select: |c| c.exemptions(colocated_test::Language::TypeScript),
            waiver_root: root,
        },
        isolation::Language::Python => Scan {
            violations: lint::find_unit_isolation_violations(root)?,
            select: |c| c.exemptions(colocated_test::Language::Python),
            waiver_root: root,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(slug: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tc-unit-lint-{slug}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_rust_arm_resolves_waivers_against_the_crate_root_above_the_scan() {
        let dir = tree("crate-root");
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"widget\"\n").unwrap();
        let src = dir.join("src");
        let scan = scan(&src, Some(&dir), isolation::Language::Rust).unwrap();
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(scan.waiver_root, dir);
        assert!(scan.violations.is_empty());
    }

    #[test]
    fn a_manifestless_rust_scan_resolves_waivers_against_the_scan_root() {
        let dir = tree("manifestless");
        let scan = scan(&dir, None, isolation::Language::Rust).unwrap();
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(scan.waiver_root, dir);
    }

    #[test]
    fn the_typescript_and_python_arms_scan_the_root_they_are_given() {
        let dir = tree("per-language");
        for language in [isolation::Language::TypeScript, isolation::Language::Python] {
            let scan = scan(&dir, None, language).unwrap();
            assert_eq!(scan.waiver_root, dir);
            assert!(scan.violations.is_empty());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_clean_tree_exits_zero() {
        let dir = tree("clean");
        std::fs::write(
            dir.join("widget.rs"),
            "pub fn one() -> u8 {\n    1\n}\n\
             #[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    \
             fn ones() {\n        assert_eq!(one(), 1);\n    }\n}\n",
        )
        .unwrap();
        let exit = run(
            &dir,
            isolation::Language::Rust,
            Path::new("no-such-unit-lint-config.toml"),
        );
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(exit.unwrap(), 0);
    }

    #[test]
    fn a_violating_tree_exits_one() {
        let dir = tree("violating");
        std::fs::write(
            dir.join("widget.rs"),
            "pub fn one() -> u8 {\n    1\n}\n\
             #[cfg(test)]\nmod tests {\n    #[test]\n    \
             fn ones() {\n        assert_eq!(crate::other::two(), 2);\n    }\n}\n",
        )
        .unwrap();
        let exit = run(
            &dir,
            isolation::Language::Rust,
            Path::new("no-such-unit-lint-config.toml"),
        );
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(exit.unwrap(), 1);
    }
}
