---
description: Why mutation testing is the verification rung coverage can't reach, and why the gate is binary, not a score.
---

# Why mutation testing

Coverage tells you which lines your tests *ran*. **Mutation testing** tells you whether your tests
would *notice if those lines were wrong* — the question coverage can't answer. It's the verification
rung above the [coverage floor](./#the-unit-ladder-exist-run-verify). To actually run it, see
[Run mutation testing](../guide/mutation).

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

A survivor you've confirmed is equivalent or deliberately defensive gets a reasoned
[exemption](../guide/configure#exempt-a-file) — the same mechanism every other rule uses; every other survivor
must be killed. It's the mutation analog of the coverage philosophy — *"zero survivors except what
you exempted with a reason"*, not *"hit a number you can't reach"* — and it ports cleanly across
languages, where a score does not.

## Why it matters for agents

An LLM can reach 100% coverage with assertion-light tests; published benchmarks put the mutation
score of LLM-generated tests around 40% despite high coverage. Mutation score is the signal an agent
**can't** satisfy by executing code — only by writing assertions that pin behavior. That makes it far
more resistant to gaming than a coverage number, which is the most exploitable target you can hand an
optimizer.

## Where it fits

Mutation is the third rung of the [unit ladder](./#the-unit-ladder-exist-run-verify), measuring the
same colocated unit suite the other rules already do. Because it re-runs that suite many times, it
rewards exactly what `unit lint` and the coverage floor already enforce: fast, deterministic,
isolated unit tests. A suite that passes those is already mutation-ready.
