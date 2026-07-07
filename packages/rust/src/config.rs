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
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub python: Option<PythonConfig>,
    pub typescript: Option<TypeScriptConfig>,
    pub rust: Option<RustConfig>,
    pub e2e: Option<E2eConfig>,
}

/// The `[python]` table. Both keys are optional, so a repo can configure just
/// coverage, just exemptions, or both. `Default` (no coverage table, no
/// exemptions) backs the zero-config path: an absent `[python]` table means the
/// rule runs against the default floor with nothing exempt (#80).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonConfig {
    pub coverage: Option<PythonCoverage>,
    #[serde(default)]
    pub exempt: Vec<Exemption>,
    /// A build escape hatch (#289, generalized to every language table in #335): a shell
    /// command a build-dependent job runs after toolchain + dependency setup and before it
    /// builds or imports the package, for a build the manifest **structurally can't express**.
    /// `detect` reads it from the package's own config and the workflow jobs run it; the binary
    /// never runs it, but the schema must accept the key so a consumer's config still loads
    /// under `deny_unknown_fields`. Absent (`None`) means no build step. For Python it's the
    /// common case — a PEP 517 backend exposes only sandboxed `build_wheel`/`build_sdist` hooks
    /// with no pre-build shell step.
    pub build_command: Option<String>,
    /// Why the manifest can't express this build — required alongside `build_command` and
    /// validated non-empty in [`Config::validate`], mirroring the exemption-reason bar. An
    /// unreasoned escape hatch can never be a silent pass.
    #[serde(default)]
    pub reason: String,
}

/// The `[e2e]` table (#333). `extra_scope` names a shared source tree beside the
/// package — a native core bound into several language bindings — whose commits
/// join the `e2e verify` freshness walk, and `exclude` carves feature-gated
/// subtrees of it back out. Both are optional lists of repo-relative directory
/// paths, so an absent `[e2e]` table (or one setting just one key) is the
/// zero-config default.
///
/// The binary never acts on these keys — the freshness walk is driven by the
/// `e2e verify --extra-scope` / `--exclude` CLI flags, which `detect` renders
/// from this table and the reusable workflow supplies — but the schema must
/// accept the table so a consumer declaring it still loads the rest of its
/// config under `deny_unknown_fields`, exactly like `[python].build_command`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct E2eConfig {
    #[serde(default)]
    pub extra_scope: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// The `[typescript]` table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeScriptConfig {
    pub coverage: Option<TypeScriptCoverage>,
    #[serde(default)]
    pub exempt: Vec<Exemption>,
    /// The build escape hatch (#335); see [`PythonConfig::build_command`]. For TypeScript this
    /// names a compile-before-`pack` that npm doesn't standardize — `npm pack` runs `prepare` /
    /// `prepack`, but the build script's own name is `build` in one package and `compile` in the
    /// next, so the tool can't derive it.
    pub build_command: Option<String>,
    /// Why the manifest can't express this build — required alongside `build_command`, validated
    /// non-empty in [`Config::validate`].
    #[serde(default)]
    pub reason: String,
}

/// The `[rust]` table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustConfig {
    pub coverage: Option<RustCoverage>,
    /// Cargo features the suite-running Rust rules enable (#266): `unit coverage`
    /// passes them to `cargo llvm-cov` (`--features`) and `unit mutation` forwards
    /// them to cargo-mutants' build/test runs, so `#[cfg(feature = ...)]` code is
    /// compiled, measured, and mutated. Cargo features are Rust's build-system
    /// concept with no Python/TypeScript analog, so the key is deliberately
    /// Rust-only (a documented asymmetry under the parity rule).
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub exempt: Vec<Exemption>,
    /// The build escape hatch (#335); see [`PythonConfig::build_command`]. Rarely needed for Rust
    /// — `cargo` compiles via `build.rs` and packages via `cargo package` from the manifest — so
    /// this is only for a pre-build step neither expresses.
    pub build_command: Option<String>,
    /// Why the manifest can't express this build — required alongside `build_command`, validated
    /// non-empty in [`Config::validate`].
    #[serde(default)]
    pub reason: String,
}

