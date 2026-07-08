# Migrations

Upgrade notes for breaking changes. New entries go under `## Unreleased`.
On release, the section is renamed to `## v<OLD> → v<NEW>`.

Each entry has five sections, in order:

1. **Summary** — one paragraph: what changed and why.
2. **Required changes** — before/after for public API. "None" if purely additive.
3. **Deprecations removed** — anything previously warned about that's now gone.
4. **Behavior changes without code changes** — same API, different runtime behavior.
5. **Verification** — commands that confirm the upgrade worked, with expected output.

## Unreleased

### Summary

Moves the suite-executing jobs' (`unit-coverage`, `coverage-changed`, `mutation`) install/build
location to the derived package root (#278, #279, epic #276, building on #277's `package_root` /
`ts_package_manager` / `python_env` / `provision_rust` primitive). All three jobs now install
TypeScript dependencies (`pnpm install --frozen-lockfile` or `npm ci`, per the detected package
manager) and provision the Python environment (`pip install coverage pytest` — or `pip install
pytest testing-conventions` for `mutation` — unchanged; or `uv sync` + a `uv pip install` of the
same packages into the project's own venv for a `[project]`-table package) at the package root
rather than the checkout root, so cosmic-ray's spawned pytest (for `mutation`) or the coverage run
can import the project's own dependencies. Rust toolchain provisioning moves ahead of that setup
and now also auto-fires when the package manifest itself declares a Rust-compiling build (a
`Cargo.toml`, a maturin `pyproject.toml` backend, a napi `package.json` key) — `rust_toolchain`
remains as a manual override for a build no manifest field expresses — and caches `target` under
the package root. No Rust-arm change to `mutation` itself — cargo-mutants already runs from the
scan root and cargo walks up to the crate. The build hook itself moves off the `uses:` call in the
same release (#289, below), so the migration is to the `[python] build_command` config key rather
than a repositioned input. A single-package pnpm/pip consumer with no Rust build declared in its
manifest is unaffected (see **Behavior changes without code changes**).

Moves the `build_command` escape hatch from a reusable-workflow input to a **`[python]
build_command` config key** (#289). The `build_command` *workflow input* is removed; a Python
package whose suite imports a compiled module now declares the build in its own
`testing-conventions.toml` as `[python] build_command`, with a required `reason` (validated
non-empty on load). `detect` reads it from the config discovered at the package root and emits a
`build_command` output; the `unit-coverage`, `coverage-changed`, and `mutation` jobs run
`needs.detect.outputs.build_command` at the package root. Breaking for a consumer setting
`build_command:` on the `uses:` call, and — because `[python] build_command` is a new public field
on `config::PythonConfig` — one breaking SDK change (see **Required changes**). Python-only,
matching the analysis that TypeScript's npm `prepare` / `postinstall` and Rust's `build.rs` are
manifest-native build hooks that never needed the input.

Adds an optional directory argument to **`e2e verify [path]`** (#281): `path` defaults to the
current directory, so a no-argument call is byte-identical to today. Passing a package
subdirectory scopes attestation discovery to it, matching `e2e verify` run with that directory as
cwd — needed so a monorepo package's attestation can be verified without a `cd`. Purely additive;
no existing invocation changes behavior (see **Required changes**).

Adds an optional `--scope <dir>` flag to **`e2e verify [path]`** (#294), decoupling where the
attestation file lives (`path`) from what counts as code for its freshness check (`--scope`,
default: `path` itself). Omitting it is byte-identical to today. The library gains
`e2e::verify_scoped(repo, scope)`; `e2e::verify(repo)` is unchanged in signature and now defined
as `verify_scoped(repo, repo)`. Purely additive; no existing invocation changes behavior.

Repoints the `install` template at the reorganized docs (#353): the managed `AGENTS.md` block's
tail drops the link to the removed CLI guide page and keeps the docs-site and machine-readable
contract (`llms.txt`) pointers. No API change; re-running `install` rewrites an existing block to
the new content (see **Behavior changes without code changes**).

Adds `functions` and `branch` floors to `[rust.coverage]` (#267), alongside the existing opt-in
`regions`. `functions` gates the llvm-cov export's functions total on the stable toolchain;
`branch` gates the branches total — the run adds `--branch`, which instruments only on a nightly
toolchain (pin one in the crate's `rust-toolchain.toml` with `llvm-tools-preview`, or set a rustup
directory override; on stable the run errors naming the requirement). A crate with no branch
points clears any `branch` floor vacuously. The zero-config default (`lines = 100`) is unchanged;
a consumer replacing a bespoke cargo-llvm-cov gate can now carry its functions/branch dimensions
across. One breaking SDK change: the public Rust coverage structs gain fields (see **Required
changes**). The coverage run also drops an inherited toolchain selection (`RUSTUP_TOOLCHAIN` /
`CARGO` / `RUSTC`) so the scanned crate's own `rust-toolchain.toml` decides (see **Behavior
changes without code changes**).

Adds cargo-feature passthrough for the suite-running Rust rules (#266). The config's `[rust]`
table takes a `features` list: `unit coverage` (whole-tree and `--base`) passes it to
`cargo llvm-cov` as `--features`, and `unit mutation` forwards it to cargo-mutants' build/test
runs — so `#[cfg(feature = ...)]` code is compiled, measured, and mutated instead of silently
compiled out of both. One breaking SDK change: the Rust measure functions gain a trailing
`features` argument (see **Required changes**). Cargo features are Rust's build-system concept
with no Python/TypeScript analog, so the key is deliberately Rust-only — a documented asymmetry
under the parity rule.

Scopes the Rust coverage floor to the unit suite (#265). `unit coverage --language rust` now runs
`cargo llvm-cov --lib`, measuring the library target and its inline `#[cfg(test)]` modules — the
tool's definition of a Rust unit and the same unit-only slice the Python and TypeScript arms
measure. Before, cargo-llvm-cov's default ran every test target, so integration tests under
`tests/` padded the number the floor gated. The diff-scoped floor (`unit coverage --base`) shares
the run and gets the same scoping. No API change; measured percentages drop wherever the
integration tier was doing the covering (see **Behavior changes without code changes**).

Makes the Rust arm provision its engine (#242, completing the #239 epic). `unit mutation --language
rust` no longer needs cargo-mutants pre-installed: on first use it runs a pinned `cargo install
cargo-mutants --locked --version <X>` into the tool's own cache directory and drives the binary from
there, so a direct CLI run works with only a cargo toolchain — the same "install nothing, never name
the engine" contract the TS/Python arms meet by resolving from the npm/wheel install. cargo has no
library form of cargo-mutants, so the tool runs the installed binary rather than embedding it — the
one deliberate asymmetry from the in-process TS/Python adapters. (The reusable workflow and selftest
run the *published* CLI, which gains provisioning only once this ships, so their `Install
cargo-mutants` step stays until then and is removed in a follow-up.) `measure_rust`'s signature is
unchanged; only its runtime behavior changes (see **Behavior changes without code changes**).

Wires the Python arm to drive cosmic-ray in-process through a **bundled Python adapter** (#248,
building on the #239 core). `unit mutation --language python` now spawns a Python adapter shipped in
the wheel (`python3 -m testing_conventions.mutation.main`) that drives cosmic-ray via its `WorkDB`
library API and emits the normalized `NormalizedMutant` schema (#239) the gate consumes — replacing
the `cosmic-ray` CLI orchestration. maturin ships the rust binary directly as the wheel's script (no
Python launcher to inject a path, unlike the TS arm), so the binary invokes the adapter as an
installed module, resolved from the wheel's environment alongside cosmic-ray. One breaking SDK change:
the cosmic-ray `dump` types are removed (see **Required changes**); `measure_python`'s signature is
unchanged.

Wires the TypeScript arm to drive Stryker through a **bundled Node adapter** (#246, building on the
#239 core). `unit mutation --language typescript` now spawns a Node adapter shipped with the npm
package that drives Stryker via its own Node API and emits the normalized `NormalizedMutant` schema
(#239) the gate consumes. The tool drives the engine; the project supplies its own test runner
(vitest) — exactly as cargo-mutants needs a buildable crate and cosmic-ray needs pytest. The npm
`testing-conventions` launcher passes the adapter's path to the binary as a `--ts-mutation-adapter`
argument on a `unit mutation` invocation. Two breaking SDK changes: the Stryker `mutation.json` report
types are removed, and `measure_typescript` takes a trailing `adapter: &Path` (see **Required
changes**).

Adds the normalized mutation-result core (#239) — the foundation for driving each engine through its
own native API instead of a Rust-spawned CLI + report-file parse. New additive public surface in the
`mutation` module: `MutantStatus`, `NormalizedMutant`, `parse_normalized_results`, and
`evaluate_normalized` (the engine-agnostic gate: line-scoped exemptions + the #226 determinism guard
over one schema). Nothing existing changes and no arm is wired to it yet — the per-language adapters
and switch-over land in #246–#249. Purely additive (see **Required changes**).

Hardens `unit mutation --language typescript` so it never downloads a mutation engine. The TypeScript
arm shelled out to `npx --yes stryker run`; with `--yes`, a project missing `@stryker-mutator/core`
would silently fetch the long-deprecated standalone `stryker` package (last released as `0.x` in 2019,
before the rename to `@stryker-mutator/core`) and crash with `MODULE_NOT_FOUND`. It now runs
`npx --no-install`, resolving only the project's own pinned Stryker via Node's parent-dir lookup and
failing fast with a clear error when it's absent — parity with the cosmic-ray and cargo-mutants arms,
which invoke their binary directly. `measure_typescript`'s signature is unchanged; only its runtime
behavior changes (see **Behavior changes without code changes**).

Makes every `[<language>].coverage` table a **partial override** (#216, parent #196): missing fields
fall back to the language's default floor instead of erroring, so a consumer sets only what they want
to change (`[typescript].coverage] branches = 90` keeps the other three at 100; `[rust].coverage]
regions = 90` keeps `lines = 100`). Purely a loosening — configs that already specified every field
parse identically, and a typo'd key is still rejected (`deny_unknown_fields` is retained). No public
API change: the `PythonCoverage` / `TypeScriptCoverage` / `RustCoverage` fields are unchanged; only
their serde defaulting moves (see **Behavior changes without code changes**).

Fixes `unit mutation --language rust --base` for a crate nested in the git repo (#204 follow-up).
The `<base>...HEAD` diff is now taken `--relative` to the crate, so cargo-mutants' `--in-diff`
matches a crate in a subdirectory (the common consumer layout) instead of nothing; and a diff with
no changed lines under the crate — or one that yields no mutants — reports no survivors instead of
erroring (`reading cargo-mutants outcomes … the run wrote none`). Purely a bug fix —
`measure_rust`'s signature is unchanged; only its `--base` runtime behavior changes (see **Behavior
changes without code changes**).

Adds `unit mutation --language python` (#203) — the Python arm, completing cross-language parity.
Wraps [cosmic-ray](https://github.com/sixty-north/cosmic-ray): a baseline check guards the suite,
then `init` / `exec` run the mutants and `cosmic-ray dump` is parsed for the `survived` outcomes,
feeding the shared `mutation::evaluate` core. Same on-by-default binary gate and reason-required
`[[python.exempt]] rules = ["mutation"]` loosening as the other arms. cosmic-ray has no native
git-diff mode, so `--base` scopes the run to the changed `.py` files and filters survivors to the
changed lines (line granularity). Purely additive — `mutation::measure_python` and the cosmic-ray
dump types; nothing existing changes. With all three languages now at parity, `unit mutation` is
still unwired from the reusable workflow until the matrix wiring lands (#199). Requires cosmic-ray
+ pytest.

Adds `unit mutation --language typescript` (#202) — the TypeScript arm of the mutation rule,
parity with the Rust vertical. Wraps [Stryker](https://stryker-mutator.io/), reads its
`mutation.json` report, and collects the surviving mutants (`Survived` / `NoCoverage`), feeding
the shared `mutation::evaluate` core extracted from the Rust arm. Same on-by-default binary gate
and reason-required `[[typescript.exempt]] rules = ["mutation"]` loosening. Stryker has no native
git-diff scoping, so `--base` translates the `<base>...HEAD` changed lines into Stryker `--mutate
<file>:<line>-<line>` ranges (line granularity; the one asymmetry is that under `--base` the ranges
replace Stryker's configured `mutate` set, filtering test/`.d.ts` files). Purely additive —
`mutation::measure_typescript`, the shared `mutation::evaluate`, and the Stryker report types;
nothing existing changes. Unwired from the reusable workflow until Python reaches parity (#199).
Requires Stryker (`@stryker-mutator/core` and a test-runner plugin).

Adds `unit mutation --language rust` (#201) — the rung above coverage: a test that *runs*
a line still passes if you delete its assertions, and a surviving mutant proves it. It wraps
[cargo-mutants](https://github.com/sourcefrog/cargo-mutants), reads its `outcomes.json`, and
finds the surviving mutants. The gate is binary, not a percentage (equivalent mutants make a
fixed score unreachable), and **on by default**: any un-exempted survivor fails the run, with no
report-only mode. The only loosening is a reason-required `[[rust.exempt]] rules = ["mutation"]`
entry for an equivalent / defensive survivor; `--base <REF>` scopes to the diff (cargo-mutants'
`--in-diff`). Purely additive — a new `unit mutation` subcommand and `mutation` module
(`measure_rust`, `unexplained_survivors`, `Survivor`), plus `config::Rule::Mutation`; nothing
existing changes. Rust-only and unwired from the reusable workflow until TypeScript and Python
reach parity (#199). Requires `cargo-mutants`.

Gives Rust a zero-config default coverage floor of `lines = 100` (#206), closing the
last gap from the strict-100 default (#194). With no `[rust].coverage` table, `unit
coverage --language rust` no longer errors asking for one — it enforces a 100% line
floor, matching Python/TypeScript, and the reusable workflow fans `unit coverage` over
a detected Rust crate whether or not a floor is configured. Two deliberate asymmetries
from the other languages, both forced by `cargo llvm-cov` on stable: there is no branch
component (branch coverage is experimental), and `regions` is opt-in (a Rust-only
sub-line metric, harsher than lines — off unless a config sets it). API change:
`RustCoverage` gains a `Default` impl, and `RustCoverage.regions` /
`coverage::RustThresholds.regions` move from `u8` to `Option<u8>` (see **Required
changes**, **Behavior changes**, and **Verification**).

Raises the zero-config default coverage floors to a strict 100% (#194). With no
`[<language>].coverage` table, `unit coverage` now enforces 100% — Python `fail_under = 100`
(branch on), TypeScript all four metrics at 100 — up from the #80 defaults (Python 85;
TypeScript lines/functions/statements 80, branches 75). The premise: the exemption system
(`# pragma: no cover`, reason-required `[[<lang>.exempt]]` entries, the empty/comment-only and
`.d.ts` auto-exemptions) already carries the trivia, so the default covers "100% of what you
didn't explicitly exempt." The `PythonCoverage` / `TypeScriptCoverage` struct fields are
unchanged — only their `Default` values move — so there is no API change; see **Behavior
changes** and **Verification**.

Renames the `unit isolation` command to `unit lint` (#160, part of the #158 CLI
taxonomy redesign), mirroring `integration lint` — each lints its test kind's files
for that kind's rules. Breaking for the command word only: the rules
(`unmocked-collaborator`, `untyped-mock`, `no-out-of-module-call`,
`no-out-of-module-import`) and their ids are unchanged, so config and
`[[<lang>.exempt]]` waivers need no edits. Internally `UnitRule::Isolation` →
`UnitRule::Lint` (`run_unit_isolation` → `run_unit_lint`); the `isolation` module and
the `isolation::Language` selector are unchanged. See **Required changes**.

Adds the `config` module: a `Config` schema holding the per-language `coverage`
thresholds (`[python]` / `[typescript]` / `[rust]`), plus `load_config()`, which
reads one TOML file into it and validates the config itself (the self-guard) —
unknown keys and malformed TOML are rejected rather than silently accepted.
Purely additive; nothing consumes the parsed config yet.

Also reshapes the colocated-test rule's CLI (#22) and then renames the rule
itself (#55). The rule ships for two languages — `missing_unit_tests` walks a
directory and returns every source file with no colocated test, and the CLI runs
it and exits non-zero on any orphan (Python #15: `foo.py` → `foo_test.py`;
TypeScript #18: `foo-bar.ts` → `foo-bar.test.ts` across `.ts`/`.tsx`/`.mts`/`.cts`,
`*.d.ts`/`*.d.mts`/`*.d.cts` ignored). The **command surface changes**, though: the
previously released `unit-location [--lang …]` (v0.0.3 / v0.0.4) first became
`unit location --language <python|typescript> <PATH>` (#22 — rules nest under
their test kind, `--language` required, the `python` default gone), and is now
`unit colocated-test …` (#55 — the rule was renamed `location` → `colocated-test`
so its name says what it checks: that a source file has a colocated,
matching-named unit test). This is a breaking change for anyone on an earlier CLI.
(#22 left the library API untouched; #32 below then changes the
`missing_unit_tests` and `measure` signatures, and #55 renames the `location`
module to `colocated_test`.)

Also adds the Python coverage rule (#26): `unit coverage --language python
--config <CONFIG> <PATH>` runs the unit suite under `coverage.py` (branch on,
`*_test.py` omitted) and enforces the config's `[python].coverage` floor, with the
supporting `testing_conventions::coverage` module (`measure`, `evaluate`,
`parse_report`, and the `Thresholds` / `CoverageReport` / `Outcome` types). Purely
additive — a new subcommand and module; nothing existing changes.

Also adds TypeScript coverage (#31), the twin of #26: `unit coverage --language
typescript --config <CONFIG> <PATH>` runs the unit suite under `vitest` v8
coverage and enforces the four `[typescript].coverage` thresholds (`lines` /
`branches` / `functions` / `statements`), excluding `*.test.*` and declaration
files — and any `coverage`-exempt path — from the denominator. vitest reports four
independent metrics rather than Python's single total, so it adds its own
`coverage::{measure_typescript, evaluate_typescript, parse_vitest_report}` plus
the `TypeScriptThresholds` / `VitestReport` / `VitestTotals` / `VitestMetric`
types, sharing the existing `Outcome`. Purely additive — `--language typescript`
previously errored as unimplemented; nothing existing changes.

Also adds the `integration lint` command and its `lint` module (#48, #49): a
deterministic, AST-based lint on Python test files. `integration lint --language
python <PATH>` parses each test file (`*_test.py`, `test_*.py`, `conftest.py`) with
`rustpython_parser` and flags the mocking-mechanism lints — `no-monkeypatch` (a
test/fixture that declares pytest's `monkeypatch` parameter), `no-inline-patch` (a
`patch(...)` call in a test body, which belongs in a `pytest.fixture`),
`no-environ-mutation` (direct `os.environ` mutation; set env via `patch.dict`), and
`no-constant-patch` (patching a module-global UPPER_CASE constant), which is waivable
per file via a `--config` `exempt` entry (`rules = ["no-constant-patch"]`, reusing #32).
Purely additive: a new command group (`integration`), a `--config` flag on `integration
lint`, the `config::Rule::NoConstantPatch` value, and the `testing_conventions::lint`
module; nothing existing changes.

Finally, adds exemptions (#32) so the checker can be an honest blocking gate.
Exemptions are **config-driven and explicit** — there is no automatic name- or
shape-based exemption. `__init__.py`, re-export barrels, and launcher shims are
all subjects now; the only files exempt automatically are empty/comment-only ones
(no logic to test). For deliberate omissions the tool can't infer, list the file
in the one config file: a `[[<language>.exempt]]` entry with the `rules` it lifts
(`colocated-test` / `coverage`) and a required `reason`. A `colocated-test`
exemption keeps the file off the orphan list; a `coverage` exemption omits it from
the coverage denominator. The list is auditable (one place, in the config diff) and enforced:
a stale entry — a path that no longer exists — is a hard error, so it can't
silently rot. New config types `Rule` and `Exemption` plus `resolve_exempt()`;
`[<language>].coverage` becomes optional (a config can carry exemptions alone);
and `missing_unit_tests` / `coverage::measure` take the resolved exemptions
(signatures below).

Zero-config coverage (#80): `unit coverage` now enforces the language's sane
default floor when coverage isn't configured — a missing config file, or a config
without the `[<language>].coverage` table — instead of erroring (Python
`branch = true, fail_under = 85`; TypeScript `lines = 80, branches = 75,
functions = 80, statements = 80`, the reasonable floors from the internals style
guides). This matches how `unit colocated-test` and `integration lint` already
treat an absent config, and lets the reusable workflow opt a new library into
every check with no config file. Additive: `Config`,
`{Python,TypeScript,Rust}Config`, `PythonCoverage`, and `TypeScriptCoverage` gain
`Default` impls; no existing signature changes.

Adds the `packaging` rule's foundation (#70): the `packaging` command and its
`packaging` module. `packaging --language <python|typescript> <PATH>` scans the
built artifact at `<PATH>` (an already-unpacked wheel or `dist/`) for that
language's test-file glob — Python `*_test.py`, TypeScript `*.test.*` — and exits
non-zero if any are present, enforcing the README "Packaging" rule that test files
must not ship. `packaging::scan(root, globs)` is the deterministic core; the
per-language *build* step that produces the artifact lands in #72 / #73 / #74
(the last also adding `--language rust`). Purely additive — a new command and
module; no existing signature or behavior changes.

Also adds the Rust colocated-test arm (#40): `unit colocated-test --language rust
<PATH>` now checks inline-`#[cfg(test)]` *presence* — a `src` file that defines a
function with a body but carries no inline `#[cfg(test)]` module is an orphan
(module-declaration / type-only files, and `tests/` / `benches/` / `examples/` /
`build.rs`, are not subjects). This is a **behavior change without a signature
change**: `unit colocated-test --language rust` previously exited 1 with an error
("Rust units are inline … see `unit isolation`") and now runs the check (exit 0 when
every source module has an inline test, 1 with an actionable list otherwise).
Additive on the library side: a new `colocated_test::missing_inline_tests(root,
exempt)`. Waivable per file via `[[rust.exempt]] rules = ["colocated-test"]`. Verify
with `testing-conventions unit colocated-test --language rust <crate>`: a crate whose
every behavior-bearing `src` module has an inline `#[cfg(test)]` exits 0; one that
doesn't lists each orphan and exits 1.

Also adds the Rust `unit isolation` rule (#44) and its `isolation`
module: a deterministic, `syn`-based lint on Rust test code. `unit isolation
--language rust <PATH>` parses each `*.rs` file under the crate root and flags a
call out of an inline `#[cfg(test)]` module's own module — `no-out-of-module-call`
(`crate::…`, `super::super::…`, an external crate from `Cargo.toml`, or effectful
`std`) — and a foreign `use` import — `no-out-of-module-import` (a glob of anything
but `super::*`, or a named import rooted at `crate::`, an external crate, or
effectful `std`). Purely additive: a new `unit` subcommand and module
(`testing_conventions::isolation::{find_violations, Violation, Language}`). The
shared `Violation` type moves to a new `violation` module and is re-exported from
`lint`, so `testing_conventions::lint::Violation` still resolves with **no code
change required**.

Also adds the Rust **integration** isolation lint (#44): `integration lint
--language rust <PATH>` flags `no-first-party-double` — a `#[double]`
(mockall_double) import of a first-party item (the crate under test or a `path`
dep) in a `tests/` integration crate, which must run first-party code for real;
doubling an external crate is fine. To add Rust without touching the file-pairing
`colocated_test::Language`, `integration lint`'s `--language` is now its own
`IntegrationLintLanguage` (python/typescript/rust). Purely additive — `--language
rust` is new, the python/typescript surface is unchanged, and the library gains
`testing_conventions::isolation::find_integration_violations`.

Also adds the first TypeScript lint (#43, #75): `integration lint --language
typescript <PATH>` extends the (previously Python-only) `integration lint` command
to TypeScript, parsing each `*.test.{ts,tsx,mts,cts}` file with `oxc` and flagging
**`no-first-party-mock`** — a `vi.mock()` / `vi.doMock()` of a first-party
(relative) module in an integration test, which must run first-party code for real
(third-party packages and Node built-ins may still be mocked). Purely additive: a
new `testing_conventions::ts` module (`find_integration_violations`, plus the shared
specifier classifier `classify` → `Origin`) and a new `--language typescript` arm on
`integration lint`; nothing existing changes.

Also extends `unit isolation` to TypeScript (#43, #76), the unit-direction
counterpart: `unit isolation --language typescript <PATH>` walks each
`*.test.{ts,tsx,mts,cts}` unit test and flags any runtime import that isn't
`vi.mock()`-ed (`unmocked-collaborator`), except the unit under test, type-only
imports, and the test runner (`vitest`). Additive — adds a `TypeScript` variant to
`isolation::Language` and the `testing_conventions::ts::find_unit_violations`
function; the Rust `unit isolation` behavior from #44 is unchanged.

Extends `packaging` to inspect a **Python wheel** (#72): `packaging --language
python <PATH>` now accepts a built `.whl` (a zip), unpacks it to a scratch
directory, and reuses `scan` to flag any `*_test.py` that shipped — `<PATH>` may
still be an already-unpacked directory. New library API `packaging::inspect(path,
globs)` (archive-or-directory → offenders relative to the artifact root); the
`packaging` command now calls it instead of `scan`. Additive — `scan` is
unchanged and the directory behavior is the same; new dependency `zip`.

Also extends `packaging` to a **TypeScript npm tarball** (#73): `packaging
--language typescript <PATH>` now accepts a built `.tgz` (an `npm pack` gzipped
tar), unpacks it, and reuses `scan` to flag any `*.test.*` that shipped in the
published `dist`. `inspect` now recognizes `.tgz` / `.tar.gz` in addition to
`.whl` and directories (the `.tar.gz` path is reused by #74's Rust `.crate` and
the Python sdist). Additive — new dependencies `flate2` + `tar`; no existing
signature changes.

Finally, `packaging --language rust` (#74) — the last packaging language.
`packaging` now accepts a Cargo `.crate` (`cargo package`, a gzipped tar) and
flags the crate-root **`tests/`** directory (`#[cfg(test)]` units compile out for
free; only the integration `tests/` needs a Cargo `exclude`). The scanner gains a
**directory pattern** (a pattern ending in `/` matches files under that dir)
alongside the file-name globs, and `colocated_test::Language` gains a `Rust`
variant so `--language rust` parses (`unit colocated-test` / `unit coverage` reject
it as separate items). Additive — a new enum variant and behavior; no existing
signature changes. Note for library consumers: matching exhaustively on the
public `Language` enum without a wildcard arm must add a `Rust` case.

Then `unit isolation --language typescript` also enforces **typed** mocks
(#43, #77): a `vi.mock(spec, factory)` whose factory carries no `vi.importActual<…>()`
type anchor is flagged `untyped-mock` (a bare `vi.mock(spec)` auto-mock and a typed
factory both pass). Behavior-only — `find_unit_violations` now reports the extra
rule; no signature changes. This completes #43's TypeScript isolation (#75/#76/#77).

Finally, adds the `workflow` guard (#92): a new `workflow` command and module that keeps the
reusable workflow's `@v0` consumption path from stranding. `workflow <PATH>` scans a
workflow file (or directory) for every `testing-conventions …` invocation and flags any
whose subcommand chain the binary no longer exposes (`no-unknown-subcommand`) — the failure
mode that broke `@v0` at 0.0.7 after the #55 `location` → `colocated-test` rename. Purely
additive: a new `testing_conventions::workflow` module (`invocations`, `unknown_subcommands`,
`check`, `Invocation`) and a `testing_conventions::command()` accessor for the binary's clap
command tree; nothing existing changes.

Also adds the e2e attestation nudge's first command (#17, #67): `e2e attest
'<command>'` runs the e2e suite, writes a committed `e2e-attestation.json` naming
the current commit (the command, a timestamp, the exit code, and the attested
SHA), and commits it on top — regardless of the command's outcome (force a run,
not a pass). Purely additive: a new `e2e` command group and the
`testing_conventions::e2e` module (`attest`, `Attestation`, `ATTESTATION_PATH`);
nothing existing changes. The CI-side `e2e verify` follows in #68.

Also makes the **isolation rules waivable** via the config `exempt` list (#102),
reusing the #32 machinery. `unit isolation` gains a `--config` flag (default
`testing-conventions.toml`); both it and `integration lint` now filter findings
against the config, so a `[[<lang>.exempt]]` entry naming an isolation rule —
`no-out-of-module-call`, `no-out-of-module-import`, `no-first-party-double`
(Rust), or `unmocked-collaborator`, `untyped-mock`, `no-first-party-mock` (TS) —
lifts it for that file (reason required; a stale entry still errors). Additive:
new `config::Rule` variants plus `Rule::id` / `Rule::from_id` and
`Config::rust_exemptions`; the `--config` flag is optional, so existing
invocations are unaffected.

Also adds **Python integration isolation** (#42): `integration lint --language
python <PATH>` now flags `no-first-party-patch` — a `patch(...)` whose string
target is first-party, e.g. `patch("ourpkg.mod.fn")`, since an integration test
must run first-party code for real (third-party packages and effectful stdlib stay
mockable). The first-party top-level package is read from the nearest
`pyproject.toml` `[project].name` (normalized), so a tree with no declared package
flags nothing. Behavior-only — `testing_conventions::lint::find_violations` now
reports the extra rule (no signature change), plus a new waivable `config::Rule`
variant `no-first-party-patch`; the python/typescript/rust surface is otherwise
unchanged.

Then adds that CI side (#17, #68): `e2e verify` reads the committed
`e2e-attestation.json` and passes iff its recorded SHA equals the latest code
commit — the newest commit touching any path other than the attestation file —
else exits non-zero with a run-`attest` hint. It never runs e2e and never judges
the recorded exit code/output. Purely additive: a new `e2e verify` subcommand and
`testing_conventions::e2e::{verify, Verification}`; nothing existing changes.

Also fixes a false positive in the TypeScript `unit isolation` typed-`vi.mock`
rule (#111): `vi.mock(spec, { spy: true })` — Vitest's options-object form, not a
factory — is no longer flagged `untyped-mock`. No API change; only a factory
*function* missing a `vi.importActual<…>` anchor is flagged now.

Also adds **Python unit isolation** (#42, slice 2): `unit isolation --language
python <PATH>` — the unit-direction twin of the above. It flags
`unmocked-collaborator` on a colocated unit test (`*_test.py` / `test_*.py`) that
imports a first-party collaborator without mocking it; the unit under test, the
test framework, pure stdlib, and type-only imports are never collaborators, and an
import counts as mocked when a `patch("…")` targets a matching last segment.
Additive: a new `Python` variant on `isolation::Language` (so `unit isolation
--language python` is now accepted) and `testing_conventions::lint::find_unit_isolation_violations`;
it emits the existing `unmocked-collaborator` rule, so the #102 waiver applies with
no new `config::Rule`. Nothing existing changes.

Also fixes the Python `conftest.py` handling (#112): `unit colocated-test` and
`unit coverage` treated `conftest.py` (pytest fixtures) as a unit-test subject —
flagging it as a missing-test orphan and counting it in the coverage denominator —
because only `*_test.py` was recognized as a non-subject. It is now test support:
never a subject, and omitted from the denominator alongside the test files. No API
change; the legacy `test_*.py` prefix stays unsupported (the colocated rule
requires `foo_test.py`).

Then extends Python `unit isolation` to **external** collaborators (#121, slice 3):
the same `unmocked-collaborator` rule now also flags an imported, un-mocked
third-party package or effectful-stdlib module (network / subprocess / process /
randomness / database / low-level OS), classifying import heads against an embedded
`sys.stdlib_module_names` set and a conservative effectful subset. Pure stdlib, the
`pytest` / `_pytest` / `mock` framework allowlist, and dual-nature heads (`os`,
`pathlib`, `datetime`, `time`, `io`) are not flagged. Behavior-only —
`find_unit_isolation_violations` reports the extra findings; no signature or rule-id
change, so the #102 waiver still applies.

Also makes the remaining Python integration lints **waivable** (#123):
`no-monkeypatch` (#49), `no-inline-patch` (#50), and `no-environ-mutation` (#51)
join `no-constant-patch` in the reason-required `[[python.exempt]]` escape hatch
(#32/#102) — they were the only blocking findings without one, because their ids
weren't `config::Rule` variants (so `apply_waivers` skipped them and the loader
rejected `rules = ["no-monkeypatch"]` outright). Additive: new `config::Rule`
variants `NoMonkeypatch` / `NoInlinePatch` / `NoEnvironMutation` (with `id()` /
`from_id()`); the lint behavior and every other surface are unchanged.

Also adds the commit-scoped `co-change` check (#33), exposed as the opt-in `--base` scope of
`unit colocated-test` (#161): `unit colocated-test --language <python|typescript> --base <REF>
<PATH>` diffs `<base>...HEAD` and flags any source file that was **modified** (and still holds
code) or **deleted** without its colocated test (`foo.py` → `foo_test.py`, `foo.ts` →
`foo.test.ts`) changing in the same diff — catching edits and removals that leave the test stale.
`--base` *adds* this on top of the tree-wide presence check and has no default, so a bare `unit
colocated-test` is presence-only. **Added** files aren't subjects (new code is the coverage
floor's job), a test file / empty file / `conftest.py` is never a subject, and a `co-change`
exemption lifts a source. Additive to the colocated-test command: the
`testing_conventions::co_change` module (`stale_sources`) and a waivable `config::Rule::CoChange`
(`co-change`); nothing existing changes. `--base --language rust` is rejected (inline
`#[cfg(test)]` units have no sibling test to go stale).

Fixes `unit colocated-test --base` so an exempt package barrel can be deleted (#252). A source
*deleted* in the `<base>...HEAD` diff is now a co-change subject only if it *had* a colocated test in
the **base** tree — a barrel (`__init__.py`, `index.ts`) that never had a sibling test can be removed
without a test co-changing, and needs no exemption. Before, the deletion was unsatisfiable: keeping the
barrel's `colocated-test` exempt entry tripped the stale-path check (the file is gone in HEAD) and
dropping it tripped co-change. Purely a bug fix — `co_change::stale_sources`'s signature is unchanged;
only its deletion-handling runtime behavior changes (see **Behavior changes without code changes**).

Finally, adds the Rust coverage arm (#37), the twin of #26 (Python) / #31 (TypeScript):
`unit coverage --language rust [--config <CONFIG>] <PATH>` runs `cargo llvm-cov --json
--summary-only` over the crate at `<PATH>` and enforces the `[rust].coverage` floor on the
export's **regions** and **lines** totals (branch coverage is still experimental, so it isn't
enforced), exiting non-zero and naming each metric below its floor. A `coverage` exemption drops a
file from the denominator via `cargo llvm-cov`'s `--ignore-filename-regex`. Two Rust-specific
caveats: inline `#[cfg(test)]` units can't be excluded by filename and `#[coverage(off)]` is still
nightly, so on a stable toolchain the inline test code is measured alongside the source; and Rust
has **no zero-config default floor** yet — `--language rust` previously errored as a separate item,
and now runs the check, but a config without a `[rust].coverage` table still errors (it does not
fall back to a default the way Python/TypeScript do under #80). Purely additive on the library
side: `coverage::{measure_rust, evaluate_rust, parse_llvm_cov_report, RustThresholds,
LlvmCovReport, LlvmCovData, LlvmCovTotals, LlvmCovMetric}`, sharing the existing `Outcome`; nothing
existing changes.

Also corrects the Python test-file recognition in the two `lint.rs` scans (#145,
follow-up to #112): `integration lint --language python` and `unit isolation
--language python` no longer treat a legacy `test_*.py` as a test file. After #112
a unit test is `*_test.py` and a `test_*.py` is ordinary source, but the scans
still recognized the legacy prefix — so a `test_*.py` carrying a `no-monkeypatch` /
`unmocked-collaborator` violation was flagged while `colocated-test` / `coverage`
treated it as source. The integration lints now scan `*_test.py` + `conftest.py`,
and the unit-isolation scan scans `*_test.py`, only. Behavior-only — no API or
rule-id change.

Also adds **patch (changed-line) coverage — Python** (#132, parent #46): `unit
patch-coverage --language python [--base <REF>] [--config <CONFIG>] <PATH>` diffs
`<base>...HEAD` and requires every line the change adds or modifies to be covered
by the unit suite — failing when a changed, executable line is a coverage.py
missing line or the source of a branch never taken (line + branch). The diff
machinery (`git diff --unified=0 <base>...HEAD`) is established here for the
forthcoming TS / Rust twins; `--base` defaults to `origin/main`. It reuses the
floor's `coverage` exemption (#32) — an exempt file's changed lines are lifted —
but, unlike `co-change` (#33), an *added* file's new lines are subjects (measured
via coverage.py `--source`). Purely additive: a new `unit` subcommand and the
`testing_conventions::patch_coverage` module (`check`, `changed_lines`,
`uncovered_changed_lines`, `Uncovered`); `coverage` gains `FileCoverage` /
`measure_patch_report` and a `files` map on `CoverageReport` (an additive,
`#[serde(default)]` field — `measure` / `evaluate` and the floor are unchanged).
`--language typescript` / `rust` are rejected as separate items.

Also adds **patch (changed-line) coverage — TypeScript** (#135, parent #46): `unit
patch-coverage --language typescript [--base <REF>] [--config <CONFIG>] <PATH>`,
the TypeScript twin of #132 built on the TypeScript coverage rule (#31). It reuses
the same `<base>...HEAD` diff machinery — scoped to `.ts` / `.tsx` / `.mts` /
`.cts` sources — and maps the changed lines against vitest's per-file v8 coverage,
flagging a changed line that carries a statement the suite never ran or the source
of a branch never taken (line + branch). It runs `npx vitest` with the `json`
reporter and `--coverage.all` (so an untested changed file is wholly uncovered) and
honors the floor's `[typescript].coverage` exemption (#32) — an excluded file's
changed lines are lifted. Purely additive: `patch_coverage` gains `check_typescript`
and `uncovered_changed_lines_ts`, and `coverage` gains `measure_patch_typescript`;
the Python arm and the existing API are unchanged. `--language rust` is still
rejected as a separate item.

Also adds **patch (changed-line) coverage — Rust** (#136, parent #46): `unit
patch-coverage --language rust [--base <REF>] [--config <CONFIG>] <PATH>`, the Rust
twin of #132 built on the Rust coverage rule (#37). It reuses the same
`<base>...HEAD` diff machinery — scoped to `.rs` sources — and maps the changed
lines against `cargo llvm-cov`'s per-line coverage, flagging a changed line
llvm-cov records no execution for (an LCOV `DA:<line>,0`). It runs `cargo llvm-cov
--lcov` with the floor's nested-run hygiene (an out-of-tree target dir, the outer
coverage env stripped) and honors the `[rust].coverage` exemption (#32) via
`--ignore-filename-regex`. Purely additive: `patch_coverage` gains `check_rust` and
`coverage` gains `measure_patch_rust` (the cargo-llvm-cov invocation is shared with
the floor via `run_cargo_llvm_cov`); the Python / TypeScript arms and the existing
API are unchanged. With Rust landed, `unit patch-coverage` covers all three
languages.

Also folds `unit patch-coverage` into `unit coverage --base` (#162, part of the #158
CLI taxonomy redesign). The diff-scoped changed-line check is no longer a separate
command: `unit coverage --language <LANG> --base <REF> [--config <CONFIG>] <PATH>`
measures the **same configured floor** (`fail_under`/`branch`; the four TypeScript
metrics; Rust regions/lines) over the `<base>...HEAD` diff instead of the whole tree.
Breaking — the `unit patch-coverage` command is removed. Two behavior changes from it:
the diff is judged against the configured floor rather than an implicit 100% (a diff
that clears the floor passes even with an uncovered changed line; they coincide only at
`fail_under = 100`), and there is no small-diff carve-out (a tiny diff below the floor
fails like any other). Config and `[[<lang>.exempt]] rules = ["coverage"]` waivers are
unchanged — both scopes already share the `coverage` rule id.

Makes **`coverage` / `mutation` exemptions line-scoped only** (#226). A `[[<language>.exempt]]` entry
naming `coverage` or `mutation` must now carry a `lines` list (`lines = [9, 10, "12-13"]`) naming the
exact lines it lifts — those rules are never whole-file. A determinism guard (the counterpart to the
stale-path rule) keeps the list minimal: a listed line that isn't actually failing — covered, or with
a killed mutant, or carrying no measured code — is a hard error, and an unlisted failing line still
fails. `lines` is rejected with a whole-file rule (`colocated-test`, the lints), so the two kinds
never share an entry. **Breaking**: a whole-file `rules = ["coverage"]` / `["mutation"]` entry, and a
combined `["colocated-test", "coverage"]` entry, no longer load — migrate them (see **Required
changes**). The mutation and diff-scoped coverage SDK functions gain an `exempt_lines` argument; the
new public items `config::{LineSpec, LineScope, resolve_exempt_scoped}`, `Exemption::{lines,
line_set}`, `coverage::measure_report`, `patch_coverage::measure_line_exempt{,_typescript,_rust}`, and
`mutation::{evaluate_scoped, mutated_lines, MutatedLines}` land alongside.

Adds the **`install`** command (#232): writes the testing contract into the repository's
`AGENTS.md` as a marker-delimited, hash-versioned block, idempotently. Purely additive — no
existing command, flag, config key, or SDK item changes.

Adds **`e2e verify --base <ref>`** (#319): scopes the freshness walk to `<base>..HEAD` (the commits
the branch introduced) instead of all reachable history, matching the changed-line coverage/mutation
gates — what makes the gate adoptable by a squash-merging repo. Purely additive at the CLI: absent,
freshness stays history-absolute, byte-identical to before. The SDK gains one public item,
`e2e::verify_since`; the existing `e2e::verify` and `e2e::verify_scoped` are unchanged (they now
delegate to it with `base = None`).

Adds **`e2e verify --extra-scope <dir>` / `--exclude <dir>`** (#333): joins a shared source tree
beside the package (repo-root-relative directories outside `--scope`) into the `<base>..HEAD`
freshness walk, with an exclude for feature-gated subtrees — so a native core bound into several
language bindings can stale each binding's attestation, which no `--scope` at-or-below the
attestation directory could reach. Purely additive at the CLI: both flags default empty, and an
empty union is byte-identical to before. The SDK gains one public item, `e2e::verify_extra_scoped`;
the existing `e2e::verify`, `e2e::verify_scoped`, and `e2e::verify_since` are unchanged (they now
delegate to it with no extra roots and no excludes).

Adds an **`[e2e]` config table** with `extra_scope` / `exclude` (#333): the declaration a package
uses to name that shared source tree in its own `testing-conventions.toml`, discovered by `detect`
like `[python] build_command`. The tool's config loader never acts on the keys — they drive the
`--extra-scope` / `--exclude` CLI flags via detect and the workflow — but the schema must accept the
table so a consumer declaring it still loads the rest of their config under `deny_unknown_fields`.
Purely additive: the SDK gains `config::E2eConfig` and a `Config::e2e` field, both defaulting to
absent/empty, so a package declaring no `[e2e]` table is unchanged.

Generalizes **`build_command`** from `[python]`-only to all three language tables and **drops its
required `reason`** (#335). It's reclassified from an escape hatch to a necessary declaration — it
supplies a build fact rather than waiving a check — so a bare `build_command` (no `reason`) now
loads, where #289's `[python].build_command` rejected it; an optional `reason` note is retained if
present. `[typescript].build_command` motivates the generalization: the packaging gate's
forthcoming auto-build needs a way to name a TS compile-before-`pack` that npm doesn't standardize
the name of. The config loader accepts the key under every table so a consumer's config loads under
`deny_unknown_fields`; the reader wiring lands as a follow-up. A package that sets no `build_command`
is unchanged.

**Behavior change without code changes:** a `[python].build_command` with no `reason` (or a blank
one) that failed to load under #289 now loads — the reason is optional. No config needs editing;
existing configs that carry a `reason` keep it (as an optional note).

### Required changes

`config::PythonConfig` gains `build_command: Option<String>` and `reason: String` (#289). Both
carry serde defaults, so a `testing-conventions.toml` without them parses unchanged; but a struct
literal must add the fields — `None` / `String::new()` preserve prior behavior:

```rust
// Before:
PythonConfig { coverage: Some(cov), exempt: vec![] }
// After:
PythonConfig { coverage: Some(cov), exempt: vec![], build_command: None, reason: String::new() }
```

`config::Config` gains `e2e: Option<E2eConfig>` (#333), with a serde default, so a
`testing-conventions.toml` without an `[e2e]` table parses unchanged; but a `Config` struct literal
must add the field — `None` preserves prior behavior:

```rust
// Before:
Config { python: p, typescript: t, rust: r }
// After:
Config { python: p, typescript: t, rust: r, e2e: None }
```

`config::TypeScriptConfig` and `config::RustConfig` each gain `build_command: Option<String>` and
`reason: String` (#335) — the same fields `PythonConfig` already carries — with serde defaults, so
a `testing-conventions.toml` without them parses unchanged; a struct literal adds the fields
(`None` / `String::new()` preserve prior behavior):

```rust
// Before:
TypeScriptConfig { coverage: Some(cov), exempt: vec![] }
// After:
TypeScriptConfig { coverage: Some(cov), exempt: vec![], build_command: None, reason: String::new() }
```

The reusable workflow's `build_command` **input is removed** (#289); a consumer setting it on the
`uses:` call moves that shell command into the package's own `testing-conventions.toml` as `[python]
build_command`, with a required `reason`. It runs at the derived package root (#278), so delete any
leading `cd ... &&` the input smuggled in:

```yaml
# Before — on the uses: call:
    with:
      build_command: cd packages/python && uv run maturin develop
```

```toml
# After — in packages/python/testing-conventions.toml:
[python]
build_command = "uv run maturin develop"
reason = "maturin's PEP 517 backend builds the wheel but exposes no pre-build shell hook"
```

Most callers need neither the key nor the `rust_toolchain` input once their manifest declares the
build: a `pyproject.toml` with a maturin `build-system.build-backend`, or a `package.json` with a
`napi` key / `@napi-rs/cli` devDependency, now builds and provisions cargo automatically (see
**Behavior changes without code changes**). Reach for `[python] build_command` only for a Python
build the manifest can't express.

`config::RustCoverage` and `coverage::RustThresholds` gain `functions: Option<u8>` and
`branch: Option<u8>`, and `coverage::LlvmCovTotals` gains `functions: LlvmCovMetric` and
`branches: Option<LlvmCovMetric>` (#267). All three have public fields, so struct literals add
the new fields — `None` preserves prior behavior:

```rust
// Before:
RustThresholds { regions: Some(80), lines: 80 }
// After:
RustThresholds { regions: Some(80), lines: 80, functions: None, branch: None }
```

The Rust SDK measure functions take a trailing `features: &[String]` (#266) —
`coverage::measure_rust`, `coverage::measure_patch_rust_detail`, `patch_coverage::measure_rust`,
`patch_coverage::measure_line_exempt_rust`, and `mutation::measure_rust`. Pass `&[]` to preserve
prior behavior, or the crate's `[rust] features` list to enable them on the run:

```rust
// Before:
coverage::measure_rust(&root, thresholds, &ignore)?;
// After:
coverage::measure_rust(&root, thresholds, &ignore, &[])?;
```

Migrate every whole-file `coverage` / `mutation` exemption to the line-scoped form, naming the lines
it covers (the determinism guard tells you the exact set — a listed line that's actually covered, or
a missed one you left out, is reported):

```toml
# Before — whole-file (no longer loads):
[[python.exempt]]
path = "mypkg/config/tomlcompat.py"
rules = ["coverage", "mutation"]
reason = "version-conditional import"

# After — line-scoped:
[[python.exempt]]
path = "mypkg/config/tomlcompat.py"
rules = ["coverage", "mutation"]
lines = [9, 10, "12-13"]
reason = "version-conditional import"
```

A combined entry that mixed a whole-file rule with a measured-line one must split in two — one
whole-file entry, one line-scoped:

```toml
# Before — combined (no longer loads):
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test", "coverage"]
reason = "thin launcher"

# After — one whole-file entry, one line-scoped:
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test"]
reason = "thin launcher"
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["coverage"]
lines = ["5-6"]
reason = "thin launcher; uncovered forwarding lines"
```

The mutation and diff-scoped coverage SDK functions gain an `exempt_lines` argument for the
line-scoped exemptions (#226). Pass an empty map to preserve the prior behavior; build the real value
from the `LineScope::Lines` entries of `config::resolve_exempt_scoped(root, exemptions, rule)`.

| Function | Before | After |
| --- | --- | --- |
| `mutation::measure_rust` / `measure_typescript` / `measure_python` | `(root, exempt, base)` | `(root, exempt, exempt_lines, base)` |
| `patch_coverage::measure` / `measure_typescript` / `measure_rust` | `(root, base, thresholds, omit)` | `(root, base, thresholds, omit, exempt_lines)` |

`exempt_lines: &BTreeMap<String, BTreeSet<u32>>` maps a `root`-relative path to its exempt lines; an
empty map is the no-op. The new whole-tree `patch_coverage::measure_line_exempt{,_typescript,_rust}`
take the same map.

The Stryker `mutation.json` report types are removed (#246): the TS arm no longer parses a report
file, so the report-parsing surface is gone. If you consumed it directly, switch to the normalized
core (#239) — the same types the bundled adapter emits and the Rust/Python arms already feed:

| Removed | Replacement |
| --- | --- |
| `mutation::parse_stryker_report`, `mutation::stryker_survivors` | `mutation::parse_normalized_results` + `mutation::evaluate_normalized` |
| `mutation::{StrykerReport, StrykerFile, StrykerMutant, StrykerLocation}` | `mutation::NormalizedMutant` |

The cosmic-ray `dump` types are removed the same way (#248): the Python arm no longer parses a
`cosmic-ray dump`. Switch to the normalized core:

| Removed | Replacement |
| --- | --- |
| `mutation::parse_cosmic_ray_dump`, `mutation::cosmic_ray_mutated_lines` | `mutation::parse_normalized_results` + `mutation::evaluate_normalized` |
| `mutation::{CosmicRayLine, CrWorkItem, CrMutation, CrResult}` | `mutation::NormalizedMutant` |

`measure_python`'s signature is unchanged; it now spawns the bundled Python adapter (`python3 -m
testing_conventions.mutation.main`) rather than the `cosmic-ray` CLI. In production the wheel makes
`testing_conventions` importable (site-packages); the rust integration tests set
`PYTHONPATH=packages/python/python` for their staged fixtures.

`measure_typescript` takes a trailing `adapter: &Path` — the path to the bundled Node adapter
(`packages/node/dist/mutation/main.js`):

| Function | Before | After |
| --- | --- | --- |
| `mutation::measure_typescript` | `(root, exempt, exempt_lines, base)` | `(root, exempt, exempt_lines, base, adapter)` |

On the CLI, the npm `testing-conventions` launcher passes this path as `--ts-mutation-adapter` on a
`unit mutation` invocation; run the rule through the published CLI rather than the raw Rust binary,
which errors clearly when the argument is absent.


`regions` on the Rust coverage types is now `Option<u8>` (#206), so the region check can
be left off (the zero-config default floors lines only). Library callers that construct
these directly must wrap the value:

| Type | Before | After |
| --- | --- | --- |
| `config::RustCoverage` | `RustCoverage { regions: 100, lines: 100 }` | `RustCoverage { regions: Some(100), lines: 100 }` |
| `coverage::RustThresholds` | `RustThresholds { regions: 80, lines: 80 }` | `RustThresholds { regions: Some(80), lines: 80 }` |

`Some(n)` reproduces the prior behavior; `None` skips the region check. A
`[rust].coverage` table written in TOML is unaffected — `regions = 100` still parses
(into `Some(100)`), and `regions` may now be omitted entirely.

`unit isolation` is now `unit lint` (#160), mirroring `integration lint`. Update any
invocation — CI steps, scripts, the reusable `testing-conventions.yml` workflow:

| Before                                      | After                                  |
| ------------------------------------------- | -------------------------------------- |
| `unit isolation --language rust .`          | `unit lint --language rust .`          |
| `unit isolation --language typescript src/` | `unit lint --language typescript src/` |
| `unit isolation --language python src/`     | `unit lint --language python src/`     |

The rules, their ids, and the `[[<lang>.exempt]]` waivers are unchanged — only the
command word moves; the library API is untouched (the `isolation` module and
`isolation::Language` keep their names).

The colocated-test CLI was renamed (twice, pre-1.0) and its language flag made
required. Update any invocation (CI steps, scripts, `npx`/`pip`/`cargo` wrappers)
to the current `unit colocated-test` form:

| Before                                                 | After                                            |
| ------------------------------------------------------ | ------------------------------------------------ |
| `unit-location src/` (≤ v0.0.4)                        | `unit colocated-test --language python src/`     |
| `unit-location --lang typescript src/` (≤ v0.0.4)      | `unit colocated-test --language typescript src/` |
| `unit location --language python src/` (v0.0.5–v0.0.8) | `unit colocated-test --language python src/`     |

- `unit-location` (flat, ≤ v0.0.4) / `unit location` (nested, v0.0.5–v0.0.8) →
  `unit colocated-test` (#22, #55).
- `--lang` → `--language`, which is required: there is no longer a `python` default.

Exemptions (#32) change the library API, and #55 renames the module these
colocated-test items live in — `testing_conventions::location` →
`testing_conventions::colocated_test` (the `Language` enum moves with it). Callers
must update the import path *and* pass the new arguments:

| Function | Before | After |
| --- | --- | --- |
| `missing_unit_tests` | `location::…(root, language)` | `colocated_test::…(root, language, exempt)` — `exempt: &BTreeSet<String>` of `colocated-test`-rule paths |
| `coverage::measure` | `(root, thresholds)` | `(root, thresholds, omit)` — `omit: &[String]` of `coverage`-rule paths |

Build both with `config::resolve_exempt(root, exemptions, rule)`. Passing an empty
set/slice preserves the prior behavior. `[<language>].coverage` is now an
`Option`, so `config.python.coverage` becomes `config.python.coverage` of type
`Option<PythonCoverage>` — match/`?` it before reading the thresholds.

Anyone relying on `__init__.py` being exempt must add it to the config: a
non-empty `__init__.py` (one with re-exports or code) is now a subject. An
**empty** `__init__.py` needs nothing — empty/comment-only files are not
subjects.

### Deprecations removed

The `--lang` flag and its implicit `python` default are gone — a clean break, not
a deprecation cycle (pre-1.0, so no prior warning was shipped).

### Behavior changes without code changes

`unit mutation --language rust` no longer duplicates the `cargo-mutants` provisioning install
under concurrent invocations (#385). A cold provisioning cache previously cost one from-source
`cargo install` per concurrent caller that observed the binary absent (a monorepo's per-package
matrix, a test runner that parallelizes across binaries); it now costs one install regardless of
how many callers race for it, with the rest waiting on a file lock and reusing the result. No API
or config change; the reported survivor set is unaffected.

`unit lint --language python` no longer reports a re-export barrel's own test (`__init___test.py`)
as importing unmocked collaborators when it imports the barrel's public surface (#382). A bare,
level-1 `from . import Thing, __all__, __version__` in `__init___test.py` is now the unit under
test, so a package with such a barrel test that previously exited 1 (one `unmocked-collaborator` per
imported name, `__all__` / `__version__` included) now exits 0 with no findings — and any file-level
`unmocked-collaborator` exemption kept solely for this shape can be deleted. The rule is unchanged
everywhere else: a sibling-direct import from a barrel test (`from .core import Thing` in
`__init___test.py`), a parent-package import (`from .. import …`), and a non-barrel `widget_test.py`
doing `from . import ledger` all still name collaborators and are still flagged. No API or config
change.

The reusable workflow's `e2e-verify` job now checks out the PR's head commit
(`github.event.pull_request.head.sha`) instead of the default `pull_request`-event ref (the
ephemeral merge commit). A PR whose attestation was reported stale purely because the base moved
since attestation (not because the package's own code changed) now correctly reports fresh. A
push-event run (no PR) is unaffected — it falls back to `github.sha`, unchanged from today.

`unit coverage --language typescript` now resolves and passes the project's own installed
vitest's default coverage excludes, instead of silently discarding them (#290). A package whose
scanned tree contains a build-tool config file (`vitest.config.ts`, `eslint.config.ts`, …) —
previously counted as 0%-covered "source" — is now excluded exactly as vitest's own defaults
already intend; its reported coverage percentage rises accordingly. A package with no such file
inside the scanned tree (the common single-package layout, where `vitest.config.ts` sits outside
the scanned `src/`) sees no change. No API or config change.

The reusable workflow's `unit-coverage`, `coverage-changed`, and `mutation` jobs now install
TypeScript dependencies and provision the Python environment at the derived package root, run
`build_command` there too, and auto-provision a Rust toolchain from the package manifest (#278,
#279). A single-package repo with a root pnpm lockfile / global pip setup and no Rust build
declared in its manifest is byte-identical to before — `package_root` derives to `.` and
`provision_rust` to `false`, so every step (including `build_command`) runs exactly where and as
it did. What changes: a per-package-lockfile monorepo TS package (its own `package-lock.json`, no
root manifest) installs at its own directory instead of failing with `ERR_PNPM_NO_PKG_MANIFEST`;
an npm TS package (`packageManager` field or a `package-lock.json`) installs with `npm ci` instead
of assuming pnpm; a Python package with its own `[project]` table installs via `uv sync` —
including building/installing the project itself, so a maturin package's native module compiles
here with no `build_command`, and (for `mutation`) the adapter wheel and pytest install into that
same venv instead of global pip — with `coverage`/`pytest` layered on for the coverage jobs and
its venv put on `PATH`; and a package whose manifest declares a Rust-compiling build gets a Rust
toolchain (and its cache, under the package root) with no `rust_toolchain: true` needed. A
monorepo consumer whose `build_command` assumed the checkout root (e.g. a relative path into a
specific package) must update it to be relative to that package's own root, or use an
absolute/`$GITHUB_WORKSPACE`-relative path instead. This reaches the `@v0` self-test and every
consumer only once a release moves the `@v0` tag (internals/repo.md) — this PR's own CI still
runs the previously published `detect` action.

`install` writes a refreshed managed block (#353): the region's content hash changes, so the next
run on a repo carrying the old block rewrites the owned region in place — the tail now carries the
docs-site and `llms.txt` pointers only. Content outside the markers is untouched, as before.

`unit coverage --language rust` now resolves the toolchain from the scanned crate (#267): the
run drops an inherited `RUSTUP_TOOLCHAIN` / `CARGO` / `RUSTC` selection, so the crate's own
`rust-toolchain.toml` (or the rustup default / directory override) decides. When the tool is
spawned by another cargo process (a test harness, an xtask), the spawning toolchain previously
overrode the crate's pin; a directly-invoked run on a crate with no pin behaves as before.

`unit coverage --language rust` (whole-tree and `--base`) now measures only the unit suite: the run
passes `--lib`, so the library target's inline `#[cfg(test)]` tests produce the number and the
integration tier under `tests/` stays out of it (#265). Reported percentages drop for any crate
where integration tests reached code the unit suite misses — that drop is the padding the Coverage
rule forbids, now made visible. Re-fit `[rust].coverage` to the honest unit-only number (or add
reasoned, line-scoped `coverage` exemptions) rather than restoring the old floor. `measure_rust`'s
signature is unchanged.

`unit mutation --language rust` provisions cargo-mutants on first use instead of requiring it on
`PATH` (#242): it runs a pinned `cargo install cargo-mutants --locked --version <X>` into the tool's
cache directory (`$XDG_CACHE_HOME`/`$HOME/.cache` → `testing-conventions/cargo-mutants-<X>`) and
invokes the binary from there. The first run on a cold machine pays a one-time from-source compile;
cache the directory in CI to skip it. If you install cargo-mutants yourself, the tool still uses its
own pinned copy, so results are reproducible across environments. `measure_rust`'s signature is
unchanged. A direct `unit mutation --language rust` now needs only a cargo toolchain; if your own
pipeline installed cargo-mutants solely for that direct call, you can drop the step once this release
is out.

`unit coverage --language typescript` likewise no longer auto-installs vitest — it runs
`npx --no-install vitest` and fails fast with a clear "must be installed" error when `vitest` /
`@vitest/coverage-v8` are absent, instead of silently downloading vitest. No API or config change.

`testing-conventions --help` no longer lists the private `workflow` command (#191): it
was always undocumented and run only from our own CI, and is now `#[command(hide = true)]`.
It still runs when invoked explicitly (hidden, not removed), and the `@v0` drift guard is
unaffected. No action needed.

Omitting the language is now a usage error (exit code `2`) instead of defaulting to
`python`. Before, running the check on a TypeScript project without a flag scanned
for `*.py`, found none, and exited `0` — a silent false green; now the language
must be stated explicitly.

Exemptions (#32) change runtime behavior:

- `__init__.py` is no longer auto-exempt — a non-empty one without a colocated
  test (and without a config entry) is now reported as an orphan. Empty/comment-
  only files (any language) are non-subjects and never reported.
- `unit colocated-test` and `unit coverage` honor the config `exempt` list: a
  `colocated-test` entry keeps a file off the orphan list; a `coverage` entry omits
  it from the denominator. A reason-less or stale entry makes the run **error**
  rather than pass.
- CLI error output now prints the full cause chain (e.g. `error: exempt entry
  \`ghost.py\` matches no file under \`…\`: …`) instead of only the outermost
  context. Exit codes are unchanged.
- `unit coverage` no longer errors on a missing config file (or a config without
  the `[<language>].coverage` table): it enforces the language's default floor
  instead — Python 85 with branch on; TypeScript lines/functions/statements 80,
  branches 75. A `[<language>].coverage` table still overrides it. (#80)
- The zero-config default coverage floors are now a strict 100% (#194): Python
  `fail_under = 100` (branch on) and all four TypeScript metrics at 100, up from the #80
  defaults above (Python 85; TypeScript 80/75/80/80). A zero-config build whose unit suite sat
  between the old floor and 100 now **fails** where it passed; restore the prior floor with an
  explicit `[<language>].coverage` table (e.g. `[python].coverage` with `fail_under = 85`). Rust
  gets its matching line floor separately in #206 (below).
- `integration lint --language typescript` (#43, #75) previously errored
  (`supports --language python only for now`); it now parses the TypeScript test
  files and runs the `no-first-party-mock` lint.
- `unit isolation --language typescript` no longer reports `untyped-mock` for the
  options-object mock `vi.mock(spec, { spy: true })` (Vitest ≥2). The spy form
  wraps the real module and can't drift, like a bare auto-mock; only a factory
  *function* without a `vi.importActual<…>` anchor is flagged. (#111)
- `unit colocated-test --language python` no longer reports `conftest.py` as a
  missing-test orphan, and `unit coverage --language python` omits `conftest.py`
  from the denominator (alongside `*_test.py`). conftest.py is pytest support,
  never a subject. (#112)
- A `[[python.exempt]]` entry naming `no-monkeypatch`, `no-inline-patch`, or
  `no-environ-mutation` is now accepted and waives that lint for the file. Previously
  the loader **rejected** those ids as an unknown `rules` variant (and even parsed,
  `integration lint` could never have waived them). A reason-less or stale entry still
  errors. (#123)
- `integration lint --language python` and `unit isolation --language python` no
  longer scan a legacy `test_*.py` (#145): it is ordinary source after #112, so a
  `test_*.py` carrying a `no-monkeypatch` / `unmocked-collaborator` violation is no
  longer flagged. The integration lints scan `*_test.py` + `conftest.py`, and the
  unit-isolation scan scans `*_test.py`, only. A `*_test.py` is unaffected.
- `unit coverage --language rust` previously errored (`Rust coverage … is a separate item`);
  it now runs `cargo llvm-cov` and enforces the `[rust].coverage` floor (#37). With no
  `[rust].coverage` table it falls back to the zero-config default added in #206 (below).
- `unit coverage --language rust` with no `[rust].coverage` table no longer errors asking for one
  (#206): it enforces a zero-config default of `lines = 100` (no branch component; `regions`
  opt-in), matching Python/TypeScript. The reusable workflow's `detect` now routes any Rust crate
  into the coverage matrix, not only crates with a configured floor. A zero-config Rust crate
  below 100% lines now **fails** where it previously had no coverage gate; lower `lines` (or omit
  it differently) with an explicit `[rust].coverage` table to restore headroom, and add a
  `regions = N` floor to opt the sub-line metric back in.
- `unit colocated-test --base` no longer flags a *deleted* source that had no colocated test in the
  base tree (#252): a package barrel (`__init__.py`, `index.ts`) can be removed without its
  (nonexistent) sibling test co-changing, and without a `co-change` exemption to silence it — the
  earlier "keep the exempt entry and the stale-path check rejects it / drop it and co-change flags it"
  deadlock is gone. A deletion whose colocated test *did* exist in base is still flagged when that test
  isn't removed or updated. No API or config change.

### Verification

```
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/testing-conventions.yml'))"
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/testing-conventions-selftest.yml'))"
grep -n 'needs.detect.outputs.package_root' .github/workflows/testing-conventions.yml
```

Expected: both parse, and matches show up in the `unit-coverage`, `coverage-changed`, and
`mutation` jobs' install / `build_command` / cache steps. There's no Rust API change for #278/#279
(workflow-only), so their own coverage is the `coverage-package-root-wired` /
`mutation-package-root-wired` static wiring jobs in `testing-conventions-selftest.yml` (green once
the jobs reference `needs.detect.outputs.package_root`) plus the `monorepo-coverage-ts` /
`monorepo-coverage-py` / `mutation-monorepo-ts` / `mutation-monorepo-py` runtime jobs — the latter
only fully exercise the new install/build location once a release moves `@v0`
(internals/repo.md); until then, verify by hand with the published-equivalent local actions, e.g.:

```
cd .github/selftest/monorepo/packages/ts && npm ci && npx vitest run
cd .github/selftest/monorepo/packages/py && uv run --with pytest pytest src/widget
```

Expected: both pass — the fixtures already meet their own coverage floors when installed at their
own package root by hand, the behavior the coverage/mutation jobs adopt.

```
cd packages/rust && cargo test --test coverage_ts
```

Expected: all pass — including `a_package_root_config_file_is_not_counted_as_uncovered_source`,
which proves a fully-tested package whose scanned root also carries its own `vitest.config.ts`
reports 100%, not diluted by the config file. Requires Node with the fixtures' vitest toolchain
installed (`npm install` under `tests/fixtures/unit_coverage/typescript/`).

```
cd packages/rust && cargo test --test e2e_verify --test e2e_verify_e2e
```

Expected: all pass — `e2e verify` with no argument still checks the current directory (unchanged
from today), and `e2e verify packages/widget` run from the repo root behaves identically to
running `e2e verify` with `packages/widget` as the current directory (fresh/stale/missing).
Includes the `#294` cases: a commit outside `--scope` but inside `path` doesn't trip staleness, a
commit inside `--scope` still does, and omitting `--scope` stays byte-identical to `#281`'s
whole-`path` freshness walk.

```
grep -n -- '--scope' .github/workflows/testing-conventions.yml
```

Expected: a match in the `e2e-verify` job's `run:` line. No Rust API change is measured by this
grep (workflow-only wiring); the `e2e::verify_scoped` behavior itself is covered by the test file
above.

```
cd packages/rust && cargo test --test install --test install_e2e
```

Expected: all pass — the block `install` writes links the docs root and `llms.txt` and carries no
link to the removed CLI guide page, and a re-run over a stale block refreshes the owned region and
leaves consumer content untouched.

```
cd packages/rust && cargo test --test coverage_metrics --test coverage_metrics_e2e
```

Expected: all pass — `functions` and `branch` parse as `[rust.coverage]` keys; the `funcs`
fixture (a never-called function) fails a 100 `functions` floor with a threshold shortfall and
clears a 60 floor; the `branchy` fixture (one of two branch outcomes taken, nightly pinned via
its own `rust-toolchain.toml`) fails a 100 `branch` floor and clears a 50 floor; and a `branch`
floor over a stable-toolchain crate errors naming the nightly requirement. Requires
`cargo-llvm-cov`; rustup fetches the fixture's pinned nightly on first run.

```
cd packages/rust && cargo test --test coverage_features --test coverage_features_e2e --test mutation_features_e2e
```

Expected: all pass — `[rust] features` parses (with and without a `coverage` table), the `gated`
fixture (feature module fully covered inline) clears a 100 floor with the feature enabled from
config, `gated_untested` fails it with a threshold shortfall over the now-measured module, and
`gated_killed` clears the mutation gate because the feature reaches cargo-mutants' build/test
runs. Requires `cargo-llvm-cov` and a cargo toolchain (cargo-mutants is provisioned by the tool).

```
cd packages/rust && cargo test --test coverage_rust --test coverage_rust_e2e
```

Expected: all pass — including `integration_tests_do_not_pad_the_unit_floor` and
`padded_exits_nonzero_against_a_100_floor`, which drive the `padded` fixture (a module covered only
by an integration test under `tests/`) and require the 100 floor to fail on its unit-only ~70%
regions / ~67% lines. Requires `cargo-llvm-cov`.

```
cd packages/rust && cargo test --lib default_python_coverage_is_the_strict_floor default_typescript_coverage_is_the_strict_floor default_rust_coverage_is_the_strict_line_floor
```

Expected: all three pass — `PythonCoverage::default()` is `fail_under = 100` with branch on,
`TypeScriptCoverage::default()` is all four metrics at 100 (the strict zero-config floor, #194),
and `RustCoverage::default()` is `lines = 100` with `regions: None` (the line floor, #206).

```
cd packages/rust && cargo test --lib mutation:: --test mutation_rust --test mutation_rust_e2e
```

Expected: the mutation tests pass — the pure `unexplained_survivors` collects only `MissedMutant`
outcomes and honors a `mutation` exemption; over the fixture crates, `killed` reports no survivors
while `survivors` (an assertion-light suite) reports several; and the always-on gate drives the
exit codes — `killed` `0`, `survivors` `1`, and `survivors` with a `mutation` exemption back to
`0`. Requires `cargo-mutants`.

```
cd packages/rust && cargo test --lib mutation:: --test mutation_typescript --test mutation_typescript_e2e --test mutation_base_ts
```

Expected: the TypeScript mutation tests pass — the rule drives the bundled Node adapter (#246), which
runs Stryker over the **runner-only** fixture projects (they install only vitest, never Stryker) and
emits the normalized schema the gate consumes; `killed` reports no survivors while `survivors` (an
assertion-light suite) reports several; `--base` scopes the run to the changed lines; and the
always-on gate drives the exit codes `0` / `1` / `0`. Requires the built node adapter
(`npm ci && npm run build` in `packages/node`) and the fixtures' vitest (`npm ci` in
`tests/fixtures/unit_mutation/typescript`); the tests pass the adapter path as
`--ts-mutation-adapter` (and to the SDK as the trailing `adapter` argument).

```
cd packages/rust && cargo test --lib mutation:: --test mutation_python --test mutation_python_e2e --test mutation_base_py
```

Expected: the Python mutation tests pass — the pure cosmic-ray-dump parser collects `survived`
outcomes and feeds the shared `evaluate` core; over the fixture projects, `killed` reports no
survivors while `survivors` (an assertion-light suite) reports several; `--base` scopes the run to
the changed files and filters survivors to the changed lines; and the always-on gate drives the
exit codes `0` / `1` / `0`. Requires cosmic-ray + pytest on PATH.

```
cd packages/rust && cargo test --test config_loader
```

Expected: the loader's integration tests pass — the canonical config loads, an
exempt-only config (no coverage thresholds) loads, and unknown-key, malformed,
missing-file, and reason-less-exemption configs are rejected.

```
cd packages/rust && cargo test --test colocated_test --test colocated_test_e2e
```

Expected: the colocated-test tests pass — clean fixtures report no orphans, red
fixtures report their missing twins, an empty `__init__.py` is not an orphan while a
content-bearing one is, config exemptions clear the listed files, and a stale
exempt entry errors. The renamed `unit colocated-test` subcommand parses while the
old `unit location` no longer does.

```
cd packages/rust && cargo test --test coverage
```

Expected: the coverage tests pass — including the `exempt_cov` codebase clearing a
100 floor once its shim is omitted by a `coverage` exemption. Requires `coverage`
+ `pytest` on `PATH`.

```
cd packages/rust && cargo test --test coverage_ts --test coverage_ts_e2e
```

Expected: the TypeScript coverage tests pass — `full` clears a 100 floor on all
four metrics, `above` fails 100 but clears the mid floor, `below` (100% lines but
~66% branches) fails the mid floor on branches, and `exempt_cov` clears 100 once
its shim is omitted by a `coverage` exemption. Requires Node with `vitest` +
`@vitest/coverage-v8` installed (run `npm ci` in
`tests/fixtures/unit_coverage/typescript`).

```
cd packages/rust && cargo test --test e2e_attest --test e2e_attest_e2e
```

Expected: the `e2e attest` tests pass — in a throwaway git repo, `attest` names
HEAD, writes `e2e-attestation.json`, and commits it on top, exiting `0` even when
the wrapped command fails (force a run, not a pass). Requires `git`.

```
cd packages/rust && cargo test --test integration_lint --test integration_lint_e2e
```

Expected: the integration-lint tests pass — including the `monkeypatch`, `inline_patch`,
and `environ` `waived` fixtures, each identical to its red fixture but cleared to exit `0`
by a reason-required `[[python.exempt]]` entry, alongside the existing `constant_patch`
and `no_first_party_patch` waivers. (#123)

```
cd packages/rust && cargo test --test integration_lint --test integration_lint_e2e
```

Expected: the lint's integration + e2e tests pass — the clean fixture reports no
violations and exits `0`, and the red fixture (a test taking `monkeypatch`) is
flagged and exits `1`.

```
cd packages/rust && cargo test --test ts_integration_lint --test ts_integration_lint_e2e
```

Expected: the TypeScript lint's integration + e2e tests pass — the clean fixture
(mocks only third-party packages and Node built-ins) reports no violations and exits
`0`, and the red fixture (a first-party `vi.mock` / `vi.doMock`) is flagged and exits
`1`.

```
cd packages/rust && cargo test --test unit_isolation --test unit_isolation_e2e
```

Expected: the TypeScript unit-isolation tests pass — both rules. For
`unmocked-collaborator`, the clean fixture (every collaborator `vi.mock()`-ed) exits
`0` and the red fixture (an un-mocked `./formatter` and `lodash`) is flagged. For
`untyped-mock` (#77), the `untyped_mock` red fixture (a `vi.mock` factory with no
`vi.importActual<…>` anchor) is flagged while its clean fixture (a typed factory and a
bare auto-mock) exits `0`.

```
cd packages/rust && cargo test --test coverage_e2e --test coverage_ts_e2e
```

Expected: the coverage e2e suites pass, including the zero-config cases (#80) — a
`--config` pointing at a nonexistent file falls back to the default floor: Python
`full` and `above_85` (85.71%) pass while `below_85` (71.43%) fails; TypeScript
`above` passes while `below` (66.66% branches) fails. Requires the coverage
toolchains (`coverage` + `pytest`; vitest installed in the TS fixture).

```
cd packages/rust && cargo test --test packaging --test packaging_e2e
```

Expected: the packaging foundation's integration + e2e suites pass — a fixture
artifact containing a test file (`python_red`'s `widget_test.py`,
`typescript_red`'s `button.test.ts`) is flagged and the built binary exits `1`,
while a clean artifact exits `0`. No toolchain required (the scanner reads the
tree directly).

```
cd packages/rust && cargo test --test isolation --test isolation_e2e
```

Expected: the isolation tests pass — the red fixture's four out-of-module forms
(first-party cross-module, effectful `std`, external crate, ancestor reach) are
each flagged and the crate exits `1`, while the clean fixture (`super::` + an
injected trait double + `Cursor`) reports nothing and exits `0`.

```
cd packages/rust && cargo test --test packaging_wheel --test packaging_wheel_e2e
```

Expected: the Python wheel suites pass — `red.whl` (which ships
`widget/core_test.py`) is flagged and the binary exits `1`, while `clean.whl`
exits `0`. The wheels are generated by the committed `make_wheels.py`. No Python
toolchain required (the checker unzips the wheel directly).

```
cd packages/rust && cargo test --test packaging_npm --test packaging_npm_e2e
```

Expected: the TypeScript npm-tarball suites pass — `red.tgz` (which ships
`package/dist/widget.test.js`) is flagged and the binary exits `1`, while
`clean.tgz` exits `0`. The tarballs are generated by the committed
`make_tarballs.py`. No Node toolchain required (the checker unpacks the tarball
directly).

```
cd packages/rust && cargo test --test packaging_crate --test packaging_crate_e2e
```

Expected: the Rust crate-tarball suites pass — `widget-0.1.0.crate` (which ships
`widget-0.1.0/tests/integration.rs`) is flagged and the binary exits `1`, while
`clean-0.1.0.crate` exits `0`. The crates are generated by the committed
`make_crates.py`. No Cargo toolchain required (the checker unpacks the `.crate`
directly).

```
cd packages/rust && cargo test --test workflow --test workflow_e2e
```

Expected: the workflow guard's integration + e2e suites pass — the clean fixture (only
live subcommands, version pins, a `\`-continuation, and a comment that must not be read as
a call) reports nothing and exits `0`, while the red fixture (`unit location` and the flat
`unit-location`) flags both and the built binary exits `1`.
cd packages/rust && cargo test --test rust_integration_lint --test rust_integration_lint_e2e
```

Expected: the Rust integration tests pass — the red fixture's `#[double] use
widget::Renderer` (doubling the crate under test) is flagged and exits `1`, while
the clean fixture (runs `gadget::compute` for real, doubles only `rand`) reports
nothing and exits `0`.

```
cd packages/rust && cargo test --test isolation --test rust_integration_lint waived
cd packages/rust && cargo test --test isolation stale_exempt
```

Expected: the waiver tests pass (#102) — a `unit/waived` out-of-module call and an
integration `waived` first-party double, each lifted by a `[[rust.exempt]]` entry,
exit `0`; a stale exempt entry makes the run error.

```
cd packages/rust && cargo test --test integration_lint --test integration_lint_e2e first_party_patch
```

Expected: the Python integration-isolation tests pass (#42) — the red fixture's
`patch("myproject.ledger.record")` (first-party, declared in `pyproject.toml`) is
flagged and exits `1`, the clean fixture (mocks only `requests.post` /
`subprocess.run`) reports nothing and exits `0`, and the `waived` fixture's
`[[python.exempt]] rules = ["no-first-party-patch"]` lifts it back to `0`.

```
cd packages/rust && cargo test --test py_unit_isolation --test py_unit_isolation_e2e
```

Expected: the Python unit-isolation tests pass (#42, slice 2) — the red fixture
(imports `from myproject.ledger import record` without mocking it) is flagged and
exits `1`, the clean fixture (imports only the unit under test, patches the
collaborator by string) reports nothing and exits `0`, and the `waived` fixture's
`[[python.exempt]] rules = ["unmocked-collaborator"]` lifts it back to `0`.

```
cd packages/rust && cargo test --test py_unit_isolation --test py_unit_isolation_e2e external
```

Expected: the external-deps tests pass (#121, slice 3) — the `external/red` fixture
(imports un-mocked `requests` + `subprocess`) is flagged and exits `1`, the
`external/clean` fixture (mocks them by string, uses only pure `json`) reports
nothing and exits `0`, and `external/waived` lifts both back to `0`.

```
cd packages/rust && cargo test --test co_change --test co_change_e2e
```

Expected: the commit-scoped co-change tests pass (#33, #161) — driven through `unit
colocated-test --base`, in a throwaway git repo, editing or deleting a source without touching
its colocated test is flagged and the binary exits `1`; changing both, changing only a test,
adding a brand-new source, or touching an empty/`conftest.py` file exits `0`; `--base` adds this
on top of presence (an orphan source still fails) while a bare `unit colocated-test` is
presence-only; a `co-change` exemption lifts a stale source; and `--base --language rust` is
rejected. Requires `git`.

```
cd packages/rust && cargo test --test patch_coverage --test patch_coverage_e2e
```

Expected: the patch-coverage tests pass (#132) — in a throwaway git repo, a change
that adds an uncovered line or branch (or a brand-new untested file) is flagged and
the binary exits `1`, while rewording a covered line, or touching only a comment,
exits `0`; a `coverage` exemption lifts an uncovered file; an unresolvable `--base`
errors; and `--language rust` is rejected. Requires `coverage` + `pytest` + `git`
on `PATH`.

```
cd packages/rust && cargo test --test patch_coverage_ts --test patch_coverage_ts_e2e
```

Expected: the TypeScript patch-coverage tests pass (#135) — in a throwaway git repo
(with the fixtures' `node_modules` symlinked so `npx vitest` resolves), a change
that adds an uncovered statement or branch (or a brand-new untested file) is flagged
and the binary exits `1`, while rewording a covered line, or touching only a
comment, exits `0`; a `[typescript].coverage` exemption lifts an uncovered file; and
an unresolvable `--base` errors. Requires `git` + a Node toolchain with `vitest` and
`@vitest/coverage-v8` installed (`npm ci` under
`packages/rust/tests/fixtures/unit_coverage/typescript`).

```
cd packages/rust && cargo test --test patch_coverage_rust --test patch_coverage_rust_e2e
```

Expected: the Rust patch-coverage tests pass (#136) — in a throwaway cargo crate (a
git repo), a change that adds an arm whose body the suite never runs (or a brand-new
untested module) is flagged and the binary exits `1`, while rewording a covered line,
or touching only a comment, exits `0`; a `[rust].coverage` exemption lifts an
uncovered file; and an unresolvable `--base` errors. Requires `git` + `cargo-llvm-cov`
on `PATH`.

```
cd packages/rust && cargo test --test coverage_rust --test coverage_rust_e2e
```

Expected: the Rust coverage tests pass (#37) — the `above` fixture crate (every region and line
exercised by colocated inline tests) clears a 100 floor, `below` (one `else` arm left uncovered)
fails 100 but clears an 80 floor, and `exempt_cov` clears 100 once its untested `src/shim.rs` is
omitted by a `coverage` exemption (`--ignore-filename-regex`). Requires `cargo-llvm-cov`.

```
testing-conventions install && testing-conventions install
```

Expected: the first run writes the `<!-- testing-conventions:begin … -->` block into `AGENTS.md`
(creating the file if needed); the second run leaves the file byte-identical. Both exit `0` and
print nothing.
