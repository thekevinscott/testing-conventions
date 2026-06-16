# Python — isolation & external-deps (design)

Design pass for [#42](https://github.com/thekevinscott/testing-conventions/issues/42)
(Phase 3, "needs a design pass first" — a spec like [#26](https://github.com/thekevinscott/testing-conventions/issues/26)).
It resolves the open questions and carves the work into red→green slices. **No
detector ships from this doc** — it is the spec the implementation slices build
against.

Read [testing.md](testing.md) first (pytest, fixtures, `unittest.mock.patch`
wrapped in a `pytest.fixture`) and the README's **Isolation** (Unit) and
**External Dependencies** (Integration) rules — this doc makes those two rules
deterministic for Python. It is the Python twin of
[../rust/isolation.md](../rust/isolation.md) and the TypeScript `no-first-party-mock`
/ `unmocked-collaborator` lints; the same first-party-vs-external bright line, drawn
for Python's `patch(...)`-based mocking.

## The rule (restated)

| Kind | Convention | Violation |
| --- | --- | --- |
| **Unit** | Every collaborator the unit imports — first-party *and* external / effectful-stdlib — is mocked (`patch` in a `pytest.fixture`, `autospec=True`), so a failure points at the unit. | An imported collaborator (not the unit under test) that is **not** mocked. |
| **Integration** | First-party code runs for real; only third-party packages and effectful stdlib may be mocked. | A `patch(...)` whose **target is first-party** — e.g. `patch("ourpkg.mod.fn")`. |

Both honor the config `exempt` waiver (see [Waiver](#waiver)).

## Approach: `rustpython` AST heuristic, no name resolution

Detection reuses [`lint.rs`](../../packages/rust/src/lint.rs)'s machinery: each
Python test file is parsed with `rustpython_parser` and walked with a `Visitor`.
Like the existing `integration lint` rules (#48–#52), this is a **deterministic
syntactic heuristic** — it reads what is written (`patch("…")` string targets,
`import` statements), not what those names resolve to at runtime. That matches the
repo's bright-line philosophy: predictable, low-false-positive, no interpreter
needed (rustpython parses Python without running it). The cases a syntactic pass
can't reach are stated as [non-goals](#precision-limits--non-goals), exactly as
#19 framed its own.

## First-party vs external (the core)

The whole rule turns on one question: **is a name first-party (the dist's own
code) or external?** The Python answer mirrors how the Rust rule reads
`Cargo.toml`'s `[package].name` — read the dist's **own top-level import
package** from its `pyproject.toml`:

- Discover the project by walking up from the scanned `<PATH>` to the nearest
  `pyproject.toml` (stopping at the repo root / a `.git` boundary so the search
  can't escape the project).
- The first-party top-level package is `[project].name`, **normalized** to an
  import name: lower-cased, with `-`/`.` → `_` (PEP 503-style — `my-project` →
  `my_project`). This is "the dist's own top-level package" the issue names.
- A **relative** import (`from . import x`, `from .mod import y`) is inherently
  first-party — it can only name the dist's own package — and needs no lookup.

If no `pyproject.toml` (or no `[project].name`) is found, the dist's package is
unknown: the integration rule then flags **nothing** rather than guess, so a tree
with no declared package is never a false positive. A project whose import name
differs from its normalized distribution name is a documented
[non-goal](#precision-limits--non-goals) (the explicit `exempt` waiver covers the
stray case).

### External, and the effectful-stdlib line

"External" is everything that isn't first-party: third-party packages
(`requests`, `stripe`, …) **and** effectful standard-library APIs. Per the README,
effectful stdlib is filesystem / clock / randomness / network / subprocess / env —
`open`/`builtins.open`, `subprocess`, `socket`, `datetime`, `time`, `random`,
`os` (environ/process), `pathlib` I/O. Mocking any of these in an **integration**
test is allowed (it's the whole point); not mocking them in a **unit** test is the
unit violation. Pure stdlib (`dataclasses`, `typing`, `collections`, `json`, …) is
neither — a unit test needn't mock it.

## Integration detection — `no-first-party-patch` (slice 1)

**Scope.** The files `integration lint` already scans (`*_test.py`, legacy
`test_*.py`, `conftest.py`), pointed at the integration suite (`tests/integration/`),
exactly as the TypeScript `integration lint` is pointed at its integration suite.

Flag a **patch of a first-party target**. The clean bright-line signal — the
canonical `patch` forms `lint.rs` already recognizes (`patch`, `mock.patch`,
`unittest.mock.patch`, `mocker.patch`, `patch.object`/`patch.dict` via their
`patch` base):

- `patch("ourpkg.mod.fn")` — a **string-literal** first argument whose **head
  dotted segment** is the first-party package → **flag** (`no-first-party-patch`).
- `patch("requests.get")`, `patch("subprocess.run")`, `patch("builtins.open")` —
  head is third-party or stdlib → **allow** (mocking the outside world is the
  point of an integration test).

This reuses the existing patch-target extraction (`patches_constant` already pulls
`call.args.first()` as a string). A patch in a `pytest.fixture` is the *right*
place (it doesn't trip `no-inline-patch`), so the red fixture puts its first-party
patch in a fixture to isolate the new rule.

## Unit detection — `unmocked-collaborator` (slice 2, deferred)

The mirror image, for `unit isolation --language python`: a colocated unit test
(`foo_test.py` next to `foo.py`) must mock every collaborator it imports — flag an
imported first-party / external / effectful-stdlib name that is **not** mocked,
leaving alone the unit under test (`foo`), pure stdlib, and pytest itself.

The hard part — and why this is its own slice — is deciding *"is this import
mocked?"* deterministically. `vi.mock('./x')` names the import specifier, but
Python's `patch("pkg.mod.name")` names the symbol's **definition site**, not the
test's `from pkg.mod import name`. Matching the two is exactly the
name-resolution #19 ruled a non-goal ("patch the name in the *consuming* module").
The slice's design pass must pick a bright line — candidates: treat a first-party
collaborator import as un-mocked unless a `patch(...)` mentions its module path; or
require the conventional autouse-fixture form. Carved out here, designed when built.

## Surface & module shape

- **CLI.** Integration: extend the existing `integration lint --language python
  <PATH>` (the home for deterministic integration-test lints) — `no-first-party-patch`
  joins #48–#52 there, no new subcommand. Unit (slice 2): a Python arm of
  `unit isolation --language python <PATH>`.
- **Module.** Lives in [`lint.rs`](../../packages/rust/src/lint.rs), the Python
  AST home, reusing its `is_patch_call` / patch-target extraction and the shared
  `Violation` shape — the Python parallel to all-Rust-in-`isolation.rs`,
  all-TypeScript-in-`ts.rs`.
- **First-party discovery.** A small `pyproject.toml` reader (reuse the existing
  `toml` dependency, as the Rust rule reuses it for `Cargo.toml`).

## Waiver

No new mechanism — the rule plugs into the config `exempt` waiver generalized to
the isolation rules in #102: register `Rule::NoFirstPartyPatch` (`id()` /
`from_id()`), and `integration lint`'s existing `apply_waivers` pass lifts a
`no-first-party-patch` finding for any file with a reason-required
`[[python.exempt]] rules = ["no-first-party-patch"]` entry. Auditable in one diff,
reason-required, stale entries error — never a silent ignore.

## Precision limits / non-goals

Deliberately **not** caught by the syntactic heuristic — left to review, and stated
plainly (à la #19) so nobody over-trusts green:

- A `patch.object(module, "attr")` whose first argument is a **module object**, not
  a string — classifying it needs resolving the object back to an import.
- A **non-literal** patch target (`patch(target)`, `patch(f"{pkg}.fn")`) — can't be
  classified deterministically (mirrors TypeScript skipping `vi.mock(name)`).
- A dist whose **import package name differs** from its normalized
  `[project].name`, or a multi-package dist — slice 1 reads the single normalized
  name; the `exempt` waiver covers the stray case until discovery is widened.
- The unit direction's import-↔-patch matching (see [slice 2](#unit-detection--unmocked-collaborator-slice-2-deferred)).

## Fixtures (per slice, red + clean — #3 guardrail)

- **Integration red:** a `pyproject.toml` (`name = "myproject"`) + a test that
  patches a first-party target in a fixture (`patch("myproject.ledger.record")`),
  importing `myproject` code for real.
- **Integration clean:** same `pyproject.toml`; a test that mocks only third-party
  (`patch("requests.get")`) and effectful stdlib (`patch("subprocess.run")`,
  `patch("builtins.open")`) — first-party runs for real. Zero findings.
- **Integration waived:** the red test, plus a `testing-conventions.toml` waiving
  `no-first-party-patch` for that file with a reason → passes.
- **Unit (slice 2):** red = a unit test with an un-mocked first-party collaborator;
  clean = every collaborator mocked.

## Implementation slices (red→green)

Each is its own test-first increment with CHANGELOG + MIGRATIONS + a VitePress doc.

1. **Integration** — `no-first-party-patch`: `pyproject.toml` first-party
   discovery + flag `patch("<first-party>…")`; register the waiver `Rule`. *(This
   slice.)*
2. **Unit** — `unmocked-collaborator` for `unit isolation --language python`, after
   its own design pass picks the import-↔-patch bright line.

Order: 1 → 2. Slice 2 reuses slice 1's first-party discovery.