/// `[python].coverage`. A **partial override** — `#[serde(default)]` fills any missing
/// field from [`PythonCoverage::default`], so a table that sets only one threshold keeps
/// our defaults for the rest (#216); `deny_unknown_fields` still rejects a typo'd key.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PythonCoverage {
    pub branch: bool,
    pub fail_under: u8,
}

/// The default Python floor used when coverage isn't configured (#80): branch
/// coverage on, `fail_under = 100` (#194). Strict by default — "100% of what you
/// didn't explicitly exempt" — because the rule already honors `# pragma: no cover`,
/// reason-required `[[python.exempt]]` entries, and the empty/comment-only
/// auto-exemption, so trivia is excluded deliberately rather than by a slack floor.
/// A config `[python].coverage` table lowers it when a project wants headroom.
impl Default for PythonCoverage {
    fn default() -> Self {
        Self {
            branch: true,
            fail_under: 100,
        }
    }
}

/// `[typescript].coverage`. A **partial override** — `#[serde(default)]` fills any
/// missing field from [`TypeScriptCoverage::default`], so a table that sets only one of
/// the four metrics keeps our defaults for the rest (#216); `deny_unknown_fields` still
/// rejects a typo'd key.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TypeScriptCoverage {
    pub lines: u8,
    pub branches: u8,
    pub functions: u8,
    pub statements: u8,
}

/// The default TypeScript floors used when coverage isn't configured (#80): all
/// four metrics at 100 (#194), matching the strict-by-default Python floor. As with
/// Python, "100" means "100% of what you didn't explicitly exempt" — the rule honors
/// reason-required `[[typescript.exempt]]` entries and skips declaration files
/// (`*.d.ts`). A config `[typescript].coverage` table lowers any of the four.
impl Default for TypeScriptCoverage {
    fn default() -> Self {
        Self {
            lines: 100,
            branches: 100,
            functions: 100,
            statements: 100,
        }
    }
}

/// `[rust].coverage`. A **partial override** — `#[serde(default)]` fills any missing
/// field from [`RustCoverage::default`] (`lines = 100`, everything else `None`), so a
/// table that sets only `regions` keeps `lines = 100` (#216); `deny_unknown_fields`
/// still rejects a typo'd key. Three opt-in floors sit alongside `lines` (#267):
/// `regions` (a Rust-only sub-line metric), `functions` (the export's functions
/// total, stable toolchain), and `branch` (adds `--branch` to the run, which
/// instruments only on a nightly toolchain).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RustCoverage {
    pub regions: Option<u8>,
    pub lines: u8,
    pub functions: Option<u8>,
    pub branch: Option<u8>,
}

/// The default Rust floor used when coverage isn't configured (#206): `lines = 100`,
/// matching Python/TypeScript's line-level 100. The other metrics are opt-in (`None`
/// unless a config sets them): `regions` is a Rust-only sub-line metric harsher than
/// lines, `functions` keeps the default line-shaped like Python's, and `branch`
/// requires a nightly toolchain (#267) — so the zero-config floor is lines only. As
/// with Python/TypeScript, "100" means "100% of what you didn't explicitly exempt" —
/// the rule honors reason-required `[[rust.exempt]]` entries. A config
/// `[rust].coverage` table lowers the line floor or adds the opt-in floors.
impl Default for RustCoverage {
    fn default() -> Self {
        Self {
            regions: None,
            lines: 100,
            functions: None,
            branch: None,
        }
    }
}

