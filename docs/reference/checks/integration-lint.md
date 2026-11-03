---
description: The integration-lint check — integration and e2e tests run first-party code for real; the standard and declared suites it scans, the per-language rules, and unknown-tier.
---

# `integration-lint`

An integration test isolates the *system*: first-party code runs for real, and only the outside
world is mocked. `integration-lint` enforces that from the suite's side. This page is the
complete record of the check; [`unit-lint`](./unit-lint) enforces the same boundary from the
opposite side.

## Why this check exists

An integration test that mocks first-party code is testing a fiction: the assembled system it
claims to exercise never actually runs. The check keeps the integration tier honest — and it
guards the suite layout itself: a test file outside a standard or declared tier is named as
`unknown-tier` so every suite receives the integration rules.

## The first-party/external line

<!--@include: ../../explanation/isolation.md#boundary-->

## Where it finds its subjects

Subjects derive from the [package root](/monorepo#source-vs-the-package-root) — the
nearest directory at or above `source` holding the language's manifest: the integration suite in
`tests/integration/` and the e2e suite in `tests/e2e/`, plus each suite declared by
`[integration-lint].suites` in the package config. Every suite receives the same integration
rules. A test file under `<package root>/tests/` outside these suites is flagged as
`unknown-tier`. Rust uses Cargo's crate-root `tests/` layout, so the check scans that whole
directory; a tree with no manifest — loose scripts — is scanned at
`source` directly.

## What it flags

<!--@include: ../../explanation/isolation.md#integration-lint-flags-->

A mock target that resists static reading is left alone — the check is deterministic first.
`vi.mock(name)` and `patch(target)` hold the target in a variable, and a `patch.object` /
`patch.dict` base bound by no import (`patch.object(get_mod(), "x")`, a local variable, a fixture
argument) resolves to no module, so neither is classified. The same holds one level deeper: a
module attribute whose module's top-level source leaves it unnamed — a dynamic assignment, a
conflicting binding, a source file the scan cannot find — resolves to no target and is left
alone.

## When it runs

Always, as a step of the `Static checks (<language>)` job, for Python, TypeScript, and Rust
alike. The [`gates` input](/reference/workflow#inputs) names it `integration-lint`.

## Configuration

The package-root `testing-conventions.toml` declares additional suite directories for Python
and TypeScript. Each name is one directory directly under `tests/`:

```toml
[integration-lint]
suites = ["perf", "smoke"]
```

A declared suite receives every integration rule. An absent suite directory is a no-op. Names
must be plain, single path segments; empty names, `..`, and path separators are errors. Rust
already scans every test in the crate-root `tests/`, including subdirectories. Its suite layout
uses Cargo's convention and does not require declarations. This check scans declared suites;
execution of their tests is configured by the consuming package.

Each rule is lifted per file by a
[`[[<language>.exempt]]` entry](/reference/config#exemptions) with a required `reason` — and for
this check's suite subjects, the entry's `path` resolves **relative to the package root** the
tiers derive from (e.g. `tests/integration/billing_test.py`), not the scanned `source`:

| Rule | Language | Flags |
| --- | --- | --- |
| `no-first-party-mock` | TypeScript | a `vi.mock()` / `vi.doMock()` of a first-party module |
| `no-first-party-patch` | Python | a `patch(...)` / `patch.object(...)` / `patch.dict(...)` targeting the dist's own package |
| `no-monkeypatch` | Python | mocking via `monkeypatch` instead of `unittest.mock` in a fixture |
| `no-inline-patch` | Python | a patch in a test body instead of a fixture |
| `no-environ-mutation` | Python | env mutated outside `patch.dict(os.environ, ...)` |
| `no-constant-patch` | Python | patching a module-global constant (either patch form) instead of injecting config |
| `no-first-party-double` | Rust | a `#[double]` of the crate under test or a `path` dependency |
| `unknown-tier` | Python, TypeScript | a test file under `<package root>/tests/` outside a standard or declared tier |

## Learn more

- [Explanation — Isolation](/explanation/isolation): the boundary both lint checks enforce, as
  one essay.
- [Scoping and exemptions](/explanation/scoping): why the layout is part of the standard.
