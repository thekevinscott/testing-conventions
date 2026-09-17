---
description: "The Static checks job — one job per language bundling the four source-scanning checks (colocated-test, one-function-per-file, unit-lint, integration-lint) as steps; what each step is, why they share a job, and how to read a red run."
---

# Static checks

`Static checks (<language>)` is the job a consumer sees in CI when any of the four source-scanning
checks fails. It bundles those checks as **steps of one job per language**, because each is a
sub-second scan that parses source text — no suite runs, no dependencies install, no build
compiles — so they share a single checkout and a single CLI download. This page is the entry
point for the job: which step went red, which check that step is, and where its full record lives.

## Why one job

The four checks share everything expensive: a checkout, a node setup, and the CLI. Each scan reads
source and parses it, so a check's step finishes in well under a second. Running them as steps of
one job turns five sub-second scans into one job's worth of setup instead of five jobs each
repeating the same checkout. The toolchain-heavy checks — [`unit-coverage`](./unit-coverage) and
[`mutation`](./mutation) — install, provision, and build, so each gets its own job.

## The steps

| Step | Check | Asks | Runs |
| --- | --- | --- | --- |
| Check colocated test | [`colocated-test`](./colocated-test) | Does a unit test **exist** for every source file? | always |
| Check co-change vs `<base>` | [`colocated-test`](./colocated-test) (co-change) | Does a changed source change its test with it? | pull requests (Python, TypeScript) |
| Check one function per file | [`one-function-per-file`](./one-function-per-file) | Does each source file hold one substantial function? | always |
| Check unit lint | [`unit-lint`](./unit-lint) | Does every unit test mock every collaborator? | always |
| Lint integration tests | [`integration-lint`](./integration-lint) | Does every integration test run first-party code for real? | always |

A step skipped by [`gates`](../workflow#inputs) is absent from the job; a step whose language
isn't present is skipped. Steps run with `!cancelled()`, so a step that follows a failed one
still runs and reports its own result — one job surfaces every violation rather than stopping at
the first.

## When the job runs

Always, one job per detected language with sources under [`source`](../workflow#inputs). The
[`gates`](../workflow#inputs) input gates the whole job: if `gates` names none of `colocated-test`,
`unit-lint`, `one-function-per-file`, `integration-lint`, the job is skipped entirely. Naming any
one of them runs the job, and each step applies its own language-membership condition — a language
present for `colocated-test` but with no integration suite still runs the colocated step and skips
the integration-lint step.

## When `Static checks` goes red

The job's name in CI is `Static checks (<language>)`. A red step names the check and the offending
file in its log. Open the red step to see which check failed, then that check's page carries the
per-language behavior, the exact rule, the exemption that lifts it, and the configuration keys
that tune it.

## Configuration

The job takes no input of its own. Its steps read the shared [`config`](../config) file, and each
check's exemption names and keys live on that check's page. The one input that changes whether the
job runs at all is [`gates`](../workflow#inputs).

## Learn more

- Each check's page (above) for the complete per-language record.
- [Explanation — The testing model](/explanation/): where these checks sit in the unit ladder and
  the isolation boundary.
- [Workflow reference](../workflow): every input and every job's run condition.
