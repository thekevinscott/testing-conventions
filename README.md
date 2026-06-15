# testing-conventions

**Enforce testing conventions in libraries (Python, TypeScript, and Rust)**

`testing-conventions` is an opinionated, config-driven standard for how tests are
structured, isolated, and measured in a library. One config file drives every
rule, and rules are enforced deterministically in CI. The point is enforcement,
not suggestion: every rule is a deterministic, bright-line check that an agent (or
a hurried human) can't quietly cross while keeping CI green.

## The three kinds of tests

Tests assert the behavior of code — they exist to give you confidence it does what
it's supposed to. This standard recognizes three kinds:

- **Unit** — cheap, plentiful, low confidence on their own. Most valuable in an
  agentic workflow, where they anchor refactors.
- **Integration** — treats the system as a black box: first-party code runs for
  real, external dependencies are mocked (databases, LLMs, the filesystem, the
  clock, …).
- **E2E** — identical to integration but with *no* mocks. Not meant for CI; meant
  to be run by an agent to confirm real third-party contracts still hold.

## Rules

Each rule states what's enforced, why, and how it varies by language. **Checked**
notes how it's verified — every rule is a deterministic check, run in CI from the
config.

### Unit

#### Location & Naming

**Rule** — unit tests are colocated with the code they test, and named after it.

**Why** — colocation makes the unit/integration boundary structural (by location,
not a tag or marker), and 1:1 naming means an orphaned test can't hide.

- **Python** — `foo.py` → `foo_test.py`, side by side in `src/`.
- **TypeScript** — `foo-bar.ts` → `foo-bar.test.ts`, side by side in `src/`.
- **Rust** — no separate file: units are an inline `#[cfg(test)]` module in the same `.rs` file, so colocation and 1:1 naming are automatic.

**Checked** — deterministic (glob + name match in Py/TS; `#[cfg(test)]` presence in Rust).

#### Isolation

**Rule** — everything except the unit under test is mocked.

**Why** — a unit test that touches a real collaborator is an integration test
wearing a unit's name, and it makes unrelated refactors break the wrong tests.

- **Python** — mock every first-party collaborator the unit imports; `autospec=True` keeps each mock's signature honest against the real object.
- **TypeScript** — `vi.mock()` each collaborator, typed so it can't drift from the source.
- **Rust** — no import monkeypatching: inject a trait (hand-rolled or `mockall`). Idiomatic Rust keeps a pure core with I/O at the edges, so many units need no mocks — the compiler guarantees the double matches the trait.

The TypeScript typed-mock pattern:

```ts
vi.mock('./service', async () => {
  const actual = await vi.importActual<typeof import('./service')>('./service');
  return { ...actual, fetchUser: vi.fn() };
});
```

**Checked** — deterministic. Py/TS: flag any un-mocked first-party/external import. Rust: flag any call out of the test's own module (cross-module, external crate, or effectful `std`); [`dylint`](https://github.com/trailofbits/dylint) for full name-resolution precision.

### Integration

#### Location

**Rule** — integration tests live in a dedicated folder, separate from the unit
suite.

**Why** — a structural home keeps black-box tests out of the unit suite that
coverage is measured on.

- **Python** — `tests/integration/`, files end in `_test.py` (non-test helpers omit the suffix).
- **TypeScript** — `tests/integration/`, files end in `.test.ts`.
- **Rust** — `tests/` at the crate root; each file compiles as its own crate, so the location is the signal.

**Checked** — deterministic (location).

#### External Dependencies

**Rule** — every external dependency is mocked; first-party code runs for real.
**External** means any package dependency *plus* effectful standard-library APIs
(filesystem, clock, randomness, network, subprocess, env). A whitelist lets
specific dependencies through unmocked.

**Why** — an un-mocked external call makes the test slow, flaky, or a charge on
someone's bill.

