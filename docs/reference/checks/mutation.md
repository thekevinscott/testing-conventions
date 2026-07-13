---
description: "The unit mutation check — a binary gate, not a score: no unexplained surviving mutant on a pull request's changed lines; the engines, timeouts, and the line-scoped exemption."
---

# `unit mutation`

Break the code, and a test has to fail. `unit mutation` mutates the lines a pull request touched,
runs the unit suite against each mutant, and fails on any surviving mutant not exempted with a
reason — **a binary gate, not a score**. This page is the complete record of the check.

## Why this check exists

Coverage proves a line executed; it says nothing about whether any test asserted on the result —
a test that calls the function and checks nothing still covers every line it touches. Mutation is
the [unit ladder](/explanation/#the-unit-ladder-exist-→-run-→-verify)'s top rung: does the test
**verify** the code? It is also the rung an agent can't game: an LLM can reach 100% coverage with
assertion-light tests, and mutation only passes when the tests actually pin behavior.

The gate is binary rather than a score because of **equivalent mutants**: some mutations are
semantically identical to the original, no test can ever kill them, and detecting them is
formally undecidable — so a score floor can be unreachable through no fault of your tests. The
rule is instead: **no unexplained surviving mutant on the lines a change touched** — zero
survivors except what you exempted with a reason. Diff-scoping is what makes a binary gate
tractable: whole-tree mutation is too slow to gate.

## What it enforces

A **mutant** is the code with one small, deliberate fault (`+` becomes `-`, `>=` becomes `>`,
`return x` becomes `return null`). The tool makes each mutant on a changed line and runs the unit
suite against it: tests fail and the mutant is **killed**; tests pass and it **survived** — and
an un-exempted survivor fails the check.

<!--@include: ../../explanation/mutation.md#engines-->

### Timeouts

A mutant whose run outlasts its budget is **inconclusive** — neither killed nor survived, dropped
from the survivor set. A **baseline** timeout (the clean, unmutated suite) is a loud error: a
suite that can't finish in its budget is untrustworthy as a judge of any mutant, exactly as a
baseline that fails is. Each engine scopes the per-mutant timeout to the clean suite's own
measured runtime, so a legitimately slow suite keeps a proportional budget.

## When it runs

Pull requests only, as its own job per language, diff-scoped to the `<base>...HEAD` changed
lines. It installs and runs from the derived
[package root](/monorepo#source-vs-the-package-root), provisioned like the
[coverage jobs](./unit-coverage#when-it-runs); for TypeScript, your project dependencies must
include `@stryker-mutator/core` and a runner plugin. The
[`gates` input](/reference/workflow#inputs) names it `mutation`.

## Configuration

The check has **no percentage key** — the gate is binary, and config can't loosen it. Its tuning
surface:

- A `mutation` exemption is **line-scoped, never whole-file**: the entry carries a `lines` list
  naming the exact lines whose survivors are explained (confirmed equivalent, or deliberately
  defensive), with a required `reason` — see
  [the exemption schema](/reference/config#line-scoped-exemptions). The determinism guard rejects
  a listed line whose mutants were all caught.
- [`[rust] features`](/reference/config#rust-features) — cargo features forwarded to
  cargo-mutants' build/test runs.
- [`build_command`](/reference/config#build-command) — a pre-suite build the manifest can't
  express.

## Learn more

- [Explanation — Mutation](/explanation/mutation): the worked example coverage can't catch,
  equivalent mutants in full, and why it matters for agents.
- [Configure the rules](/guide/configure#exempt-specific-lines-coverage-mutation): exempting a
  surviving mutant, step by step.
