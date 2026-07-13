---
description: The unit lint check — every collaborator in a unit test is mocked; the first-party/external line, the per-language rules, and their exemptions.
---

# `unit lint`

A unit test isolates one unit: everything it depends on is mocked, so a failure points at the
unit, not a collaborator. `unit lint` enforces that from the unit suite's side. This page is the
complete record of the check; [`integration lint`](./integration-lint) enforces the same boundary
from the opposite side.

## Why this check exists

A unit test that touches a real collaborator behaves like an integration test: slower, and
ambiguous when it fails. The unit tier is built on failures that localize — and on staying fast
and deterministic enough for [mutation](./mutation) to re-run the suite many times. The check
makes that discipline deterministic instead of aspirational: a collaborator is either mocked, or
named in the log.

## The first-party/external line

<!--@include: ../../explanation/isolation.md#boundary-->

## What it flags

<!--@include: ../../explanation/isolation.md#unit-lint-flags-->

A non-literal mock target (`vi.mock(name)`, `patch(target)`) can't be classified
deterministically and is left alone — the check is deterministic first.

## When it runs

Always, as a step of the `Static checks (<language>)` job, for Python, TypeScript, and Rust
alike. It scans the colocated unit tests under `source`, leaving `<package root>/tests/` to
[`integration lint`](./integration-lint). The [`gates` input](/reference/workflow#inputs) names
it `unit-lint`.

## Configuration

The check takes no keys of its own. Each rule is lifted per file by a
[`[[<language>.exempt]]` entry](/reference/config#exemptions) with a required `reason`:

| Rule | Language | Flags |
| --- | --- | --- |
| `unmocked-collaborator` | Python, TypeScript | a collaborator the unit test imports and doesn't mock |
| `untyped-mock` | TypeScript | a mock factory with no `vi.importActual<typeof import(...)>()` type anchor |
| `no-out-of-module-call` | Rust | a unit test calling out of its own module |
| `no-out-of-module-import` | Rust | a unit test importing out of its own module |

The bar for exempting is high: what feels untestable usually needs a technique — inject the
dependency, patch by string in a fixture — not a waiver. See
[Scoping and exemptions](/explanation/scoping#the-bar-for-exempting).

## Learn more

- [Explanation — Isolation](/explanation/isolation): the boundary both lint checks enforce, as
  one essay.
- [Configure the rules](/guide/configure#exempt-a-file): exempting a file, step by step.
