---
description: The unit coverage check — the unit suite clears a 100% floor per language, whole-tree and over a pull request's changed lines; every key, default, and the line-scoped exemption.
---

# `unit coverage`

The unit suite clears **a 100% floor** — whole-tree on every run, and over the changed lines on
every pull request. This page is the complete record of the check: why the floor is strict, what
each language measures, when the two jobs run, and every key that tunes it.

## Why this check exists

A test can exist without running the code — coverage is the
[unit ladder](/explanation/#the-unit-ladder-exist-→-run-→-verify)'s second rung: does the test
**run** it? And the floor is 100 rather than a comfortable 85 because a lower floor hides a
permanent, unexamined gap: at `fail_under = 85`, fifteen percent of the code is uncovered *and
nobody has said which fifteen or why* — the number stays green while the uncovered region drifts
to wherever tests are hardest to write. The standard inverts this: the floor is **100% of what
you didn't explicitly exempt**, so every uncovered line is either covered or named with a reason.

A covered line proves execution, nothing more — a test that calls the function and asserts
nothing covers every line it touches. [`unit mutation`](./mutation) is the rung above.

## What it enforces

<!--@include: ../../explanation/coverage.md#enforces-->

## The changed-line job

On pull requests, the same configured floor is also measured over only the lines the
`<base>...HEAD` diff added or modified. The thresholds stay the single source of truth; the
diff-scoped job only changes what they're measured over — a one-line change below the floor fails
however small the diff is, an added file's new lines are subjects, and a change touching no
measured line passes vacuously.

## When it runs

| Variant | Runs | As |
| --- | --- | --- |
| Whole-tree | always | its own job per language |
| Changed-line | pull requests only, over `<base>...HEAD` | its own job per language |

Both jobs install, provision, and build at the derived
[package root](/monorepo#source-vs-the-package-root): Python is provisioned by **uv**
(`uv sync` for an installable package, else a fresh `uv venv`, with the suite toolchain installed
into that same `.venv`); TypeScript installs with the package's own lockfile
(`pnpm install --frozen-lockfile` or `npm ci`) and runs `vitest` v8 coverage; Rust runs
`cargo llvm-cov --lib` with no install step of its own. A package whose suite imports a compiled
module builds it first via [`build_command`](/reference/config#build-command). The
[`gates` input](/reference/workflow#inputs) names it `unit-coverage`; the changed-line variant
rides with it.

## Configuration

A `[<language>].coverage` table is a **partial override** — set only the fields you want to move,
the rest keep their default, and a typo'd key is rejected:

| Language | Keys | Default |
| --- | --- | --- |
| **Python** | `branch`, `fail_under` | `branch = true`, `fail_under = 100` — coverage.py's combined line + branch total. |
| **TypeScript** | `lines`, `branches`, `functions`, `statements` | All four at `100`, each enforced independently. |
| **Rust** | `lines`, `regions`, `functions`, `branch` | `lines = 100`; the rest are opt-in floors. A `branch` floor adds `--branch`, which runs on the nightly toolchain the crate pins in its own `rust-toolchain.toml` (with `llvm-tools-preview`). |

Related keys and surfaces:

- [`[rust] features`](/reference/config#rust-features) — cargo features the coverage run enables,
  so `#[cfg(feature = ...)]` code is compiled and measured.
- [`build_command`](/reference/config#build-command) — a pre-suite build the manifest can't
  express (a maturin/PyO3 extension).
- The floor is also published as a [shared test config](/reference/config#shared-test-configs)
  your own runner extends, so a local run is held to the same floor CI enforces.

A `coverage` exemption is **line-scoped, never whole-file**: the entry carries a `lines` list
naming the exact failing lines, with a required `reason`, and a determinism guard rejects a
listed line that isn't actually failing — see
[the exemption schema](/reference/config#line-scoped-exemptions).

## Learn more

- [Explanation — Coverage](/explanation/coverage): why 100, in full.
- [Configure the rules](/guide/configure#relax-a-coverage-floor): lowering a floor, step by step.
