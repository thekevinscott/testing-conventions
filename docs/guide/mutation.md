# Mutation testing

Coverage tells you which lines your tests *ran*. **Mutation testing** tells you
whether your tests would *notice if those lines were wrong* — the question coverage
can't answer. It's the verification layer above the coverage floor.

## Coverage runs your code; mutation checks your tests

A coverage report says a line executed during a test. It says nothing about whether
any test *asserted* on the result. A test that calls your function and checks
nothing still covers every line it touches — 100% coverage, zero verification.

Mutation testing closes that gap by deliberately breaking the code and watching
what your tests do.

## How it works

A **mutant** is your code with one small, deliberate fault introduced — `+` becomes
`-`, `>=` becomes `>`, a `return x` becomes `return null`. The tool makes the
mutant, runs your unit suite, and records the outcome:

- Tests **fail** → the mutant is **killed**. Good: a test caught the bug.
- Tests **pass** → the mutant **survived**. Bad: a real, bug-shaped change slipped
  past every test.

The **mutation score** is the fraction of mutants your suite kills. Unlike
coverage, it can't be satisfied by executing code — only by asserting on it.

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

That test has **100% line and branch coverage** — there isn't even a branch to
miss. Now mutate `>=` into `>`:

```ts
return age > 18;
```

`20` and `10` still return the same answers, so every assertion still passes — the
mutant **survives**. Coverage said "perfect"; mutation testing found a real
off-by-one at the boundary. The only thing that kills it is the assertion you
didn't write:

```ts
expect(isAdult(18)).toBe(true); // now `>` and `>=` diverge
```

## Mutation operators

Tools apply a catalog of small, realistic edits: arithmetic (`+`↔`-`), comparison
(`<`↔`<=`, `==`↔`!=`), logical (`&&`↔`||`), negated conditionals (`if (x)` →
`if (!x)`), replaced return values, swapped constants (`true`↔`false`), and removed
statements. Each is a stand-in for a plausible real bug.

## Why a number won't do: equivalent mutants

You might expect a mutation-score floor like "≥ 95%." Mutation score is a poor
*target*, for one dominant reason: **equivalent mutants**. Some mutations produce
code that is *semantically identical* to the original (`x = x + 0`, a boundary that
can't be reached). No test can ever kill them — and detecting which survivors are
equivalent is formally undecidable. So for any given file you don't actually know
what the maximum achievable score is; a fixed floor can be unreachable through no
fault of your tests. It's the "last 20% is defensive trivia" problem from coverage,
but worse — you often can't tell a real gap from an equivalent mutant without
reading it.

## The floor: no *unexplained* survivors on the diff

So the gate isn't a percentage. When enforced, it's **binary and diff-scoped**:

> **No unexplained surviving mutant on the lines a change touched.**

A survivor you've confirmed is equivalent or defensive gets a reasoned exemption
(the same `[[<language>.exempt]]` mechanism the other rules use); every other
survivor must be killed. It's the mutation analog of the coverage philosophy —
*"zero survivors except what you exempted with a reason,"* not *"hit a number you
can't reach."* And it ports cleanly across languages, where a score number does
not.

The gate is **on by default**: an unexplained survivor fails the build, the way
`unit coverage` does — there's no separate report-only mode, and config can't loosen
it. The only escape is a reasoned, per-file exemption (above), so a passing run means
"every survivor was either killed or explained."

## Where it fits: the unit ladder

Mutation testing is the third rung of the same unit suite the other rules already
measure:

| Rule | Question it answers |
| --- | --- |
| `unit colocated-test` | Does a test **exist**? |
| `unit coverage` | Does the test **run** the code? |
| **`unit mutation`** | Does the test **verify** the code? |

Because it re-runs your unit tests many times, it rewards exactly what the other
rules already enforce — fast, deterministic, isolated unit tests. A suite that
passes `unit lint` and the coverage floor is already mutation-ready.

## Why it matters for agents

An LLM can reach 100% coverage with assertion-light tests; published benchmarks put
the mutation score of LLM-generated tests around 40% despite high coverage. Mutation
score is the signal an agent **can't** satisfy by executing code — only by writing
assertions that pin behavior. That makes it far more resistant to gaming than a
coverage number, which is the most exploitable target you can hand an optimizer.

## Per-language engines

The rule wraps each language's mutation tool behind one contract:

| Language | Engine | Diff-scoping |
| --- | --- | --- |
| TypeScript | [Stryker](https://stryker-mutator.io/) | changed-line `--mutate` ranges |
| Rust | [cargo-mutants](https://github.com/sourcefrog/cargo-mutants) | `--in-diff` |
| Python | [mutmut](https://github.com/boxed/mutmut) | via the wrapper |

## Status

The `unit mutation` rule is landing **one language at a time** — see the
[epic](https://github.com/thekevinscott/testing-conventions/issues/199).

- **Rust** — available now as `unit mutation --language rust` (via
  [cargo-mutants](https://github.com/sourcefrog/cargo-mutants)). The gate is **on by
  default**: any un-exempted surviving mutant fails the run (exit `1`), with reasoned
  `[[rust.exempt]] rules = ["mutation"]` entries the only loosening. Pass `--base <ref>`
  to scope it to a diff.
- **TypeScript** — available now as `unit mutation --language typescript` (via
  [Stryker](https://stryker-mutator.io/)). Same on-by-default gate and same reasoned
  `[[typescript.exempt]] rules = ["mutation"]` loosening as Rust. Stryker has no native
  git-diff mode, so `--base <ref>` is implemented by translating the `<base>...HEAD`
  changed lines into Stryker `--mutate <file>:<line>-<line>` ranges — **line** granularity,
  matching cargo-mutants' `--in-diff`. The one called-out asymmetry: under `--base`, the
  changed-line ranges *replace* the project's configured `mutate` set (test and `.d.ts`
  files are filtered out), where cargo-mutants' `--in-diff` intersects with its own file
  selection.
- **Python** — still planned.

Because the bar is **least parity** (a rule ships to consumers only once all three languages
meet one contract), `unit mutation` is **not yet wired into the [reusable workflow](./ci)**.
The Rust and TypeScript commands run today for local use and experimentation; the CI rule
turns on once Python reaches parity.
