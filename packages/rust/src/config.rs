//! The testing-conventions config schema and loader.
//!
//! One config file is read into the in-memory [`Config`] below. The loader
//! parses *and* validates the config itself (the "self-guard" from issue #12):
//! a malformed or unknown-key config is an error, never a silently-accepted
//! default. Validation also covers the per-file [`Exemption`] list (issue #32):
//! every exemption must name at least one rule and carry a non-empty reason.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// A fully-parsed testing-conventions config file.
///
/// Holds the per-language coverage thresholds — the `[python]` / `[typescript]`
/// / `[rust]` tables from the README's "Configuration" section — and the
/// per-language `exempt` lists. Each table is optional so a repo can configure
/// only the languages it ships. Test locations follow convention, not config, so
/// there are no location keys here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub python: Option<PythonConfig>,
    pub typescript: Option<TypeScriptConfig>,
    pub rust: Option<RustConfig>,
}

/// The `[python]` table. Both keys are optional, so a repo can configure just
/// coverage, just exemptions, or both.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonConfig {
    pub coverage: Option<PythonCoverage>,
    #[serde(default)]
    pub exempt: Vec<Exemption>,
}

/// The `[typescript]` table.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeScriptConfig {
    pub coverage: Option<TypeScriptCoverage>,
    #[serde(default)]
    pub exempt: Vec<Exemption>,
}

/// The `[rust]` table.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustConfig {
    pub coverage: Option<RustCoverage>,
    #[serde(default)]
    pub exempt: Vec<Exemption>,
}

/// `[python].coverage`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonCoverage {
    pub branch: bool,
    pub fail_under: u8,
}

/// `[typescript].coverage`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeScriptCoverage {
    pub lines: u8,
    pub branches: u8,
    pub functions: u8,
    pub statements: u8,
}

/// `[rust].coverage`. Branch coverage is still experimental, so only
/// regions/lines are configurable.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustCoverage {
    pub regions: u8,
    pub lines: u8,
}

/// A rule a file can be exempted from (issue #32).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rule {
    /// The unit-test location/naming check ([`crate::location`]).
    Location,
    /// The unit-test coverage floor ([`crate::coverage`]).
    Coverage,
}

/// One auditable per-file exemption — a `[[<language>.exempt]]` entry.
///
/// The opposite of a silent ignore-glob: an exemption is declared in the one
/// config file, names the rules it lifts, and **must say why**. Empty
/// (comment-only) files need no entry — they carry no logic and are not
/// subjects — so this is for deliberate omissions the tool can't infer (a
/// launcher shim, generated code, a re-export barrel).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exemption {
    /// Path to the exempt file, relative to the scanned root.
    pub path: String,
    /// Which rules the exemption lifts (`location`, `coverage`).
    pub rules: Vec<Rule>,
    /// Why the omission is deliberate — required, and never empty.
    pub reason: String,
}

/// Read one config file at `path` into a [`Config`], validating it on the way.
///
/// The validation is the config's self-guard: `serde`'s `deny_unknown_fields`
/// rejects keys that aren't part of the schema, missing required keys and
/// wrong-typed values are type errors, malformed TOML fails to parse, and every
/// `exempt` entry must name a rule and carry a non-empty reason. Any of these
/// surfaces as an `Err` rather than a silently-accepted default.
pub fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let path = path.as_ref();
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("reading config file `{}`", path.display()))?;
    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("parsing config file `{}`", path.display()))?;
    config
        .validate()
        .with_context(|| format!("validating config file `{}`", path.display()))?;
    Ok(config)
}

impl Config {
    /// The `exempt` list for `language` (empty when the table is absent).
    pub fn exemptions(&self, language: crate::location::Language) -> &[Exemption] {
        match language {
            crate::location::Language::Python => self.python.as_ref().map_or(&[], |c| &c.exempt),
            crate::location::Language::TypeScript => {
                self.typescript.as_ref().map_or(&[], |c| &c.exempt)
            }
        }
    }

    /// Reject any `exempt` entry that names no rule or carries an empty reason —
    /// a reasonless or scopeless exemption can never be a silent pass.
    fn validate(&self) -> Result<()> {
        let tables = [
            ("python", self.python.as_ref().map(|c| &c.exempt)),
            ("typescript", self.typescript.as_ref().map(|c| &c.exempt)),
            ("rust", self.rust.as_ref().map(|c| &c.exempt)),
        ];
        for (table, exempt) in tables.into_iter().filter_map(|(t, e)| e.map(|e| (t, e))) {
            for entry in exempt {
                if entry.rules.is_empty() {
                    bail!(
                        "[{table}].exempt entry for `{}` names no rules — set \
                         `rules = [\"location\"]` and/or `\"coverage\"`",
                        entry.path
                    );
                }
                if entry.reason.trim().is_empty() {
                    bail!(
                        "[{table}].exempt entry for `{}` has an empty reason — \
                         every exemption must say why the file is exempt",
                        entry.path
                    );
                }
            }
        }
        Ok(())
    }
}

