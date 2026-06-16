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

Also reshapes the colocated-test rule's CLI (#22) and then renames the rule
itself (#55). The rule ships for two languages — `missing_unit_tests` walks a
directory and returns every source file with no colocated test, and the CLI runs
it and exits non-zero on any orphan (Python #15: `foo.py` → `foo_test.py`;
TypeScript #18: `foo-bar.ts` → `foo-bar.test.ts` across `.ts`/`.tsx`/`.mts`/`.cts`,
`*.d.ts`/`*.d.mts`/`*.d.cts` ignored). The **command surface changes**, though: the
previously released `unit-location [--lang …]` (v0.0.3 / v0.0.4) first became
`unit location --language <python|typescript> <PATH>` (#22 — rules nest under
their test kind, `--language` required, the `python` default gone), and is now
`unit colocated-test …` (#55 — the rule was renamed `location` → `colocated-test`
so its name says what it checks: that a source file has a colocated,
matching-named unit test). This is a breaking change for anyone on an earlier CLI.
(#22 left the library API untouched; #32 below then changes the
`missing_unit_tests` and `measure` signatures, and #55 renames the `location`
module to `colocated_test`.)

Also adds the Python coverage rule (#26): `unit coverage --language python
--config <CONFIG> <PATH>` runs the unit suite under `coverage.py` (branch on,
`*_test.py` omitted) and enforces the config's `[python].coverage` floor, with the
supporting `testing_conventions::coverage` module (`measure`, `evaluate`,
`parse_report`, and the `Thresholds` / `CoverageReport` / `Outcome` types). Purely
additive — a new subcommand and module; nothing existing changes.

Also adds TypeScript coverage (#31), the twin of #26: `unit coverage --language
typescript --config <CONFIG> <PATH>` runs the unit suite under `vitest` v8
coverage and enforces the four `[typescript].coverage` thresholds (`lines` /
`branches` / `functions` / `statements`), excluding `*.test.*` and declaration
files — and any `coverage`-exempt path — from the denominator. vitest reports four
independent metrics rather than Python's single total, so it adds its own
`coverage::{measure_typescript, evaluate_typescript, parse_vitest_report}` plus
the `TypeScriptThresholds` / `VitestReport` / `VitestTotals` / `VitestMetric`
types, sharing the existing `Outcome`. Purely additive — `--language typescript`
previously errored as unimplemented; nothing existing changes.

Also adds the `integration lint` command and its `lint` module (#48, #49): a
deterministic, AST-based lint on Python test files. `integration lint --language
python <PATH>` parses each test file (`*_test.py`, `test_*.py`, `conftest.py`) with
`rustpython_parser` and flags the mocking-mechanism lints — `no-monkeypatch` (a
test/fixture that declares pytest's `monkeypatch` parameter), `no-inline-patch` (a
`patch(...)` call in a test body, which belongs in a `pytest.fixture`),
`no-environ-mutation` (direct `os.environ` mutation; set env via `patch.dict`), and
`no-constant-patch` (patching a module-global UPPER_CASE constant), which is waivable
per file via a `--config` `exempt` entry (`rules = ["no-constant-patch"]`, reusing #32).
Purely additive: a new command group (`integration`), a `--config` flag on `integration
lint`, the `config::Rule::NoConstantPatch` value, and the `testing_conventions::lint`
module; nothing existing changes.

Finally, adds exemptions (#32) so the checker can be an honest blocking gate.
Exemptions are **config-driven and explicit** — there is no automatic name- or
shape-based exemption. `__init__.py`, re-export barrels, and launcher shims are
all subjects now; the only files exempt automatically are empty/comment-only ones
(no logic to test). For deliberate omissions the tool can't infer, list the file
in the one config file: a `[[<language>.exempt]]` entry with the `rules` it lifts
(`colocated-test` / `coverage`) and a required `reason`. A `colocated-test`
exemption keeps the file off the orphan list; a `coverage` exemption omits it from
the coverage denominator. The list is auditable (one place, in the config diff) and enforced:
a stale entry — a path that no longer exists — is a hard error, so it can't
silently rot. New config types `Rule` and `Exemption` plus `resolve_exempt()`;
`[<language>].coverage` becomes optional (a config can carry exemptions alone);
and `missing_unit_tests` / `coverage::measure` take the resolved exemptions
(signatures below).

Zero-config coverage (#80): `unit coverage` now enforces the language's sane
default floor when coverage isn't configured — a missing config file, or a config
without the `[<language>].coverage` table — instead of erroring (Python
`branch = true, fail_under = 85`; TypeScript `lines = 80, branches = 75,
functions = 80, statements = 80`, the reasonable floors from the internals style
guides). This matches how `unit colocated-test` and `integration lint` already
treat an absent config, and lets the reusable workflow opt a new library into
every check with no config file. Additive: `Config`,
`{Python,TypeScript,Rust}Config`, `PythonCoverage`, and `TypeScriptCoverage` gain
`Default` impls; no existing signature changes.

### Required changes

The colocated-test CLI was renamed (twice, pre-1.0) and its language flag made
required. Update any invocation (CI steps, scripts, `npx`/`pip`/`cargo` wrappers)
to the current `unit colocated-test` form:

| Before                                                 | After                                            |
| ------------------------------------------------------ | ------------------------------------------------ |
| `unit-location src/` (≤ v0.0.4)                        | `unit colocated-test --language python src/`     |
| `unit-location --lang typescript src/` (≤ v0.0.4)      | `unit colocated-test --language typescript src/` |
| `unit location --language python src/` (v0.0.5–v0.0.8) | `unit colocated-test --language python src/`     |

- `unit-location` (flat, ≤ v0.0.4) / `unit location` (nested, v0.0.5–v0.0.8) →
  `unit colocated-test` (#22, #55).
- `--lang` → `--language`, which is required: there is no longer a `python` default.

Exemptions (#32) change the library API, and #55 renames the module these
colocated-test items live in — `testing_conventions::location` →
`testing_conventions::colocated_test` (the `Language` enum moves with it). Callers
must update the import path *and* pass the new arguments:

| Function | Before | After |
| --- | --- | --- |
| `missing_unit_tests` | `location::…(root, language)` | `colocated_test::…(root, language, exempt)` — `exempt: &BTreeSet<String>` of `colocated-test`-rule paths |
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
- `unit colocated-test` and `unit coverage` honor the config `exempt` list: a
  `colocated-test` entry keeps a file off the orphan list; a `coverage` entry omits
  it from the denominator. A reason-less or stale entry makes the run **error**
  rather than pass.
- CLI error output now prints the full cause chain (e.g. `error: exempt entry
  \`ghost.py\` matches no file under \`…\`: …`) instead of only the outermost
  context. Exit codes are unchanged.
- `unit coverage` no longer errors on a missing config file (or a config without
  the `[<language>].coverage` table): it enforces the language's default floor
  instead — Python 85 with branch on; TypeScript lines/functions/statements 80,
  branches 75. A `[<language>].coverage` table still overrides it. (#80)

### Verification

```
cd packages/rust && cargo test --test config_loader
```

Expected: the loader's integration tests pass — the canonical config loads, an
exempt-only config (no coverage thresholds) loads, and unknown-key, malformed,
missing-file, and reason-less-exemption configs are rejected.

```
cd packages/rust && cargo test --test colocated_test --test colocated_test_e2e
```

Expected: the colocated-test tests pass — clean fixtures report no orphans, red
fixtures report their missing twins, an empty `__init__.py` is not an orphan while a
content-bearing one is, config exemptions clear the listed files, and a stale
exempt entry errors. The renamed `unit colocated-test` subcommand parses while the
old `unit location` no longer does.

```
cd packages/rust && cargo test --test coverage
```

Expected: the coverage tests pass — including the `exempt_cov` codebase clearing a
100 floor once its shim is omitted by a `coverage` exemption. Requires `coverage`
+ `pytest` on `PATH`.

```
cd packages/rust && cargo test --test coverage_ts --test coverage_ts_e2e
```

Expected: the TypeScript coverage tests pass — `full` clears a 100 floor on all
four metrics, `above` fails 100 but clears the mid floor, `below` (100% lines but
~66% branches) fails the mid floor on branches, and `exempt_cov` clears 100 once
its shim is omitted by a `coverage` exemption. Requires Node with `vitest` +
`@vitest/coverage-v8` installed (run `npm ci` in
`tests/fixtures/unit_coverage/typescript`).

```
cd packages/rust && cargo test --test integration_lint --test integration_lint_e2e
```

Expected: the lint's integration + e2e tests pass — the clean fixture reports no
violations and exits `0`, and the red fixture (a test taking `monkeypatch`) is
flagged and exits `1`.

```
cd packages/rust && cargo test --test coverage_e2e --test coverage_ts_e2e
```

Expected: the coverage e2e suites pass, including the zero-config cases (#80) — a
`--config` pointing at a nonexistent file falls back to the default floor: Python
`full` and `above_85` (85.71%) pass while `below_85` (71.43%) fails; TypeScript
`above` passes while `below` (66.66% branches) fails. Requires the coverage
toolchains (`coverage` + `pytest`; vitest installed in the TS fixture).
