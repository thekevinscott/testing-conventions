---
description: The integration lint check — integration and e2e tests run first-party code for real; the suite tiers it scans, the per-language rules, the hygiene lints, and unknown-tier.
---

# `integration lint`

An integration test isolates the *system*: first-party code runs for real, and only the outside
world is mocked. `integration lint` enforces that from the suite's side. This page is the
complete record of the check; [`unit lint`](./unit-lint) enforces the same boundary from the
opposite side.

## Why this check exists

An integration test that mocks first-party code is testing a fiction: the assembled system it
claims to exercise never actually runs. The check keeps the integration tier honest — and it
guards the suite layout itself: a test file outside a standard tier is a suite the scan would
silently miss, so it is named as an error (`unknown-tier`) instead.

## The first-party/external line

<!--@include: ../../explanation/isolation.md#boundary-->

## Where it finds its subjects

Subjects derive from the [package root](/monorepo#source-vs-the-package-root) — the
nearest directory at or above `source` holding the language's manifest: the integration suite in
`tests/integration/` and the e2e suite in `tests/e2e/` (Rust: the crate root's `tests/`, cargo's
own layout, holding both out-of-crate suites). Both suites run first-party code for real, so both
are held to the integration rules. A test file under `<package root>/tests/` outside a standard
tier is flagged as `unknown-tier`; a tree with no manifest — loose scripts — is scanned at
`source` directly.

## What it flags

<!--@include: ../../explanation/isolation.md#integration-lint-flags-->

A non-literal mock target (`vi.mock(name)`, `patch(target)`) can't be classified
deterministically and is left alone — the check is deterministic first.

## When it runs

Always, as a step of the `Static checks (<language>)` job, for Python, TypeScript, and Rust
alike. The [`gates` input](/reference/workflow#inputs) names it `integration-lint`.

## Configuration

The check takes no keys of its own. Each rule is lifted per file by a
[`[[<language>.exempt]]` entry](/reference/config#exemptions) with a required `reason` — and for
this check's suite subjects, the entry's `path` resolves **relative to the package root** the
tiers derive from (e.g. `tests/integration/billing_test.py`), not the scanned `source`:

| Rule | Language | Flags |
| --- | --- | --- |
| `no-first-party-mock` | TypeScript | a `vi.mock()` / `vi.doMock()` of a first-party module |
| `no-first-party-patch` | Python | a `patch(...)` whose string target is the dist's own package |
| `no-monkeypatch` | Python | mocking via `monkeypatch` instead of `unittest.mock` in a fixture |
| `no-inline-patch` | Python | a patch in a test body instead of a fixture |
| `no-environ-mutation` | Python | env mutated outside `patch.dict(os.environ, ...)` |
| `no-constant-patch` | Python | patching a module global instead of injecting config |
| `no-first-party-double` | Rust | a `#[double]` of the crate under test or a `path` dependency |
| `unknown-tier` | all | a test file under `<package root>/tests/` outside a standard tier |

## Learn more

- [Explanation — Isolation](/explanation/isolation): the boundary both lint checks enforce, as
  one essay.
- [Scoping and exemptions](/explanation/scoping): why the layout is part of the standard.