/// A rule a file can be exempted from (issue #32).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rule {
    /// The unit-test colocated-test check ([`crate::colocated_test`]).
    ColocatedTest,
    /// The unit-test coverage floor ([`crate::coverage`]).
    Coverage,
    /// The commit-scoped `co-change` check ([`crate::co_change`], #33) — a
    /// changed source whose colocated test needn't co-change.
    CoChange,
    /// `integration lint` — a test/fixture takes pytest's `monkeypatch` fixture ([`crate::lint`], #49).
    NoMonkeypatch,
    /// `integration lint` — a `patch(...)` called inline in a Python test body ([`crate::lint`], #50).
    NoInlinePatch,
    /// `integration lint` — direct mutation of `os.environ` in a Python test ([`crate::lint`], #51).
    NoEnvironMutation,
    /// The `no-constant-patch` lint ([`crate::lint`], issue #52).
    NoConstantPatch,
    /// `integration lint` — patching a first-party target in a Python integration test ([`crate::lint`], #42).
    NoFirstPartyPatch,
    /// `unit lint` — a call out of a Rust unit's own module ([`crate::isolation`], #44).
    NoOutOfModuleCall,
    /// `unit lint` — a foreign `use` in a Rust unit test ([`crate::isolation`], #44).
    NoOutOfModuleImport,
    /// `integration lint` — doubling a first-party item in a Rust integration test (#44).
    NoFirstPartyDouble,
    /// `unit lint` — an un-mocked first-party/external import in a TS unit test ([`crate::ts`], #76).
    UnmockedCollaborator,
    /// `unit lint` — a `vi.mock` without a typed anchor in a TS unit test (#77).
    UntypedMock,
    /// `integration lint` — a `vi.mock` of a first-party module in a TS integration test (#75).
    NoFirstPartyMock,
    /// `unit mutation` — a surviving mutant the unit suite didn't catch ([`crate::mutation`], #201).
    Mutation,
}

impl Rule {
    /// Whether a `lines` list may scope this rule (#226). The measured-line rules —
    /// `coverage` and `mutation` — judge individual lines, so an exemption can name
    /// the exact lines it lifts. Every other rule is whole-file (presence, a lint, a
    /// folder convention), so a `lines` key alongside it is a config error.
    pub fn is_line_scopable(self) -> bool {
        matches!(self, Rule::Coverage | Rule::Mutation)
    }

    /// The rule's kebab-case id — the string used in a `Violation` and in a config
    /// `rules` value. Mirrors the `serde(rename_all = "kebab-case")` encoding.
    pub fn id(self) -> &'static str {
        match self {
            Rule::ColocatedTest => "colocated-test",
            Rule::Coverage => "coverage",
            Rule::CoChange => "co-change",
            Rule::NoMonkeypatch => "no-monkeypatch",
            Rule::NoInlinePatch => "no-inline-patch",
            Rule::NoEnvironMutation => "no-environ-mutation",
            Rule::NoConstantPatch => "no-constant-patch",
            Rule::NoFirstPartyPatch => "no-first-party-patch",
            Rule::NoOutOfModuleCall => "no-out-of-module-call",
            Rule::NoOutOfModuleImport => "no-out-of-module-import",
            Rule::NoFirstPartyDouble => "no-first-party-double",
            Rule::UnmockedCollaborator => "unmocked-collaborator",
            Rule::UntypedMock => "untyped-mock",
            Rule::NoFirstPartyMock => "no-first-party-mock",
            Rule::Mutation => "mutation",
        }
    }

    /// The [`Rule`] for a lint id, or `None` for an unknown / non-waivable id.
    pub fn from_id(id: &str) -> Option<Rule> {
        [
            Rule::ColocatedTest,
            Rule::Coverage,
            Rule::CoChange,
            Rule::NoMonkeypatch,
            Rule::NoInlinePatch,
            Rule::NoEnvironMutation,
            Rule::NoConstantPatch,
            Rule::NoFirstPartyPatch,
            Rule::NoOutOfModuleCall,
            Rule::NoOutOfModuleImport,
            Rule::NoFirstPartyDouble,
            Rule::UnmockedCollaborator,
            Rule::UntypedMock,
            Rule::NoFirstPartyMock,
            Rule::Mutation,
        ]
        .into_iter()
        .find(|rule| rule.id() == id)
    }
}

/// One element of an exemption's `lines` list (#226): a single 1-based line, or an
/// inclusive `"start-end"` range.
///
/// Parses from a TOML integer (`9`) or a string range (`"12-13"`). Semantic checks
/// (a line ≥ 1, a range's start ≤ end) live in [`Config::validate`] so the error can
/// name the offending exemption; the deserializer only rejects what isn't a line spec
/// at all (a non-integer, a malformed range).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineSpec {
    /// A single line.
    Single(u32),
    /// An inclusive line range, `start..=end`.
    Range(u32, u32),
}

