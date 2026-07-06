---
description: Why CI never runs the e2e suite — and how the attest/verify pair still guarantees it ran against the current code.
---

# E2E attestation

E2E tests run with **no mocks at all** — real services, real credentials, real bills. That's their
value and their cost: they're slow, flaky by nature (the outside world is), and often need secrets
CI shouldn't hold. So the standard takes a position: **CI never runs the e2e suite.** What CI
enforces instead is that *someone ran it, against this code* — a freshness receipt, checked
deterministically.

## The nudge: attest locally, verify in CI

The mechanism is a pair:

- **`e2e attest`** — the write half, run locally (by you or your agent) from the repository root
  (or a package's own root, in a monorepo):

  ```sh
  testing-conventions e2e attest 'pnpm run e2e'
  ```

  It runs your e2e command with output streamed through, then writes `e2e-attestation.json` —
  recording the command, a timestamp, the exit code, and the commit SHA it ran against — and
  commits that file on top. The attestation names the code commit *beneath* it, since a commit
  can't name its own SHA. It writes regardless of the command's exit code: the record is that the
  suite **ran**, and the honest result is part of the record.

- **`e2e verify [path] [--scope <dir>] [--base <ref>]`** — the CI half, run by the
  [workflow](../reference/workflow). It reads the committed attestation at `path` (default: the
  current directory) and passes only when its recorded SHA equals the **latest code commit** under
  `--scope` (default: `path` itself) — the newest commit that changed any path other than the
  attestation itself. It never runs the suite and never inspects the recorded exit code: presence
  and freshness only. In a monorepo, `path` names the package — `e2e verify packages/widget` behaves
  exactly like running `e2e verify` with `packages/widget` as the current directory (#281).
  `--scope` narrows what counts as code independently of where the attestation lives (#294): the
  reusable workflow passes the package's own root for `path` (a manifest's natural home for its
  attestation) but the caller's own `path` input for `--scope`, so a commit touching the package's
  `tests/`, docs, or config — outside what the caller actually scoped their call to — doesn't trip a
  false-stale.

Push new code without re-attesting, and the recorded SHA no longer names the latest code commit —
`verify` fails with a message naming the fix (re-run `attest`). That staleness is the whole nudge:
the e2e suite gets re-run exactly when the code it vouched for has moved on.

## Freshness relative to a branch: `--base`

By default `verify` measures freshness against all reachable history — the latest scoped commit
anywhere in the tree. `--base <ref>` scopes it instead to the commits this branch introduced
(`<base>..HEAD`), the same diff-relative model the [changed-line coverage](./coverage) and
[mutation](./mutation) gates use (#319). A branch that touched the scoped source must name its
newest scoped commit; a branch that touched none of it has nothing to re-attest and passes.

This is what lets a **squash-merging** repo adopt the gate. A squash rewrites a source PR's commits
— including its attestation commit — into one new commit on the base branch, so the SHA the
attestation names no longer exists there. Against absolute history that reads as permanently stale,
reddening every later PR, even ones that never touched the package. Scoped to `<base>..HEAD`,
`verify` asks the only question that matters on a pull request — *did **this** branch change the
scoped source without re-attesting?* — so an unrelated PR stays green and the PR that changes the
source is exactly the one asked to re-attest.

## Why a receipt is enough

The attestation is deliberately weak — it proves a run happened against a commit, and trusts the
runner about everything else. That's the right trade: the alternative (e2e in CI) buys stronger
proof at the cost of flaky builds, leaked credentials, and a suite everyone learns to re-run until
green — which is weaker in practice. A deterministic freshness check on an honest local run keeps
the guarantee CI *can* make: this code was exercised against the real world, by someone who had to
look at the result.

In the workflow the check is **verify-if-present**: it runs whenever a committed
`e2e-attestation.json` sits at the [package root](../monorepo) — the repo root for a
single-package repo, a package's own root in a monorepo — and is skipped, never failed,
otherwise — so a library without an e2e suite adopts the [drop-in](../getting-started) unchanged.
The reusable job passes `--base`, so it runs on pull requests and measures freshness over the
scoped source the branch changed — diff-relative like the changed-line coverage and mutation
jobs, and adoptable by a squash-merging repo.
