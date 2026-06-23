# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Added

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

### Deprecated

### Removed

### Fixed
