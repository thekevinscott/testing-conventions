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

**Scope.** The files `integration lint` already scans (`*_test.py`,
`conftest.py`), pointed at the integration suite (`tests/integration/`),
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

## Unit detection — `unmocked-collaborator` (slice 2)

The mirror image, for `unit lint --language python <PATH>`: a colocated unit
test (`foo_test.py` next to `foo.py`) must isolate the unit under test, so an
imported **first-party collaborator** that isn't mocked is the violation
(`unmocked-collaborator` — the same rule id TypeScript's #76 emits, so the #102
waiver `Rule` already covers it).

**The bright line for "is this import mocked?"** `vi.mock('./x')` names the import
specifier, but Python's `patch("pkg.mod.name")` names a *symbol*, and the
convention patches the name in the **consuming** module (`patch("pkg.foo.thing")`,
not `patch("pkg.other.thing")`) — the name-resolution #19 ruled a non-goal. The
deterministic signal the convention *does* give us: a mocked collaborator is
**patched by string** and, in the canonical form, **not imported** — the unit uses
it internally. So the rule keys off imports:

- Scan `*_test.py` only (not `conftest.py` — that holds fixtures, not units; and
  not a legacy `test_*.py`, which is ordinary source — #145).
- The **unit under test** is the import whose module's last segment equals the
  test's base name (`widget_test.py` → `widget`; `from pkg.widget import build`,
  `from .widget import build`, and `import pkg.widget` all match). Never flagged.
- An import is **first-party** when it's relative (`from . import x`,
  `from .mod import y`) or its head segment is the dist package (slice 1's
  `pyproject.toml` discovery).
- An import is **mocked** when some `patch("…")` target in the file has a **last
  dotted segment** equal to the imported symbol: `from pkg.other import thing` +
  `patch("pkg.widget.thing")` → last segment `thing` matches (catches the
  consuming-module patch); `patch("pkg.other.thing")` matches too.
- **Flag** a first-party import that is neither the unit under test nor mocked.

Pure stdlib, the test framework (`pytest`, `unittest`, `unittest.mock`),
`__future__`, and `TYPE_CHECKING`-guarded (type-only) imports are never
collaborators.

