# Migrations

Upgrade notes for breaking changes, mirroring each package's `MIGRATIONS.md`. Entries follow
the five-part structure (Summary · Required changes · Deprecations removed · Behavior changes ·
Verification) documented in `internals/repo.md`.

## Unreleased

### Summary

Adds the `config` module (a `Config` schema for per-language `coverage` thresholds plus a
validating `load_config()`), and reshapes the unit-test **location & naming** rule's CLI (#22).
The rule ships for Python (#15) and TypeScript (#18, including `.mts` / `.cts`); its command,
previously released as `unit-location [--lang …]` (v0.0.3 / v0.0.4), is now
`unit location --language <python|typescript> <PATH>` — rules nest under their test kind
(`unit` is a command group, `location` its first rule) and `--language` is required. Breaking
for anyone on v0.0.4 or earlier; the library API is unchanged.

### Required changes

The unit-location CLI was renamed and its language flag made required:

| Before (≤ v0.0.4)                      | After                                      |
| -------------------------------------- | ------------------------------------------ |
| `unit-location src/`                   | `unit location --language python src/`     |
| `unit-location --lang typescript src/` | `unit location --language typescript src/` |

`unit-location` → `unit location`; `--lang` → `--language`; `--language` is required (no
`python` default).

### Deprecations removed

The `--lang` flag and its implicit `python` default are gone — a clean pre-1.0 break, not a
deprecation cycle.

### Behavior changes without code changes

Omitting the language is now a usage error (exit `2`) instead of defaulting to `python` — so a
flag-less run on a TypeScript project no longer scans for `*.py`, finds none, and exits `0`.

### Verification

```sh
testing-conventions unit location --help
testing-conventions unit location --language python src/   # exits 0 on a fully-paired tree
```
