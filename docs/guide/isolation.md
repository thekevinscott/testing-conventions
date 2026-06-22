---
description: Enforce the isolation boundary — mock every collaborator in a unit test, but only the external world in an integration test.
---

# Isolate tests

Isolation is about which collaborators a test may fake. The answer is *opposite* for the two
kinds of test:

- A **unit test** isolates one unit. Everything it depends on is mocked (first-party
  collaborators *and* external packages / Node built-ins), so a failure points at the unit, not
  a collaborator.
- An **integration test** isolates the *system*. First-party code runs for real, and only the
  outside world is mocked: third-party packages and Node built-ins (databases, the network, the
  clock, the filesystem).

Both rules turn on the same distinction: **first-party vs. external**. `testing-conventions`
draws it deterministically, with no module resolution:

| Specifier                          | Origin        | Example                    |
| ---------------------------------- | ------------- | -------------------------- |
| relative / absolute path           | first-party   | `./service`, `../core`     |
| `node:`-prefixed or a builtin name | Node built-in | `node:fs`, `child_process` |
| any other bare specifier           | third-party   | `stripe`, `@scope/pkg`     |

## Don't mock first-party code in an integration test

Point `integration lint` at your integration suite. For TypeScript, the `no-first-party-mock`
lint flags any `vi.mock()` / `vi.doMock()` of a first-party module:

```sh
testing-conventions integration lint --language typescript test/integration/
```

```ts
// ❌ flagged: ../src/ledger is first-party; an integration test must run it for real
vi.mock('../src/ledger', () => ({ record: vi.fn() }));

// ✅ fine: a third-party package and a Node built-in
vi.mock('stripe');
vi.mock('node:fs');
```

Python enforces the same direction with `no-first-party-patch`:

```sh
testing-conventions integration lint --language python tests/integration/
```

It flags any `patch(...)` whose string target is first-party. The dist's own top-level package
is read from the nearest `pyproject.toml` `[project].name`:

```python
# ❌ flagged: `ourpkg` is this dist's package; an integration test must run it for real
with patch("ourpkg.ledger.record") as mock:
    ...

# ✅ fine: a third-party package and effectful stdlib
with patch("requests.post") as mock: ...
with patch("subprocess.run") as mock: ...
```

A first-party mock prints to stderr and the command exits non-zero, so CI fails. A non-literal
target (`vi.mock(name)` / `patch(target)`) can't be classified deterministically and is left
alone.

## Mock every collaborator in a unit test

The mirror-image rule for the **unit** suite, enforced by `unit lint`:

```sh
testing-conventions unit lint --language typescript src/
```

It flags any runtime import a unit test doesn't `vi.mock()` (`unmocked-collaborator`), leaving
only three alone: the unit under test (`widget.test.ts` → `./widget`), type-only imports, and
the test runner (`vitest`). Mock each collaborator, typed so the double can't drift from the
real module:

```ts
vi.mock('./service', async () => {
  const actual = await vi.importActual<typeof import('./service')>('./service');
  return { ...actual, fetchUser: vi.fn() };
});
```

Python works the same way, against the colocated `*_test.py` suite:

```sh
testing-conventions unit lint --language python src/
```

It flags an imported collaborator the unit test doesn't mock (`unmocked-collaborator`), both
**first-party** (the dist's own package, per `pyproject.toml`) and **external**: a third-party
package or an effectful-stdlib module (`socket`, `subprocess`, `random`). The unit under test,
`pytest` / `unittest`, **pure** stdlib (`json`, `dataclasses`), and type-only imports are never
collaborators. The idiom is to *not* import the collaborator at all: patch it by string in a
fixture, so the unit uses the double internally:

```python
# ❌ flagged: a first-party collaborator and an external dep, imported and used for real
from myproject.ledger import record
import requests

# ✅ fine: patch the names as the unit under test looks them up; don't import them
@pytest.fixture(autouse=True)
def mock_record():
    with patch("myproject.widget.record") as mock:
        yield mock
```

(Dual-nature stdlib like `os` / `pathlib` / `datetime` isn't flagged at the import; the clock and
filesystem are caught by patching, not by the import check.)

(Rust enforces the same intent structurally with `unit lint --language rust`, via
`no-out-of-module-call`.)

## See also

- [The testing model — isolation](../explanation/#isolation-first-party-vs-external): why unit and integration tests mock opposite things.
- [Reference — `integration lint`](../reference/#integration-lint): the lint list and exit codes.
- [Configure the rules — exempt a file](./configure#exempt-a-file): the reason-required escape hatch, shared by every rule.
