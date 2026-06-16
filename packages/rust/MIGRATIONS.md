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

Adds the `packaging` rule's foundation (#70): the `packaging` command and its
`packaging` module. `packaging --language <python|typescript> <PATH>` scans the
built artifact at `<PATH>` (an already-unpacked wheel or `dist/`) for that
language's test-file glob — Python `*_test.py`, TypeScript `*.test.*` — and exits
non-zero if any are present, enforcing the README "Packaging" rule that test files
must not ship. `packaging::scan(root, globs)` is the deterministic core; the
per-language *build* step that produces the artifact lands in #72 / #73 / #74
(the last also adding `--language rust`). Purely additive — a new command and
module; no existing signature or behavior changes.

Also adds the Rust colocated-test arm (#40): `unit colocated-test --language rust
<PATH>` now checks inline-`#[cfg(test)]` *presence* — a `src` file that defines a
function with a body but carries no inline `#[cfg(test)]` module is an orphan
(module-declaration / type-only files, and `tests/` / `benches/` / `examples/` /
`build.rs`, are not subjects). This is a **behavior change without a signature
change**: `unit colocated-test --language rust` previously exited 1 with an error
("Rust units are inline … see `unit isolation`") and now runs the check (exit 0 when
every source module has an inline test, 1 with an actionable list otherwise).
Additive on the library side: a new `colocated_test::missing_inline_tests(root,
exempt)`. Waivable per file via `[[rust.exempt]] rules = ["colocated-test"]`. Verify
with `testing-conventions unit colocated-test --language rust <crate>`: a crate whose
every behavior-bearing `src` module has an inline `#[cfg(test)]` exits 0; one that
doesn't lists each orphan and exits 1.

Also adds the Rust `unit isolation` rule (#44) and its `isolation`
module: a deterministic, `syn`-based lint on Rust test code. `unit isolation
--language rust <PATH>` parses each `*.rs` file under the crate root and flags a
call out of an inline `#[cfg(test)]` module's own module — `no-out-of-module-call`
(`crate::…`, `super::super::…`, an external crate from `Cargo.toml`, or effectful
`std`) — and a foreign `use` import — `no-out-of-module-import` (a glob of anything
but `super::*`, or a named import rooted at `crate::`, an external crate, or
effectful `std`). Purely additive: a new `unit` subcommand and module
(`testing_conventions::isolation::{find_violations, Violation, Language}`). The
shared `Violation` type moves to a new `violation` module and is re-exported from
`lint`, so `testing_conventions::lint::Violation` still resolves with **no code
change required**.

Also adds the Rust **integration** isolation lint (#44): `integration lint
--language rust <PATH>` flags `no-first-party-double` — a `#[double]`
(mockall_double) import of a first-party item (the crate under test or a `path`
dep) in a `tests/` integration crate, which must run first-party code for real;
doubling an external crate is fine. To add Rust without touching the file-pairing
`colocated_test::Language`, `integration lint`'s `--language` is now its own
`IntegrationLintLanguage` (python/typescript/rust). Purely additive — `--language
rust` is new, the python/typescript surface is unchanged, and the library gains
`testing_conventions::isolation::find_integration_violations`.

Also adds the first TypeScript lint (#43, #75): `integration lint --language
typescript <PATH>` extends the (previously Python-only) `integration lint` command
to TypeScript, parsing each `*.test.{ts,tsx,mts,cts}` file with `oxc` and flagging
**`no-first-party-mock`** — a `vi.mock()` / `vi.doMock()` of a first-party
(relative) module in an integration test, which must run first-party code for real
(third-party packages and Node built-ins may still be mocked). Purely additive: a
new `testing_conventions::ts` module (`find_integration_violations`, plus the shared
specifier classifier `classify` → `Origin`) and a new `--language typescript` arm on
`integration lint`; nothing existing changes.

Also extends `unit isolation` to TypeScript (#43, #76), the unit-direction
counterpart: `unit isolation --language typescript <PATH>` walks each
`*.test.{ts,tsx,mts,cts}` unit test and flags any runtime import that isn't
`vi.mock()`-ed (`unmocked-collaborator`), except the unit under test, type-only
imports, and the test runner (`vitest`). Additive — adds a `TypeScript` variant to
`isolation::Language` and the `testing_conventions::ts::find_unit_violations`
function; the Rust `unit isolation` behavior from #44 is unchanged.

Extends `packaging` to inspect a **Python wheel** (#72): `packaging --language
python <PATH>` now accepts a built `.whl` (a zip), unpacks it to a scratch
directory, and reuses `scan` to flag any `*_test.py` that shipped — `<PATH>` may
still be an already-unpacked directory. New library API `packaging::inspect(path,
globs)` (archive-or-directory → offenders relative to the artifact root); the
`packaging` command now calls it instead of `scan`. Additive — `scan` is
unchanged and the directory behavior is the same; new dependency `zip`.

Also extends `packaging` to a **TypeScript npm tarball** (#73): `packaging
--language typescript <PATH>` now accepts a built `.tgz` (an `npm pack` gzipped
tar), unpacks it, and reuses `scan` to flag any `*.test.*` that shipped in the
published `dist`. `inspect` now recognizes `.tgz` / `.tar.gz` in addition to
`.whl` and directories (the `.tar.gz` path is reused by #74's Rust `.crate` and
the Python sdist). Additive — new dependencies `flate2` + `tar`; no existing
signature changes.

Finally, `packaging --language rust` (#74) — the last packaging language.
`packaging` now accepts a Cargo `.crate` (`cargo package`, a gzipped tar) and
flags the crate-root **`tests/`** directory (`#[cfg(test)]` units compile out for
free; only the integration `tests/` needs a Cargo `exclude`). The scanner gains a
**directory pattern** (a pattern ending in `/` matches files under that dir)
alongside the file-name globs, and `colocated_test::Language` gains a `Rust`
variant so `--language rust` parses (`unit colocated-test` / `unit coverage` reject
it as separate items). Additive — a new enum variant and behavior; no existing
signature changes. Note for library consumers: matching exhaustively on the
public `Language` enum without a wildcard arm must add a `Rust` case.

Then `unit isolation --language typescript` also enforces **typed** mocks
(#43, #77): a `vi.mock(spec, factory)` whose factory carries no `vi.importActual<…>()`
type anchor is flagged `untyped-mock` (a bare `vi.mock(spec)` auto-mock and a typed
factory both pass). Behavior-only — `find_unit_violations` now reports the extra
rule; no signature changes. This completes #43's TypeScript isolation (#75/#76/#77).

Finally, adds the `workflow` guard (#92): a new `workflow` command and module that keeps the
reusable workflow's `@v0` consumption path from stranding. `workflow <PATH>` scans a
workflow file (or directory) for every `testing-conventions …` invocation and flags any
whose subcommand chain the binary no longer exposes (`no-unknown-subcommand`) — the failure
mode that broke `@v0` at 0.0.7 after the #55 `location` → `colocated-test` rename. Purely
additive: a new `testing_conventions::workflow` module (`invocations`, `unknown_subcommands`,
`check`, `Invocation`) and a `testing_conventions::command()` accessor for the binary's clap
command tree; nothing existing changes.

Also adds the e2e attestation nudge's first command (#17, #67): `e2e attest
'<command>'` runs the e2e suite, writes a committed `e2e-attestation.json` naming
the current commit (the command, a timestamp, the exit code, and the attested
SHA), and commits it on top — regardless of the command's outcome (force a run,
not a pass). Purely additive: a new `e2e` command group and the
`testing_conventions::e2e` module (`attest`, `Attestation`, `ATTESTATION_PATH`);
nothing existing changes. The CI-side `e2e verify` follows in #68.

Also makes the **isolation rules waivable** via the config `exempt` list (#102),
reusing the #32 machinery. `unit isolation` gains a `--config` flag (default
`testing-conventions.toml`); both it and `integration lint` now filter findings
against the config, so a `[[<lang>.exempt]]` entry naming an isolation rule —
`no-out-of-module-call`, `no-out-of-module-import`, `no-first-party-double`
(Rust), or `unmocked-collaborator`, `untyped-mock`, `no-first-party-mock` (TS) —
lifts it for that file (reason required; a stale entry still errors). Additive:
new `config::Rule` variants plus `Rule::id` / `Rule::from_id` and
`Config::rust_exemptions`; the `--config` flag is optional, so existing
invocations are unaffected.

Also adds **Python integration isolation** (#42): `integration lint --language
python <PATH>` now flags `no-first-party-patch` — a `patch(...)` whose string
target is first-party, e.g. `patch("ourpkg.mod.fn")`, since an integration test
must run first-party code for real (third-party packages and effectful stdlib stay
mockable). The first-party top-level package is read from the nearest
`pyproject.toml` `[project].name` (normalized), so a tree with no declared package
flags nothing. Behavior-only — `testing_conventions::lint::find_violations` now
reports the extra rule (no signature change), plus a new waivable `config::Rule`
variant `no-first-party-patch`; the python/typescript/rust surface is otherwise
unchanged.

Then adds that CI side (#17, #68): `e2e verify` reads the committed
`e2e-attestation.json` and passes iff its recorded SHA equals the latest code
commit — the newest commit touching any path other than the attestation file —
else exits non-zero with a run-`attest` hint. It never runs e2e and never judges
the recorded exit code/output. Purely additive: a new `e2e verify` subcommand and
`testing_conventions::e2e::{verify, Verification}`; nothing existing changes.

Also fixes a false positive in the TypeScript `unit isolation` typed-`vi.mock`
rule (#111): `vi.mock(spec, { spy: true })` — Vitest's options-object form, not a
factory — is no longer flagged `untyped-mock`. No API change; only a factory
*function* missing a `vi.importActual<…>` anchor is flagged now.

Also adds **Python unit isolation** (#42, slice 2): `unit isolation --language
python <PATH>` — the unit-direction twin of the above. It flags
`unmocked-collaborator` on a colocated unit test (`*_test.py` / `test_*.py`) that
imports a first-party collaborator without mocking it; the unit under test, the
test framework, pure stdlib, and type-only imports are never collaborators, and an
import counts as mocked when a `patch("…")` targets a matching last segment.
Additive: a new `Python` variant on `isolation::Language` (so `unit isolation
--language python` is now accepted) and `testing_conventions::lint::find_unit_isolation_violations`;
it emits the existing `unmocked-collaborator` rule, so the #102 waiver applies with
no new `config::Rule`. Nothing existing changes.

Also fixes the Python `conftest.py` handling (#112): `unit colocated-test` and
`unit coverage` treated `conftest.py` (pytest fixtures) as a unit-test subject —
flagging it as a missing-test orphan and counting it in the coverage denominator —
because only `*_test.py` was recognized as a non-subject. It is now test support:
never a subject, and omitted from the denominator alongside the test files. No API
change; the legacy `test_*.py` prefix stays unsupported (the colocated rule
requires `foo_test.py`).

Then extends Python `unit isolation` to **external** collaborators (#121, slice 3):
the same `unmocked-collaborator` rule now also flags an imported, un-mocked
third-party package or effectful-stdlib module (network / subprocess / process /
randomness / database / low-level OS), classifying import heads against an embedded
`sys.stdlib_module_names` set and a conservative effectful subset. Pure stdlib, the
`pytest` / `_pytest` / `mock` framework allowlist, and dual-nature heads (`os`,
`pathlib`, `datetime`, `time`, `io`) are not flagged. Behavior-only —
`find_unit_isolation_violations` reports the extra findings; no signature or rule-id
change, so the #102 waiver still applies.

Also makes the remaining Python integration lints **waivable** (#123):
`no-monkeypatch` (#49), `no-inline-patch` (#50), and `no-environ-mutation` (#51)
join `no-constant-patch` in the reason-required `[[python.exempt]]` escape hatch
(#32/#102) — they were the only blocking findings without one, because their ids
weren't `config::Rule` variants (so `apply_waivers` skipped them and the loader
rejected `rules = ["no-monkeypatch"]` outright). Additive: new `config::Rule`
variants `NoMonkeypatch` / `NoInlinePatch` / `NoEnvironMutation` (with `id()` /
`from_id()`); the lint behavior and every other surface are unchanged.

Also adds the commit-scoped `co-change` check (#33): `unit co-change --language
<python|typescript> --base <REF> <PATH>` diffs `<base>...HEAD` and flags any source file that
was **modified** (and still holds code) or **deleted** without its colocated test (`foo.py` →
`foo_test.py`, `foo.ts` → `foo.test.ts`) changing in the same diff — catching edits and removals
that leave the test stale. **Added** files aren't subjects (new code is the coverage floor's
job), a test file / empty file / `conftest.py` is never a subject, and a `co-change` exemption
lifts a source. Purely additive: a new `unit` subcommand, the `testing_conventions::co_change`
module (`stale_sources`), and a waivable `config::Rule::CoChange` (`co-change`); nothing existing
changes. `--language rust` is rejected (inline `#[cfg(test)]` units have no sibling test to go
stale).

Finally, adds the **coverage non-regression ratchet — Python** (#131, parent #46):
`unit coverage --language python` now also fails on a regression. A committed
`coverage-baseline.json` beside the measured tree records the last total per
language (`{ "python": { "percent_covered": 100.0 } }`), and a run whose Python
total drops below the recorded baseline fails even when it still clears the
configured floor. Purely additive — an absent baseline file means no ratchet
(floor-only, unchanged). New library API `coverage::{read_baseline,
evaluate_ratchet, measure_report, Baseline, PythonBaseline, BASELINE_PATH}`; the
existing `measure` / `evaluate` are unchanged. The TypeScript / Rust arms and the
explicit baseline-record step are later slices.

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
- `integration lint --language typescript` (#43, #75) previously errored
  (`supports --language python only for now`); it now parses the TypeScript test
  files and runs the `no-first-party-mock` lint.
- `unit isolation --language typescript` no longer reports `untyped-mock` for the
  options-object mock `vi.mock(spec, { spy: true })` (Vitest ≥2). The spy form
  wraps the real module and can't drift, like a bare auto-mock; only a factory
  *function* without a `vi.importActual<…>` anchor is flagged. (#111)
- `unit colocated-test --language python` no longer reports `conftest.py` as a
  missing-test orphan, and `unit coverage --language python` omits `conftest.py`
  from the denominator (alongside `*_test.py`). conftest.py is pytest support,
  never a subject. (#112)
- A `[[python.exempt]]` entry naming `no-monkeypatch`, `no-inline-patch`, or
  `no-environ-mutation` is now accepted and waives that lint for the file. Previously
  the loader **rejected** those ids as an unknown `rules` variant (and even parsed,
  `integration lint` could never have waived them). A reason-less or stale entry still
  errors. (#123)
- `unit coverage --language python` now also enforces a **non-regression ratchet**
  (#131): with a `coverage-baseline.json` beside `<PATH>`, a run whose total drops
  below the recorded `python` baseline exits non-zero — printing `coverage NN.NN%
  regressed below the recorded baseline MM.MM%` — even when the floor is still met.
  Without the file, behavior is unchanged.

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
cd packages/rust && cargo test --test e2e_attest --test e2e_attest_e2e
```

Expected: the `e2e attest` tests pass — in a throwaway git repo, `attest` names
HEAD, writes `e2e-attestation.json`, and commits it on top, exiting `0` even when
the wrapped command fails (force a run, not a pass). Requires `git`.

```
cd packages/rust && cargo test --test integration_lint --test integration_lint_e2e
```

Expected: the integration-lint tests pass — including the `monkeypatch`, `inline_patch`,
and `environ` `waived` fixtures, each identical to its red fixture but cleared to exit `0`
by a reason-required `[[python.exempt]]` entry, alongside the existing `constant_patch`
and `no_first_party_patch` waivers. (#123)

```
cd packages/rust && cargo test --test integration_lint --test integration_lint_e2e
```

Expected: the lint's integration + e2e tests pass — the clean fixture reports no
violations and exits `0`, and the red fixture (a test taking `monkeypatch`) is
flagged and exits `1`.

```
cd packages/rust && cargo test --test ts_integration_lint --test ts_integration_lint_e2e
```

Expected: the TypeScript lint's integration + e2e tests pass — the clean fixture
(mocks only third-party packages and Node built-ins) reports no violations and exits
`0`, and the red fixture (a first-party `vi.mock` / `vi.doMock`) is flagged and exits
`1`.

```
cd packages/rust && cargo test --test unit_isolation --test unit_isolation_e2e
```

Expected: the TypeScript unit-isolation tests pass — both rules. For
`unmocked-collaborator`, the clean fixture (every collaborator `vi.mock()`-ed) exits
`0` and the red fixture (an un-mocked `./formatter` and `lodash`) is flagged. For
`untyped-mock` (#77), the `untyped_mock` red fixture (a `vi.mock` factory with no
`vi.importActual<…>` anchor) is flagged while its clean fixture (a typed factory and a
bare auto-mock) exits `0`.

```
cd packages/rust && cargo test --test coverage_e2e --test coverage_ts_e2e
```

Expected: the coverage e2e suites pass, including the zero-config cases (#80) — a
`--config` pointing at a nonexistent file falls back to the default floor: Python
`full` and `above_85` (85.71%) pass while `below_85` (71.43%) fails; TypeScript
`above` passes while `below` (66.66% branches) fails. Requires the coverage
toolchains (`coverage` + `pytest`; vitest installed in the TS fixture).

```
cd packages/rust && cargo test --test packaging --test packaging_e2e
```

Expected: the packaging foundation's integration + e2e suites pass — a fixture
artifact containing a test file (`python_red`'s `widget_test.py`,
`typescript_red`'s `button.test.ts`) is flagged and the built binary exits `1`,
while a clean artifact exits `0`. No toolchain required (the scanner reads the
tree directly).

```
cd packages/rust && cargo test --test isolation --test isolation_e2e
```

Expected: the isolation tests pass — the red fixture's four out-of-module forms
(first-party cross-module, effectful `std`, external crate, ancestor reach) are
each flagged and the crate exits `1`, while the clean fixture (`super::` + an
injected trait double + `Cursor`) reports nothing and exits `0`.

```
cd packages/rust && cargo test --test packaging_wheel --test packaging_wheel_e2e
```

Expected: the Python wheel suites pass — `red.whl` (which ships
`widget/core_test.py`) is flagged and the binary exits `1`, while `clean.whl`
exits `0`. The wheels are generated by the committed `make_wheels.py`. No Python
toolchain required (the checker unzips the wheel directly).

```
cd packages/rust && cargo test --test packaging_npm --test packaging_npm_e2e
```

Expected: the TypeScript npm-tarball suites pass — `red.tgz` (which ships
`package/dist/widget.test.js`) is flagged and the binary exits `1`, while
`clean.tgz` exits `0`. The tarballs are generated by the committed
`make_tarballs.py`. No Node toolchain required (the checker unpacks the tarball
directly).

```
cd packages/rust && cargo test --test packaging_crate --test packaging_crate_e2e
```

Expected: the Rust crate-tarball suites pass — `widget-0.1.0.crate` (which ships
`widget-0.1.0/tests/integration.rs`) is flagged and the binary exits `1`, while
`clean-0.1.0.crate` exits `0`. The crates are generated by the committed
`make_crates.py`. No Cargo toolchain required (the checker unpacks the `.crate`
directly).

```
cd packages/rust && cargo test --test workflow --test workflow_e2e
```

Expected: the workflow guard's integration + e2e suites pass — the clean fixture (only
live subcommands, version pins, a `\`-continuation, and a comment that must not be read as
a call) reports nothing and exits `0`, while the red fixture (`unit location` and the flat
`unit-location`) flags both and the built binary exits `1`.
cd packages/rust && cargo test --test rust_integration_lint --test rust_integration_lint_e2e
```

Expected: the Rust integration tests pass — the red fixture's `#[double] use
widget::Renderer` (doubling the crate under test) is flagged and exits `1`, while
the clean fixture (runs `gadget::compute` for real, doubles only `rand`) reports
nothing and exits `0`.

```
cd packages/rust && cargo test --test isolation --test rust_integration_lint waived
cd packages/rust && cargo test --test isolation stale_exempt
```

Expected: the waiver tests pass (#102) — a `unit/waived` out-of-module call and an
integration `waived` first-party double, each lifted by a `[[rust.exempt]]` entry,
exit `0`; a stale exempt entry makes the run error.

```
cd packages/rust && cargo test --test integration_lint --test integration_lint_e2e first_party_patch
```

Expected: the Python integration-isolation tests pass (#42) — the red fixture's
`patch("myproject.ledger.record")` (first-party, declared in `pyproject.toml`) is
flagged and exits `1`, the clean fixture (mocks only `requests.post` /
`subprocess.run`) reports nothing and exits `0`, and the `waived` fixture's
`[[python.exempt]] rules = ["no-first-party-patch"]` lifts it back to `0`.

```
cd packages/rust && cargo test --test py_unit_isolation --test py_unit_isolation_e2e
```

Expected: the Python unit-isolation tests pass (#42, slice 2) — the red fixture
(imports `from myproject.ledger import record` without mocking it) is flagged and
exits `1`, the clean fixture (imports only the unit under test, patches the
collaborator by string) reports nothing and exits `0`, and the `waived` fixture's
`[[python.exempt]] rules = ["unmocked-collaborator"]` lifts it back to `0`.

```
cd packages/rust && cargo test --test py_unit_isolation --test py_unit_isolation_e2e external
```

Expected: the external-deps tests pass (#121, slice 3) — the `external/red` fixture
(imports un-mocked `requests` + `subprocess`) is flagged and exits `1`, the
`external/clean` fixture (mocks them by string, uses only pure `json`) reports
nothing and exits `0`, and `external/waived` lifts both back to `0`.

```
cd packages/rust && cargo test --test co_change --test co_change_e2e
```

Expected: the commit-scoped co-change tests pass (#33) — in a throwaway git repo,
editing or deleting a source without touching its colocated test is flagged and the
binary exits `1`; changing both, changing only a test, adding a brand-new source, or
touching an empty/`conftest.py` file exits `0`; a `co-change` exemption lifts a stale
source; and `--language rust` is rejected. Requires `git`.

```
cd packages/rust && cargo test --test coverage_ratchet --test coverage_ratchet_e2e
```

Expected: the non-regression ratchet tests pass (#131) — `ratchet_regressed`
(~86%, clears the 85 floor) regresses below its committed 100% baseline and exits
`1`, while `ratchet_clean` (100% meeting a 100% baseline) exits `0`. Requires
`coverage` + `pytest` on `PATH`.
