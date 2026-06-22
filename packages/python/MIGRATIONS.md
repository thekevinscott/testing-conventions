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
