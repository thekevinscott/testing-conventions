//! The exemption-approval gate's deterministic detection (#229).
//!
//! Exemptions are a **last resort**: a `[[<language>.exempt]]` entry turns a blocking
//! rule off for a file, so adding one should be hard and discouraged — the path of
//! least resistance must be to write the test or isolate the code, not to reach for an
//! exemption. This module is the deterministic engine that makes that true: it flags
//! every exemption a PR **adds or changes**, and the CLI turns that into a non-zero exit
//! that only a human greenlight (a `tc:exemption-approved` label applied by someone who
//! is not the PR author, read by the reusable workflow) can clear. The agent can't clear
//! it itself — that is the entire source of friction.
//!
//! It is the same diff-scoped shape as co-change ([`crate::co_change`]) and changed-line
//! coverage ([`crate::patch_coverage`]): [`changed`] compares the `[[<language>.exempt]]`
//! entries in the working tree's config against those in the config at a base ref
//! (`git show <base>:<config>`) and returns every entry the PR added or modified.
//!
//! **Identity is the whole entry** (path + rules + reason). An entry needs a greenlight
//! unless that *exact* entry already existed at the base, so:
//!   - adding an entry → gated;
//!   - **modifying** an entry — widening it, lifting an extra rule, even rewording the
//!     reason — → gated (an agent can't broaden an existing exemption to slip through);
//!   - removing an entry, or leaving it byte-for-byte unchanged → free.
//!
//! Keying on the diff is the anti-loophole: pre-seeding exemptions on the base branch
//! doesn't dodge the gate, because that base change is itself a gated diff. One config
//! schema drives all three languages, so the gate is language-agnostic.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::config::{self, Config, Exemption};

/// One exemption a PR **added or changed** relative to the base — the unit that needs a
/// human greenlight, as rendered for the CLI. A modified entry (a wider line scope, an
/// extra rule, a reworded reason) is reported just like a brand-new one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChangedExemption {
    /// The config table the entry lives under: `python`, `typescript`, or `rust`.
    pub language: &'static str,
    /// The exempt entry's `path`, as written (with `\` normalized to `/`).
    pub path: String,
    /// The rules the entry lifts, by kebab-case id, sorted (e.g. `["coverage"]`).
    pub rules: Vec<String>,
    /// The 1-based lines a `coverage` / `mutation` entry is scoped to (#226), sorted and
    /// range-expanded. Empty for a whole-file exemption.
    pub lines: Vec<u32>,
}

