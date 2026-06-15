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

### Changed

### Deprecated

### Removed

### Fixed
