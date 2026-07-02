---
description: The single source of truth for every default — coverage floors, what runs without config, and the workflow inputs — and why.
---

# Defaults

`testing-conventions` is built to run with **no configuration**: the six-line
[reusable-workflow drop-in](../getting-started) opts a library into every rule with
sensible defaults. This page is the single source of truth for what those defaults are,
what each value is, and why.

Everything here is overridable — in a [`testing-conventions.toml`](./#configuration) or via
the [workflow inputs](../guide/ci#inputs) — but nothing here requires it.

## Coverage floors

Coverage floors apply to the **unit suite only** (test files are excluded from the
denominator). Each language's default is a strict **100%** floor — 100% of what you
didn't explicitly exempt; a `[<language>].coverage` table lowers it.

| Language       | Default floor                                                  | Why |
| -------------- | -------------------------------------------------------------- | --- |
| **Python**     | `branch = true`, `fail_under = 100`                             | Strict by default — 100% of what you don't explicitly exempt. The rule honors `# pragma: no cover`, reason-required `[[python.exempt]]` entries, and the empty/comment-only auto-exemption, so trivia is excluded deliberately, not by a slack floor. |
| **TypeScript** | `lines = 100`, `branches = 100`, `functions = 100`, `statements = 100` | Strict by default, like Python. Still four independent metrics — line coverage can read 100% while a branch lags, so each is enforced separately. |
| **Rust**       | `lines = 100` (`regions` opt-in)                               | Strict by default, like the others — but on **lines** only. `regions` is a Rust-only sub-line metric (opt-in), and branch coverage is experimental on stable, so there's no branch component (see below). |

### Rust: a line floor, no branch

Rust defaults to `lines = 100` — the same line-level floor Python and TypeScript enforce. A
zero-config Rust crate's coverage job runs and gates on lines (it no longer skips or errors for
want of a `[rust].coverage` table). Two deliberate, documented asymmetries:

- **No branch component.** Branch coverage is experimental on stable Rust / `llvm-cov`, so Rust
  can't offer it (Python folds branch into its total; TypeScript has a `branches` metric).
- **`regions` is opt-in.** Region coverage is a Rust-only, sub-line metric with no Python/TypeScript
  analog and a harsher bar, so it isn't in the default. Add it (or lower `lines`) explicitly:

```toml
[rust]
coverage = { lines = 90 }                  # lines only — regions stays unenforced
# coverage = { lines = 90, regions = 90 }  # opt into the finer region floor too
```

### Mutation: a binary gate, no score floor

`unit mutation` (all three languages) has **no percentage default** — and deliberately so. Equivalent
mutants (mutations no test can ever kill) put an unknown, undecidable ceiling below 100%, so a
fixed "≥ N%" floor can be unreachable through no fault of the tests; a score also isn't comparable
across the per-language engines. The gate is instead **binary and on by default**: *no un-exempted
surviving mutant* (on the changed lines, under `--base`) fails the run — there is no report-only
mode and config can't loosen it. The only escape is a reason-required `mutation` exemption for a
survivor that is equivalent or deliberately defensive, so a passing run means every survivor was
killed or explained.

## What runs by default

With the inputs-free [drop-in](../getting-started), the workflow auto-detects every
supported language present under `path` and runs every applicable rule — each as its own
job that fails the build on a violation:

| Rule                  | Default                              | Notes |
| --------------------- | ------------------------------------ | --- |
| `unit colocated-test` | on                                   | Plus the diff-scoped co-change (`--base`) job on pull requests. |
| `unit coverage`       | on                                   | Python / TypeScript / Rust on their default floor above. |
| `unit lint`           | on                                   | Python, TypeScript, Rust. |
| `integration lint`    | on                                   | Python, TypeScript, Rust. |
| `unit mutation`       | on (pull requests only)              | A binary gate over all three languages (`--language <rust\|typescript\|python>`), diff-scoped to the `<base>...HEAD` changed lines — whole-tree mutation is too slow to gate. A PR fails on any un-exempted survivor on a changed line. See below. |
| `packaging`           | on when a built dist is discoverable | Inspects a `dist/` found in the checkout, or a named `packaging_artifact`; **skipped, never failed** when neither exists. |
| `e2e verify`          | on when an attestation is present    | Runs when a committed `e2e-attestation.json` sits at the repo root; **skipped, never failed** otherwise. `run_e2e` forces it on. |

`packaging` and `e2e verify` are *conditionally* on: each needs a precondition (a built
distribution, a committed attestation) and is skipped — never failed — when it's absent, so
the drop-in is safe on a brand-new library.

## Workflow inputs

The [reusable-workflow](../guide/ci) input defaults, all overridable:

| Input                | Default                    | Meaning |
| -------------------- | -------------------------- | --- |
| `languages`          | `''` (empty)               | Empty **auto-detects** every supported language present under `path`. A JSON array (e.g. `'["python"]'`) restricts the run to the languages it names. |
| `path`               | `src`                      | Directory scanned recursively for sources. |
| `config`             | `testing-conventions.toml` | Optional; if absent, every rule runs on its default. |
| `base`               | `origin/main`              | Base ref for the diff-scoped `--base` jobs (pull requests only). |
| `run_e2e`            | `false`                    | Forces `e2e verify` on; it is already on when an attestation is committed. |
| `packaging_artifact` | `''`                       | A named upload artifact to inspect; when empty, packaging still runs over a discoverable `dist/`. |
| `build_command`      | `''` (empty)               | A shell command run before the suite-executing jobs (`unit coverage`, changed-line `coverage`, `unit mutation`) invoke the suite — e.g. `uv run maturin develop` to build a native module the suite imports. Empty ⇒ no build step; the static rules and `e2e verify` never run it. |
| `gates`              | `''` (all applicable)      | Empty runs **every applicable gate**. A JSON array (e.g. `'["colocated-test", "unit-lint", "integration-lint"]'`) runs exactly the gates it names — a named gate's diff-scoped variant rides with it, and the allowlist decides even when `run_e2e` / `packaging_artifact` is set. |
| `version`            | latest                     | The `testing-conventions` version to run. |

## Automatic exemptions

Two kinds of files are skipped from the source rules with **no configuration** — the only
non-explicit exclusions:

- **Empty or comment-only files** — nothing to test (a bare `__init__.py`, say).
- **Declaration files** (`*.d.ts` / `*.d.mts` / `*.d.cts`) — they carry no runtime code.

Anything else that genuinely shouldn't be tested needs an explicit, reason-required
[`exempt`](./#exemptions) entry; see [Configure the rules](../guide/configure#exempt-a-file).
