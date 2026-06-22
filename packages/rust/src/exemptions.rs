//! The exemption-approval gate's deterministic detection (#229).
//!
//! A reason-required `[[<language>.exempt]]` entry keeps an exemption *honest*, but
//! it's still cheap to add — an agent (or a hurried human) can write a plausible
//! `reason` and slip one through. So adding a *new* exemption should cost a deliberate
//! human greenlight. This module is the deterministic half of that gate: the same
//! diff-scoped shape as co-change ([`crate::co_change`]) and changed-line coverage
//! ([`crate::patch_coverage`]).
//!
//! [`newly_added`] compares the `[[<language>.exempt]]` entries in the working tree's
//! config against those in the config at a base ref (`git show <base>:<config>`), and
//! returns every exemption the diff **added**. The CLI turns a non-empty result into a
//! non-zero exit, and the human greenlight — a reviewer applying a `tc:exemption-approved`
//! PR label, read by the reusable workflow — rides on that exit code.
//!
//! The unit of detection is one *(language, path, rule)* triple — each entry expands to
//! one per rule it lifts — so:
//!   - adding an entry, or lifting an **extra** rule on an existing entry, is *new*;
//!   - removing or keeping an entry is not;
//!   - editing only the `reason` is not (the gate keys on what is lifted, not the prose).
//!
//! Keying on *newly-added* units is the anti-loophole: pre-seeding exemptions on the
//! base branch doesn't dodge the gate, because that base change is itself a gated diff.
//! One config schema drives all three languages, so the gate is language-agnostic.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::config::{self, Config, Exemption};

/// One newly-added exemption — a *(language, path, rule)* the diff lifted that the base
/// did not — printed by the CLI and the unit the gate keys on.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddedExemption {
    /// The config table the entry lives under: `python`, `typescript`, or `rust`.
    pub language: &'static str,
    /// The exempt entry's `path`, as written (with `\` normalized to `/`).
    pub path: String,
    /// The rule the entry lifts, by its kebab-case id (e.g. `coverage`).
    pub rule: &'static str,
}

/// Every exemption newly added between the config at `base` and the working tree's
/// config at `config_path`, sorted and de-duplicated.
///
/// The working-tree ("after") config is read from disk and validated like every other
/// command loads it (an absent file → no exemptions). The base ("before") config is read
/// with `git show <base>:<config>`, run in the config's directory so a config in a
/// subdirectory resolves too; a config absent at `base` (e.g. one added in this diff)
/// means no exemptions there, so everything present now is newly added. An unresolvable
/// `base` is an error, never a silent "clean".
pub fn newly_added(base: &str, config_path: &Path) -> Result<Vec<AddedExemption>> {
    let head = head_config(config_path)?;
    let dir = config_dir(config_path);
    verify_base(dir, base)?;
    let base_config = base_config(dir, base, config_path)?;
    Ok(added(&head, &base_config))
}

/// The working tree's config at `config_path`, validated (an absent file → an empty
/// [`Config`], the zero-config drop-in). Reuses [`config::load_config`], so a malformed
/// or reason-less config surfaces here as it would for any other command.
fn head_config(config_path: &Path) -> Result<Config> {
    if config_path.exists() {
        config::load_config(config_path)
    } else {
        Ok(Config::default())
    }
}

