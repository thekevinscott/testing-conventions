# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Added

- **A `testing_conventions.mutation` adapter** (#248, epic #239). The wheel now ships a Python module
  that drives cosmic-ray through its `WorkDB` library API (`commands.init` → `execute`, read the
  `WorkResult`s) and emits the normalized mutation-result schema the rust core gates on — one function
  per file (`parse_args`, `normalize`, `config`, `baseline`, `session`, `cli`, `main`), mirroring the
  TypeScript adapter's layout. The rust binary spawns it as `python3 -m
  testing_conventions.mutation.main` for `unit mutation --language python`: the tool drives the
  engine, and the project supplies its own test runner (pytest). `cosmic_ray` is imported lazily so
  the package imports without the engine. Additive — the `bin` entry and the `pytest11` entry point
  are unchanged, and `cosmic-ray` was already a runtime dependency.
- **The Python mutation/coverage engines now ship with the wheel.** `cosmic-ray` and `coverage` are
  declared as runtime dependencies, so a `pip install` / `uvx` of testing-conventions brings them and
  `unit mutation`/`unit coverage --language python` resolve them from the same environment — no
  separate engine install. The test runner (`pytest`) stays the consumer's, since it runs their suite.
  Left unpinned so pip selects an interpreter-compatible release across the supported 3.9+ range.
- A `testing_conventions` pytest plugin, shipped in the wheel and auto-loaded via
  the `pytest11` entry point. On `pip install testing-conventions` it brings the
  project's recommended coverage floor to a local `pytest --cov` run — branch
  coverage on, `fail_under = 100`, and test files omitted — as *defaults* the
  consumer's own coverage config or CLI flags override. The wheel is now a mixed
  bin + Python project (maturin `python-source`); the bundled CLI binary is
  unchanged. ([#218](https://github.com/thekevinscott/testing-conventions/issues/218))

### Changed

- **The per-mutant pytest run stops at the first failure** (`-x`) (#380, epic #366). A killed
  mutant's suite run ends at the test that kills it; a surviving mutant's all-green run is
  unchanged, and cosmic-ray classifies outcomes by exit status, so the survivor set is identical —
  only faster. The baseline check runs the same command; a passing baseline never triggers `-x`.

### Deprecated

### Removed

### Fixed

- **The mutation baseline no longer false-greens on a slow suite** (#395). The baseline guard
  raised only when the unmutated run reported `killed`, so a baseline that timed out (or ended
  abnormally, `test_outcome=None`) passed silently; combined with a fixed 30s per-run timeout, any
  suite slower than 30s made the baseline time out, then every mutant time out and drop, and the
  adapter wrote an empty survivor set — a green gate over zero mutants. The guard now requires the
  clean suite to *pass* (`survived`): a `killed`, timed-out, or abnormal baseline is a loud error.
  The per-mutant timeout is derived from the clean suite's observed runtime (`observed x
  multiplier`, floored) instead of the fixed 30s, so a legitimately slow suite earns a
  proportionally larger budget. The spawned `python3 -m testing_conventions.mutation.main` interface
  is unchanged.
