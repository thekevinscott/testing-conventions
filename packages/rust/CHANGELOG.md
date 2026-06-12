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
- `location` module — `missing_unit_tests()` walks a directory tree and returns
  every Python source file (`*.py`) that has no colocated `*_test.py`, enforcing
  the README's "Location & Naming" rule. Test files (`*_test.py`) and the package
  marker (`__init__.py`) are exempt; the orphan list is sorted for deterministic
  output. (#15)
- `unit-location <PATH>` CLI subcommand — runs that check over a directory,
  printing each source file missing its colocated `_test.py` and exiting
  non-zero when any are missing. (#15)

### Changed

### Deprecated

### Removed

### Fixed