/// The directory `git show` runs in: the config's parent, or `.` when the path is a bare
/// file name (so `git show <base>:./<file>` resolves relative to it).
fn config_dir(config_path: &Path) -> &Path {
    match config_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

/// Fail unless `base` resolves to a commit in `dir`'s repo — so an unresolvable ref is an
/// error rather than passing as "no exemptions added".
fn verify_base(dir: &Path, base: &str) -> Result<()> {
    let spec = format!("{base}^{{commit}}");
    let out = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--verify", "--quiet", &spec])
        .output()
        .with_context(|| format!("running `git rev-parse {spec}` in `{}`", dir.display()))?;
    if !out.status.success() {
        bail!(
            "base ref `{base}` is not a resolvable commit in `{}` — pass a ref reachable from \
             here (in CI, the PR's base, e.g. `origin/main`)",
            dir.display()
        );
    }
    Ok(())
}

/// The config committed at `base`, via `git show <base>:./<file>`. A config absent at
/// `base` (the show fails) means no exemptions there — so a newly-added config file can't
/// smuggle exemptions in. Parsed leniently (the schema, without the reason-required
/// self-guard): the base is historical, and only its exempt *sets* matter here.
fn base_config(dir: &Path, base: &str, config_path: &Path) -> Result<Config> {
    let file = config_path
        .file_name()
        .with_context(|| format!("config path `{}` has no file name", config_path.display()))?;
    let spec = format!("{base}:./{}", Path::new(file).display());
    let out = Command::new("git")
        .current_dir(dir)
        .args(["show", &spec])
        .output()
        .with_context(|| format!("running `git show {spec}` in `{}`", dir.display()))?;
    if !out.status.success() {
        // Absent at base (e.g. the config file is new in this diff) → nothing exempt there.
        return Ok(Config::default());
    }
    let contents = String::from_utf8_lossy(&out.stdout);
    toml::from_str(&contents).with_context(|| format!("parsing the base config from `{spec}`"))
}

/// The exemptions in `head` whose *(language, path, rule)* unit is absent in `base` —
/// the additions the gate flags. Sorted (the [`BTreeSet`] iteration order) for
/// deterministic output.
fn added(head: &Config, base: &Config) -> Vec<AddedExemption> {
    let base_units = lifted(base);
    lifted(head)
        .into_iter()
        .filter(|unit| !base_units.contains(unit))
        .map(|(language, path, rule)| AddedExemption {
            language,
            path,
            rule,
        })
        .collect()
}

/// Every *(language, path, rule)* a config lifts — each `[[<language>.exempt]]` entry
/// expanded to one unit per rule in its `rules`.
fn lifted(config: &Config) -> BTreeSet<(&'static str, String, &'static str)> {
    let tables: [(&'static str, &[Exemption]); 3] = [
        (
            "python",
            config
                .python
                .as_ref()
                .map_or(&[][..], |c| c.exempt.as_slice()),
        ),
        (
            "typescript",
            config
                .typescript
                .as_ref()
                .map_or(&[][..], |c| c.exempt.as_slice()),
        ),
        (
            "rust",
            config
                .rust
                .as_ref()
                .map_or(&[][..], |c| c.exempt.as_slice()),
        ),
    ];
    let mut units = BTreeSet::new();
    for (language, exempt) in tables {
        for entry in exempt {
            let path = entry.path.replace('\\', "/");
            for rule in &entry.rules {
                units.insert((language, path.clone(), rule.id()));
            }
        }
    }
    units
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse a config from TOML (lenient, like the base side) for the pure unit tests.
    fn parse(toml_src: &str) -> Config {
        toml::from_str(toml_src).expect("valid test config")
    }

    const REASON: &str = "reason = \"deliberate\"\n";

    #[test]
    fn an_added_entry_is_reported() {
        let base = Config::default();
        let head = parse(&format!(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\n{REASON}"
        ));
        assert_eq!(
            added(&head, &base),
            vec![AddedExemption {
                language: "python",
                path: "cli.py".to_string(),
                rule: "coverage",
            }]
        );
    }

    #[test]
    fn an_unchanged_entry_is_not_reported() {
        let toml =
            format!("[[rust.exempt]]\npath = \"build.rs\"\nrules = [\"coverage\"]\n{REASON}");
        assert!(added(&parse(&toml), &parse(&toml)).is_empty());
    }

    #[test]
    fn a_removed_entry_is_not_reported() {
        let base = parse(&format!(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\n{REASON}"
        ));
        let head = Config::default();
        assert!(added(&head, &base).is_empty());
    }

    #[test]
    fn an_extra_rule_on_an_existing_entry_is_reported() {
        let base = parse(&format!(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"colocated-test\"]\n{REASON}"
        ));
        let head = parse(&format!(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"colocated-test\", \"coverage\"]\n{REASON}"
        ));
        // Only the newly-lifted (cli.py, coverage) unit is flagged.
        assert_eq!(
            added(&head, &base),
            vec![AddedExemption {
                language: "python",
                path: "cli.py".to_string(),
                rule: "coverage",
            }]
        );
    }

    #[test]
    fn a_reason_only_change_is_not_reported() {
        let base = parse(
            "[[typescript.exempt]]\npath = \"index.ts\"\nrules = [\"colocated-test\"]\nreason = \"old\"\n",
        );
        let head = parse(
            "[[typescript.exempt]]\npath = \"index.ts\"\nrules = [\"colocated-test\"]\nreason = \"new and improved\"\n",
        );
        assert!(added(&head, &base).is_empty());
    }

    #[test]
    fn additions_across_languages_are_sorted() {
        let base = Config::default();
        let head = parse(&format!(
            "[[rust.exempt]]\npath = \"build.rs\"\nrules = [\"coverage\"]\n{REASON}\
             [[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\n{REASON}"
        ));
        let langs: Vec<_> = added(&head, &base)
            .into_iter()
            .map(|a| a.language)
            .collect();
        // BTreeSet order: "python" sorts before "rust".
        assert_eq!(langs, vec!["python", "rust"]);
    }

    #[test]
    fn config_dir_falls_back_to_dot_for_a_bare_file_name() {
        assert_eq!(
            config_dir(Path::new("testing-conventions.toml")),
            Path::new(".")
        );
        assert_eq!(
            config_dir(Path::new("sub/dir/tc.toml")),
            Path::new("sub/dir")
        );
    }
}
