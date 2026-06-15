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

Also adds **exemptions** (#32), so the checker can be an honest blocking gate. Exemptions are
config-driven and explicit — there's no automatic name- or shape-based exemption, so
`__init__.py`, re-export barrels, and launcher shims are all subjects now. Only empty/comment-
only files (no logic) are skipped automatically. For a deliberate omission, list the file in a
`[[<language>.exempt]]` config entry with the `rules` it lifts (`location` / `coverage`) and a
required `reason`; a stale entry (missing path) is a hard error. See the
[reference](/reference/#exemptions) and the [exemptions guide](/guide/exemptions).

### Required changes

The unit-location CLI was renamed and its language flag made required:

| Before (≤ v0.0.4)                      | After                                      |
| -------------------------------------- | ------------------------------------------ |
| `unit-location src/`                   | `unit location --language python src/`     |
| `unit-location --lang typescript src/` | `unit location --language typescript src/` |

`unit-location` → `unit location`; `--lang` → `--language`; `--language` is required (no
`python` default).

Exemptions (#32) also change the library API: `location::missing_unit_tests` gains an `exempt`
argument and `coverage::measure` gains an `omit` argument (build both with
`config::resolve_exempt`); `[<language>].coverage` becomes optional. Anyone relying on
`__init__.py` being exempt must add a non-empty one to the config — empty ones need nothing.

### Deprecations removed

The `--lang` flag and its implicit `python` default are gone — a clean pre-1.0 break, not a
deprecation cycle.

### Behavior changes without code changes

Omitting the language is now a usage error (exit `2`) instead of defaulting to `python` — so a
flag-less run on a TypeScript project no longer scans for `*.py`, finds none, and exits `0`.

Exemptions (#32) change runtime behavior: `__init__.py` is no longer auto-exempt (a non-empty
one without a test is now an orphan), `unit location` / `unit coverage` honor the config
`exempt` list, and a reason-less or stale exempt entry makes the run **error** rather than
pass. Empty/comment-only files of any language are non-subjects and never reported.

### Verification

```sh
testing-conventions unit location --help
testing-conventions unit location --language python src/   # exits 0 on a fully-paired tree
```
