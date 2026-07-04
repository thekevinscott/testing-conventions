---
description: The testing model the checks encode — the three kinds of test, the unit ladder, and why it's built to resist gaming.
---

# The testing model

`testing-conventions` isn't a test runner. It's an opinionated *standard* for how a library's tests
are structured, isolated, and measured — enforced deterministically in CI so the standard can't
quietly erode. This page explains the model; each check then has its own page explaining exactly
what it enforces and why.

## Why a standard at all

Tests are the contract an automated change has to honor. When an agent (or a human in a hurry) edits
code, the test suite is what catches a regression — but only if the suite actually asserts behavior,
runs the real code it claims to, and can't be quietly weakened. Those are properties of *how tests
are written*, and they're exactly what a normal test runner doesn't check. This project makes them
explicit, deterministic rules so an agentic workflow has a floor it can't fall through.

The bar is deliberately high — a strict **100% coverage floor**, a **binary mutation gate** — because
the cost of a too-low bar is silent rot, and the escape hatch (a reason-required
[exemption](./scoping)) is cheap and auditable.

## The three kinds of test

The standard recognizes three kinds of test, by how much of the system they exercise:

- **Unit** — cheap, plentiful, low-confidence on their own. Everything around the unit is mocked, so
  a failure points at the unit. These anchor refactors: when an agent changes code, the unit suite is
  the fast signal.
- **Integration** — treats the system as a black box. First-party code runs *for real*; only the
  outside world is mocked (databases, the network, the clock, the filesystem, LLMs).
- **E2E** — like integration, but with no mocks at all. Slow, flaky, and costly, so CI never runs
  them; an agent runs them on demand to confirm real third-party contracts still hold, and
  [attests](./e2e) that it did.

The unit/integration split is **structural**, not a tag or marker: unit tests are
[colocated](./colocated-test) with their source, integration tests live in their own folder, and
the boundary is enforced behaviorally — unit tests must mock every collaborator, integration tests
must run first-party code for real (see [Isolation](./isolation)).

## The unit ladder: exist → run → verify

Three checks measure the *same* colocated unit suite, each a stronger question than the last:

| Check | Question |
| --- | --- |
| [`unit colocated-test`](./colocated-test) | Does a test **exist**? |
| [`unit coverage`](./coverage) | Does the test **run** the code? |
| [`unit mutation`](./mutation) | Does the test **verify** the code? |

Each rung answers a gap in the one below. Coverage exists because a test can exist without running a
line; mutation exists because a line can run without any assertion checking it. The ladder rewards
exactly what the other checks already demand: fast, deterministic, isolated unit tests.

## The checks

Every check is a CI job that fails the build on a violation, with the offending files in the log:

- [Colocated tests](./colocated-test) — every source file has a colocated, matching-named unit
  test, and a changed source changes its test with it.
- [Coverage](./coverage) — the unit suite clears a 100% floor, whole-tree and on the changed lines
  of a pull request.
- [Mutation](./mutation) — every changed line is *verified*, not just executed: break the code, and
  a test has to fail.
- [Isolation](./isolation) — unit tests mock every collaborator; integration tests run first-party
  code for real.
- [Packaging](./packaging) — test files never ship in the built artifact.
- [E2E attestation](./e2e) — the e2e suite ran against the current code, without CI ever running it.
- [Scoping and exemptions](./scoping) — how the scan is scoped and how a deliberate omission is
  recorded.

## Why it's built for agents

An LLM optimizes for the target you give it. Coverage is the most exploitable target there is — an
agent can hit 100% with assertion-light tests that execute every line and check almost nothing. The
mutation gate is the rung an agent **can't** satisfy by executing code; it only passes when the tests
actually pin behavior. That's the throughline of the whole standard: prefer signals that can't be
gamed by running code without asserting on it.

## Parity over cleverness

A check is offered only to the level the *least-capable* supported language can meet — **least
parity**. There are no language-only rules: if a capability can't be matched in Python, TypeScript,
and Rust alike, the feature is scoped down to the common denominator or held until parity is
reachable. The payoff is that the standard means the same thing everywhere; the cost is the
occasional deliberate asymmetry (Rust's coverage has no branch component by default, because branch
coverage is experimental on stable), and those are always called out where they occur.
