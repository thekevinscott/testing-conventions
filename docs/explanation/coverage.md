---
description: Why the coverage floor is a strict 100% with reasoned exemptions — and how the same floor is measured over a pull request's changed lines.
---

# Coverage

`unit coverage` is the second rung of the [unit ladder](./#the-unit-ladder-exist-→-run-→-verify): does
the test **run** the code? This page explains why the floor is a strict 100% rather than a
comfortable 85, and how the same floor gates a pull request's changed lines.

## Why a 100% floor

A coverage floor below 100 hides a permanent, unexamined gap. At `fail_under = 85`, fifteen percent
of the code is uncovered *and nobody has said which fifteen or why* — the number stays green while
the uncovered region drifts to wherever tests are hardest to write, which is exactly where the bugs
are. The floor rots silently, because a regression from 91 to 87 still passes.

The standard inverts this: the floor is **100% of what you didn't explicitly exempt**. Code that
genuinely shouldn't count — a version-conditional import, a defensive branch — is excluded by a
reason-required, [line-scoped exemption](./scoping#exemptions-are-line-scoped-where-it-counts)
rather than by slack in the number. The result is that every uncovered line is either covered,
or named with a reason, and the whole omission surface is auditable in one file.

## What it enforces

<!-- #region enforces -->
The floor applies to the **unit suite only** — test files are excluded from the denominator, and
exempted files/lines are lifted from it. The exact keys and defaults are in the
[configuration reference](/reference/config#coverage).

- **Python** — the suite runs under `coverage.py` with branch coverage on; the combined line +
  branch total meets `fail_under`.
- **TypeScript** — the suite runs under `vitest` v8 coverage; **four independent metrics** (lines,
  branches, functions, statements) each meet their floor, because line coverage can read 100% while
  a branch lags.
- **Rust** — the suite runs under `cargo llvm-cov --lib`, so the floor measures the same unit-only
  slice the other languages measure (the integration tier under `tests/` stays out of the number).
  The default floors **lines** only; three finer metrics are opt-in: `regions` is a Rust-only
  sub-line metric with a harsher bar and no cross-language analog, `functions` mirrors
  TypeScript's, and `branch` instruments on a nightly toolchain, which the crate pins in its own
  `rust-toolchain.toml` (with `llvm-tools-preview`) — the coverage run reads that pin, in CI and
  locally alike (the [config reference](/reference/config#coverage) has the exact keys). Keeping
  them opt-in keeps Rust's default floor line-shaped like Python's — the
  [parity](/explanation/#parity-over-cleverness) call, with the asymmetry named here.
<!-- #endregion enforces -->

## The changed-line floor

On pull requests, the same configured floor is *also* measured over only the lines the
`<base>...HEAD` diff added or modified. The thresholds stay the single source of truth; the
diff-scoped job only changes what they're measured over. So a one-line change below the floor
fails however small the diff is, an added file's new lines are subjects (brand-new code must be
covered too), and a change touching no measured line passes vacuously.

Because the diff is judged against the configured floor rather than an implicit 100%, the two
coincide only at a 100 floor — one more reason the strict default earns its keep: with it, *every
changed line is exercised*, on every pull request.

## What coverage can't tell you

A covered line is a line that *executed* — nothing more. A test that calls the function and asserts
nothing covers every line it touches. That gap is the next rung's job:
[mutation](./mutation) breaks the covered line and requires a test to notice.
