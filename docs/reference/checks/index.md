---
description: "The GitHub Actions jobs the workflow runs — one page per job, each carrying the check(s) it runs, its run conditions, and links to each check's complete record."
---

# Checks

The reusable workflow runs a consumer's checks as **GitHub Actions jobs**. Each job that can fail
has a page here: a job that bundles several checks has its own page, and each check it runs has its
own page below it carrying the complete per-language record — why it exists, what it enforces,
when it runs, and every configuration key and exemption name that touches it.

## The jobs

One [`uses:` call](../workflow) runs these jobs, one per detected language where the job applies.
The name in the table is the name you see on a pull request's check list.

| Job | Check(s) | Runs |
| --- | --- | --- |
| [`Static checks`](./static-checks) (`<language>`) | `colocated-test`, `one-function-per-file`, `unit-lint`, `integration-lint` | always — the four source-scanning checks as steps of one job |
| [`Unit-test coverage`](./unit-coverage) (`<language>`) | `unit-coverage` | always |
| [`Unit-test coverage — changed lines`](./unit-coverage#the-changed-line-job) (`<language>`) | `unit-coverage` | pull requests only |
| [`Unit mutation — changed lines`](./mutation) (`<language>`) | `mutation` | pull requests only |
| [`Packaging`](./packaging) | `packaging` | when a build is derivable, an artifact is named, or a `dist/` is committed |
| [`E2E attestation freshness`](./e2e-verify) | `e2e-verify` | when receipts are present, on pull requests |
| [`CHANGELOG + MIGRATIONS touched`](./changelog) | `changelog` | pull requests only, when the gate is selected |

A job skipped by [`gates`](../workflow#inputs) is absent from CI. A check left out of `gates` is
skipped whether it runs as its own job or as a step of `Static checks`, and a check's diff-scoped
mode rides with it (`colocated-test` covers the co-change step, `unit-coverage` the changed-line
job).

## The checks

The check name is the value the [`gates` input](../workflow#inputs) takes to name a check, and
the value that appears in a `rules = […]` exemption. Each check's page opens with the why and
carries the complete factual record.

| Check | Job | Asks |
| --- | --- | --- |
| [`colocated-test`](./colocated-test) | Static checks | Does a unit test **exist** for every source file — and move with it on a pull request? |
| [`one-function-per-file`](./one-function-per-file) | Static checks | Does each source file hold at most **one substantial function** — so the file name is the subject? |
| [`unit-lint`](./unit-lint) | Static checks | Does every unit test **mock every collaborator**? |
| [`integration-lint`](./integration-lint) | Static checks | Does every integration test run first-party code **for real**? |
| [`unit-coverage`](./unit-coverage) | Unit-test coverage | Does the unit suite **run** the code — whole-tree and on the changed lines? |
| [`mutation`](./mutation) | Unit mutation | Does the unit suite **verify** the code — break it, and a test fails? |
| [`packaging`](./packaging) | Packaging | Does the **built artifact** ship no test files? |
| [`e2e-verify`](./e2e-verify) | E2E attestation freshness | Does a branch that changed the code record one visible **e2e decision**? |
| [`changelog`](./changelog) | run from the CLI | Does a pull request that changed a package's **public surface** add a fragment recording it? |

Each check's page states the facts and opens with the why; the [explanation section](/explanation/)
carries the same ground as discursive essays — the testing model, the unit ladder, and the design
trade-offs behind each check. One deliberate asymmetry: the two lint checks share one essay,
[Isolation](/explanation/isolation), because they enforce a single boundary from opposite sides.
