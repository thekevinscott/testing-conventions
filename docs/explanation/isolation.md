---
description: Why unit and integration tests mock opposite things — the first-party/external line, and what the two lint checks flag per language.
---

# Isolation

Two checks — `unit lint` and `integration lint` — enforce one boundary from opposite sides.
Isolation is about which collaborators a test may fake, and the answer is *opposite* for the two
kinds of test:

- A **unit test** isolates one *unit*. Everything it depends on is mocked — first-party
  collaborators *and* external packages alike — so a failure points at the unit, not a
  collaborator. A unit test that touches a real collaborator behaves like an integration test:
  slower, and ambiguous when it fails.
- An **integration test** isolates the *system*. First-party code runs **for real**, and only the
  outside world is mocked. An integration test that mocks first-party code is testing a fiction:
  the assembled system it claims to exercise never actually runs.

## The first-party/external line

Both checks turn on the same distinction, drawn deterministically with no module resolution:

| Specifier | Origin | Example |
| --- | --- | --- |
| relative / absolute path | first-party | `./service`, `../core` |
| the dist's own package (Python, from `pyproject.toml` `[project].name`; Rust, from `Cargo.toml`) | first-party | `ourpkg.ledger` |
| `node:`-prefixed or a builtin name | external (built-in) | `node:fs`, `child_process` |
| any other bare specifier | external (third-party) | `stripe`, `@scope/pkg` |

"External" means more than third-party packages: it includes **effectful standard-library APIs** —
the filesystem, the clock, randomness, the network, subprocess. An un-mocked external call is what
makes a test slow, flaky, or a charge on someone's bill, so the boundary is drawn there. Pure
stdlib (`json`, `dataclasses`, `std::collections`) is nobody's collaborator.

## What `unit lint` flags

The unit suite's side: every collaborator is mocked.

- **TypeScript** — `unmocked-collaborator`: any runtime import a unit test doesn't `vi.mock()`.
  Three imports are never collaborators: the unit under test (`widget.test.ts` → `./widget`),
  type-only imports, and the test runner (`vitest`). Plus `untyped-mock`: a mock factory with no
  `vi.importActual<typeof import(...)>()` type anchor, so the double can't drift from the real
  module.
- **Python** — `unmocked-collaborator`: an imported collaborator the colocated `*_test.py` doesn't
  mock, both first-party and external (a third-party package, or effectful stdlib such as
  `socket`, `subprocess`, `random`). Never collaborators: the unit under test, the test framework,
  pure stdlib, and type-only imports. The canonical unit test imports only the unit under test and
  patches collaborators by string in a fixture — so it has no collaborator imports at all. A
  re-export barrel is the unit under test too: in `__init___test.py`, a bare `from . import …`
  names the package's own `__init__.py` surface — `Thing`, `__all__`, `__version__` — so those
  names are what the test verifies, never collaborators. (Reaching around the barrel into a sibling
  module — `from .core import Thing` in `__init___test.py` — imports a real collaborator and is
  flagged.) This matches TypeScript, where `index.test.ts` importing `./index.js` is the unit under
  test.
- **Rust** — the same intent, structurally: `no-out-of-module-call` and `no-out-of-module-import`
  flag a unit test (an inline `#[cfg(test)]` module) that reaches out of its own module —
  `crate::…`, an external crate, or effectful `std` (`fs`, `net`, `process`, `env`, `thread`, the
  clock). A single `super::` (the unit under test), `self`, and pure `std` stay in-module. Inject
  a trait double for a collaborator instead. A unit test is a module gated by a positively-required
  `test` (`#[cfg(test)]`, `#[cfg(all(test, …))]`); a `#[cfg(not(test))]` module compiles in
  *non-test* builds, so it is production code and its out-of-module calls are left alone.

## What `integration lint` flags

The integration suite's side: first-party code runs for real.

- **TypeScript** — `no-first-party-mock`: a `vi.mock()` / `vi.doMock()` of a first-party module.
  Mocking `stripe` or `node:fs` is fine; mocking `../src/ledger` is the violation.
- **Python** — `no-first-party-patch`: a `patch(...)` whose string target is the dist's own
  package. Patching `requests.post` or `subprocess.run` is fine; patching `ourpkg.ledger.record`
  is the violation. Four hygiene lints ride along, keeping the *mechanism* of mocking disciplined:
  `no-monkeypatch` (use `unittest.mock` in a fixture, so patches are declared, not sprinkled),
  `no-inline-patch` (a patch lives in a fixture, not a test body), `no-environ-mutation` (set env
  via `patch.dict(os.environ, ...)`, so it's restored), and `no-constant-patch` (inject config
  explicitly instead of patching a module global).
- **Rust** — `no-first-party-double`: a `#[double]` (mockall_double) of the crate under test or a
  `path` dependency. Only external crates and `std` may be doubled.

A non-literal target (`vi.mock(name)`, `patch(target)`) can't be classified deterministically and
is left alone — the checks are deterministic first.

## When a lint fires on a real design constraint

Every one of these rules is waivable per file with a reason-required
[exemption](../guide/configure#exempt-a-file) — and the bar for using one is high: what feels
untestable usually needs a technique (inject the dependency, patch by string in a fixture, drive
the boundary directly), not a waiver. The reasoned entry exists so that when a file genuinely earns
one, the omission is named and auditable rather than silent.
