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

Adds the `config` module: a `Config` schema holding the per-language `coverage`
thresholds (`[python]` / `[typescript]` / `[rust]`), plus `load_config()`, which
reads one TOML file into it and validates the config itself (the self-guard) —
unknown keys and malformed TOML are rejected rather than silently accepted.
Purely additive; nothing consumes the parsed config yet.

Also reshapes the unit-test location/naming rule's CLI (#22). The rule itself
ships for two languages — `missing_unit_tests(root, language)` walks a directory
and returns every source file with no colocated test, and the CLI runs it and
exits non-zero on any orphan (Python #15: `foo.py` → `foo_test.py`, `__init__.py`
exempt; TypeScript #18: `foo-bar.ts` → `foo-bar.test.ts` across
`.ts`/`.tsx`/`.mts`/`.cts`, `*.d.ts`/`*.d.mts`/`*.d.cts` ignored). The
**command surface changes**, though: the previously released `unit-location
[--lang …]` (v0.0.3 / v0.0.4) becomes `unit location --language
<python|typescript> <PATH>` — rules now nest under their test kind (`unit` is a
command group, `location` its first rule) and `--language` is required (the
`python` default is gone). This is a breaking change for anyone on v0.0.4 or
earlier; the library API (`missing_unit_tests`, `Language`, `Config`,
`load_config`) is unchanged.

Also adds the Python coverage rule (#26): `unit coverage --language python
--config <CONFIG> <PATH>` runs the unit suite under `coverage.py` (branch on,
`*_test.py` omitted) and enforces the config's `[python].coverage` floor, with the
supporting `testing_conventions::coverage` module (`measure`, `evaluate`,
`parse_report`, and the `Thresholds` / `CoverageReport` / `Outcome` types). Purely
additive — a new subcommand and module; nothing existing changes.

### Required changes

The unit-location CLI was renamed and its language flag made required. Update any
invocation (CI steps, scripts, `npx`/`pip`/`cargo` wrappers):

| Before (≤ v0.0.4)                      | After                                      |
| -------------------------------------- | ------------------------------------------ |
| `unit-location src/`                   | `unit location --language python src/`     |
| `unit-location --lang typescript src/` | `unit location --language typescript src/` |

- `unit-location` → `unit location` (a `location` subcommand under the new `unit` group).
- `--lang` → `--language`.
- `--language` is required: there is no longer a `python` default to fall back on.

The library API is unchanged — `testing_conventions::config::{Config, load_config}`
and `testing_conventions::location::{missing_unit_tests, Language}` keep their
signatures.

### Deprecations removed

The `--lang` flag and its implicit `python` default are gone — a clean break, not
a deprecation cycle (pre-1.0, so no prior warning was shipped).

### Behavior changes without code changes

Omitting the language is now a usage error (exit code `2`) instead of defaulting to
`python`. Before, running the check on a TypeScript project without a flag scanned
for `*.py`, found none, and exited `0` — a silent false green; now the language
must be stated explicitly.

### Verification

```
cd packages/rust && cargo test --test config_loader
```

Expected: the loader's integration tests pass — the canonical config loads into
memory, and unknown-key, malformed, and missing-file configs are rejected.

```
cd packages/rust && cargo test --test unit_location
```

Expected: the location check's integration tests pass for both languages — each
clean fixture reports no orphans, each red fixture reports its missing twins, and
`unit location` exits non-zero on the red fixtures.
