# testing-conventions

**Enforce testing conventions in libraries (Python, TypeScript, and Rust)**

`testing-conventions` is a config-driven standard for how tests are structured,
isolated, and measured in a library. One config file drives every rule, and rules
are enforced deterministically in CI: a violation fails the build. It's primarily
useful for enforcing agent (LLM) behavior.

<!-- Single source of truth for the rule list: docs/index.md pulls the region below via VitePress @include. Keep it in sync with the #3 checklist. -->
<!-- #region rules -->
## Rules at a glance

Every rule is a CLI command that fails CI on a violation, and links to its reference entry. Where a rule isn't shipped for all three languages yet, that's noted inline.

**Unit**

- [`unit colocated-test`](https://thekevinscott.github.io/testing-conventions/reference/#unit-colocated-test) — every source file has a colocated, matching-named unit test (Python, TypeScript, Rust); with `--base`, a source changed in the diff must also change its colocated test (co-change; Python, TypeScript; [#33](https://github.com/thekevinscott/testing-conventions/issues/33)).
- [`unit coverage`](https://thekevinscott.github.io/testing-conventions/reference/#unit-coverage) — enforce a coverage floor on the unit suite (Python, TypeScript, Rust); with `--base`, the same floor is measured over the changed lines of a `<base>...HEAD` diff instead of the whole tree ([#162](https://github.com/thekevinscott/testing-conventions/issues/162)).
- [`unit lint`](https://thekevinscott.github.io/testing-conventions/reference/#unit-lint) — a unit test mocks every collaborator: no out-of-module calls or imports (Rust); no un-mocked first-party or external collaborators (Python, TypeScript); typed mocks (TypeScript).

**Integration**

- [`integration lint`](https://thekevinscott.github.io/testing-conventions/reference/#integration-lint) — integration tests run first-party code for real: no first-party mock, double, or patch (Python, TypeScript, Rust); plus Python mock-mechanism hygiene (`no-monkeypatch`, `no-inline-patch`, `no-environ-mutation`, `no-constant-patch`).

**Packaging**

- [`packaging`](https://thekevinscott.github.io/testing-conventions/reference/#packaging) — test files never ship in the built artifact (wheel, sdist, npm tarball, crate).

**E2E**

- [`e2e attest`](https://thekevinscott.github.io/testing-conventions/reference/#e2e-attest) / [`e2e verify`](https://thekevinscott.github.io/testing-conventions/reference/#e2e-verify) — `attest` runs the e2e suite locally and records the commit it ran against; `verify` checks that receipt in CI and never runs e2e.
<!-- #endregion rules -->

## The three kinds of tests

Tests assert the behavior of code. This standard recognizes three kinds:

- **Unit:** cheap and plentiful, low confidence on their own. They anchor refactors
  in an agentic workflow.
- **Integration:** treats the system as a black box. First-party code runs for real;
  external dependencies are mocked (databases, LLMs, the filesystem, the clock).
- **E2E:** like integration, but with no mocks. Not run in CI. An agent runs them to
  confirm real third-party contracts still hold.

## Rules

Each rule states what it enforces, why, and how it varies by language. **Checked**
says how it's verified. Most rules are deterministic checks run in CI from the
config. Where a rule is a structural convention rather than its own gate (the
integration/e2e folder layout), **Checked** says so.

### Unit

#### Colocated Test

**Rule:** unit tests are colocated with the code they test, and named after it.

**Why:** colocation makes the unit/integration boundary structural, by location
rather than a tag or marker. 1:1 naming means an orphaned test can't hide.

- **Python:** `foo.py` → `foo_test.py`, side by side in `src/`.
- **TypeScript:** `foo-bar.ts` → `foo-bar.test.ts`, side by side in `src/`.
- **Rust:** no separate file. Units are an inline `#[cfg(test)]` module in the same `.rs` file, so colocation and 1:1 naming are automatic.

**Checked:** deterministic (glob + name match in Python/TS; `#[cfg(test)]` presence in Rust).

#### Isolation

**Rule:** everything except the unit under test is mocked.

**Why:** a unit test that touches a real collaborator behaves like an integration
test, so unrelated refactors break the wrong tests.

- **Python:** mock every first-party collaborator the unit imports. `autospec=True` keeps each mock's signature matching the real object.
- **TypeScript:** `vi.mock()` each collaborator, typed so it can't drift from the source.
- **Rust:** no import monkeypatching. Inject a trait (hand-rolled or `mockall`). Idiomatic Rust keeps a pure core with I/O at the edges, so many units need no mocks, and the compiler guarantees the double matches the trait.

The TypeScript typed-mock pattern:

```ts
vi.mock('./service', async () => {
  const actual = await vi.importActual<typeof import('./service')>('./service');
  return { ...actual, fetchUser: vi.fn() };
});
```

**Checked:** deterministic. Python/TS flag any un-mocked first-party/external import. Rust flags any call out of the test's own module (cross-module, external crate, or effectful `std`); [`dylint`](https://github.com/trailofbits/dylint) gives full name-resolution precision.

#### Co-change

**Rule:** when a source file changes, its colocated unit test changes with it.

**Why:** an edit or removal that leaves the colocated test untouched lets the test
go stale; the test should move with the code it pins. Adding new code is the
coverage floor's job, so this targets edits and removals.

- **Python:** a modified or deleted `foo.py` requires `foo_test.py` in the same diff.
- **TypeScript:** a modified or deleted `foo.ts` requires `foo.test.ts` in the same diff.
- **Rust:** not applicable. Units are an inline `#[cfg(test)]` module in the same file, so the test moves with the source.

**Checked:** commit-scoped and deterministic. `unit colocated-test --base <ref>` diffs `<ref>...HEAD` and flags any changed source whose colocated test didn't change — an opt-in, additive scope of the colocated-test command (tree-wide presence still runs). Added files and exempt sources are excused.

### Integration

#### Location

**Rule:** integration tests live in a dedicated folder, separate from the unit
suite.

**Why:** a structural home keeps black-box tests out of the unit suite that
coverage is measured on.

- **Python:** `tests/integration/`, files end in `_test.py` (non-test helpers omit the suffix).
- **TypeScript:** `tests/integration/`, files end in `.test.ts`.
- **Rust:** `tests/` at the crate root. Each file compiles as its own crate, so the location is the signal.

**Checked:** the boundary is enforced behaviorally, not by a folder gate.
`unit lint` requires unit tests to mock every collaborator, `integration lint`
requires integration tests to run first-party code for real, and `unit coverage`
measures only the colocated unit suite. The `tests/integration/` folder is a
convention, not a separately checked rule.

#### External Dependencies

**Rule:** every external dependency is mocked; first-party code runs for real.
**External** means any package dependency plus effectful standard-library APIs
(filesystem, clock, randomness, network, subprocess, env). A whitelist lets
specific dependencies through unmocked.

**Why:** an un-mocked external call makes the test slow, flaky, or a charge on
someone's bill.

- **Python:** patch third-party imports and effectful stdlib (`open`, `datetime`, `subprocess`).
- **TypeScript:** `vi.mock()` third-party packages and Node built-ins (`fs`, `Date`, `child_process`).
- **Rust:** external crates and `std` I/O are mocked behind injected traits (e.g. `mockall`).

**Note:** if the library ships a CLI, back it with an SDK and point integration
tests at the SDK. Keep the CLI a thin wrapper.

**Checked:** deterministic. Python/TS flag any un-mocked external import. Rust flags external-crate or effectful-`std` use that isn't behind an injected trait or whitelisted; [`dylint`](https://github.com/trailofbits/dylint) gives full precision.

#### Mocking mechanism (Python only)

**Rule:** Python integration tests get three additional mechanism-hygiene lints:

- `no-monkeypatch`: patch with `unittest.mock` in a `pytest.fixture` rather than pytest's `monkeypatch`.
- `no-inline-patch`: a `patch(...)` belongs in a fixture, not a test body.
- `no-environ-mutation`: set env with `patch.dict(os.environ, {...})`, never mutate `os.environ` in place.

**Why Python only:** each mechanism these target is a pytest/Python idiom with no
TypeScript or Rust analog. `monkeypatch` is a pytest fixture, fixture-vs-inline
patching is pytest's model, and in-place `os.environ` mutation is Python-specific.
TypeScript's "don't hand-roll an untyped mock" concern is already the `untyped-mock`
unit rule, and Rust injects trait doubles the compiler checks. So TypeScript and Rust
have no mechanism-hygiene integration lints: their `integration lint` is the
first-party direction check alone (`no-first-party-mock` / `no-first-party-double`).

**Checked:** deterministic, Python only (an AST walk of each integration test file).

### E2E

**Rule:** e2e tests live in a dedicated folder and run with no mocks.

**Why:** they confirm real external contracts still hold. They're for an agent to
run on demand, not for CI.

- **Python:** `tests/e2e/`, files end in `_test.py`.
- **TypeScript:** `tests/e2e/`, files end in `.test.ts`.
- **Rust:** under `tests/`, typically driving the built binary (`CARGO_BIN_EXE_<name>` or `assert_cmd`).

**Attestation:** CI never runs e2e (real contracts are slow, flaky, and cost
money), but the suite shouldn't silently rot either. The agent runs it locally and
attests that it did:

```
testing-conventions e2e attest '<your e2e command>'
```

`attest` runs the suite and commits an `e2e-attestation.json` recording the command,
the exit code, and the commit it ran against. In CI, `testing-conventions e2e verify`
passes only if that attestation names the latest code commit. Push code without
re-attesting and it goes stale, so CI prompts you to re-run e2e. CI confirms someone
ran the suite against this code; it never runs the suite itself.

**Checked:** the e2e location is a convention, not its own gate, and CI never runs
the suite. CI checks the attestation: `e2e verify` requires the committed
`e2e-attestation.json` to name the latest code commit.

### Coverage

**Rule:** coverage floors are enforced on the **unit suite only** and exclude
test code from the denominator. The thresholds are set per library, in
each tool's native coverage primitives.

**Why:** coverage measures execution, not assertion. Measuring it on anything but
real unit tests lets integration tests inflate the number.

- **Python:** `pytest --cov` (coverage.py). Set `branch` and `fail_under`; omit `*_test.py`.
- **TypeScript:** `vitest` coverage (v8/istanbul). Set the `lines` / `branches` / `functions` / `statements` thresholds; exclude `*.test.ts`.
- **Rust:** `cargo llvm-cov`. Set `regions` / `lines` (branch coverage is still experimental). Inline units can't be excluded by filename; use `#[coverage(off)]` (toolchain-dependent).

**Checked:** deterministic (run coverage; compare to the configured thresholds).

### Packaging

**Rule:** test files never ship in the built package.

**Why:** colocated unit tests live in `src/`, so packaging has to strip them.
Shipping test code bloats the artifact and leaks fixtures.

- **Python:** exclude `*_test.py` from the wheel/sdist (`build_py` + `MANIFEST.in`).
- **TypeScript:** exclude `*.test.*` from the published `dist` (source `.test.{ts,tsx,mts,cts}` and any compiled `.test.js`).
- **Rust:** free in the compiled artifact. `#[cfg(test)]` is stripped and `tests/` isn't built for consumers (add a Cargo `exclude` only to keep them out of the source tarball).

**Checked:** deterministic (inspect the built artifact for test files).

## Exemptions

A blocking gate needs an escape hatch for files that genuinely shouldn't be tested,
or it forces pointless tests and gets disabled. Exemptions are explicit and
config-driven, never a silent ignore:

- **Empty files** are skipped automatically. A file with no code (empty or
  comment-only, e.g. a bare `__init__.py`) and declaration files (`*.d.ts`) have
  nothing to test. This is the only automatic exclusion; there is no name- or
  shape-based magic.
- **Everything else is explicit.** A launcher shim, a re-export barrel, generated
  code, or a non-empty `__init__.py` is exempted by a `[[<language>.exempt]]` config
  entry naming the `rules` it lifts (`colocated-test` / `coverage`) and a required
  `reason`. The whole exemption surface lives in one file, auditable in a single
  diff. A stale entry (a path that no longer exists) is a hard error, so the list
  can't rot.

## Configuration

One file drives every rule (TOML shown; native-language config files are also
supported):

```toml
[python]
coverage = { branch = true, fail_under = 100 }

# A deliberate, reason-required omission (see Exemptions above):
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test", "coverage"]
reason = "thin launcher; logic in run(), tested in run_test.py"

[typescript]
coverage = { lines = 100, branches = 100, functions = 100, statements = 100 }

[rust]
coverage = { regions = 100, lines = 100 }
```

## License

Released under the [MIT License](LICENSE).
