---
description: The unit colocated-test check — every source file has a colocated, matching-named unit test, and on a pull request a changed source changes its test with it.
---

# `unit colocated-test`

Every source file under the scan root has a colocated, matching-named unit test — and on a pull
request, a changed source changes its test with it. This page is the complete record of the
check: why it exists, what it enforces per language, when it runs, and its configuration surface.

## Why this check exists

Colocation makes the unit/integration boundary **structural** — by location rather than a tag or
marker: a test that sits next to `widget.py` and is named `widget_test.py` is a unit test of
`widget`, by construction. 1:1 naming makes an orphan visible: when every source file maps to
exactly one test file, a source with no test is caught by a deterministic scan — and so is a test
whose source went away. This is the first rung of the
[unit ladder](/explanation/#the-unit-ladder-exist-→-run-→-verify) — does a test **exist**? — with
[coverage](./unit-coverage) and [mutation](./mutation) asking the stronger questions above it.

The co-change variant exists because presence isn't enough on a pull request: a source edit that
leaves the colocated test untouched lets the test silently go stale — it still exists, but it
pins the old behavior.

## What it enforces

<!--@include: ../../explanation/colocated-test.md#enforces-->

### Co-change, on pull requests

On pull requests the check also runs **commit-scoped** over the `<base>...HEAD` diff (Python and
TypeScript):

- a **modified** source must have its colocated test in the diff too;
- a **deleted** source that had a test in the base tree must delete or update that test with it;
- an **added** source is not a subject — brand-new code is the
  [changed-line coverage floor](./unit-coverage#the-changed-line-job)'s concern.

Changing a test on its own always passes. Rust units are inline in the same file, so a sibling
test can't go stale and co-change doesn't apply to Rust — a deliberate asymmetry.

## When it runs

| Variant | Runs | As |
| --- | --- | --- |
| Presence | always, tree-wide | a step of the `Static checks (<language>)` job |
| Co-change | pull requests only (Python, TypeScript), over `<base>...HEAD` | a step of the same job |

The scan covers every file under `source`, leaving `<package root>/tests/` to the suite tiers.
The [`gates` input](/reference/workflow#inputs) names it `colocated-test`; the diff-scoped
co-change variant rides with it.

## Configuration

The check takes no keys of its own. Its exemption rules, each a
[`[[<language>.exempt]]` entry](/reference/config#exemptions) with a required `reason`:

| Rule | Lifts |
| --- | --- |
| `colocated-test` | the presence requirement for one file (a launcher shim, a re-export barrel) |
| `co-change` | the co-change requirement for one file, independently of presence |

Both are whole-file rules. Empty and comment-only files are never subjects, with no
configuration.

## Learn more

- [Explanation — Colocated tests](/explanation/colocated-test): why colocation, and why a stale
  test is an invisible orphan.
- [Configure the rules](/guide/configure#exempt-a-file): exempting a file, step by step.
