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
earlier. (The rename left the library API untouched; #32 below then changes the
`missing_unit_tests` and `measure` signatures.)

Also adds the Python coverage rule (#26): `unit coverage --language python
--config <CONFIG> <PATH>` runs the unit suite under `coverage.py` (branch on,
`*_test.py` omitted) and enforces the config's `[python].coverage` floor, with the
supporting `testing_conventions::coverage` module (`measure`, `evaluate`,
`parse_report`, and the `Thresholds` / `CoverageReport` / `Outcome` types). Purely
additive — a new subcommand and module; nothing existing changes.

Also adds the `integration lint` command and its `lint` module (#48, #49): a
deterministic, AST-based lint on Python test files. `integration lint --language
python <PATH>` parses each test file (`*_test.py`, `test_*.py`, `conftest.py`) with
`rustpython_parser` and flags the first mocking-mechanism lint — `no-monkeypatch`, a
test/fixture that declares pytest's `monkeypatch` parameter. Purely additive: a new
command group (`integration`) and module (`testing_conventions::lint`); nothing
existing changes.

Finally, adds exemptions (#32) so the checker can be an honest blocking gate.
Exemptions are **config-driven and explicit** — there is no automatic name- or
shape-based exemption. `__init__.py`, re-export barrels, and launcher shims are
all subjects now; the only files exempt automatically are empty/comment-only ones
(no logic to test). For deliberate omissions the tool can't infer, list the file
in the one config file: a `[[<language>.exempt]]` entry with the `rules` it lifts
(`location` / `coverage`) and a required `reason`. A `location` exemption keeps
the file off the orphan list; a `coverage` exemption omits it from the coverage
denominator. The list is auditable (one place, in the config diff) and enforced:
a stale entry — a path that no longer exists — is a hard error, so it can't
silently rot. New config types `Rule` and `Exemption` plus `resolve_exempt()`;
`[<language>].coverage` becomes optional (a config can carry exemptions alone);
and `missing_unit_tests` / `coverage::measure` take the resolved exemptions
(signatures below).

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

Exemptions (#32) change the library API. Callers of these functions must pass the
new arguments:

| Function | Before | After |
| --- | --- | --- |
| `location::missing_unit_tests` | `(root, language)` | `(root, language, exempt)` — `exempt: &BTreeSet<String>` of `location`-rule paths |
| `coverage::measure` | `(root, thresholds)` | `(root, thresholds, omit)` — `omit: &[String]` of `coverage`-rule paths |

Build both with `config::resolve_exempt(root, exemptions, rule)`. Passing an empty
set/slice preserves the prior behavior. `[<language>].coverage` is now an
`Option`, so `config.python.coverage` becomes `config.python.coverage` of type
`Option<PythonCoverage>` — match/`?` it before reading the thresholds.

Anyone relying on `__init__.py` being exempt must add it to the config: a
non-empty `__init__.py` (one with re-exports or code) is now a subject. An
**empty** `__init__.py` needs nothing — empty/comment-only files are not
subjects.

### Deprecations removed

The `--lang` flag and its implicit `python` default are gone — a clean break, not
a deprecation cycle (pre-1.0, so no prior warning was shipped).

### Behavior changes without code changes

Omitting the language is now a usage error (exit code `2`) instead of defaulting to
`python`. Before, running the check on a TypeScript project without a flag scanned
for `*.py`, found none, and exited `0` — a silent false green; now the language
must be stated explicitly.

Exemptions (#32) change runtime behavior:

- `__init__.py` is no longer auto-exempt — a non-empty one without a colocated
  test (and without a config entry) is now reported as an orphan. Empty/comment-
  only files (any language) are non-subjects and never reported.
- `unit location` and `unit coverage` honor the config `exempt` list: a
  `location` entry keeps a file off the orphan list; a `coverage` entry omits it
  from the denominator. A reason-less or stale entry makes the run **error**
  rather than pass.
- CLI error output now prints the full cause chain (e.g. `error: exempt entry
  \`ghost.py\` matches no file under \`…\`: …`) instead of only the outermost
  context. Exit codes are unchanged.

### Verification

```
cd packages/rust && cargo test --test config_loader
```

Expected: the loader's integration tests pass — the canonical config loads, an
exempt-only config (no coverage thresholds) loads, and unknown-key, malformed,
missing-file, and reason-less-exemption configs are rejected.

```
cd packages/rust && cargo test --test unit_location
```

Expected: the location tests pass — clean fixtures report no orphans, red fixtures
report their missing twins, an empty `__init__.py` is not an orphan while a
content-bearing one is, config exemptions clear the listed files, and a stale
exempt entry errors.

```
cd packages/rust && cargo test --test coverage
```

Expected: the coverage tests pass — including the `exempt_cov` codebase clearing a
100 floor once its shim is omitted by a `coverage` exemption. Requires `coverage`
+ `pytest` on `PATH`.

```
cd packages/rust && cargo test --test integration_lint --test integration_lint_e2e
```

Expected: the lint's integration + e2e tests pass — the clean fixture reports no
violations and exits `0`, and the red fixture (a test taking `monkeypatch`) is
flagged and exits `1`.
