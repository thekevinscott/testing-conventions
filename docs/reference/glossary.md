---
description: "The repo's standard for check, rule, and gate — three words with one meaning each, and the named exceptions where a fourth population borrows one of them."
---

# Glossary

**check**, **rule**, and **gate** each name one thing. This page is the standard; every other page
in this repo — the docs site, `README.md`, and the root `AGENTS.md` — uses the words this way.

- **check** — one of the runnable units the workflow enforces: `colocated-test`,
  `one-function-per-file`, `unit-lint`, `unit-coverage`, `mutation`, `integration-lint`,
  `packaging`, `e2e-verify`, `changelog`. Each is named by the value the [`gates` input](./workflow#inputs)
  takes and that appears in a `rules = […]` [exemption](./config). A check is a unit of
  enforcement, not a CI job — see [Checks](./checks/) for the job shape: some checks run as their
  own job, several share one job as steps, and one check produces two jobs.
- **rule** — an individual assertion inside a lint check: `no-first-party-patch`,
  `no-monkeypatch`, `no-inline-patch`, `no-environ-mutation`, `no-constant-patch`,
  `no-out-of-module-call`, `no-out-of-module-import`, `no-first-party-double`,
  `unmocked-collaborator`, `untyped-mock`, `no-first-party-mock`, `unknown-tier`. A rule lives one
  level below a check, the way an ESLint rule (`no-unused-vars`) lives one level below the linter
  that runs it. [Isolation](../explanation/isolation) and
  [`docs/internals/python/isolation.md`](https://github.com/thekevinscott/testing-conventions/blob/main/docs/internals/python/isolation.md)
  write every sentence at this level and are the pages to copy.
- **gate** — a check in its merge-blocking role, and the `gates` workflow input that names which
  checks run. Use "gate" for the blocking role and the input — "mutation is a binary gate, not a
  score", "an escape hatch needs a reason" — and "check" for the unit itself.

## Three named exceptions

Three populations use one of these words for something other than the definitions above. Each is
its own thing, named here once, so a page that meets it doesn't need to re-argue the boundary.

**The `rules` config key is a mixed namespace.** [`rules = […]`](./config) in an exemption entry
accepts a check id (`colocated-test`, `coverage`, `mutation`) or a rule id
(`no-monkeypatch`, `unmocked-collaborator`) side by side, in the same array. The key name predates
this glossary and stays: it names "any exemptable unit, check or rule," a namespace one level wider
than either definition above, not a fourth term.

**GitHub's own check-runs are a different population.** GitHub Actions reports **check-runs** — a
per-job pass/fail status on a pull request — and the word collides with this repo's "check." The
two do not map 1:1: the workflow declares eight job names, five checks share the one `Static
checks` job, and `unit-coverage` produces two jobs. "Check" in a sentence about the pull-request UI
means a GitHub check-run; "check" in a sentence about `colocated-test` or `mutation` means one of
the nine above. Context carries the distinction; the words are the same.

**`tc-checks` is a third population.** The repo's own internal CI-assertion tooling
(`internals/checks`, invoked as `tc-checks <subcommand>`) verifies this repo's own wiring —
changelog fragments present, a lint rule still registered, a workflow output still consumed. Its
subcommands are not one of the eight checks a consumer's build runs; they're this repo dogfooding
its own gates on itself.
