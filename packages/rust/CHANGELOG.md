# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Added

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
