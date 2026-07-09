---
description: Why CI never runs the e2e suite — and how the attest/verify pair nudges one visible e2e decision out of every branch that changes the code.
---

# E2E attestation

E2E tests run with **no mocks at all** — real services, real credentials, real bills. That's their
value and their cost: they're slow, expensive, flaky by nature (the outside world is), and often
need secrets CI shouldn't hold. So the standard takes a position: **CI never runs the e2e suite,
and never demands a fixed amount of e2e running.** E2E is run ad hoc, on the runner's judgment.
What CI enforces is smaller and sharper: **a branch that changes the code records one visible e2e
decision** — a receipt in the branch's own diff, checked deterministically.

## The nudge: attest locally, verify in CI

The mechanism is a pair:

- **`e2e attest`** — the write half, run locally (by you or your agent) from the repository root
  (or a package's own root, in a monorepo), on the branch carrying the work:

  ```sh
  testing-conventions e2e attest 'pnpm run e2e'
  ```

  It runs your command with output streamed through, then writes
  `e2e-attestations/<branch>.json` — recording the command, a timestamp, the exit code, and the
  commit it ran against — and commits that file on top. **The command is yours to choose, and the
  choice is the judgment the receipt records**: the full suite, the one suite covering the
  contract this change touches, or a no-op for a change you judge needs no run at all. All are
  valid receipts. `attest` writes regardless of the command's exit code: the record is the
  decision and what ran, and the honest result is part of the record.

  The receipt is keyed by the branch name, sanitized to a lowercase, truncated slug so any branch
  name yields a valid, portable filename; the raw branch name is recorded inside the receipt.
  Parallel pull requests therefore write distinct files and merge cleanly beside each other. The
  derivation is public: `testing-conventions e2e slug [branch]` prints the slug (default: the
  checked-out branch), so a script locates a branch's receipt at
  `e2e-attestations/$(testing-conventions e2e slug).json`. `attest` also deletes the receipts other
  branches left behind — once their PRs merge those files are dead weight, since `verify` reads
  only the current branch's diff — so the directory carries one live receipt at a time.

- **`e2e verify [path] [--scope <dir>] [--base <ref>] [--extra-scope <dir>]… [--exclude <dir>]…`** —
  the CI half, run by the [workflow](../reference/workflow) on pull requests. It asks two
  questions, each a plain content diff of `<base>...HEAD`:

  1. **Did this branch change the scoped source?** The scope is `--scope` (default: `path`
     itself), joined with every `--extra-scope` and minus every `--exclude`, with the receipts
     themselves excluded. An empty diff passes: the branch owes no decision.
  2. **Does this branch's diff add or update a receipt?** A receipt added or updated under
     `path`'s `e2e-attestations/` passes; otherwise the gate fails, naming the fix — run
     `e2e attest` with the command of your choosing.

  It never runs the suite, never inspects the recorded command or exit code, and never compares
  commit SHAs. Deleting another branch's receipt (the prune above) is not a decision — only an
  added or updated receipt answers question 2. In a monorepo, `path` names the package —
  `e2e verify packages/widget` behaves exactly like running `e2e verify` with `packages/widget`
  as the current directory (#281) — and `--scope` narrows what counts as code independently of
  where the receipts live (#294), so a commit touching the package's `tests/`, docs, or config —
  outside what the caller actually scoped their call to — owes no decision. `--scope` names
  `path` or a directory beneath it that git tracks; a `--scope` that resolves to no tracked path
  (a typo, or a directory outside `path`) is an error that names the bad scope and exits
  non-zero, so a misconfigured scope fails loudly at the gate rather than waving a branch through
  (#391).

Change the scoped source without attesting, and `verify` fails with a message naming the fix. That
is the whole nudge: the question "does this change warrant an e2e run, and which one?" is put to
the branch's author exactly once, at the moment it applies, and the answer lands in the diff where
review can see it.

## One decision per branch

The receipt is a decision for the **branch**, not a stamp on its newest commit. Pushing more
commits after attesting leaves the gate green — the branch already made its call, and re-running a
heavy suite on every push would price the gate out of the ad-hoc, judgment-driven use e2e is meant
for. An author who judges that later commits change the picture re-runs `attest`; the receipt is
overwritten in place.

Both of `verify`'s questions are **content** questions, not history questions: `git diff
<base>...HEAD` reads what the branch changed relative to the merge base, not which SHAs carry it.
A rebase onto a moved default branch, a squash merge, a force-push — none disturb a receipt,
because none change the answer to either question. The gate runs on pull requests with `--base`,
diff-relative exactly like the [changed-line coverage](./coverage) and [mutation](./mutation)
gates: a branch that touched none of the scoped source passes trivially, so unrelated PRs stay
green and a squash-merging repo adopts the gate unchanged (#319).

Without `--base`, `verify` has no branch to read a diff from, so it checks presence: a committed
receipt at `path` passes. The reusable workflow always passes `--base`.

## A shared source tree beside the package

`--scope` narrows the scoped diff to a directory *at or below* where the receipts live, so a
package's own subtree defines what counts as code. That misses one monorepo shape: a package whose
e2e artifact is compiled from a **shared source tree that sits beside the package** — a native core
bound into several language bindings (dirsql's `packages/rust` core, compiled into its Python and
TypeScript bindings via PyO3 and napi). That core lives in no binding's subtree, so no binding can
point `--scope` at it, and a PR changing only the core would owe no binding a decision while their
e2e coverage is exactly what it puts at risk.

`--extra-scope <dir>` closes that gap. It names a **repo-root-relative** directory — outside the
package's own `path`, which is the whole point — that joins the scoped diff. A binding declares
the shared core as an extra scope, and a core change puts the e2e question to the branch the same
way a change to the binding's own source would. The flag is repeatable; each occurrence adds one
root. `--exclude <dir>` carves a feature-gated subtree back out: dirsql's core `cli/` and `bin/`
are compiled out of both bindings, so a `cli`-only core change owes them nothing. Excludes are
repo-root relative and repeatable, like extra scopes. Each `--extra-scope` must name a
repo-root-relative directory that git tracks; one resolving to no tracked path (a misspelled core
path) is an error that names the bad root and exits non-zero, so a shared tree stays wired to the
scoped diff rather than silently dropping out of it (#391).

This is a fact about the package's build — *my artifact is compiled from that tree* — so it lives
in the package's own `testing-conventions.toml`, discovered by `detect` like `build_command` and
`config` already are, not in the `uses:` call:

```toml
[e2e]
extra_scope = ["packages/rust/src"]
exclude = ["packages/rust/src/cli", "packages/rust/src/bin"]
```

The diff is git-level and language-agnostic, so this holds across Python, TypeScript, and Rust by
construction. A package that declares nothing scopes the diff to `--scope` alone.

## Why a receipt is enough

The receipt is deliberately weak — it proves a decision was made and records what ran, and trusts
the runner about everything else. That's the right trade twice over. Running e2e in CI buys
stronger proof at the cost of flaky builds, leaked credentials, and a suite everyone learns to
re-run until green. And a *deterministic floor* of local e2e runs — demanding the full suite per
branch, or a fresh run per push — prices the receipt in hours and real money, which teaches the
same lesson: make the suite trivial, or make the gate lie. A receipt that records an honest
judgment keeps the guarantee CI *can* make: someone looked at this change, decided what it needed
against the real world, and put that decision where review sees it.

In the workflow the check is **verify-if-present**: it runs whenever committed receipts sit at the
[package root](../monorepo) (`e2e-attestations/`, the directory `attest` maintains — the repo root
for a single-package repo, a package's own root in a monorepo) and is skipped, never failed,
otherwise — so a library without an e2e suite adopts the [drop-in](../getting-started) unchanged.