/// Resolve the set of exempt paths for `rule` from `exemptions`, validating that
/// each still points to a file under `root`.
///
/// A stale entry — a path that no longer exists — is an error, so the exempt
/// list can't silently rot (the auditable counterpart to an ignore-glob, which
/// would just stop matching). Returns the matching paths as `/`-joined,
/// `root`-relative strings, sorted and de-duplicated.
pub fn resolve_exempt(
    root: &Path,
    exemptions: &[Exemption],
    rule: Rule,
) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::new();
    for entry in exemptions {
        if !entry.rules.contains(&rule) {
            continue;
        }
        if !root.join(&entry.path).is_file() {
            bail!(
                "exempt entry `{}` matches no file under `{}` — remove the stale \
                 entry or fix the path",
                entry.path,
                root.display()
            );
        }
        paths.insert(entry.path.replace('\\', "/"));
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn parse(toml_src: &str) -> Result<Config> {
        let config: Config = toml::from_str(toml_src)?;
        config.validate()?;
        Ok(config)
    }

    #[test]
    fn an_exemption_with_no_rules_is_rejected() {
        let err = parse(
            "[python]\ncoverage = { branch = true, fail_under = 100 }\n\
             [[python.exempt]]\npath = \"cli.py\"\nrules = []\nreason = \"shim\"\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("names no rules"), "got: {err}");
    }

    #[test]
    fn an_exemption_with_an_empty_reason_is_rejected() {
        let err = parse(
            "[python]\ncoverage = { branch = true, fail_under = 100 }\n\
             [[python.exempt]]\npath = \"cli.py\"\nrules = [\"location\"]\nreason = \"  \"\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("empty reason"), "got: {err}");
    }

    #[test]
    fn an_unknown_rule_is_rejected() {
        assert!(parse(
            "[python]\ncoverage = { branch = true, fail_under = 100 }\n\
             [[python.exempt]]\npath = \"cli.py\"\nrules = [\"packaging\"]\nreason = \"x\"\n",
        )
        .is_err());
    }

    #[test]
    fn a_valid_exemption_parses() {
        let config = parse(
            "[python]\ncoverage = { branch = true, fail_under = 100 }\n\
             [[python.exempt]]\npath = \"cli.py\"\nrules = [\"location\", \"coverage\"]\n\
             reason = \"thin launcher\"\n",
        )
        .unwrap();
        let exempt = &config.python.unwrap().exempt;
        assert_eq!(exempt.len(), 1);
        assert_eq!(exempt[0].rules, vec![Rule::Location, Rule::Coverage]);
    }

    /// A throwaway directory tree, removed on drop.
    struct TempTree(std::path::PathBuf);

    impl TempTree {
        fn new(files: &[&str]) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "tc-exempt-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            for rel in files {
                let path = root.join(rel);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, "x = 1\n").unwrap();
            }
            TempTree(root)
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn exemption(path: &str, rules: &[Rule]) -> Exemption {
        Exemption {
            path: path.to_string(),
            rules: rules.to_vec(),
            reason: "deliberate".to_string(),
        }
    }

    #[test]
    fn resolve_keeps_only_the_requested_rule_and_returns_sorted_paths() {
        let tree = TempTree::new(&["cli.py", "pkg/gen.py", "loc_only.py"]);
        let exemptions = [
            exemption("cli.py", &[Rule::Location, Rule::Coverage]),
            exemption("pkg/gen.py", &[Rule::Coverage]),
            exemption("loc_only.py", &[Rule::Location]),
        ];
        let coverage = resolve_exempt(&tree.0, &exemptions, Rule::Coverage).unwrap();
        assert_eq!(
            coverage.into_iter().collect::<Vec<_>>(),
            vec!["cli.py".to_string(), "pkg/gen.py".to_string()],
        );
        let location = resolve_exempt(&tree.0, &exemptions, Rule::Location).unwrap();
        assert_eq!(
            location.into_iter().collect::<Vec<_>>(),
            vec!["cli.py".to_string(), "loc_only.py".to_string()],
        );
    }

    #[test]
    fn a_stale_exempt_path_is_an_error() {
        let tree = TempTree::new(&["cli.py"]);
        let exemptions = [exemption("ghost.py", &[Rule::Location])];
        let err = resolve_exempt(&tree.0, &exemptions, Rule::Location).unwrap_err();
        assert!(err.to_string().contains("matches no file"), "got: {err}");
    }
}
