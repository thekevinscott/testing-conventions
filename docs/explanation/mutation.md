---
description: Why mutation testing is the verification rung coverage can't reach, why the gate is binary, not a score, and the engines that run it.
---

# Mutation

Coverage tells you which lines your tests *ran*. **Mutation testing** tells you whether your tests
would *notice if those lines were wrong* — the question coverage can't answer. It's the
verification rung above the [coverage floor](./coverage): the top of the
[unit ladder](./#the-unit-ladder-exist-→-run-→-verify). The [workflow](../reference/workflow) runs it
on every pull request, diff-scoped to the changed lines.

## Coverage runs your code; mutation checks your tests

A coverage report says a line executed during a test. It says nothing about whether any test
*asserted* on the result. A test that calls your function and checks nothing still covers every line
it touches — 100% coverage, zero verification. Mutation testing closes that gap by deliberately
breaking the code and watching what your tests do.

A **mutant** is your code with one small, deliberate fault introduced — `+` becomes `-`, `>=` becomes
`>`, `return x` becomes `return null`. The tool makes the mutant, runs your unit suite, and records
the outcome:

- Tests **fail** → the mutant is **killed**. Good: a test caught the bug.
- Tests **pass** → the mutant **survived**. Bad: a real, bug-shaped change slipped past every test.

Unlike coverage, the result can't be satisfied by executing code — only by asserting on it.

## The example coverage can't catch

```ts
// adult.ts
export function isAdult(age: number): boolean {
  return age >= 18;
}
```

```ts
// adult.test.ts
import { expect, test } from 'vitest';
import { isAdult } from './adult';

test('isAdult', () => {
  expect(isAdult(20)).toBe(true);
  expect(isAdult(10)).toBe(false);
});
```

That test has **100% line and branch coverage** — there isn't even a branch to miss. Now mutate `>=`
into `>`:

```ts
return age > 18;
```

`20` and `10` still return the same answers, so every assertion still passes — the mutant
**survives**. Coverage said "perfect"; mutation testing found a real off-by-one at the boundary. The
only thing that kills it is the assertion you didn't write:

```ts
expect(isAdult(18)).toBe(true); // now `>` and `>=` diverge
```

## Why a number won't do: equivalent mutants

You might expect a mutation-score floor like "≥ 95%". Mutation score — the fraction of mutants a
suite kills — is a poor *target*, for one dominant reason: **equivalent mutants**. Some mutations
produce code that is *semantically identical* to the original (`x = x + 0`, a boundary that can't be
reached). No test can ever kill them — and detecting which survivors are equivalent is formally
undecidable. So for any given file you don't actually know what the maximum achievable score is; a
fixed floor can be unreachable through no fault of your tests. It's the "last 20% is defensive
trivia" problem from coverage, but worse — you often can't tell a real gap from an equivalent mutant
without reading it. A score also isn't comparable across the per-language engines.

So the gate isn't a percentage. It's **binary and diff-scoped**:

> **No unexplained surviving mutant on the lines a change touched.**

A survivor you've confirmed is equivalent or deliberately defensive gets a reasoned, line-scoped
[exemption](../guide/configure#exempt-specific-lines-coverage-mutation) — the same mechanism every
other check uses; every other survivor must be killed. It's the mutation analog of the coverage
philosophy — *"zero survivors except what you exempted with a reason"*, not *"hit a number you
can't reach"* — and it ports cleanly across languages, where a score does not. Diff-scoping is
what makes a binary gate tractable: whole-tree mutation is too slow to gate, so the job runs on
pull requests only, over the `<base>...HEAD` changed lines.

## A pass names its evidence

The gate has two green outcomes, and they are different facts — so the run reports which one it
earned. A run that tested mutants states the count:

> `unit mutation: no surviving mutants — every mutation was caught (6 mutant(s) tested)`

so the pass carries its own evidence. A diff-scoped run whose changed lines hold nothing
mutatable — a docs-only or workflow-only pull request — skips the engine and says so:

> `unit mutation: no mutatable changed lines — engine not run`

Both pass (an empty diff owes no mutation run; this is reporting, not gating), but the log tells
a validated pass from a vacuous one: a gate that has only ever printed the second line has never
exercised the engine, the sandbox, or the toolchain path, and the first source-touching pull
request is where an environment problem would surface.

## The engines

<!-- #region engines -->
Each language wraps its standard engine; the tool drives the engine, and you provide the test
runner that runs your own suite:

| Language | Engine | You provide |
| --- | --- | --- |
| TypeScript | [Stryker](https://stryker-mutator.io/), driven via its Node API by an adapter bundled in the npm package | `vitest` |
| Python | [cosmic-ray](https://github.com/sixty-north/cosmic-ray), driven via its library API by an adapter bundled in the wheel, with a baseline check that requires the clean suite to pass; each mutant's run ends at its first failing test | `pytest` |
| Rust | [cargo-mutants](https://github.com/sourcefrog/cargo-mutants), provisioned on first use (a pinned `cargo install` into the tool's own cache) and run from there; concurrent invocations share one provisioning install rather than each racing to install their own | the cargo toolchain that builds and tests your crate |

A run lists each survivor with its file, line, and mutation, and fails on any un-exempted one.
Survivor paths are reported relative to the scanned path, so exemptions address the same paths
every other check uses.

Each engine runs where its ecosystem expects: cargo-mutants at the crate root, cosmic-ray at the
scanned path (it mutates the tree in place), and Stryker at the **package root** — the nearest
directory at or above the scanned path holding a `package.json`. Stryker copies the project into
a sandbox and runs the suite there; rooting that sandbox at the package root puts the manifest
and every other package-level file inside it, so a source that reaches above the scanned path —
`import pkg from '../package.json'`, a shared `../tsconfig` — resolves in the sandbox exactly as
it does in the tree. Mutation stays scoped to the scanned path, and the scanned path's colocated
unit suite is what judges each mutant; the package's other suite tiers (`tests/`) stay out of the
run.
<!-- #endregion engines -->

## Timeouts: a mutant timeout is inconclusive, a baseline timeout is fatal

A mutant whose run outlasts its budget **times out**. A timeout is *inconclusive* — the tool
counts it as neither a caught mutant nor a survivor, so a diff-scoped run where every mutant on
the touched lines times out passes with zero findings. This holds across engines: cargo-mutants
signals it with its own timeout exit status, and cosmic-ray reports the mutant with no usable
outcome; both drop out of the survivor set.

A **baseline** timeout is the opposite — a loud error. The baseline is the clean, unmutated suite;
a suite that can't finish in its budget is untrustworthy as a judge of any mutant, exactly as a
baseline that *fails* is. So an unmutated suite that times out or ends abnormally stops the run
with an error. A silently timed-out baseline would instead let every mutant time out and drop,
and slip an empty survivor set through as a false pass.

Each engine scopes the per-mutant timeout to the clean suite's own measured runtime: a slow suite
earns a proportionally larger budget, and only a mutant that runs *far* longer than the clean
suite is judged hung — so a legitimately slow suite keeps its budget instead of hitting a fixed
ceiling.

## Why it matters for agents

An LLM can reach 100% coverage with assertion-light tests; published benchmarks put the mutation
score of LLM-generated tests around 40% despite high coverage. Mutation score is the signal an agent
**can't** satisfy by executing code — only by writing assertions that pin behavior. That makes it far
more resistant to gaming than a coverage number, which is the most exploitable target you can hand an
optimizer.

## Where it fits

Mutation measures the same colocated unit suite the other checks already do. Because it re-runs
that suite many times, it rewards exactly what [isolation](./isolation) and the coverage floor
already enforce: fast, deterministic, isolated unit tests. A suite that passes those is already
mutation-ready.
