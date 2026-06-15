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

Also adds the first structural rule. The `location` module's
`missing_unit_tests()` walks a directory and returns every Python source file
with no colocated `*_test.py`, and the new `unit-location <PATH>` subcommand
runs that check and exits non-zero on any orphan. Purely additive.

### Required changes

None. New, additive API: `testing_conventions::config::{Config, load_config}`,
`testing_conventions::location::missing_unit_tests`, and the
`unit-location <PATH>` CLI subcommand.

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

Expected: the location check's integration tests pass — the clean fixture
reports no orphans, the red fixture reports both missing twins, and the
`unit-location` subcommand exits non-zero on the red fixture.