impl LineSpec {
    /// Parse a string spec: `"12-13"` → a range, `"9"` → a single line. The two parts
    /// of a range are trimmed, so `"12 - 13"` is accepted. A part that isn't a
    /// non-negative integer (or a range with more than one `-`) is an error.
    fn parse_str(s: &str) -> Result<LineSpec, String> {
        let parse = |part: &str| {
            part.trim()
                .parse::<u32>()
                .map_err(|_| format!("`{s}` is not a line number or \"start-end\" range"))
        };
        match s.split_once('-') {
            Some((start, end)) => Ok(LineSpec::Range(parse(start)?, parse(end)?)),
            None => Ok(LineSpec::Single(parse(s)?)),
        }
    }

    /// The lines this spec expands to, pushed into `set`.
    fn extend_into(self, set: &mut BTreeSet<u32>) {
        match self {
            LineSpec::Single(n) => {
                set.insert(n);
            }
            LineSpec::Range(start, end) => {
                for n in start..=end {
                    set.insert(n);
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for LineSpec {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SpecVisitor;
        impl serde::de::Visitor<'_> for SpecVisitor {
            type Value = LineSpec;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a line number or a \"start-end\" range string")
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> std::result::Result<LineSpec, E> {
                u32::try_from(v)
                    .map(LineSpec::Single)
                    .map_err(|_| E::custom(format!("line number {v} is out of range")))
            }

            // TOML integers arrive as i64; a negative line number is nonsense.
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> std::result::Result<LineSpec, E> {
                u64::try_from(v)
                    .map_err(|_| E::custom(format!("line number {v} must be positive")))
                    .and_then(|v| self.visit_u64(v))
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> std::result::Result<LineSpec, E> {
                LineSpec::parse_str(v).map_err(E::custom)
            }
        }
        deserializer.deserialize_any(SpecVisitor)
    }
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
    /// Which rules the exemption lifts (`colocated-test`, `coverage`).
    pub rules: Vec<Rule>,
    /// Lines this exemption is scoped to (#226). Empty (the default, the `lines` key
    /// omitted) is a **whole-file** exemption — today's behavior. A non-empty list
    /// narrows a `coverage` / `mutation` exemption to exactly those lines, guarded so
    /// every listed line must actually be failing.
    #[serde(default)]
    pub lines: Vec<LineSpec>,
    /// Why the omission is deliberate — required, and never empty.
    pub reason: String,
}

impl Exemption {
    /// The 1-based line numbers this exemption is scoped to, with ranges expanded.
    /// Empty when the entry carries no `lines` (a whole-file exemption).
    pub fn line_set(&self) -> BTreeSet<u32> {
        let mut set = BTreeSet::new();
        for spec in &self.lines {
            spec.extend_into(&mut set);
        }
        set
    }
}

/// What an exemption lifts for one file (#226): the whole file, or only specific lines.
///
/// The resolved counterpart of [`Exemption::lines`] — [`resolve_exempt_scoped`] turns
/// each entry into one of these, so the `coverage` / `mutation` rules can apply a
/// file-level omit or a line-level guard uniformly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineScope {
    /// The whole file is exempt (no `lines` key) — today's behavior.
    WholeFile,
    /// Only these 1-based lines are exempt.
    Lines(BTreeSet<u32>),
}

impl LineScope {
    /// Merge two scopes for the same path: a whole-file exemption subsumes any
    /// line-scoped one (the file is wholly lifted either way), otherwise the line sets
    /// union. Lets two entries naming the same file for the same rule combine cleanly.
    fn merged_with(self, other: LineScope) -> LineScope {
        match (self, other) {
            (LineScope::WholeFile, _) | (_, LineScope::WholeFile) => LineScope::WholeFile,
            (LineScope::Lines(mut a), LineScope::Lines(b)) => {
                a.extend(b);
                LineScope::Lines(a)
            }
        }
    }
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
    pub fn exemptions(&self, language: crate::colocated_test::Language) -> &[Exemption] {
        match language {
            crate::colocated_test::Language::Python => {
                self.python.as_ref().map_or(&[], |c| &c.exempt)
            }
            crate::colocated_test::Language::TypeScript => {
                self.typescript.as_ref().map_or(&[], |c| &c.exempt)
            }
            crate::colocated_test::Language::Rust => self.rust_exemptions(),
        }
    }

    /// The `[[rust.exempt]]` list (empty when the table is absent). The named
    /// accessor the Rust isolation rules (#44) waive through; equivalent to
    /// [`Self::exemptions`]`(Language::Rust)`.
    pub fn rust_exemptions(&self) -> &[Exemption] {
        self.rust.as_ref().map_or(&[], |c| &c.exempt)
    }

    /// Reject any `exempt` entry that names no rule or carries an empty reason —
    /// a reasonless or scopeless exemption can never be a silent pass.
    fn validate(&self) -> Result<()> {
        // The `build_command` escape hatch (#289, generalized to every language table in #335) is
        // held to the exemption bar: a reasonless build command can never be a silent pass, so a
        // present command with an empty reason fails to load, in the same style as the exemption
        // reason check below.
        let build_commands = [
            (
                "python",
                self.python.as_ref().map(|c| (&c.build_command, &c.reason)),
            ),
            (
                "typescript",
                self.typescript
                    .as_ref()
                    .map(|c| (&c.build_command, &c.reason)),
            ),
            (
                "rust",
                self.rust.as_ref().map(|c| (&c.build_command, &c.reason)),
            ),
        ];
        for (table, (build_command, reason)) in build_commands
            .into_iter()
            .filter_map(|(t, p)| p.map(|p| (t, p)))
        {
            if build_command.is_some() && reason.trim().is_empty() {
                bail!(
                    "[{table}].build_command has an empty reason — say why the manifest can't \
                     express this build (an ecosystem that standardizes the build needs no \
                     build_command; name only one it structurally can't)"
                );
            }
        }
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
                         `rules = [\"colocated-test\"]` and/or `\"coverage\"`",
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
                // Line-scoping and whole-file exemptions don't mix (#226). The
                // measured-line rules (`coverage` / `mutation`) **require** `lines` —
                // an exemption may not lift a whole file from coverage or mutation, only
                // the exact lines it can prove are failing. The whole-file rules
                // (presence, lints) **reject** `lines`. So an entry is either all
                // line-scopable rules with `lines`, or all whole-file rules without.
                let has_scopable = entry.rules.iter().any(|rule| rule.is_line_scopable());
                let has_whole_file = entry.rules.iter().any(|rule| !rule.is_line_scopable());
                if entry.lines.is_empty() {
                    if has_scopable {
                        let rule = entry.rules.iter().find(|r| r.is_line_scopable()).unwrap();
                        bail!(
                            "[{table}].exempt entry for `{}` names `{}` but lists no `lines` — \
                             a `coverage` / `mutation` exemption must name the exact lines it \
                             covers (whole-file exemptions are for presence / lint rules only)",
                            entry.path,
                            rule.id()
                        );
                    }
                } else {
                    if has_whole_file {
                        let rule = entry.rules.iter().find(|r| !r.is_line_scopable()).unwrap();
                        bail!(
                            "[{table}].exempt entry for `{}` has `lines` alongside rule \
                             `{}` — line-scoped exemptions apply only to `coverage` and \
                             `mutation`; move the whole-file rules to a separate entry",
                            entry.path,
                            rule.id()
                        );
                    }
                    for spec in &entry.lines {
                        let invalid = match spec {
                            LineSpec::Single(n) => *n == 0,
                            LineSpec::Range(start, end) => *start == 0 || start > end,
                        };
                        if invalid {
                            bail!(
                                "[{table}].exempt entry for `{}` has an invalid line spec — \
                                 line numbers are 1-based and a range's start must not exceed \
                                 its end",
                                entry.path
                            );
                        }
                    }
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
    Ok(resolve_exempt_scoped(root, exemptions, rule)?
        .into_keys()
        .collect())
}

/// Resolve the per-file exempt **scope** for `rule` (#226) — whole-file or line-scoped.
///
/// Like [`resolve_exempt`], a stale path is a hard error so the list can't rot. An
/// entry with no `lines` resolves to [`LineScope::WholeFile`] (today's behavior); one
/// with `lines` to [`LineScope::Lines`]. Two entries naming the same file for the same
/// rule merge ([`LineScope::merged_with`]). The `coverage` / `mutation` rules read this
/// to apply a file-level omit or a line-level guard; the file-level rules go through the
/// [`resolve_exempt`] shim above, which keeps only the keys.
pub fn resolve_exempt_scoped(
    root: &Path,
    exemptions: &[Exemption],
    rule: Rule,
) -> Result<std::collections::BTreeMap<String, LineScope>> {
    let mut scopes: std::collections::BTreeMap<String, LineScope> =
        std::collections::BTreeMap::new();
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
        let key = entry.path.replace('\\', "/");
        let scope = if entry.lines.is_empty() {
            LineScope::WholeFile
        } else {
            LineScope::Lines(entry.line_set())
        };
        let merged = match scopes.remove(&key) {
            Some(existing) => existing.merged_with(scope),
            None => scope,
        };
        scopes.insert(key, merged);
    }
    Ok(scopes)
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
             [[python.exempt]]\npath = \"cli.py\"\nrules = [\"colocated-test\"]\nreason = \"  \"\n",
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
    fn default_python_coverage_is_the_strict_floor() {
        // The zero-config floor (#80, #194) is strict by default: branch on, 100.
        // Locked here so it can't silently drift from the Defaults reference.
        assert_eq!(
            PythonCoverage::default(),
            PythonCoverage {
                branch: true,
                fail_under: 100,
            }
        );
    }

    #[test]
    fn default_typescript_coverage_is_the_strict_floor() {
        // The zero-config floor (#80, #194) is strict by default: all four metrics
        // at 100. Locked here so it can't silently drift from the Defaults reference.
        assert_eq!(
            TypeScriptCoverage::default(),
            TypeScriptCoverage {
                lines: 100,
                branches: 100,
                functions: 100,
                statements: 100,
            }
        );
    }

    #[test]
    fn default_rust_coverage_is_the_strict_line_floor() {
        // The zero-config Rust floor (#206) is `lines = 100` — matching Python/TS — with
        // every other metric opt-in (None): `regions` (a Rust-only sub-line metric),
        // `functions`, and `branch` (nightly-only instrumentation, #267). Locked here
        // so it can't silently drift from the Defaults reference.
        assert_eq!(
            RustCoverage::default(),
            RustCoverage {
                regions: None,
                lines: 100,
                functions: None,
                branch: None,
            }
        );
    }

    #[test]
    fn rust_coverage_table_parses_with_regions_omitted() {
        // `regions` is opt-in (#206): a `[rust].coverage` table may set `lines` alone,
        // leaving the region check off.
        let config = parse("[rust]\ncoverage = { lines = 90 }\n").unwrap();
        let coverage = config.rust.unwrap().coverage.unwrap();
        assert_eq!(coverage.regions, None);
        assert_eq!(coverage.lines, 90);
    }

    #[test]
    fn a_python_build_command_with_a_reason_parses() {
        // #289: the escape hatch and its required reason both survive into the [python] table.
        let config = parse(
            "[python]\nbuild_command = \"uv run maturin develop\"\n\
             reason = \"maturin's PEP 517 backend has no pre-build shell hook\"\n",
        )
        .unwrap();
        let python = config.python.unwrap();
        assert_eq!(
            python.build_command.as_deref(),
            Some("uv run maturin develop")
        );
        assert_eq!(
            python.reason,
            "maturin's PEP 517 backend has no pre-build shell hook"
        );
    }

    #[test]
    fn a_python_build_command_with_an_empty_reason_is_rejected() {
        let err = parse("[python]\nbuild_command = \"uv run maturin develop\"\nreason = \"  \"\n")
            .unwrap_err();
        assert!(err.to_string().contains("empty reason"), "got: {err}");
    }

    #[test]
    fn a_python_build_command_with_no_reason_is_rejected() {
        // The `reason` key is absent entirely (serde-defaulted to empty) — still rejected.
        let err = parse("[python]\nbuild_command = \"uv run maturin develop\"\n").unwrap_err();
        assert!(err.to_string().contains("empty reason"), "got: {err}");
    }

    #[test]
    fn a_python_table_without_a_build_command_needs_no_reason() {
        // The reason is required *only* alongside a build_command: a coverage-only [python]
        // table (no build_command) loads with an empty defaulted reason, byte-identical to today.
        let config = parse("[python]\ncoverage = { branch = true, fail_under = 90 }\n").unwrap();
        let python = config.python.unwrap();
        assert!(python.build_command.is_none());
        assert!(python.reason.is_empty());
    }

    #[test]
    fn a_valid_exemption_parses() {
        // A whole-file presence exemption (a launcher shim with no colocated test).
        let config = parse(
            "[python]\ncoverage = { branch = true, fail_under = 100 }\n\
             [[python.exempt]]\npath = \"cli.py\"\nrules = [\"colocated-test\"]\n\
             reason = \"thin launcher\"\n",
        )
        .unwrap();
        let exempt = &config.python.unwrap().exempt;
        assert_eq!(exempt.len(), 1);
        assert_eq!(exempt[0].rules, vec![Rule::ColocatedTest]);
        assert!(exempt[0].lines.is_empty());
    }

    #[test]
    fn exemptions_reads_the_rust_table() {
        let config = parse(
            "[[rust.exempt]]\npath = \"build.rs\"\nrules = [\"no-out-of-module-call\"]\n\
             reason = \"generated\"\n",
        )
        .unwrap();
        let rust = config.exemptions(crate::colocated_test::Language::Rust);
        assert_eq!(rust.len(), 1);
        assert_eq!(rust[0].path, "build.rs");
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
            lines: vec![],
            reason: "deliberate".to_string(),
        }
    }

    #[test]
    fn resolve_keeps_only_the_requested_rule_and_returns_sorted_paths() {
        let tree = TempTree::new(&["cli.py", "pkg/gen.py", "loc_only.py"]);
        let exemptions = [
            exemption("cli.py", &[Rule::ColocatedTest, Rule::Coverage]),
            exemption("pkg/gen.py", &[Rule::Coverage]),
            exemption("loc_only.py", &[Rule::ColocatedTest]),
        ];
        let coverage = resolve_exempt(&tree.0, &exemptions, Rule::Coverage).unwrap();
        assert_eq!(
            coverage.into_iter().collect::<Vec<_>>(),
            vec!["cli.py".to_string(), "pkg/gen.py".to_string()],
        );
        let colocated_test = resolve_exempt(&tree.0, &exemptions, Rule::ColocatedTest).unwrap();
        assert_eq!(
            colocated_test.into_iter().collect::<Vec<_>>(),
            vec!["cli.py".to_string(), "loc_only.py".to_string()],
        );
    }

    #[test]
    fn a_stale_exempt_path_is_an_error() {
        let tree = TempTree::new(&["cli.py"]);
        let exemptions = [exemption("ghost.py", &[Rule::ColocatedTest])];
        let err = resolve_exempt(&tree.0, &exemptions, Rule::ColocatedTest).unwrap_err();
        assert!(err.to_string().contains("matches no file"), "got: {err}");
    }

    // --- line-scoped exemptions (#226) ---

    #[test]
    fn line_specs_parse_from_ints_and_range_strings() {
        // `lines = [9, 10, "12-13"]` — a TOML integer is a single line, a "start-end"
        // string is an inclusive range.
        let config = parse(
            "[[python.exempt]]\npath = \"shim.py\"\nrules = [\"coverage\"]\n\
             lines = [9, 10, \"12-13\"]\nreason = \"dead branch\"\n",
        )
        .unwrap();
        let exempt = &config.python.unwrap().exempt[0];
        assert_eq!(
            exempt.lines,
            vec![
                LineSpec::Single(9),
                LineSpec::Single(10),
                LineSpec::Range(12, 13),
            ]
        );
        // `line_set` expands the range and de-duplicates into a sorted set.
        assert_eq!(
            exempt.line_set().into_iter().collect::<Vec<_>>(),
            vec![9, 10, 12, 13]
        );
    }

    #[test]
    fn a_coverage_exemption_without_lines_is_rejected() {
        // `lines` is required for the measured-line rules (#226): an exemption can't
        // lift a whole file from coverage, only the lines it can prove are uncovered.
        let err = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\nreason = \"gen\"\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("lists no `lines`"), "got: {err}");
    }

    #[test]
    fn a_mutation_exemption_without_lines_is_rejected() {
        let err = parse(
            "[[rust.exempt]]\npath = \"src/lib.rs\"\nrules = [\"mutation\"]\nreason = \"eq\"\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("lists no `lines`"), "got: {err}");
    }

    #[test]
    fn lines_on_a_whole_file_rule_is_rejected() {
        // `colocated-test` is whole-file presence, so a `lines` key alongside it can't
        // mean anything — rejected on load.
        let err = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"colocated-test\", \"coverage\"]\n\
             lines = [3]\nreason = \"shim\"\n",
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("line-scoped exemptions apply only"),
            "got: {err}"
        );
    }

    #[test]
    fn a_zero_line_is_rejected() {
        let err = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\n\
             lines = [0]\nreason = \"x\"\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid line spec"), "got: {err}");
    }