/// Every exemption the working tree's config at `config_path` **adds or changes** versus
/// the config at `base`, sorted and de-duplicated.
///
/// The working-tree ("after") config is read from disk and validated like every other
/// command loads it (an absent file → no exemptions). The base ("before") config is read
/// with `git show <base>:<config>`, run in the config's directory so a config in a
/// subdirectory resolves too; a config absent at `base` (e.g. one added in this diff)
/// means no exemptions there, so everything present now is newly added. An unresolvable
/// `base` is an error, never a silent "clean".
pub fn changed(base: &str, config_path: &Path) -> Result<Vec<ChangedExemption>> {
    let head = head_config(config_path)?;
    let dir = config_dir(config_path);
    verify_base(dir, base)?;
    let base_config = base_config(dir, base, config_path)?;
    Ok(added_or_modified(&head, &base_config))
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
/// self-guard): the base is historical, and only its exempt *entries* matter here.
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

/// Every entry in `head` whose **whole-entry identity** is absent in `base` — the
/// additions *and* modifications the gate flags. Sorted (the [`BTreeSet`] dedup +
/// `sort`) for deterministic output.
fn added_or_modified(head: &Config, base: &Config) -> Vec<ChangedExemption> {
    let base_keys: BTreeSet<EntryKey> = entries(base).into_iter().map(|(key, _)| key).collect();
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (key, display) in entries(head) {
        // Unchanged (identical entry already at base) → free. A first sighting of a
        // new-or-modified identity → needs a greenlight (dedup repeated identical entries).
        if base_keys.contains(&key) || !seen.insert(key) {
            continue;
        }
        out.push(display);
    }
    out.sort();
    out
}

/// A whole-entry identity: `(language, path, sorted rule ids, line set, reason)`. Two
/// entries are "the same exemption" only when all five match, so any edit — line scope
/// (#226), rules, or the reason prose — is a *different* identity and therefore a change
/// the gate flags. The line set is range-expanded and sorted, so `[12, 13]` and
/// `["12-13"]` are the same scope, but **widening** it (e.g. to `"12-200"`) is a change.
type EntryKey = (&'static str, String, Vec<String>, BTreeSet<u32>, String);

/// Each `[[<language>.exempt]]` entry in `config`, paired as `(identity, display)`.
fn entries(config: &Config) -> Vec<(EntryKey, ChangedExemption)> {
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
    let mut out = Vec::new();
    for (language, exempt) in tables {
        for entry in exempt {
            let path = entry.path.replace('\\', "/");
            // Sort + dedup the rule ids so reordering `rules = [...]` isn't seen as a
            // change, while adding or removing one is.
            let mut rules: Vec<String> = entry.rules.iter().map(|r| r.id().to_string()).collect();
            rules.sort();
            rules.dedup();
            // The range-expanded, sorted line set (#226): canonical, so re-spelling the
            // same lines isn't a change, but widening the scope is.
            let line_set = entry.line_set();
            let key: EntryKey = (
                language,
                path.clone(),
                rules.clone(),
                line_set.clone(),
                entry.reason.clone(),
            );
            out.push((
                key,
                ChangedExemption {
                    language,
                    path,
                    rules,
                    lines: line_set.into_iter().collect(),
                },
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse a config from TOML (lenient, like the base side) for the pure unit tests.
    fn parse(toml_src: &str) -> Config {
        toml::from_str(toml_src).expect("valid test config")
    }

    fn changed_of(head: &Config, base: &Config) -> Vec<ChangedExemption> {
        added_or_modified(head, base)
    }

    #[test]
    fn an_added_entry_is_reported() {
        let base = Config::default();
        let head = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\nreason = \"shim\"\n",
        );
        assert_eq!(
            changed_of(&head, &base),
            vec![ChangedExemption {
                language: "python",
                path: "cli.py".to_string(),
                rules: vec!["coverage".to_string()],
                lines: vec![],
            }]
        );
    }

    #[test]
    fn an_unchanged_entry_is_not_reported() {
        let toml =
            "[[rust.exempt]]\npath = \"build.rs\"\nrules = [\"coverage\"]\nreason = \"gen\"\n";
        assert!(changed_of(&parse(toml), &parse(toml)).is_empty());
    }

    #[test]
    fn a_removed_entry_is_not_reported() {
        let base = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\nreason = \"shim\"\n",
        );
        assert!(changed_of(&Config::default(), &base).is_empty());
    }

    #[test]
    fn an_extra_rule_on_an_existing_entry_is_reported() {
        // Two whole-file rules (so the mix is valid under #226); lifting an extra rule
        // changes the entry's identity, so it gates.
        let base = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"colocated-test\"]\nreason = \"shim\"\n",
        );
        let head = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"colocated-test\", \"co-change\"]\nreason = \"shim\"\n",
        );
        assert_eq!(
            changed_of(&head, &base),
            vec![ChangedExemption {
                language: "python",
                path: "cli.py".to_string(),
                rules: vec!["co-change".to_string(), "colocated-test".to_string()],
                lines: vec![],
            }]
        );
    }

    #[test]
    fn widening_a_line_scope_is_reported() {
        // The #226 hole: an agent broadens an existing line-scoped exemption. The line
        // set is part of the identity, so widening it gates.
        let base = parse(
            "[[python.exempt]]\npath = \"cfg.py\"\nrules = [\"coverage\"]\nlines = [\"12-13\"]\nreason = \"x\"\n",
        );
        let head = parse(
            "[[python.exempt]]\npath = \"cfg.py\"\nrules = [\"coverage\"]\nlines = [\"12-200\"]\nreason = \"x\"\n",
        );
        let changed = changed_of(&head, &base);
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].lines, (12..=200).collect::<Vec<u32>>());
    }

    #[test]
    fn an_equivalent_line_spec_is_not_a_change() {
        // The line set is range-expanded, so `[12, 13]` and `["12-13"]` are the same scope.
        let base = parse(
            "[[python.exempt]]\npath = \"cfg.py\"\nrules = [\"coverage\"]\nlines = [12, 13]\nreason = \"x\"\n",
        );
        let head = parse(
            "[[python.exempt]]\npath = \"cfg.py\"\nrules = [\"coverage\"]\nlines = [\"12-13\"]\nreason = \"x\"\n",
        );
        assert!(changed_of(&head, &base).is_empty());
    }

    #[test]
    fn a_reason_only_edit_is_reported() {
        // Modifying an existing entry gates — even a reworded reason — so an agent can't
        // quietly broaden the justification (or, later, the scope) of an exemption.
        let base = parse(
            "[[typescript.exempt]]\npath = \"index.ts\"\nrules = [\"colocated-test\"]\nreason = \"old\"\n",
        );
        let head = parse(
            "[[typescript.exempt]]\npath = \"index.ts\"\nrules = [\"colocated-test\"]\nreason = \"new, more thorough\"\n",
        );
        assert_eq!(changed_of(&head, &base).len(), 1);
    }

    #[test]
    fn reordering_rules_is_not_a_change() {
        let base = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\", \"colocated-test\"]\nreason = \"shim\"\n",
        );
        let head = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"colocated-test\", \"coverage\"]\nreason = \"shim\"\n",
        );
        assert!(changed_of(&head, &base).is_empty());
    }

    #[test]
    fn changes_across_languages_are_sorted() {
        let base = Config::default();
        let head = parse(
            "[[rust.exempt]]\npath = \"build.rs\"\nrules = [\"coverage\"]\nreason = \"gen\"\n\
             [[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\nreason = \"shim\"\n",
        );
        let langs: Vec<_> = changed_of(&head, &base)
            .into_iter()
            .map(|c| c.language)
            .collect();
        // ChangedExemption sorts by language first: "python" before "rust".
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
