# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Changed

- **BREAKING: `coverage` / `mutation` exemptions are now line-scoped only** (#226). A
  `[[<language>.exempt]]` entry naming `coverage` or `mutation` must carry a `lines` list; a
  whole-file `rules = ["coverage"]` (or `["mutation"]`) entry — accepted before — is now rejected on
  load, as is mixing a measured-line rule with a whole-file rule in one entry. Migrate each to the
  line-scoped form (or split a combined entry in two). See [MIGRATIONS](./MIGRATIONS.md).
- **BREAKING: SDK measure functions take an `exempt_lines` argument** (#226).
  `mutation::measure_rust` / `measure_typescript` / `measure_python` and
  `patch_coverage::measure{,_typescript,_rust}` gain a trailing
  `exempt_lines: &BTreeMap<String, BTreeSet<u32>>`. Pass an empty map to preserve prior behavior. See
  [MIGRATIONS](./MIGRATIONS.md).
- **A `[<language>].coverage` table is now a partial override** (#216, parent #196). Set only the
  fields you want to change; the rest fall back to the language's default floor — so
  `[typescript].coverage` with just `branches = 90` keeps `lines`/`functions`/`statements` at 100,
  and `[rust].coverage` with just `regions = 90` keeps `lines = 100`. Previously every field was
  required, so a partial table errored (and `[rust].coverage` errored without `lines`). A typo'd key
  is still rejected — only *missing* keys default. No API change (the struct fields are unchanged).

### Fixed

- **`unit mutation --language rust --base` now handles a crate nested in the git repo, and a diff
  that doesn't touch it** (#204 follow-up). The `<base>...HEAD` diff is taken `--relative` to the
  crate, so cargo-mutants' `--in-diff` matches a crate in a subdirectory (the common consumer
  layout) rather than nothing; and a diff with no changed lines under the crate — or one that
  produces no mutants — now reports no survivors instead of erroring with `reading cargo-mutants
  outcomes … the run wrote none`. No API change (`measure_rust`'s signature is unchanged).

### Added

- **`testing-conventions exemptions --base <REF> [--approved]`** (#229) — the exemption-approval
  gate. An exemption turns a blocking rule off for a file, so adding one is a last resort; this
  command makes that true by costing a human greenlight. It diffs the `[[<language>.exempt]]`
  entries between `<REF>` (read with `git show <REF>:<config>`) and the working tree's config and
  exits `1` when the diff **adds or modifies** one, unless `--approved` is passed. Identity is the
  **whole entry** (path + rules + reason + `lines`): adding an entry, lifting an extra rule, widening
  its line scope (#226), or rewording its `reason` all gate; removing an entry or leaving it
  byte-for-byte unchanged is free — so an agent can't quietly broaden an existing exemption (e.g.
  expand a `lines` range). Keying on the diff is the anti-loophole (pre-seeding the base is itself a
  gated diff). `--approved` is the binary human greenlight: with it, added/modified entries pass and
  are echoed as an audit trail; the reusable workflow will set it only when the
  `tc:exemption-approved` label was applied by a reviewer who is **not** the PR author, so the agent
  can't approve its own exemption. On a violation the command steers toward writing a test instead.
  One schema drives all three languages; an unresolvable `<REF>` errors rather than passing as clean.
  New library surface: the `exemptions` module (`exemptions::changed`, `exemptions::ChangedExemption`).
  The **reusable-workflow job** that reads the label (verifying the non-author actor and passing
  `--approved`) is the remaining wiring step, mirroring how `unit mutation` shipped as a command
  (#201–#203) before its workflow job (#204).

- **Line-scoped `coverage` / `mutation` exemptions** (#226). A `coverage` or `mutation`
  `[[<language>.exempt]]` entry now **requires** a `lines` list (`lines = [9, 10, "12-13"]` — single
  line numbers and inclusive `"start-end"` ranges) naming the exact lines it lifts — those two rules
  are **never whole-file**. A determinism guard rejects a listed line that isn't actually failing
  (covered, a killed mutant, or no measured code), and an unlisted failing line still fails. `lines`
  is rejected with a whole-file rule (`colocated-test`, the lints), so the two never share an entry. Whole-tree `unit coverage` recomputes its floor from per-line detail over the
  measured-minus-exempt lines (no coverage tool excludes line *numbers* from the outside); `unit
  coverage --base` lifts the exempt lines from the diff; and `unit mutation` lifts the survivors on
  the listed lines. New public API: `config::{LineSpec, LineScope, resolve_exempt_scoped}`,
  `Exemption::{lines, line_set}`, `coverage::measure_report`,
  `patch_coverage::measure_line_exempt{,_typescript,_rust}`,
  `mutation::{evaluate_scoped, mutated_lines, MutatedLines}`.
- **`unit mutation --language python`** (#203) — the Python arm of the mutation rule, completing
  cross-language parity. Wraps [cosmic-ray](https://github.com/sixty-north/cosmic-ray): a baseline
  check guards the suite, then `init` / `exec` run the mutants and `cosmic-ray dump` is parsed for
  the `survived` outcomes (file + line), feeding the shared `mutation::evaluate` core. Same
  **on-by-default binary gate** as the other arms — any un-exempted survivor fails the run — with
  reasoned `[[python.exempt]] rules = ["mutation"]` entries the only loosening. cosmic-ray has no
  native git-diff scoping, so `--base <REF>` scopes the run to the changed `.py` files and filters
  the survivors to the `<base>...HEAD` changed lines (line granularity, matching the Rust/TS arms).
  All cosmic-ray artifacts (config + session) live in an out-of-tree temp dir. New library surface:
  `mutation::measure_python` and the cosmic-ray dump types. With all three languages at parity,
  the rule is still **not wired into the reusable workflow** — that matrix wiring is the remaining
  step (#199). cosmic-ray + pytest must be installed.

- **`unit mutation --language typescript`** (#202) — the TypeScript arm of the mutation rule,
  parity with the Rust vertical. Wraps [Stryker](https://stryker-mutator.io/): runs the engine,
  reads its `mutation.json` report, and collects the surviving mutants (`Survived` and `NoCoverage`)
  the suite ran but didn't catch, feeding the shared evaluation core (`mutation::evaluate`). Same
  **on-by-default binary gate** as Rust — any un-exempted survivor fails the run — with reasoned
  `[[typescript.exempt]] rules = ["mutation"]` entries the only loosening. Stryker has no native
  git-diff scoping, so `--base <REF>` translates the `<base>...HEAD` changed lines into Stryker
  `--mutate <file>:<line>-<line>` ranges — **line** granularity, matching cargo-mutants' `--in-diff`
  (one called-out asymmetry: under `--base` the ranges replace Stryker's configured `mutate` set,
  filtering test/`.d.ts` files). New library surface: `mutation::measure_typescript`, the shared
  `mutation::evaluate` core, and the Stryker report types. Still **not wired into the reusable
  workflow** — that waits on Python parity (#199). Stryker (`@stryker-mutator/core` and a
  test-runner plugin) must be installed/resolvable.

- **`unit mutation --language rust`** (#201) — the rung above coverage. Wraps
  [cargo-mutants](https://github.com/sourcefrog/cargo-mutants): runs the engine, reads its
  `outcomes.json`, and finds the surviving mutants the suite ran but didn't catch. The gate is
  **binary, not a percentage** (equivalent mutants make a fixed score unreachable) and **on by
  default**: any *un-exempted* surviving mutant fails the run (exit `1`), with no report-only mode.
  The only loosening is a reason-required `[[rust.exempt]] rules = ["mutation"]` entry for an
  equivalent / deliberately-defensive survivor — so a passing run means every survivor was killed
  or explained. `--base <REF>` scopes to the diff via cargo-mutants' `--in-diff`. New library
  surface: the `mutation` module (`measure_rust`, `unexplained_survivors`, `Survivor`, the
  `outcomes.json` types) and `config::Rule::Mutation`. Rust-only for now and **not yet wired into
  the reusable workflow** — it ships per-language and turns on in CI once TypeScript and Python
  reach parity (#199). `cargo-mutants` must be installed.

### Changed

- **BREAKING — Rust coverage now has a zero-config default floor of `lines = 100`** (#206).
  Closing the last gap from the strict-100 default (#194): with no `[rust].coverage` table,
  `unit coverage --language rust` no longer errors asking for one — it enforces a 100% **line**
  floor, matching Python/TypeScript. Two deliberate asymmetries from the other languages, both
  forced by `cargo llvm-cov` on stable and documented in the Defaults reference: there is **no
  branch component** (branch coverage is experimental), and **`regions` is opt-in** (a Rust-only
  sub-line metric, harsher than lines — off unless a config sets it). The reusable workflow now
  fans `unit coverage` over a detected Rust crate whether or not a floor is configured. A
  zero-config Rust crate whose unit suite is below 100% lines will now **fail** where it
  previously had no coverage gate; restore the prior posture with an explicit `[rust].coverage`
  table (lower `lines`, or add a `regions` floor). API: `RustCoverage` gains a `Default` impl, and
  its `regions` field — plus `coverage::RustThresholds.regions` — becomes `Option<u8>` (see
  MIGRATIONS).
- **BREAKING — default coverage floors raised to a strict 100%** (#194). With no
  `[<language>].coverage` table, `unit coverage` now requires 100%: Python `fail_under = 100`
  (branch on), TypeScript `lines`/`branches`/`functions`/`statements` all 100 — up from the #80
  defaults (Python 85; TypeScript 80/75/80/80). The premise is that the exemption system
  (`# pragma: no cover`, reason-required `[[<lang>.exempt]]` entries, the empty/comment-only and
  `.d.ts` auto-exemptions) already carries trivia, so the default enforces "100% of what you didn't
  explicitly exempt." A zero-config build whose unit suite sat between the old floor and 100 will
  now **fail**; restore the previous floor with an explicit `[<language>].coverage` table (see
  MIGRATIONS). Rust's line floor lands separately in #206 (above).
- The private `workflow` guard command is now **hidden from `--help`** (#191). It was
  always undocumented and run only from our own CI; `#[command(hide = true)]` makes that
  explicit. It still runs when invoked directly (hidden, not removed), and the `@v0` drift
  guard — which introspects the in-process command tree — is unaffected. Non-breaking.
- **BREAKING — `unit isolation` renamed to `unit lint`** (#160, part of the #158 CLI
  taxonomy redesign). The unit-suite lint command is now `unit lint`, mirroring
  `integration lint`: each lints its test kind's files for that kind's rules. The rules
  it runs are unchanged — `unmocked-collaborator`, `untyped-mock` (TypeScript),
  `no-out-of-module-call`, `no-out-of-module-import` (Rust), Python
  `unmocked-collaborator` — and so are their ids, so **config and `[[<lang>.exempt]]`
  waivers need no change**. Internally `UnitRule::Isolation` became `UnitRule::Lint`
  (`run_unit_isolation` → `run_unit_lint`); the `isolation` module, the
  `isolation::Language` selector, and every library entry point are untouched.
  Migration: replace `unit isolation` with `unit lint` wherever you invoke it (e.g. the
  reusable `testing-conventions.yml` workflow). (#160)
- **BREAKING — `unit patch-coverage` folded into `unit coverage --base`** (#162, part of the
  #158 CLI taxonomy redesign). The diff-scoped changed-line check is no longer a separate
  command: `unit coverage --language <LANG> --base <REF> [--config <CONFIG>] <PATH>` measures the
  **same configured floor** (`fail_under`/`branch` for Python; the four vitest metrics for
  TypeScript; regions/lines for Rust) over the `<base>...HEAD` diff instead of the whole tree.
  Two behavior changes from the old command: the diff is judged against the configured floor
  rather than an implicit 100% (a diff that clears the floor passes even with an uncovered changed
  line — they coincide only at `fail_under = 100`), and there is no small-diff carve-out (a tiny
  diff below the floor fails like any other). Config and `[[<lang>.exempt]] rules = ["coverage"]`
  waivers are unchanged — both scopes already share the `coverage` rule id. Migration: replace
  `unit patch-coverage --base X` with `unit coverage --base X` wherever you invoke it (the reusable
  `testing-conventions.yml` workflow, CI). (#162)

### Added

- **Patch (changed-line) coverage — Rust** (#136, parent #46). `unit patch-coverage --language rust
  [--base <REF>] [--config <CONFIG>] <PATH>`: the Rust twin of the patch-coverage core (#132), built
  on the Rust coverage rule (#37). Reuses the same `<base>...HEAD` diff machinery — scoped to `.rs`
  sources — and maps the changed lines against `cargo llvm-cov`'s per-line coverage: a changed line
  is *uncovered* when llvm-cov records no execution for it (an LCOV `DA:<line>,0` record). Runs
  `cargo llvm-cov --lcov` with the floor's nested-run hygiene (an out-of-tree target dir and the
  outer coverage env stripped), so it works under the package's own `cargo llvm-cov` job;
  `cargo-llvm-cov` must be installed. A file with a `[rust].coverage` exemption (reusing #32) is
  dropped from the run via `--ignore-filename-regex`, so its changed lines are lifted. As with the
  Rust floor, inline `#[cfg(test)]` code is measured alongside the source (it can't be excluded by
  filename on a stable toolchain). Prints each uncovered line to stderr as `<path>:<line>` and exits
  non-zero. New library API `testing_conventions::patch_coverage::check_rust` and
  `coverage::measure_patch_rust`; the vitest invocation is shared with the floor via
  `run_cargo_llvm_cov`. With Rust landed, `unit patch-coverage` now covers all three languages.
  (#136)
- **Patch (changed-line) coverage — TypeScript** (#135, parent #46). `unit patch-coverage
  --language typescript [--base <REF>] [--config <CONFIG>] <PATH>`: the TypeScript twin of the
  Python patch-coverage core (#132), built on the TypeScript coverage rule (#31). Reuses the same
  `<base>...HEAD` diff machinery — scoped to `.ts` / `.tsx` / `.mts` / `.cts` sources — and maps the
  changed lines against vitest's per-file v8 coverage: a changed line is *uncovered* when it carries
  a statement the suite never executed, or the source of a branch a path of which the suite never
  took (line + branch), exactly mirroring the Python arm's missing-line / missing-branch test. Runs
  `npx vitest` with the `json` reporter and `--coverage.all` (so an untested changed file is seen as
  wholly uncovered); `vitest` and `@vitest/coverage-v8` must be installed under `<PATH>`. A file with
  a `[typescript].coverage` exemption (reusing #32) is excluded from the run, so its changed lines
  are lifted. Prints each uncovered line to stderr as `<path>:<line>` and exits non-zero. New library
  API `testing_conventions::patch_coverage::{check_typescript, uncovered_changed_lines_ts}` and
  `coverage::measure_patch_typescript`. `--language rust` (`cargo llvm-cov`) remains a separate item.
  (#135)
- **Patch (changed-line) coverage — Python** (#132, parent #46). New `unit patch-coverage
  --language python [--base <REF>] [--config <CONFIG>] <PATH>` command: a diff-scoped coverage
  check that every line `<base>...HEAD` adds or modifies is covered by the unit suite. Where
  `unit coverage` measures the whole suite against a floor (#26), this measures only the changed
  lines — failing when a changed, executable line is a
  coverage.py *missing line* or the *source of a branch* the suite never took (line + branch). The
  diff machinery (`git diff --unified=0 <base>...HEAD`) is established here and shared by the
  forthcoming TypeScript / Rust twins; `--base` defaults to `origin/main` (override for another
  base or an explicit range). Non-executable changed lines (comments, blanks) have nothing to
  cover, and a file with a `coverage` exemption (reusing #32) is omitted — so its changed lines are
  lifted, the same waiver the floor honors. **Added** files differ from the co-change rule (#33):
  their new lines *are* subjects (measured via coverage.py `--source`, so an untested new file is
  wholly uncovered). Complementary to `unit colocated-test --base` — co-change enforces that a
  changed source and its colocated test move together; patch coverage enforces that the changed
  lines are exercised. Prints each uncovered line to stderr as `<path>:<line>` and exits non-zero. New
  library API `testing_conventions::patch_coverage::{check, changed_lines, uncovered_changed_lines,
  Uncovered}` and `coverage::{FileCoverage, measure_patch_report}` (plus `CoverageReport` gains a
  `files` map); reuses the `coverage` `config::Rule`. Python only this slice — `--language
  typescript` / `rust` are rejected as separate items. (#132)
- **Commit-scoped `co-change` — `unit colocated-test --base`** (#33, #161). With `--base`,
  `unit colocated-test --language <python|typescript> --base <REF> [--config <CONFIG>] <PATH>`
  adds a diff-scoped check that a source file **modified** (and still holding code) or **deleted**
  between `<base>...HEAD` also changed its colocated test (the #15/#18 pairing — `foo.py` →
  `foo_test.py`, `foo.ts` → `foo.test.ts`), so an edit or removal can't leave the test silently
  stale. It is **additive and opt-in**: `--base` runs co-change *on top of* the tree-wide presence
  check (an orphan source still fails) and has no default, so absent means presence-only. **Added**
  source files are not subjects (brand-new code is the coverage floor's job); a test file, an
  empty/comment-only file, and Python's `conftest.py` are never subjects; and a source with a
  `co-change` exemption needn't co-change. `<base>...HEAD` is the changes this branch introduced
  (what a PR shows), so CI passes the PR base (e.g. `--base origin/main`). `--base --language rust`
  is rejected — Rust units are inline `#[cfg(test)]` in the same file, so a sibling test can't go
  stale (presence without `--base` still supports Rust). New library API
  `testing_conventions::co_change::stale_sources(repo, base, language, exempt)` and a waivable
  `config::Rule` variant `co-change` (`[[<language>.exempt]] rules = ["co-change"]`, reusing #32).
  (#33, #161)
- **Waivers for the remaining Python integration lints** (#123). The reason-required
  `[[python.exempt]]` escape hatch (#32/#102) now covers the last three lints that
  lacked it — `no-monkeypatch` (#49), `no-inline-patch` (#50), and
  `no-environ-mutation` (#51) — so they waive like their sibling `no-constant-patch`
  (#52) and every other rule. Previously their ids weren't `config::Rule` variants, so
  `apply_waivers` could never waive them and the loader rejected
  `rules = ["no-monkeypatch"]` outright. Decision per #3: the waiver is reason-required,
  single-file, auditable, and stale-checked, so an honest hatch doesn't weaken the gate.
  New `config::Rule` variants `NoMonkeypatch` / `NoInlinePatch` / `NoEnvironMutation`
  (with `id()` / `from_id()`). A waived file passes; an un-waived violation still fails;
  a reason-less or stale entry still errors. Example:
  `[[python.exempt]] rules = ["no-inline-patch"]`.
- **Rust unit coverage** — `unit coverage --language rust [--config <CONFIG>] <PATH>` now
  enforces a `cargo llvm-cov` floor on the unit suite (#37), the Rust arm of the coverage rule
  (Python #26 / TypeScript #31). It runs `cargo llvm-cov --json --summary-only` over the crate at
  `<PATH>` and checks the export's **regions** and **lines** totals against `[rust].coverage`
  (`regions` / `lines`) — branch coverage is still experimental, so it isn't enforced — exiting
  non-zero, and naming each metric below its floor, when either falls short. `cargo-llvm-cov` must
  be installed. Files with a `coverage` exemption are dropped from the denominator via
  `--ignore-filename-regex` (`[[rust.exempt]] rules = ["coverage"]`, reusing #32). Two caveats are
  Rust-specific: inline `#[cfg(test)]` units can't be excluded by filename and `#[coverage(off)]`
  is still nightly, so on a stable toolchain the inline test code is measured alongside the source;
  and Rust has **no zero-config default floor** yet (unlike #80's Python/TypeScript defaults), so a
  config without a `[rust].coverage` table errors rather than guessing one. New library API:
  `coverage::{measure_rust, evaluate_rust, parse_llvm_cov_report, RustThresholds, LlvmCovReport,
  LlvmCovData, LlvmCovTotals, LlvmCovMetric}`, sharing the existing `Outcome`. (#37)
- **Python unit isolation — external deps** (#121, slice 3). `unit isolation
  --language python` now also flags an imported, un-mocked **external** collaborator:
  a **third-party** package (any bare import that isn't first-party or stdlib) or an
  **effectful stdlib** module (a conservative set — network / subprocess / process /
  randomness / database / low-level OS: `socket`, `subprocess`, `ssl`, `random`,
  `sqlite3`, …). Pure stdlib (`json`, `dataclasses`, …), the test framework
  (`pytest` / `_pytest` / `mock`; `unittest` is stdlib), `__future__`, and
  `TYPE_CHECKING` imports stay exempt. Dual-nature heads (`os`, `pathlib`,
  `datetime`, `time`, `io`) are excluded from the effectful set — their pure vs.
  effectful use can't be told at the import head, so the clock / filesystem stay
  caught by the patch-by-string convention (a documented non-goal). Same
  `unmocked-collaborator` rule (still waivable via #102), no new `config::Rule`. See
  [`internals/python/isolation.md`](../../internals/python/isolation.md).
- **Python unit isolation** — `unmocked-collaborator` (#42, slice 2). `unit isolation
  --language python <PATH>` now flags a colocated unit test (`*_test.py` / `test_*.py`)
  that **imports a first-party collaborator without mocking it** — a unit test must
  isolate the unit under test. The unit under test (the import whose module's last
  segment matches the test's base name), the test framework (`pytest` / `unittest`),
  pure stdlib, `__future__`, and `TYPE_CHECKING`-guarded imports are never
  collaborators; an import counts as mocked when a `patch("…")` in the file targets a
  matching last dotted segment (catching the consuming-module patch
  `patch("pkg.widget.record")`). First-party is the dist's own package, read from the
  nearest `pyproject.toml` (as in slice 1). Emits the same `unmocked-collaborator`
  rule as TypeScript, so the #102 waiver covers it: `[[python.exempt]] rules =
  ["unmocked-collaborator"]`. Library API: `testing_conventions::lint::find_unit_isolation_violations`.
  Un-mocked third-party / effectful-stdlib imports are a follow-up slice; value-type
  imports and cross-file (`conftest.py`) mocks are documented non-goals. See
  [`internals/python/isolation.md`](../../internals/python/isolation.md).
- **Rust colocated-test** — `unit colocated-test --language rust <PATH>` now checks
  inline-`#[cfg(test)]` **presence** (#40), the Rust arm of the colocated-test rule.
  Rust units are inline `#[cfg(test)]` modules, so a `src` file that defines a
  function with a body but carries no inline `#[cfg(test)]` module is flagged as an
  orphan; module-declaration and type-only files (and `tests/` / `benches/` /
  `examples/` / `build.rs`) are not subjects. Previously this combination errored
  ("Rust units are inline … see `unit isolation`"). Waivable per file via
  `[[rust.exempt]] rules = ["colocated-test"]`. New library function
  `colocated_test::missing_inline_tests(root, exempt)`.
- **Python integration isolation** — `no-first-party-patch` (#42). `integration lint
  --language python` now flags a `patch(...)` whose string target is **first-party**
  — e.g. `patch("ourpkg.mod.fn")` — because an integration test must run first-party
  code for real; only third-party packages and effectful stdlib (`requests.get`,
  `subprocess.run`, `builtins.open`, …) may be patched. The dist's own top-level
  package is read from the nearest `pyproject.toml` `[project].name` (normalized to
  an import name), mirroring how the Rust rule reads `Cargo.toml`; a tree with no
  declared package flags nothing. Waivable like the other lints via
  `[[python.exempt]] rules = ["no-first-party-patch"]` (#32/#102). The
  `patch.object(module, …)` and non-literal-target forms are documented non-goals.
  See [`internals/python/isolation.md`](../../internals/python/isolation.md) for the
  design and the deferred unit direction.
- Config-driven **waivers for the isolation rules** (#102). The escape hatch from
  #32 (a reason-required `[[<lang>.exempt]]` entry, auditable in one diff) now
  lifts the isolation rules too: `unit isolation` gains a `--config` flag (default
  `testing-conventions.toml`, like the other `unit` rules), and both `unit isolation`
  and `integration lint` filter findings against the config. New `config::Rule`
  variants (`no-out-of-module-call`, `no-out-of-module-import`, `no-first-party-double`,
  `unmocked-collaborator`, `untyped-mock`, `no-first-party-mock`) plus
  `Rule::id()` / `Rule::from_id()` and `Config::rust_exemptions()`. A waived file
  passes; an un-waived violation still fails; a reason-less or stale entry still
  errors. Example: `[[rust.exempt]] rules = ["no-out-of-module-call"]`.
- `config` module — `load_config()` parses one TOML config file into the
  in-memory `Config` and self-validates on load: unknown keys, malformed TOML,
  and (#32) any `exempt` entry that names no rule or has an empty reason are
  rejected. Each `[python]` / `[typescript]` / `[rust]` table carries an optional
  `coverage` block and an `exempt` list. Types: `Config`,
  `{Python,TypeScript,Rust}Config`, `{Python,TypeScript,Rust}Coverage`, plus
  `Rule` and `Exemption`; `resolve_exempt()` turns the list into the exempt paths
  for a rule, erroring on any stale (missing) path. (#12, #32)
- `colocated_test` module — `missing_unit_tests(root, language, exempt)` walks a
  directory tree and returns every source file with no colocated unit test
  (sorted), enforcing the README's "Colocated Test" rule per `Language`:
  - **Python** (#15) — `foo.py` → `foo_test.py`; `*_test.py` files are tests.
  - **TypeScript** (#18) — `foo-bar.ts` → `foo-bar.test.ts` across `.ts`/`.tsx`/`.mts`/`.cts`; `*.test.{ts,tsx,mts,cts}` are tests, `*.d.ts`/`*.d.mts`/`*.d.cts` are ignored.
  - Empty/comment-only files carry no logic and are never subjects; files listed in `exempt` are deliberate, reason-required omissions. (#32)
- `unit colocated-test --language <python|typescript> [--config <CONFIG>] <PATH>` CLI —
  runs the check and exits non-zero, printing each orphan. `--config` (optional,
  default `testing-conventions.toml`) supplies the `exempt` list; an absent file
  means no exemptions. (#15, #18, #22, #32, #55)
- `coverage` module + `unit coverage` CLI — enforce the Python coverage floor.
  `unit coverage --language python [--config <CONFIG>] <PATH>` runs the unit suite
  under `coverage.py` (branch on, `*_test.py` plus every `coverage`-exempt path
  omitted from the denominator), then checks the total against the config's
  `[python].coverage` `fail_under` / `branch` and exits non-zero if below. Library
  API: `coverage::{measure, evaluate, parse_report, Thresholds, CoverageReport,
  Outcome}`. (#26, #32)
- `unit coverage` is zero-config by default (#80): with no config file — or a
  config that omits the `[<language>].coverage` table — it enforces the language's
  sane default floor instead of erroring, the same way `unit colocated-test` and
  `integration lint` treat an absent config. The defaults are the reasonable
  floors from `internals/<lang>/testing.md`: Python `branch = true, fail_under = 85`;
  TypeScript `lines = 80, branches = 75, functions = 80, statements = 80`. A
  `[<language>].coverage` table still overrides them, and `exempt` lists still
  apply. Library API: `Config`, `{Python,TypeScript,Rust}Config`, `PythonCoverage`,
  and `TypeScriptCoverage` now implement `Default` (the two coverage structs
  default to those floors). (#80)
- `unit coverage --language typescript` — the TypeScript twin (#31). Runs the unit
  suite under `vitest` v8 coverage (json-summary reporter), excludes `*.test.*`,
  declaration files, and every `coverage`-exempt path from the denominator, and
  enforces the four `[typescript].coverage` thresholds (`lines` / `branches` /
  `functions` / `statements`) independently — exiting non-zero, and naming every
  metric below its floor, if any falls short. `npx vitest` runs the project-local
  toolchain, so `vitest` and `@vitest/coverage-v8` must be installed under the
  scanned path. New library API: `coverage::{measure_typescript,
  evaluate_typescript, parse_vitest_report, TypeScriptThresholds, VitestReport,
  VitestTotals, VitestMetric}`, sharing the existing `Outcome`. (#31)
- `lint` module + `integration lint` CLI — the first deterministic *lint* on test
  code. `integration lint --language python <PATH>` parses each Python test file
  (`*_test.py`, `test_*.py`, `conftest.py`) with `rustpython_parser` and walks the
  AST, exiting non-zero on any violation. Lints:
  **`no-monkeypatch`** (#49) — a test/fixture that declares the `monkeypatch`
  parameter; **`no-inline-patch`** (#50) — a `patch(...)` / `patch.object(...)` /
  `patch.dict(...)` call in a test body (`with patch(...)` or a bare call);
  **`no-environ-mutation`** (#51) — direct `os.environ` mutation (`os.environ[...] = …`,
  `del`, or `update`/`pop`/`setdefault`/`clear`/`popitem`); and
  **`no-constant-patch`** (#52) — patching a module-global UPPER_CASE constant
  (`patch("pkg.config.CACHE_DIR", …)`), waivable per file via a `--config` `exempt`
  entry (`rules = ["no-constant-patch"]`, reusing #32). Library API:
  `testing_conventions::lint::{find_violations, Violation}`. (#48, #49, #50, #51, #52)
- `packaging` module + `packaging` CLI command (foundation) — enforce the README's
  Packaging rule that test files never ship in the built artifact.
  `packaging --language <python|typescript> <PATH>` scans the built artifact at
  `<PATH>` (an already-unpacked wheel or `dist/`) for that language's test-file
  glob — Python `*_test.py`, TypeScript `*.test.*` — and exits non-zero, printing
  each offending path, when any are present. Library API:
  `packaging::scan(root, globs)`, the deterministic walker returning the matching
  files (sorted, `*` wildcards). The per-language *build* step that produces the
  artifact follows in #72 (Python wheel/sdist), #73 (TypeScript `dist`), and #74
  (Rust crate tarball, which also adds `--language rust`). (#41, #70)
- `ts` module + `integration lint --language typescript` — the first TypeScript
  lint, on a new `oxc`-based AST foundation (#43, #75). `integration lint --language
  typescript <PATH>` parses each `*.test.{ts,tsx,mts,cts}` file with `oxc_parser` and
  walks it for **`no-first-party-mock`**: an integration test runs first-party code
  for real, so a `vi.mock()` / `vi.doMock()` whose target is a **first-party** module
  (a relative specifier) is flagged; third-party packages and Node built-ins may still
  be mocked. The shared, resolution-free specifier classifier (`ts::classify` →
  `Origin::{FirstParty, Builtin, ThirdParty}`) is the foundation the unit-isolation
  slices (#76, #77) build on. Library API:
  `testing_conventions::ts::{find_integration_violations, classify, Origin}`. (#43, #75)
- `isolation` module + `unit isolation` CLI — the first deterministic lint on
  *Rust* test code. `unit isolation --language rust <PATH>` parses each `*.rs`
  file under the crate root with `syn` and walks its inline `#[cfg(test)]` modules,
  exiting non-zero on any violation. Detectors (#44): **`no-out-of-module-call`** —
  a call out of a unit test's own module — `crate::…` (another first-party module),
  `super::super::…` (an ancestor), an external crate (named in `Cargo.toml`, with
  `[dev-dependencies]` test tooling excluded), or effectful `std`
  (`fs`/`net`/`process`/`env`/`thread`/`os`, the clock, or real-handle I/O); and
  **`no-out-of-module-import`** — a `use` that pulls a foreign surface into a test
  module: a glob of anything but `super::*`, or a named import rooted at `crate::`,
  an external crate, or effectful `std` (closing the gap where a collaborator is
  imported then called unqualified). A single `super::`, `self`/`Self`, a bare
  unqualified call, and pure `std` (including `std::io::Cursor` and the I/O traits)
  stay in-module. Library API:
  `testing_conventions::isolation::{find_violations, Violation, Language}`. (#44)
- `integration lint --language rust <PATH>` — the Rust arm of `integration lint`,
  enforcing the README "External Dependencies" rule on `tests/` integration crates.
  Detector **`no-first-party-double`** (#44): a `#[double]` (mockall_double) import
  of a first-party item — the crate under test (its `Cargo.toml` `[package].name`)
  or a `path` dependency — which an integration test must run for real. Doubling an
  external crate / `std` is fine, and `crate::` (the test crate itself, not the
  library under test) is not flagged. `integration lint` gains its own
  `IntegrationLintLanguage` (python/typescript/rust), distinct from the file-pairing
  `colocated_test::Language`. Library API:
  `testing_conventions::isolation::find_integration_violations`. (#44)
- `unit isolation --language typescript <PATH>` — the TypeScript arm of `unit
  isolation` (#43, #76), the unit-direction counterpart to slice #75's
  `no-first-party-mock`. A unit test must isolate the unit under test, so every
  runtime import that isn't `vi.mock()` / `vi.doMock()`-ed is flagged
  (**`unmocked-collaborator`**), except three: the **unit under test**
  (`widget.test.ts` → `./widget`), **type-only** imports (`import type …`), and the
  **test runner** (`vitest` / `@vitest/*`). Adds `TypeScript` to `isolation::Language`
  and reuses slice #75's `oxc` parser. Library API:
  `testing_conventions::ts::find_unit_violations`. (#43, #76)
- `unit isolation --language typescript` also enforces **typed** mocks (#43, #77):
  a `vi.mock(spec, factory)` whose factory has no `vi.importActual<…>()` type anchor
  is flagged **`untyped-mock`**, since an untyped double can drift from the source.
  A bare `vi.mock(spec)` (vitest auto-mock, typed from the real module) and a typed
  factory (`vi.importActual<typeof import(spec)>()`) both pass. With this, #43's
  TypeScript isolation is complete (#75 / #76 / #77). (#43, #77)
- `violation` module — the `Violation` type is hoisted here and shared by the
  Python `lint` and Rust `isolation` detectors so the CLI prints every rule the
  same way. `testing_conventions::lint::Violation` remains as a re-export, so the
  prior path still resolves (no break). (#44)
- `packaging` now inspects a **Python wheel** (#72) — point `packaging --language
  python <PATH>` at a built `.whl` and it unpacks the archive and scans for
  `*_test.py`, so a colocated test that leaked into the wheel fails the check.
  `<PATH>` still accepts an already-unpacked directory too. New library API
  `packaging::inspect(path, globs)` unpacks an archive (a `.whl`/`.zip`) or reads
  a directory, then reuses `scan`, returning offenders relative to the artifact
  root. New dependency: `zip` (read-only). (sdist `.tar.gz`, and the TypeScript /
  Rust archives, are still to come.) (#41, #72)
- `packaging` now inspects a **TypeScript npm tarball** (#73) — point `packaging
  --language typescript <PATH>` at a built `.tgz` (from `npm pack`) and it unpacks
  the gzipped tar and scans for `*.test.*`, so a test file that leaked into the
  published package fails the check. `inspect` now also accepts `.tgz` / `.tar.gz`
  (the `.tar.gz` path is reused by the Rust `.crate` in #74 and the Python sdist).
  New dependencies: `flate2` + `tar` (read-only). (#41, #73)
- `packaging --language python` also inspects a built **sdist** (`name-version.tar.gz`),
  not just a wheel — the `.tar.gz` support added in #73 already applies, and dedicated
  sdist fixtures + integration/e2e tests now lock the case in. Test coverage only; no
  behavior change. (#41, #106)
- `packaging --language rust` (#74) — the last packaging language. `packaging` now
  accepts a Cargo `.crate` (from `cargo package`, a gzipped tar) and flags the crate-root
  **`tests/`** directory: `#[cfg(test)]` units compile out of the consumer artifact for
  free, so the only thing to keep out of the source tarball is the integration `tests/`
  (via a Cargo `exclude`). The scanner gains a **directory pattern** (a pattern ending in
  `/`, like `tests/`, matches files under that directory) alongside the file-name globs.
  `Language` (`colocated_test::Language`) gains a `Rust` value, so `--language rust` parses;
  `unit colocated-test` / `unit coverage` reject it (separate items), while `unit isolation`
  / `integration lint` already support Rust through their own enums. (#41, #74)
- `workflow` module + `workflow` CLI command — guard the reusable workflow against
  CLI subcommand drift (#92). `workflow <PATH>` scans a workflow file (or a directory
  of them) for every `testing-conventions …` invocation and checks each one's
  subcommand chain against the binary's own command tree, exiting non-zero — and
  naming each offender as `path:line: no-unknown-subcommand — …` — when a workflow
  invokes a subcommand the binary no longer exposes. This keeps the documented `@v0`
  consumption path from stranding the way it did at 0.0.7 (broken by the #55
  `location` → `colocated-test` rename). Library API:
  `testing_conventions::workflow::{invocations, unknown_subcommands, check, Invocation}`,
  plus `testing_conventions::command()` exposing the binary's clap command tree. (#92)
- `e2e` module + `e2e attest '<command>'` CLI (#17, #67) — run a local e2e suite
  and record that it ran against the current commit. `attest` runs `<command>` via
  the shell (streaming its output), writes an `e2e-attestation.json` recording the
  command, a timestamp, the exit code, and the **commit SHA it was run against**
  (HEAD), then commits that file on top. It writes regardless of the command's exit
  code (force a run, not a pass) and exits `0` once recorded. Library API:
  `testing_conventions::e2e::{attest, Attestation, ATTESTATION_PATH}`. The CI-side
  `e2e verify` gate is a later slice (#68). (#17, #67)
- `e2e verify` CLI (#17, #68) — the CI side of the nudge. Reads the committed
  `e2e-attestation.json` and passes iff its recorded SHA equals the latest *code*
  commit (the newest commit touching any path other than the attestation file);
  otherwise exits non-zero with a run-`attest` hint. Never runs e2e and never
  judges the recorded exit code or output. Library API:
  `testing_conventions::e2e::{verify, Verification}`. (#17, #68)

### Changed

- **BREAKING** — exemptions are config-driven and explicit (#32). There is **no
  automatic name- or shape-based exemption**: `__init__.py`, re-export barrels,
  and launcher shims are all subjects. Only empty/comment-only files (no logic)
  are non-subjects automatically; everything else needs a colocated test or a
  `[[<language>.exempt]]` entry naming the `rules` it lifts (`colocated-test` /
  `coverage`) and a required `reason`. A stale exempt path (file gone) is a hard
  error. Library API: `missing_unit_tests` gains an `exempt` argument and
  `coverage::measure` gains an `omit` argument; `[<language>].coverage` is now
  optional (a config can carry exemptions alone).
- **BREAKING** — the unit-test location check moved from the flat `unit-location`
  subcommand to `unit location`: rules now nest under their test kind (`unit` is a
  command group, `location` its first rule). The `--lang` flag is renamed to
  `--language` and is now **required** — the `python` default is gone, so omitting
  the language is a usage error instead of a silently-empty `python` scan. Migrate
  `unit-location --lang typescript src/` → `unit location --language typescript src/`;
  see [MIGRATIONS](./MIGRATIONS.md#unreleased). (#22)
- **BREAKING** — the unit-test rule was renamed `location` → `colocated-test` so its
  name states what it checks: that every source file has a colocated, matching-named
  unit test. The CLI subcommand `unit location` is now `unit colocated-test`; the
  library module `testing_conventions::location` is now
  `testing_conventions::colocated_test` (its `missing_unit_tests` / `Language` items
  are otherwise unchanged); and the config `exempt` rules value `"location"` is now
  `"colocated-test"` (`rules = ["colocated-test"]`). Migrate
  `unit location --language python src/` → `unit colocated-test --language python src/`;
  see [MIGRATIONS](./MIGRATIONS.md#unreleased). (#55)

### Deprecated

### Removed

### Fixed

- The CLI now prints the full error cause chain (`{err:#}`) instead of only the
  outermost context, so a wrapped failure shows both the *where* and the *why* —
  e.g. a stale exempt entry reports the offending path and config. (#32)
- `unit isolation --language typescript` no longer flags Vitest's options-object
  mock `vi.mock(spec, { spy: true })` as `untyped-mock`. The options form is not a
  factory — it spies on the real module, so the double can't drift, exactly like a
  bare `vi.mock(spec)` auto-mock; only a factory *function* missing a
  `vi.importActual<…>` anchor is flagged. (#111)
- `unit colocated-test` and `unit coverage` (`--language python`) no longer treat
  `conftest.py` as a unit-test subject: it holds pytest fixtures (test support), so
  it is never reported as a missing-test orphan, and it is omitted from the
  coverage denominator alongside `*_test.py`. (#112)
- `integration lint` and `unit isolation` (`--language python`) no longer recognize
  a legacy `test_*.py` as a test file (#145, follow-up to #112). After #112 a unit
  test is `*_test.py` and a `test_*.py` is ordinary source, but the two `lint.rs`
  scans still scanned the legacy prefix — so a `test_*.py` carrying a
  `no-monkeypatch` / `unmocked-collaborator` violation was flagged even though
  `colocated-test` / `coverage` treat it as source. The integration lints now scan
  `*_test.py` + `conftest.py`, and the unit-isolation scan scans `*_test.py`, only.
  No API or rule-id change. (#145)
