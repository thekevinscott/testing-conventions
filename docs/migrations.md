# Migrations

Upgrade notes for breaking changes, mirroring each package's `MIGRATIONS.md`. Entries follow
the five-part structure (Summary · Required changes · Deprecations removed · Behavior changes ·
Verification) documented in `internals/repo.md`.

## Unreleased

### Summary

Adds the `config` module (a `Config` schema for per-language `coverage` thresholds plus a
validating `load_config()`), and the first structural rule — unit-test **location & naming** —
for Python (#15) and TypeScript (#18, including `.mts` / `.cts`), exposed as the
`unit-location [--lang python|typescript] <PATH>` subcommand. All additive.

### Required changes

None — new, additive API and a new subcommand. Nothing existing changed.

### Deprecations removed

None.

### Behavior changes without code changes

None.

### Verification

```sh
testing-conventions unit-location --help
testing-conventions unit-location src/   # exits 0 on a fully-paired tree
```
