# Testing Strategy — Open Discussion Log

> **Status: UNRESOLVED. No agreed-upon solution.** Parked 2026-06-27.
>
> This records a long design discussion about how to test `dirsql` (Rust core +
> Python + TypeScript SDKs) when the code is written by an LLM agent. It captures
> the goals, every attempt, and the dead ends. The companion
> [`TESTING-PHILOSOPHIES.md`](./TESTING-PHILOSOPHIES.md) is the literature survey
> that came out of it. **Neither document represents an adopted policy.**

## Why this exists

We set out to define a testing strategy that an *agent* (not a human) could
follow mechanically. We did a lot of research and produced a strong literature
map, but we did **not** converge on a usable answer for dirsql — and part way
through, the research drifted onto a *different* question than the one we
started with. This is the record so a future session can resume cold without
re-litigating. The most important thing preserved here is the set of
high-level goals, because those never changed.

## The high-level goals (stated repeatedly, the fixed point)

These are the requirements any solution must satisfy. They held constant across
the entire discussion.

1. **Deterministically enforceable, unambiguous rules — written for an agent,
   not a human.** The code is written by an LLM, and agents vary by seed.
   "Best practices" that need human judgment (e.g. "mock some dependencies but
   not others") are unacceptable. Goal: *make it as easy as possible to do the
   right thing*, and mechanically catch the wrong thing.
2. **Three test tiers, with crisp boundaries:**
   - **unit** — isolate the unit; mock every non-trivial dependency.
   - **integration** — exercise *all* of the dirsql code **and nothing else**;
     every third-party dependency replaced.
   - **e2e** — real everything (real SQLite, real filesystem, real install);
     mock nothing.
3. **Ban in-code mocks.** No `vitest.mock`, no `unittest.patch`, no
   `monkeypatch`. The only thing faked is the **external environment** — the
   network — via an *environment fixture*. The user's framing: "a mock of the
   environment instead of a mock in the code."
4. **Uniform across all three languages** (Rust, Python, TypeScript) — the
   one-implementation principle applied to tests. Ideally a **single command**
   that runs equivalently on local (Ubuntu + macOS, ideally Windows) and in CI.
5. **Lightweight.** A container per test run is "super heavy" and was rejected;
   wants something cheaper than Docker.
6. **Grounded in rigorous evidence, not invented.** "I'd prefer not to invent
   something myself. I'd prefer to follow somebody else's advice, particularly
   if it emerges from rigorous scientific or academic study." Primary sources
   required.
7. **No static exemptions.** "Don't sandbox code that can't reach the network /
   let CI be the gate" was rejected emphatically and must not be re-proposed.
   All unit + integration tests must be sandboxed *uniformly*.

## Why it's hard (the core tension)

- **Rust cannot do runtime import-mocking.** There is no monkeypatch; you cannot
  swap a dependency (e.g. SQLite/rusqlite) at runtime the way Python/TS can.
  Substitution requires dependency injection or compile-time seams. So the
  Python/TS "mock all deps in the integration tier" approach **does not port to
  Rust** — which broke goals #2/#3 as originally conceived and started the whole
  thread.
- That pushed us from "mock the dependencies" to "fake the *environment*" (block
  the network), which the user liked in principle. But the *mechanism* for
  blocking the network then collided with goals #4 + #5 + #7 simultaneously, and
  we could not satisfy all of them at once (see attempt D).

## What we tried (the twists and turns)

**A. Mock SQLite in Rust integration tests, like Python/TS would.**
→ Impossible: Rust has no runtime monkeypatch. Dead end. This is what kicked off
the whole inquiry.

**B. Dependency injection / compile-time seams** (`mockall`, `faux`, Cargo
`[patch]`).
→ The user dislikes DI and avoids it in Python/TS too; rejected as the general
answer (retained only as a last resort).

**C. Reframe "mock" → fake the environment by blocking the network.**
An *environment fixture*, not an in-code mock. Liked in theory. The bright line
was identified as **effect-based**: block *external* network; allow loopback +
filesystem. This idea is still alive in principle — it's the isolation
*mechanism* underneath it that stalled.

**D. Find a network-block mechanism that is uniform (3 languages) + cross-OS
(incl. Windows) + lightweight.** Where it stalled:
  - *In-process blockers* (pytest-socket, nock/undici): per-language (not
    uniform), and not airtight.
  - *OS network namespaces* (`unshare --net`): Linux-only.
  - *Container `--network none`*: the **only** airtight, cross-OS (incl.
    Windows), all-language option found — but rejected as too heavy (#5).
  → **Dead end.** The constraints (#4 uniform + #5 lightweight + airtight +
  Windows) may be **mutually unsatisfiable**. This was never said out loud at the
  time; it is the central unresolved fact.

**E. Static exemption** ("don't sandbox code that can't reach the network; let
CI be the gate").
→ Rejected emphatically ("the worst idea you've had; never bring it up again").
Recorded only so it is not re-proposed.

**F. Pivot to the literature** (→ [`TESTING-PHILOSOPHIES.md`](./TESTING-PHILOSOPHIES.md)).
A philosophy-organized survey with primary sources, plus a first-principles
thread:
  - **Tests = an independent oracle** asserting intended behavior; their value is
    proportional to their *independence from the code's author*.
  - **Two-quantifier decomposition:** "does what it says" (testable via an
    independent oracle) vs "does no more than it says" (a universal negative —
    *not* testable; won only by confinement / least-authority).
  - **Formal methods (TLA+)** verify the *model*, not the running code, so they
    relocate the oracle problem rather than remove it; tests are still needed.
  - **Industrial evidence for agent code** converges on *assured gating*: never
    trust LLM output; accept an artifact only if it passes mechanical,
    tamper-resistant checks (build + non-flaky + **mutation-kill**) — Meta's
    Assured LLMSE / ACH.

**G. Self-critical realization (where we stopped).**
The research mostly answered a **different question** than the one we started
with. Meta's mutation-kill gating is about whether *tests are any good* — it
does **not** tell you what to mock, or how to isolate dependencies uniformly
across three languages. The original blocker (goals #2–#5) is therefore still
**open**.

## Where it actually stands

**No agreed solution.** The genuine fork — none of these chosen — is:

1. **Relax a constraint** on the isolation mechanism: drop Windows, or accept a
   *per-language* isolation mechanism (not one uniform command), or accept
   "good enough" blocking rather than airtight, or accept heavier tooling.
2. **Change the objective:** abandon network-isolation-as-the-mechanism and
   adopt **mutation-kill gating** (Meta's approach) as the enforceable,
   deterministic rule. Well-evidenced and agent-shaped — but it answers "are the
   tests good?", not "what is mocked / how are deps isolated?", so it is a
   *different* goal, not a drop-in answer to the original one.
3. **Accept the original goal as stated may be over-constrained** (uniform +
   lightweight + airtight + cross-OS incl. Windows + deterministic) and decide
   explicitly what to give up.

Until one of those is chosen, there is nothing to implement.

## Do NOT re-propose (already rejected)

- A container per test run — too heavy.
- Static exemption / "CI is the gate" / sandbox only the code that can reach the
  network — hard no.
- Dependency injection as the *general* substitution mechanism — disliked.

## Pointers

- Literature survey: [`TESTING-PHILOSOPHIES.md`](./TESTING-PHILOSOPHIES.md)
- Unrelated in-flight work (not part of this decision): PR #228 (SQLite
  extension loading, #225) is green but unmerged; #229 / #230 are the Python /
  TypeScript parity follow-ups.
