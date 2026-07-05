# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Changed

- **BREAKING: the suite-executing jobs (`unit coverage`, changed-line coverage, `mutation`)
  install, provision, and build at the derived package root, not the checkout root** (#278, #279,
  epic #276, building on #277's `package_root`/`ts_package_manager`/`python_env`/`provision_rust`
  primitive). All three jobs now install TypeScript dependencies — `npm ci` or `pnpm install
  --frozen-lockfile`, picked from detect's `ts_package_manager` — and provision the Python
  environment — `pip` unchanged; `uv` runs `uv sync` then installs the adapter wheel and pytest
  into the project's own venv — at `needs.detect.outputs.package_root`, and `build_command` runs
  there too. So a per-package-lockfile monorepo TS package (its own `package-lock.json`, no root
  manifest) no longer hits `ERR_PNPM_NO_PKG_MANIFEST`, a `uv`-managed Python package's own
  dependencies are installed before cosmic-ray's spawned pytest needs them, and `build_command` no
  longer needs a smuggled-in `cd`. Rust toolchain provisioning (`rust_toolchain` / detect's
  `provision_rust`) also moves ahead of the language-env setup and caches `target` under the
  package root, since `uv sync` and an npm `prepare` script may themselves compile a Rust core. The
  Rust language arm itself is unchanged — cargo-mutants already runs from the scan root and cargo
  walks up to the crate. See [MIGRATIONS](./MIGRATIONS.md).

- **`unit coverage --language typescript` no longer discards vitest's own default coverage
  excludes** (#290). Passing any `--coverage.exclude` to vitest replaces its built-in default
  list rather than extending it; the rule always passed its own test-file/declaration-file
  excludes, silently dropping vitest's defaults — which already exclude build-tool config files
  (`vitest.config.ts`, `eslint.config.ts`, …), `dist/`, and `node_modules/`. A package whose
  scanned root contained one of those (any monorepo package whose own `vitest.config.ts` sits
  alongside `src/`, since a per-package call scans the whole package root) had it counted as
  0%-covered "source," under-reporting real coverage. The rule now resolves the project's own
  installed vitest's `coverageConfigDefaults.exclude` live (rather than hand-maintaining a list of
  ecosystem config-file conventions) and passes that alongside the config-driven `coverage`
  exemptions. A previously-affected consumer's reported percentage goes *up*; an unaffected one
  (no config file inside the scanned tree) is unchanged. No API or config change.

