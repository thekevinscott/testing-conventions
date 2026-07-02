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

Adds the `testing_conventions.mutation` adapter (#248, part of #239): a Python module that drives
cosmic-ray via its `WorkDB` library API and emits the normalized mutation-result schema the rust core
gates on. The rust binary spawns it as `python3 -m testing_conventions.mutation.main` for `unit
mutation --language python`, replacing the `cosmic-ray` CLI orchestration on the rust side. Purely
additive on the wheel: the `bin` entry, the `pytest11` entry point, and the declared dependencies are
unchanged (`cosmic-ray` was already there). `cosmic_ray` is imported lazily, so the package imports
without the engine.

The wheel now declares the Python mutation/coverage engines (`cosmic-ray`, `coverage`) as runtime
dependencies, so installing testing-conventions brings them automatically — the `unit mutation` /
`unit coverage --language python` rules resolve the engine from the same environment instead of
requiring a separate install. Purely additive: the bundled CLI binary is unchanged, and the test
runner (`pytest`) is still the consumer's. Deps are unpinned so pip backtracks to an
interpreter-compatible release across 3.9+.

The wheel now ships an importable `testing_conventions` package with a pytest
plugin (`pytest11` entry point) alongside the CLI binary, applying the
recommended coverage floor to a local `pytest --cov` run unless the consumer has
configured it themselves. Purely additive: the CLI binary and its behavior are
unchanged, and the plugin only engages when a coverage run is active.

### Required changes

None.

### Deprecations removed

None.

### Behavior changes without code changes

None.

### Verification

None.
