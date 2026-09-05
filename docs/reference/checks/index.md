---
description: The checks the workflow runs — one page per check, each carrying its motivation, per-language behavior, run conditions, and configuration surface.
---

# Checks

Every check the workflow runs has one page here carrying its complete picture: why the
check exists, what it enforces per language, when it runs, and every configuration key and
exemption rule that touches it. Land on a check's page and you have everything you need for that
check.

| Check | `gates` name | Asks |
| --- | --- | --- |
| [`unit colocated-test`](./colocated-test) | `colocated-test` | Does a unit test **exist** for every source file — and move with it on a pull request? |
| [`unit one-function-per-file`](./one-function-per-file) | `one-function-per-file` | Does each source file hold at most **one substantial function** — so the file name is the subject? |
| [`unit lint`](./unit-lint) | `unit-lint` | Does every unit test **mock every collaborator**? |
| [`unit coverage`](./unit-coverage) | `unit-coverage` | Does the unit suite **run** the code — whole-tree and on the changed lines? |
| [`unit mutation`](./mutation) | `mutation` | Does the unit suite **verify** the code — break it, and a test fails? |
| [`integration lint`](./integration-lint) | `integration-lint` | Does every integration test run first-party code **for real**? |
| [`packaging`](./packaging) | `packaging` | Does the **built artifact** ship no test files? |
| [`e2e verify`](./e2e-verify) | `e2e-verify` | Does a branch that changed the code record one visible **e2e decision**? |

The `gates` name is the value the [`gates` input](/reference/workflow#inputs) takes to name a
check.

Each page states the facts and opens with the why; the [explanation section](/explanation/)
carries the same ground as discursive essays — the testing model, the unit ladder, and the design
trade-offs behind each check. One deliberate asymmetry: the two lint checks share one essay,
[Isolation](/explanation/isolation), because they enforce a single boundary from opposite sides.

## Running a check directly

The [reusable workflow](/reference/workflow) runs these subcommands for you. A repository on
another CI system runs the same binary itself, one invocation per check per language:

```sh
testing-conventions unit colocated-test src --language python
testing-conventions unit one-function-per-file src --language typescript
testing-conventions unit lint src --language rust
testing-conventions unit coverage src --language python
testing-conventions unit mutation src --language typescript --base origin/main
testing-conventions integration lint src --language python
testing-conventions packaging dist --language python
testing-conventions e2e verify . --base origin/main
```

Each check takes a path — the [`source`](/monorepo#source-vs-the-package-root) scan root, except
`packaging` (the built artifact's root) and `e2e verify` (the package root holding the receipts) —
and these flags:

| Flag | Checks | Meaning |
| --- | --- | --- |
| `--language` | every check except `e2e verify` | Required: `python`, `typescript`, or `rust`. One invocation enforces one language's convention. |
| `--config <file>` | every check except `packaging` and `e2e verify` | The [config file](/reference/config) supplying floors and exemptions, defaulting to `testing-conventions.toml` in the working directory. It is what the workflow's [`config` input](/reference/workflow#inputs) resolves to a path and passes through. Where no file exists, every check runs on its default. |
| `--base <ref>` | `unit colocated-test`, `unit coverage`, `unit mutation`, `e2e verify` | Diffs `<base>...HEAD` and adds that check's diff-scoped behavior: the co-change check, the changed-line floor, diff-scoped mutants, the receipt question. |

`e2e verify` also takes `--scope`, `--extra-scope`, and `--exclude`; [its page](./e2e-verify)
carries them. `testing-conventions --help` prints the full command tree.
