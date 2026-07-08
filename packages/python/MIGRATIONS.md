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

The mutation adapter's per-mutant `pytest` command gains `-x` (#380, epic #366): a killed
mutant's suite run now stops at the test that kills it instead of running to completion.
cosmic-ray classifies outcomes by the test command's exit status, not its output, so a
surviving mutant's all-green run is unaffected and the survivor set is unchanged — only the
wall-clock cost of killed mutants drops.

Hardens the mutation baseline against slow suites (#395). The baseline guard raised only on a
`killed` outcome, so a baseline that timed out (or ended abnormally, `test_outcome=None`) passed
silently; with a fixed 30s per-run timeout, any suite slower than 30s timed the baseline out, then
every mutant timed out and dropped, and the adapter emitted an empty survivor set — a false green.
The guard now requires the clean suite to *pass* (`survived`), and the per-mutant timeout is derived
from the clean suite's observed runtime rather than the fixed 30s. The spawned adapter interface
(`python3 -m testing_conventions.mutation.main`) is unchanged; the internal `config`/`baseline`
helpers gained a `timeout` argument and an observed-runtime return.

### Required changes

None for consumers — the spawned adapter CLI is unchanged. (Internally, `config.render_config` /
`config.build_config` take a `timeout`, and `baseline.check_baseline` returns the observed runtime
and accepts an injectable `clock`; these modules are not a public surface.)

### Deprecations removed

None.

### Behavior changes without code changes

`unit mutation --language python`'s per-mutant test run now stops at the first failing test
instead of running the whole suite. A killed mutant's captured test output may show fewer
results than before; the killed/survived classification, the survivor set, and the gate's
pass/fail verdict are unchanged.

`unit mutation --language python`'s baseline is now stricter and its per-mutant timeout adaptive
(#395). A previously-silent baseline timeout or abnormal outcome now fails the run loudly (the
clean suite must report `survived`), so a suite too slow for its budget surfaces as an error rather
than an empty, falsely-green survivor set. The per-mutant timeout scales with the clean suite's
measured runtime instead of a fixed 30s, so a legitimately slow suite keeps its budget.

### Verification

None.
