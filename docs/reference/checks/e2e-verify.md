---
description: "The e2e verify check — a branch that changes the scoped source records one visible e2e decision: the attest/verify pair, the two diff questions, and the extra-scope keys."
---

# `e2e verify`

A branch that changes the scoped source records one visible e2e decision — a receipt in the
branch's own diff, checked deterministically, with CI never running the e2e suite. This page is
the complete record of the check: the attest/verify pair, the two questions the gate asks, when
it runs, and its configuration surface.

## Why this check exists

E2E tests run with no mocks at all — real services, real credentials, real bills. They're slow,
expensive, and flaky by nature, so the standard takes a position: **CI never runs the e2e suite,
and never demands a fixed amount of e2e running.** E2E runs ad hoc, on the runner's judgment.
What CI enforces is smaller and sharper: the question "does this change warrant an e2e run, and
which one?" is put to the branch's author exactly once, at the moment it applies, and the answer
lands in the diff where review can see it.

The receipt is deliberately weak — it proves a decision was made and records what ran. Running
e2e in CI buys stronger proof at the cost of flaky builds, leaked credentials, and a suite
everyone learns to re-run until green; a deterministic floor of local runs prices the receipt in
hours and real money, which teaches the same lesson: make the suite trivial, or make the gate
lie.

Both of the gate's questions are **content** questions over the `<base>...HEAD` diff, not history
questions (#319): a rebase, a squash merge, a force-push — none disturb a receipt, because none
change what the branch changed.

## What it enforces

`e2e verify` asks two questions, each a plain content diff of `<base>...HEAD`:

1. **Did this branch change the scoped source?** The scope is the caller's own `source`, joined
   with every declared extra scope and minus every exclude, with the receipts themselves
   excluded. An empty diff passes: the branch owes no decision.
2. **Does this branch's diff add or update a receipt?** A receipt added or updated under
   `e2e-attestations/` passes; otherwise the check fails, naming the fix.

It never runs the suite, never inspects the recorded command or exit code, and never compares
commit SHAs. The receipt is a decision for the **branch**, not a stamp on its newest commit:
pushing more commits after attesting leaves the check green, and an author who judges that later
commits change the picture re-runs `attest`, overwriting the receipt in place.

## The receipt: `e2e attest`

The write half runs locally (by you or your agent) on the branch carrying the work:

```sh
testing-conventions e2e attest 'pnpm run e2e'
```

It runs your command with output streamed through, then writes and commits
`e2e-attestations/<branch>.json` — the command, a timestamp, the exit code, and the commit it ran
against. **The command is yours to choose, and the choice is the judgment the receipt records**:
the full suite, the one suite covering the contract this change touches, or a no-op for a change
you judge needs no run at all. `attest` writes regardless of exit code — the honest result is
part of the record. The receipt is keyed by the branch name as a sanitized slug
(`testing-conventions e2e slug` prints it), so parallel pull requests write distinct files, and
`attest` prunes the receipts other branches left behind.

## When it runs

On pull requests, whenever committed receipts (`e2e-attestations/`) sit at the
[package root](/monorepo#source-vs-the-package-root); **skipped, never failed**
otherwise, so a library without an e2e suite adopts the drop-in unchanged. The
[`run_e2e` input](/reference/workflow#inputs) forces it on. It is diff-scoped like the
changed-line coverage and mutation jobs, so it runs on `pull_request` only and needs full
history. The [`gates` input](/reference/workflow#inputs) names it `e2e-verify`.

## Configuration

- [`run_e2e`](/reference/workflow#inputs) — the workflow input forcing the job on before the
  first receipt exists.
- [`[e2e] extra_scope` and `exclude`](/reference/config#e2e-extra-scope-and-exclude) — for a
  package whose e2e artifact is compiled from a **shared source tree beside it** (a native core
  bound into several language bindings), which no scope at or below the package root can reach.
  Declared in the package's own `testing-conventions.toml`:

  ```toml
  [e2e]
  extra_scope = ["packages/rust/src"]
  exclude = ["packages/rust/src/cli", "packages/rust/src/bin"]
  ```

  `extra_scope` names repo-root-relative directories whose changes join the scoped diff;
  `exclude` carves feature-gated subtrees back out. A path that resolves to no git-tracked
  directory is an error that names the bad root and exits non-zero, so a misspelled core path
  fails loudly rather than silently dropping out of the diff (#391).

The check honors no exemption rules — a branch that changed the scoped source either carries a
receipt or fails.

## Learn more

- [Explanation — E2E attestation](/explanation/e2e): why a receipt is enough, in full.
