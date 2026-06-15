# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Added

- `config` module — `load_config()` parses one TOML config file into the new
  in-memory `Config` schema (per-language `coverage` thresholds under the
  `[python]` / `[typescript]` / `[rust]` tables) and self-validates on load:
  unknown keys and malformed TOML are rejected. The parsed config is not
  consumed yet. (#12)
- `location` module — `missing_unit_tests(root, language)` walks a directory tree
  and returns every source file with no colocated unit test (sorted), enforcing
  the README's "Location & Naming" rule per `Language`:
  - **Python** (#15) — `foo.py` → `foo_test.py`; `*_test.py` and `__init__.py` are exempt.
  - **TypeScript** (#18) — `foo-bar.ts` → `foo-bar.test.ts` across `.ts`/`.tsx`/`.mts`/`.cts`; `*.test.{ts,tsx,mts,cts}` are tests, `*.d.ts`/`*.d.mts`/`*.d.cts` are ignored, nothing is exempt.
- `unit-location [--lang python|typescript] <PATH>` CLI subcommand — runs the check
  over a directory and exits non-zero, printing each source file missing its
  colocated test. `--lang` defaults to `python`. (#15, #18)
- `coverage` module + `unit coverage` CLI — enforce the Python coverage floor.
  `unit coverage --language python --config <CONFIG> <PATH>` runs the unit suite
  under `coverage.py` (branch on, `*_test.py` omitted from the denominator), then
  checks the total against the config's `[python].coverage` `fail_under` / `branch`
  and exits non-zero if below. Library API: `coverage::{measure, evaluate,
  parse_report, Thresholds, CoverageReport, Outcome}`. First rule to consume
  `load_config`. (#26)
- `waiver` module — auditable, reason-required exemptions for the deliberate
  omissions the tool can't infer (launcher shims, generated code). An in-file
  marker `testing-conventions:waiver(<scope>): <reason>` (scope `location`,
  `coverage`, or `all`) exempts the file it lives in; the reason is **required**
  and a malformed marker is a hard error, never a silent pass. Library API:
  `waiver::{Scope, Waiver, parse_waivers, waived_reason}`. (#32)

### Changed

- `unit location` no longer flags two new classes of file as orphans (#32): pure
  re-export **barrels** (a `.ts`/`.tsx`/`.mts`/`.cts` whose only statements are
  `export … from "…"`), matched by *shape* not name — the TypeScript analog of
  the already-exempt `__init__.py` — and any file carrying a `location`/`all`
  waiver. A waiver with no reason makes the check error instead of pass.
  `missing_unit_tests` keeps its signature.
- `unit coverage` now omits files carrying a `coverage`/`all` waiver from the
  coverage denominator (folded into coverage.py's `--omit`), so a deliberately
  uncovered file no longer fails the floor — with its reason recorded in the file
  rather than a silent ignore-glob. A waiver with no reason makes the run error.
  `coverage::measure` keeps its signature. (#32)

- **BREAKING** — the unit-test location check moved from the flat `unit-location`
  subcommand to `unit location`: rules now nest under their test kind (`unit` is a
  command group, `location` its first rule). The `--lang` flag is renamed to
  `--language` and is now **required** — the `python` default is gone, so omitting
  the language is a usage error instead of a silently-empty `python` scan. Migrate
  `unit-location --lang typescript src/` → `unit location --language typescript src/`;
  see [MIGRATIONS](./MIGRATIONS.md#unreleased). (#22)

### Deprecated

### Removed

### Fixed

- The CLI now prints the full error cause chain (`{err:#}`) instead of only the
  outermost context, so a wrapped failure shows both the *where* and the *why* —
  e.g. a malformed waiver reports the offending file *and* "waiver missing a
  reason". (#32)
