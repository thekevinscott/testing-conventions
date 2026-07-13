---
description: The seven checks the workflow runs — one page per check, each carrying its motivation, per-language behavior, run conditions, and configuration surface.
---

# The seven checks

The workflow runs seven checks, and each has one page here carrying its complete picture: why the
check exists, what it enforces per language, when it runs, and every configuration key and
exemption rule that touches it. Land on a check's page and you have everything you need for that
check.

| Check | `gates` name | Asks |
| --- | --- | --- |
| [`unit colocated-test`](./colocated-test) | `colocated-test` | Does a unit test **exist** for every source file — and move with it on a pull request? |
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
