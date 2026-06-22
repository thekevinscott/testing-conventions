---
description: The testing model the rules encode — the three kinds of test, the unit ladder, and why it's built to resist gaming.
---

# The testing model

`testing-conventions` isn't a test runner. It's an opinionated *standard* for how a library's tests
are structured, isolated, and measured — enforced deterministically in CI so the standard can't
quietly erode. This page explains the model the rules encode and why it's shaped the way it is. For
the commands that enforce it, see the [how-to guides](../guide/); for exact flags, the
[reference](../reference/).

## Why a standard at all

Tests are the contract an automated change has to honor. When an agent (or a human in a hurry) edits
code, the test suite is what catches a regression — but only if the suite actually asserts behavior,
runs the real code it claims to, and can't be quietly weakened. Those are properties of *how tests
are written*, and they're exactly what a normal test runner doesn't check. This project makes them
explicit, deterministic rules so an agentic workflow has a floor it can't fall through.

The bar is deliberately high — a strict **100% coverage floor**, a **binary mutation gate** — because
the cost of a too-low bar is silent rot, and the escape hatch (a reason-required
[exemption](../guide/configure#exempt-a-file)) is cheap and auditable.

## The three kinds of test

The standard recognizes three kinds of test, by how much of the system they exercise:

- **Unit** — cheap, plentiful, low-confidence on their own. Everything around the unit is mocked, so
  a failure points at the unit. These anchor refactors: when an agent changes code, the unit suite is
  the fast signal.
- **Integration** — treats the system as a black box. First-party code runs *for real*; only the
  outside world is mocked (databases, the network, the clock, the filesystem, LLMs).
- **E2E** — like integration, but with no mocks at all. Slow, flaky, and costly, so CI never runs
  them; an agent runs them on demand to confirm real third-party contracts still hold, and
  [attests](../reference/#e2e-attest) that it did.

The unit/integration split is **structural**, not a tag or marker: unit tests are colocated with
their source, integration tests live in their own folder, and the boundary is enforced behaviorally —
unit tests must mock every collaborator, integration tests must run first-party code for real.

## Isolation: first-party vs. external

Both the unit and integration lints turn on one distinction — **first-party vs. external** — drawn
deterministically, with no module resolution:

- A **unit test** mocks *everything* but the unit: first-party collaborators and external packages
  alike.
- An **integration test** mocks *only* the external world and runs first-party code for real.

"External" means more than third-party packages: it includes effectful standard-library APIs (the
filesystem, the clock, randomness, the network, subprocess). An un-mocked external call is what makes
a test slow, flaky, or a charge on someone's bill, so the boundary is drawn there. See
[Isolate tests](../guide/isolation) for the rules that enforce it.

## The unit ladder: exist → run → verify

Three rules measure the *same* colocated unit suite, each a stronger question than the last:

| Rule | Question |
| --- | --- |
| `unit colocated-test` | Does a test **exist**? |
| `unit coverage` | Does the test **run** the code? |
| `unit mutation` | Does the test **verify** the code? |

Each rung answers a gap in the one below. Coverage exists because a test can exist without running a
line; mutation exists because a line can run without any assertion checking it — see
[Why mutation testing](./mutation). The ladder rewards exactly what the other rules already demand:
fast, deterministic, isolated unit tests.

## Why it's built for agents

An LLM optimizes for the target you give it. Coverage is the most exploitable target there is — an
agent can hit 100% with assertion-light tests that execute every line and check almost nothing. The
mutation gate is the rung an agent **can't** satisfy by executing code; it only passes when the tests
actually pin behavior. That's the throughline of the whole standard: prefer signals that can't be
gamed by running code without asserting on it.

## Parity over cleverness

A rule is offered only to the level the *least-capable* supported language can meet — **least
parity**. There are no language-only rules: if a capability can't be matched in Python, TypeScript,
and Rust alike, the feature is scoped down to the common denominator or held until parity is
reachable. The payoff is that the standard means the same thing everywhere; the cost is the
occasional deliberate asymmetry (Rust's coverage has no branch component, because branch coverage is
experimental on stable), and those are always called out where they occur.

## Exemptions: a gate needs a door

A blocking gate with no escape hatch gets disabled. So every rule has one — but it's **explicit,
reason-required, and in one file**, never a silent ignore. A launcher shim, a re-export barrel, or
generated code earns an exemption that names the rules it lifts and *why*; the whole exemption
surface is auditable in a single diff, and a stale entry is a hard error so the list can't rot. The
philosophy is "zero violations except what you exempted with a reason" — not "hit a number you can
soften when it's inconvenient." And keep each exemption as small as the code that genuinely can't be
tested: extract the irreducible part into its own small unit and exempt only that, rather than waving
a whole module past the gate. See [Configure the rules](../guide/configure#exempt-a-file).