#42's acceptance ("un-mocked first-party (unit)") is met by **first-party** alone.
**Slice 3 (#121)** broadens the same rule to un-mocked **external** imports, so the
README's full "flag any un-mocked first-party / external import" holds.

### External classification (slice 3)

An import head that isn't first-party is classified against an embedded copy of the
stdlib module set (`sys.stdlib_module_names`) and a curated **effectful** subset:

| Head | Class | Verdict |
| --- | --- | --- |
| relative, or `== dist package` | first-party | check (slice 2) |
| `pytest` / `_pytest` / `mock` | test framework | allow |
| in the **effectful-stdlib** set | external | **check** |
| in the stdlib set (otherwise) | pure stdlib | allow |
| anything else (bare, non-stdlib) | third-party | **check** |

The effectful set is deliberately **conservative** — the modules that are
effectful *at the head* (network / subprocess / process & IPC / randomness /
database / low-level OS): `socket`, `ssl`, `ftplib`, `smtplib`, `subprocess`,
`multiprocessing`, `signal`, `random`, `secrets`, `sqlite3`, `ctypes`, … It
**excludes dual-nature** modules whose head can't distinguish a pure use from an
effectful one — `os` (`os.path` vs `os.system`), `pathlib` (`PurePath` vs
`Path.read_text`), `datetime` / `time` (a literal vs `now()`), `io`, `logging`. At
the head level those are a [non-goal](#precision-limits--non-goals): the clock /
filesystem are caught when patched-by-string (the convention), not at import. The
list is a tunable heuristic, not an exhaustive map (à la #19).

## Surface & module shape

- **CLI.** Integration: extend the existing `integration lint --language python
  <PATH>` (the home for deterministic integration-test lints) — `no-first-party-patch`
  joins #48–#52 there, no new subcommand. Unit (slice 2): a Python arm of
  `unit lint --language python <PATH>`.
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
- **Unit (slice 2):** a first-party **value/type** import used to build test data
  (rather than a collaborator to mock) — distinguishing the two needs data-flow, so
  the rule treats every un-mocked first-party import as a collaborator; the `exempt`
  waiver covers the legitimate case. A collaborator **mocked across files** (an
  autouse fixture in `conftest.py`) — the rule only reads the test file's own
  patches; the canonical form patches *and doesn't import*, so this bites only the
  import-and-mock-elsewhere mix. Alias edges (`import x as y`).
- **Unit (slice 3):** a **dual-nature stdlib** module used purely (`os.path.join`,
  `pathlib.PurePath`, `datetime(2020, 1, 1)`) — its head can't tell pure from
  effectful, so those heads are excluded from the effectful set and not flagged;
  the clock / filesystem stay caught by the patch-by-string convention, not at
  import. A pure **test-helper** third-party package (`freezegun`, `responses`, …)
  is imported-and-used, not mocked — beyond the `pytest` / `_pytest` / `mock`
  allowlist it's flagged like any third-party import; waive it (or extend the
  allowlist) until a config-level test-dep list exists.

## Fixtures (per slice, red + clean — #3 guardrail)

- **Integration red:** a `pyproject.toml` (`name = "myproject"`) + a test that
  patches a first-party target in a fixture (`patch("myproject.ledger.record")`),
  importing `myproject` code for real.
- **Integration clean:** same `pyproject.toml`; a test that mocks only third-party
  (`patch("requests.get")`) and effectful stdlib (`patch("subprocess.run")`,
  `patch("builtins.open")`) — first-party runs for real. Zero findings.
- **Integration waived:** the red test, plus a `testing-conventions.toml` waiving
  `no-first-party-patch` for that file with a reason → passes.
- **Unit red:** a `pyproject.toml` + a colocated `widget_test.py` that imports an
  un-mocked first-party collaborator (`from myproject.ledger import record`)
  alongside the unit under test (`from myproject.widget import build`).
- **Unit clean:** same `pyproject.toml`; the canonical form — imports only the unit
  under test (and `pytest` / `patch`), patches the collaborator by string in a
  fixture. Zero findings.
- **Unit waived:** the red test, plus a `testing-conventions.toml` waiving
  `unmocked-collaborator` for that file with a reason → passes.

## Implementation slices (red→green)

Each is its own test-first increment with CHANGELOG + MIGRATIONS + a VitePress doc.

1. **Integration** — `no-first-party-patch`: `pyproject.toml` first-party
   discovery + flag `patch("<first-party>…")`; register the waiver `Rule`. ✅
2. **Unit (first-party)** — `unmocked-collaborator` for `unit lint --language
   python`: flag an imported first-party collaborator (not the unit under test, not
   `patch`-ed by string) — reuses slice 1's first-party discovery. ✅ (#117)
3. **Unit (external)** — extend slice 2 to un-mocked third-party / effectful-stdlib
   imports, behind the [stdlib effectful/pure classifier](#external-classification-slice-3)
   above. Same `unmocked-collaborator` rule (no new `config::Rule`). *(This slice — #121.)*

Order: 1 → 2 → 3. Slice 3 reuses slice 2's visitor, `is_mocked`, and waiver.

## Unit fixtures (slice 3)

- **External red:** the `pyproject.toml` + a `widget_test.py` importing un-mocked
  `requests` (third-party) and `subprocess` (effectful stdlib), plus `json` (pure
  stdlib, allowed) and the unit under test.
- **External clean:** the canonical form — mocks the external collaborators by
  string (so they're never imported) and uses only pure stdlib. Zero findings.
- **External waived:** the red test waived for `unmocked-collaborator` → passes.