- **`packaging` discovers `dist/` at the derived package root, not the checkout root** (#280).
  A per-package `uses:` call now inspects its own package's `dist/`; a repo-root `dist/` counts
  only for a call whose derived package root IS the repo root. Every single-package consumer's
  derived package root is the checkout root, so this is byte-identical for them. `packaging_artifact`
  is untouched. Additive/non-breaking — a monorepo consumer that was silently skipped (never
  failed) for a per-package `dist/` the workflow couldn't see now gets it inspected.

- **`install` block points at the reorganized docs** (#353). The managed `AGENTS.md` block's tail
  links the docs site and the machine-readable contract (`llms.txt`); the pointer to the removed
  CLI guide page is gone. Re-running `install` refreshes an existing block in place (the begin
  marker's content hash advances with the content).

### Fixed

- **`e2e-verify` checks out the PR's head commit, not the ephemeral merge ref** (epic #276
  follow-up). On a `pull_request` event, `actions/checkout`'s default ref is the synthetic merge
  commit GitHub rebuilds on every push to the base or the PR — a commit an e2e attestation (which
  names a real, attested code commit) can never match. Once a PR's base has moved since the
  attestation, `git log`'s pathspec walk over the merge ref resolves the merge commit itself as
  "latest," so a genuinely fresh attestation reports stale for reasons that have nothing to do
  with the package's own code. The job now checks out `github.event.pull_request.head.sha`,
  falling back to `github.sha` for a non-PR trigger. No API or config change.

### Added

- **The coverage jobs auto-derive the TypeScript package manager, the Python environment model,
  and Rust auto-provisioning from the package manifest** (#278, epic #276). TS installs run
  `pnpm install --frozen-lockfile` or `npm ci` per `needs.detect.outputs.ts_package_manager`
  (npm joins pnpm); a Python package with its own `[project]` table (`python_env == 'uv'`) is
  installed with `uv sync` — building/installing the project itself, so a maturin package's
  native module compiles with no `build_command` — with `coverage`/`pytest` layered on and the
  venv's `bin` put on `PATH`; a plain `pip`-based package is unchanged. `rust_toolchain`'s cache
  and provisioning now also fire automatically when the package manifest declares a Rust-compiling
  build (`needs.detect.outputs.provision_rust`), with `rust_toolchain` remaining as a manual
  override. See [MIGRATIONS](./MIGRATIONS.md).
- **`e2e verify [path]`** (#281). `e2e verify` takes an optional directory argument (default: the
  current directory) whose committed `e2e-attestation.json` is checked — `testing-conventions e2e
  verify packages/widget` behaves identically to running `e2e verify` with `packages/widget` as
  the current directory, so a monorepo package's attestation can be verified without a `cd`. No
  argument is byte-identical to today. `detect`'s `e2e_attestation` flag (internal to this repo's
  own CI, not shipped) moves with it: it now looks for the attestation at the package root rather
  than the checkout root.

- **`e2e verify [path] --scope <dir>`** (#294). `e2e verify` takes an optional `--scope` flag
  narrowing the "latest code commit" freshness walk to `<dir>`, independently of `path` (where the
  attestation file lives) — default (omitted) is `path` itself, byte-identical to today. A new
  `e2e::verify_scoped(repo, scope)` function backs it; `e2e::verify(repo)` is now defined as
  `verify_scoped(repo, repo)`. Lets a caller whose attestation sits at a package root that also
  holds `tests/`, docs, or config outside its actual source directory scope freshness to just that
  source directory, instead of the whole package root.

- **`install`** (#232). Writes the testing contract into the repository's `AGENTS.md` as a
  marker-delimited, hash-versioned block (the beads `bd init` pattern), so a coding agent learns
  the contract before writing code. Idempotent: re-running refreshes the owned region; everything
  outside the markers is untouched. Refuses a symlinked target; writes atomically.

### Added

- **`functions` and `branch` floors on `[rust.coverage]`** (#267). Two opt-in floors join
  `regions`: `functions` gates the export's functions total on the stable toolchain, and
  `branch` gates the branches total — the run adds `--branch`, which instruments only on a
  nightly toolchain (pin one in the crate's `rust-toolchain.toml` with `llvm-tools-preview`,
  or run under `cargo +nightly`; on stable the run errors naming the requirement). A crate
  with no branch points clears any `branch` floor vacuously. The zero-config default
  (`lines = 100`) is unchanged. A consumer replacing a bespoke cargo-llvm-cov gate can now
  migrate its functions/branch dimensions instead of dropping them.
- **`[rust] features` — cargo-feature passthrough for the suite-running Rust rules** (#266).
  The config's `[rust]` table takes a `features` list that `unit coverage` (whole-tree and
  `--base`) passes to `cargo llvm-cov` as `--features`, and `unit mutation` forwards to
  cargo-mutants' build/test runs — so `#[cfg(feature = ...)]` code is compiled, measured, and
  mutated. Cargo features are Rust's build-system concept with no Python/TypeScript analog, so
  the key is deliberately Rust-only (a documented asymmetry under the parity rule).

### Changed

- **BREAKING: `RustCoverage` / `RustThresholds` gain `functions` and `branch` fields** (#267).
  Both structs have public fields, so struct literals need the two new `Option<u8>` fields
  (`None` preserves prior behavior); `LlvmCovTotals` likewise gains `functions` and
  `branches`. See [MIGRATIONS](./MIGRATIONS.md).
- **BREAKING: the Rust SDK measure functions take a `features` argument** (#266).
  `coverage::measure_rust`, `patch_coverage::measure_rust` / `measure_line_exempt_rust`,
  `coverage::measure_patch_rust_detail`, and `mutation::measure_rust` gain a trailing
  `features: &[String]`. Pass `&[]` to preserve prior behavior. See
  [MIGRATIONS](./MIGRATIONS.md).
- **BREAKING: `unit coverage --language rust` measures the unit suite only** (#265). The Rust arm
  now passes `--lib` to `cargo llvm-cov`, scoping the floor to the library target and its inline
  `#[cfg(test)]` modules — the tool's definition of a Rust unit, and the same unit-only slice the
  Python and TypeScript arms measure. Before, cargo-llvm-cov's default ran every test target, so
  integration tests under `tests/` padded the "unit" number. The diff-scoped floor
  (`unit coverage --base`) shares the run, so it gets the same scoping. Reported percentages drop
  for any crate whose integration tier reached code the unit suite misses; re-fit the
  `[rust].coverage` floor to the honest unit-only number. See [MIGRATIONS](./MIGRATIONS.md).
- **`unit mutation --language rust` provisions cargo-mutants itself** (#242, epic #239). The Rust arm
  no longer requires cargo-mutants to be pre-installed: on first use it runs a pinned `cargo install
  cargo-mutants --locked --version <X>` into the tool's own cache directory and invokes the binary
  from there, so a direct `testing-conventions unit mutation --language rust` works with only a cargo
  toolchain present — parity with the TS/Python arms, which resolve their engines from the npm/wheel
  install. cargo ships no library form of the engine, so the tool drives the installed binary (the
  one unavoidable asymmetry from the in-process TS/Python adapters, called out per the
  cross-language-parity rule). No SDK change — `measure_rust`'s signature is unchanged. (The reusable
  workflow and selftest run the *published* CLI, which gains provisioning only once this ships, so
  their `Install cargo-mutants` step stays until then; it's removed in a follow-up once the
  provisioning binary is released.)
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
- **BREAKING: `unit mutation --language typescript` bundles and drives Stryker through a Node
  adapter** (#246, epic #239). The TS arm now spawns a Node adapter shipped with the npm package,
  which drives Stryker through its own Node API and emits the normalized `NormalizedMutant` schema
  (#239) the gate consumes; the tool drives the engine, and the project supplies its own test runner
  (vitest), exactly as cargo-mutants needs a buildable crate and cosmic-ray needs pytest. The npm
  `testing-conventions` launcher passes the adapter's path to the binary as a `--ts-mutation-adapter`
  argument on a `unit mutation` invocation; the SDK `measure_typescript` takes it as a trailing
  `adapter: &Path`. See [MIGRATIONS](./MIGRATIONS.md).
- **`unit mutation --language python` drives cosmic-ray in-process through a bundled adapter** (#248,
  epic #239). The Python arm now spawns a Python adapter shipped in the wheel (`python3 -m
  testing_conventions.mutation.main`) that drives cosmic-ray via its `WorkDB` library API and emits
  the normalized `NormalizedMutant` schema (#239) the gate consumes — replacing the `cosmic-ray` CLI
  orchestration (baseline/init/exec/dump spawns + JSONL dump parse). The tool drives the engine; the
  project supplies its own test runner (pytest). maturin ships the binary directly as the wheel's
  script, so — unlike the TS arm's launcher-injected path — the binary resolves the adapter as an
  installed module (from the wheel's site-packages; the diff-scoped run passes the changed `.py`
  files as `--module` and the core filters survivors to the changed lines). `measure_python`'s
  signature is unchanged. See [MIGRATIONS](./MIGRATIONS.md).

### Removed

- **BREAKING: the Stryker `mutation.json` report types are gone** (#246). `mutation::{StrykerReport,
  StrykerFile, StrykerMutant, StrykerLocation, parse_stryker_report, stryker_survivors}` are removed —
  the TS arm no longer parses a Stryker report file; the bundled adapter emits the normalized schema
  (#239) directly. Consume `parse_normalized_results` + `evaluate_normalized` instead. See
  [MIGRATIONS](./MIGRATIONS.md).
- **BREAKING: the cosmic-ray `dump` types are gone** (#248). `mutation::{parse_cosmic_ray_dump,
  cosmic_ray_mutated_lines, CosmicRayLine, CrWorkItem, CrMutation, CrResult}` are removed — the Python
  arm no longer parses a `cosmic-ray dump`; the bundled adapter emits the normalized schema (#239)
  directly. Consume `parse_normalized_results` + `evaluate_normalized` instead. See
  [MIGRATIONS](./MIGRATIONS.md).

### Fixed

- **`unit colocated-test --base` no longer makes an exempt package barrel undeletable** (#252). A
  source *deleted* in the `<base>...HEAD` diff is now a co-change subject only if it *had* a colocated
  test in the **base** tree — the test actually at risk of being orphaned. A file that never had a
  sibling test (a package barrel: `__init__.py`, `index.ts`) can be removed without a test appearing
  in the diff, so co-change no longer flags it. Before, deleting an exempt barrel was unsatisfiable:
  keeping its `colocated-test` exempt entry tripped the stale-exempt check (the file is gone in HEAD),
  and removing the entry — the documented move — tripped co-change. Now the barrel and its (now-stale)
  exempt entry are both simply deleted. No API change (`co_change::stale_sources`'s signature is
  unchanged).

- **`unit mutation --language rust --base` now handles a crate nested in the git repo, and a diff
  that doesn't touch it** (#204 follow-up). The `<base>...HEAD` diff is taken `--relative` to the
  crate, so cargo-mutants' `--in-diff` matches a crate in a subdirectory (the common consumer
  layout) rather than nothing; and a diff with no changed lines under the crate — or one that
  produces no mutants — now reports no survivors instead of erroring with `reading cargo-mutants
  outcomes … the run wrote none`. No API change (`measure_rust`'s signature is unchanged).

- **`unit mutation --language typescript` no longer auto-downloads a deprecated package.** The TS
  arm shelled out to `npx --yes stryker run`, which — when `@stryker-mutator/core` wasn't installed —
  silently fetched the long-deprecated standalone `stryker` package (last published as `0.x` in 2019,
  renamed to `@stryker-mutator/core`) and crashed with a confusing `MODULE_NOT_FOUND`. It now runs
  `npx --no-install`, so it uses only the project's own pinned Stryker and fails fast with a clear
  error when the engine is absent — parity with the cosmic-ray (Python) and cargo-mutants (Rust) arms,
  which already invoke their binary directly. A project that relied on the implicit download must now
  install `@stryker-mutator/core` + a test-runner plugin (the rule always documented this as a
  prerequisite; the reusable workflow already installs it). *(Superseded within this same unreleased
  window by #246 above: the consumer no longer installs Stryker at all — the tool bundles and drives
  it. This entry is retained only because the coverage fix below refers back to its `npx` footgun.)*
- **`unit coverage --language typescript` no longer auto-downloads vitest.** The same `npx --yes`
  footgun as the mutation arm: `run_vitest_coverage` shelled out to `npx --yes vitest`, silently
  fetching vitest when it wasn't installed. It now runs `npx --no-install`, using only the project's
  own vitest and failing fast with a clear error otherwise — parity with the coverage.py / cargo
  llvm-cov arms.

### Added

- **Normalized mutation-result core** (#239, epic foundation). A new engine-agnostic result
  representation — `mutation::{MutantStatus, NormalizedMutant, parse_normalized_results,
  evaluate_normalized}` — so the gate (line-scoped exemptions + the #226 determinism guard + binary
  pass/fail) runs over **one** schema instead of three per-engine report formats. `MutantStatus` is the
  union of the engines' outcomes (`survived` / `killed` / `no_coverage` / `timeout` / `compile_error` /
  `runtime_error`, `snake_case` on the wire); `survived` + `no_coverage` are survivors, the viable ones
  feed the guard. Purely additive — nothing existing changes and the per-language arms are not yet wired
  to it (that lands per #246–#249 as each engine gains a native-API adapter that emits this schema).
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
