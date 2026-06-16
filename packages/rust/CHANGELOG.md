# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Added

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
  `unit coverage --language python --config <CONFIG> <PATH>` runs the unit suite
  under `coverage.py` (branch on, `*_test.py` plus every `coverage`-exempt path
  omitted from the denominator), then checks the total against the config's
  `[python].coverage` `fail_under` / `branch` and exits non-zero if below. Library
  API: `coverage::{measure, evaluate, parse_report, Thresholds, CoverageReport,
  Outcome}`. (#26, #32)
- `lint` module + `integration lint` CLI — the first deterministic *lint* on test
  code. `integration lint --language python <PATH>` parses each Python test file
  (`*_test.py`, `test_*.py`, `conftest.py`) with `rustpython_parser` and walks the
  AST, exiting non-zero on any violation. Lints:
  **`no-monkeypatch`** (#49) — a test/fixture that declares the `monkeypatch`
  parameter; and **`no-inline-patch`** (#50) — a `patch(...)` / `patch.object(...)`
  / `patch.dict(...)` call in a test body (`with patch(...)` or a bare call), which
  belongs in a `pytest.fixture`. Library API:
  `testing_conventions::lint::{find_violations, Violation}`. (#48, #49, #50)

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