    #[test]
    fn a_reversed_range_is_rejected() {
        let err = parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\n\
             lines = [\"13-12\"]\nreason = \"x\"\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid line spec"), "got: {err}");
    }

    #[test]
    fn a_non_numeric_line_spec_is_a_parse_error() {
        // Not a line number or range at all — rejected by the deserializer.
        assert!(parse(
            "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"coverage\"]\n\
             lines = [\"oops\"]\nreason = \"x\"\n",
        )
        .is_err());
    }

    #[test]
    fn resolve_scoped_distinguishes_whole_file_from_lines() {
        // A `coverage` entry resolves to its lines; a `colocated-test` entry (whole-file
        // presence) to the whole file.
        let tree = TempTree::new(&["barrel.py", "scoped.py"]);
        let exemptions = [
            exemption("barrel.py", &[Rule::ColocatedTest]),
            Exemption {
                path: "scoped.py".to_string(),
                rules: vec![Rule::Coverage],
                lines: vec![LineSpec::Single(2), LineSpec::Range(4, 5)],
                reason: "dead branch".to_string(),
            },
        ];
        let coverage = resolve_exempt_scoped(&tree.0, &exemptions, Rule::Coverage).unwrap();
        assert_eq!(
            coverage["scoped.py"],
            LineScope::Lines([2, 4, 5].into_iter().collect())
        );
        let presence = resolve_exempt_scoped(&tree.0, &exemptions, Rule::ColocatedTest).unwrap();
        assert_eq!(presence["barrel.py"], LineScope::WholeFile);
    }

    #[test]
    fn resolve_scoped_merges_two_entries_for_one_file() {
        // Two line-scoped entries for one file union their lines; two whole-file entries
        // stay whole-file.
        let tree = TempTree::new(&["a.py", "b.py"]);
        let line = |n: u32| Exemption {
            path: "a.py".to_string(),
            rules: vec![Rule::Mutation],
            lines: vec![LineSpec::Single(n)],
            reason: "equivalent mutant".to_string(),
        };
        let mutation = [line(3), line(7)];
        let scopes = resolve_exempt_scoped(&tree.0, &mutation, Rule::Mutation).unwrap();
        assert_eq!(
            scopes["a.py"],
            LineScope::Lines([3, 7].into_iter().collect())
        );

        let presence = [
            exemption("b.py", &[Rule::ColocatedTest]),
            exemption("b.py", &[Rule::ColocatedTest]),
        ];
        let scopes = resolve_exempt_scoped(&tree.0, &presence, Rule::ColocatedTest).unwrap();
        assert_eq!(scopes["b.py"], LineScope::WholeFile);
    }
}