- **Python** — patch third-party imports and effectful stdlib (`open`, `datetime`, `subprocess`, …).
- **TypeScript** — `vi.mock()` third-party packages and Node built-ins (`fs`, `Date`, `child_process`, …).
- **Rust** — external crates and `std` I/O are mocked behind injected traits (e.g. `mockall`).

**Note** — if the library ships a CLI, back it with an SDK and point integration
tests at the SDK; keep the CLI a thin wrapper.

**Checked** — deterministic. Py/TS: flag any un-mocked external import. Rust: flag external-crate or effectful-`std` use that isn't behind an injected trait or whitelisted; [`dylint`](https://github.com/trailofbits/dylint) for full precision.

### E2E

**Rule** — e2e tests live in a dedicated folder and run with no mocks.

**Why** — they confirm real external contracts still hold; they're for an agent to
run on demand, not for CI.

- **Python** — `tests/e2e/`, files end in `_test.py`.
- **TypeScript** — `tests/e2e/`, files end in `.test.ts`.
- **Rust** — under `tests/`, typically driving the built binary (`CARGO_BIN_EXE_<name>` or `assert_cmd`).

**Checked** — deterministic (location only; e2e is excluded from the CI gate).

### Coverage

**Rule** — coverage floors are enforced on the **unit suite only**, can't regress,
and exclude test code from the denominator. The thresholds themselves are set per
library, in each tool's native coverage primitives.

**Why** — coverage measures execution, not assertion; measuring it on anything but
real unit tests lets integration tests inflate the number.

- **Python** — `pytest --cov` (coverage.py): set `branch` and `fail_under`; omit `*_test.py`.
- **TypeScript** — `vitest` coverage (v8/istanbul): set the `lines` / `branches` / `functions` / `statements` thresholds; exclude `*.test.ts`.
- **Rust** — `cargo llvm-cov`: set `regions` / `lines` (branch coverage is still experimental). Inline units can't be excluded by filename — use `#[coverage(off)]` (toolchain-dependent).

**Checked** — deterministic (run coverage; compare to the configured thresholds and to the previous run).

### Packaging

**Rule** — test files never ship in the built package.

**Why** — colocated unit tests live in `src/`, so packaging has to strip them;
shipping test code bloats the artifact and leaks fixtures.

- **Python** — exclude `*_test.py` from the wheel/sdist (`build_py` + `MANIFEST.in`).
- **TypeScript** — exclude `*.test.ts` from the published `dist`.
- **Rust** — free in the compiled artifact: `#[cfg(test)]` is stripped and `tests/` isn't built for consumers (add a Cargo `exclude` only to keep them out of the source tarball).

**Checked** — deterministic (inspect the built artifact for test files).

## Exemptions

A blocking gate needs an honest escape hatch for files that genuinely shouldn't be tested —
otherwise it forces pointless tests or gets disabled. Exemptions are **explicit and config-
driven**, never a silent ignore:

- **Empty files** are skipped automatically — a file with no code (empty or comment-only, e.g.
  a bare `__init__.py`) and declaration files (`*.d.ts`) have nothing to test. This is the only
  automatic exclusion; there is no name- or shape-based magic.
- **Everything else is explicit.** A launcher shim, a re-export barrel, generated code, or a
  non-empty `__init__.py` is exempted by a `[[<language>.exempt]]` config entry naming the
  `rules` it lifts (`location` / `coverage`) and a **required** `reason`. The whole exemption
  surface lives in one file (auditable in a single diff), and a stale entry — a path that no
  longer exists — is a hard error, so the list can't quietly rot.

## Configuration

One file drives every rule (TOML shown; native-language config files are also
supported):

```toml
[python]
coverage = { branch = true, fail_under = 100 }

# A deliberate, reason-required omission (see Exemptions above):
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["location", "coverage"]
reason = "thin launcher; logic in run(), tested in run_test.py"

[typescript]
coverage = { lines = 100, branches = 100, functions = 100, statements = 100 }

[rust]
coverage = { regions = 100, lines = 100 }
```

## License

Released under the [MIT License](LICENSE).
