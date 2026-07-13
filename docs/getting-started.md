---
description: Adopt the full standard on a repository — add one workflow file, watch a check go red on a pull request, fix it, and watch it go green.
---

# Getting Started

This tutorial adopts `testing-conventions` on a repository and walks you through the loop you'll
live in from then on: a pull request trips a check, CI goes red with the exact violation, you fix
it, CI goes green. By the end, every rule runs on every pull request — and you'll have seen one
fire for real.

You need a GitHub repository you can push to. The steps below use a fresh Python library so the
output matches exactly; the same flow works for TypeScript and Rust, or on an existing project.

## 1. Add the workflow

Add one file to your repository — no inputs, no config — and commit it to your default branch:

```yaml
# .github/workflows/conventions.yml
name: Conventions
on: [pull_request]

jobs:
  conventions:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
```

On every pull request it **auto-detects the languages present** (Python, TypeScript, and Rust),
scans `src`, and runs [every check](./explanation/) with strict defaults — each as its own job that
fails the build on a violation. A language with no sources is skipped, never failed, so the default
is safe on any library.

That one file is the whole adoption: your repository is now held to the full standard.

## 2. Open a pull request that breaks a rule

See it fire. On a branch, add a source file with **no test**:

```python
# src/greet.py
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

Push the branch and open a pull request.

## 3. Watch the check go red

The `unit colocated-test (python)` check fails, and its log names the violation:

```
missing colocated unit test: greet.py
```

This is the standard's core move: every source file carries a colocated, matching-named unit test,
and the check is a blocking gate — the pull request stays red until the test exists (or the file
carries a reasoned [exemption](./guide/configure#exempt-a-file)).

## 4. Make it green

Add the colocated test, named after the source file, side by side with it:

```python
# src/greet_test.py
from greet import greet


def test_greet():
    assert greet("Ada") == "Hello, Ada!"
```

Push. The checks run again and come back green: the test exists (`unit colocated-test`), it runs
the new lines at a 100% floor (`unit coverage`), and its assertion pins the behavior
(`unit mutation` breaks the code and requires a test to fail — an assertion-free test would leave
the check red).

## Where you are

Every pull request now runs the full standard: colocated tests, a 100% coverage floor, mocked-out
collaborators in unit tests, real first-party code in integration tests, a binary mutation gate on
changed lines, and clean packaging. When a check goes red, its log names the file and the rule;
you fix the code, or record a reasoned exemption in
[`testing-conventions.toml`](./guide/configure).

## Teach your agent

Run `npx testing-conventions install` to write the contract into your repo's `AGENTS.md`, so a
coding agent knows the rules before it writes code. The managed block carries the contract's
non-negotiables and points at this site and its machine-readable digest (`llms.txt`). Re-running
refreshes the block; everything outside its markers stays yours. `install` reads the block by its
`begin`/`end` marker pair — if the `end` marker is missing (a hand edit deleted or fenced it),
`install` stops and names it so you can restore the marker, keeping your surrounding prose intact.

## Next

- A repository with several packages adopts per package — see [Adopt on a monorepo](./monorepo).
  If a package's integration or e2e suite isn't showing up in any check, read
  [monorepo suite discovery](./monorepo#everything-derives-from-the-package) first — discovery
  looks at fixed, plural-named paths (`tests/integration/`, `tests/e2e/`), not wherever `source`
  happens to point.
- [The testing model](./explanation/) explains what each check enforces and why.
- [Configure the rules](./guide/configure) tunes a floor or exempts a file, with a reason.
