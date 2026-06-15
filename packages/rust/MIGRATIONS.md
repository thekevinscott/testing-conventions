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

Also adds the first structural rule — unit-test location/naming — for two
languages. `missing_unit_tests(root, language)` walks a directory and returns
every source file with no colocated test, and `unit-location [--lang …] <PATH>`
runs that check and exits non-zero on any orphan. Python (#15): `foo.py` →
`foo_test.py`, `__init__.py` exempt. TypeScript (#18): `foo-bar.ts` →
`foo-bar.test.ts` (and `.tsx`), `*.d.ts` ignored. Purely additive.

### Required changes

None. New, additive API: `testing_conventions::config::{Config, load_config}`,
`testing_conventions::location::{missing_unit_tests, Language}`, and the
`unit-location [--lang python|typescript] <PATH>` CLI subcommand.

### Deprecations removed

None.

### Behavior changes without code changes

None.

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
`unit-location` exits non-zero on the red fixtures.
