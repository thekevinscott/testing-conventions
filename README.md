# testing-conventions

`testing-conventions` enforces testing conventions in libraries (Python, TypeScript, and Rust).
Primarily useful for enforcing agent (LLM) behavior.

<!-- Single source of truth for the rule list: docs/index.md pulls the region below via VitePress @include. Keep it in sync with the #3 checklist. -->
<!-- #region rules -->
## Rules at a glance

Every rule is a CLI command that fails CI on a violation.

**Unit**

- [`unit colocated-test`](https://thekevinscott.github.io/testing-conventions/reference/#unit-colocated-test) — every source file has a colocated, matching-named unit test (Python, TypeScript, Rust); with `--base`, a source changed in the diff must also change its colocated test (co-change; Python, TypeScript; [#33](https://github.com/thekevinscott/testing-conventions/issues/33)).
- [`unit coverage`](https://thekevinscott.github.io/testing-conventions/reference/#unit-coverage) — enforce a coverage floor on the unit suite (Python, TypeScript, Rust); with `--base`, the same floor is measured over the changed lines of a `<base>...HEAD` diff instead of the whole tree ([#162](https://github.com/thekevinscott/testing-conventions/issues/162)).
- [`unit lint`](https://thekevinscott.github.io/testing-conventions/reference/#unit-lint) — a unit test mocks every collaborator: no out-of-module calls or imports (Rust); no un-mocked first-party or external collaborators (Python, TypeScript); typed mocks (TypeScript).
- [`unit mutation`](https://thekevinscott.github.io/testing-conventions/guide/mutation) — every line a change touches is *verified*, not just executed: mutation testing breaks the code and requires a test to fail. The gate is binary and diff-scoped — no unexplained surviving mutant on the diff — not a score percentage (Python, TypeScript, Rust; wired into the reusable workflow as a diff-scoped, PR-only job, [#204](https://github.com/thekevinscott/testing-conventions/issues/204)).

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
- **Rust:** `cargo llvm-cov`, default `lines = 100`. `regions` is opt-in (a finer, Rust-only metric) and branch coverage is still experimental. Inline units can't be excluded by filename; use `#[coverage(off)]` (toolchain-dependent).

**Diff-scoped (`--base`):** `unit coverage --base <ref>` measures the same configured floor over only the lines a `<base>...HEAD` diff changed — the slice a PR introduces — instead of the whole tree. An opt-in, additive scope of `unit coverage`: without `--base`, the whole-tree floor runs.

**Checked:** deterministic (run coverage; compare to the configured thresholds).

### Mutation

**Rule:** every line a change touches is *verified*, not just executed — some test
must fail when that line is broken.

**Why:** coverage measures execution, not assertion; a test can run a line without
checking it. Mutation testing introduces a small fault (a *mutant*) and requires a
test to catch it — the verification rung above the coverage floor, and the signal an
agent can't satisfy without real assertions.

- **Python:** [cosmic-ray](https://github.com/sixty-north/cosmic-ray) over the unit suite.
- **TypeScript:** [Stryker](https://stryker-mutator.io/) over the unit suite.
- **Rust:** [cargo-mutants](https://github.com/sourcefrog/cargo-mutants) over the unit suite.

The gate is not a score percentage — equivalent mutants (mutations no test can ever
kill) make 100% unreachable, and a score isn't comparable across engines. Instead
it's binary and diff-scoped: **no unexplained surviving mutant on changed lines**,
with reasoned `[[<language>.exempt]]` entries for the rest.

**Checked:** all three languages are available now — **Rust** (`unit mutation --language rust`, via cargo-mutants), **TypeScript** (`unit mutation --language typescript`, via Stryker), and **Python** (`unit mutation --language python`, via cosmic-ray) — a binary gate, on by default: any un-exempted survivor fails, with reasoned `[[<language>.exempt]] rules = ["mutation"]` entries (naming the survivor's `lines`) the only loosening. They're at parity and **wired into the reusable workflow** as a diff-scoped, PR-only job across the matrix ([#204](https://github.com/thekevinscott/testing-conventions/issues/204)). See the [mutation-testing guide](https://thekevinscott.github.io/testing-conventions/guide/mutation). Deterministic (a diff-scoped mutation run; any unexplained survivor on a changed line fails the build).

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
- **Line-scoped, always, for `coverage` / `mutation`.** These two rules are never
  whole-file: an exemption carries a `lines` list (`lines = [9, 10, "12-13"]`) naming
  the exact lines it lifts. A determinism guard checks the list — a listed line that
  *isn't* failing is a hard error, an unlisted failing line still fails — so it's
  exactly the failing lines. (`lines` is for these two rules only; a `lines` key on
  `colocated-test` is rejected.)

## Configuration

One file drives every rule (TOML shown; native-language config files are also
supported):

```toml
[python]
coverage = { branch = true, fail_under = 100 }

# A whole-file presence exemption (a launcher shim with no colocated test):
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test"]
reason = "thin launcher; logic in run(), tested in run_test.py"

# A line-scoped coverage/mutation exemption (`lines` required; its own entry):
[[python.exempt]]
path = "mypkg/config/tomlcompat.py"
rules = ["coverage", "mutation"]
lines = [9, 10, "12-13"]
reason = "version-conditional tomllib/tomli import; one branch is dead on any single interpreter"

[typescript]
coverage = { lines = 100, branches = 100, functions = 100, statements = 100 }

[rust]
coverage = { regions = 100, lines = 100 }
```

## Surface the contract to your agent

CI enforces the rules; `testing-conventions agents install` teaches them. It writes a small
managed block into your repository's `AGENTS.md` — the few non-negotiables plus pointers to the
full contract — so a coding agent (Claude Code, Codex, Cursor, Gemini CLI, Copilot) knows the
discipline before it writes code. The block is sentinel-delimited and hash-versioned: re-running
`install` is idempotent, `agents check` fails CI when the block goes stale, and `agents remove`
takes it back out. Everything outside the markers is yours and stays byte-for-byte intact. See
the [how-to](https://thekevinscott.github.io/testing-conventions/guide/agents).

## License

Released under the [MIT License](LICENSE).
