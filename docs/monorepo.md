---
description: Adopt the standard on a monorepo — one workflow call per package, scoped by `source`.
---

# Adopt on a Monorepo

A monorepo adopts the standard **one workflow call per package**: each `uses:` job names its
package's real source directory, and the workflow derives everything else from the package
itself. All seven gates run per call. This repository consumes itself the same way — its
dogfood workflow is exactly this file with our package paths.

```yaml
# .github/workflows/conventions.yml
name: Conventions
on: [pull_request]

jobs:
  python:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      source: packages/python/yourpkg

  typescript:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      source: packages/ts/src

  rust:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      source: packages/rust
```

Each `source` names a **source directory** — nothing more. It is the recursive scan root for
exactly two things: language detection and the unit-tier checks (colocated-test, unit-lint, unit
coverage, mutation). It is *not* the package root, and it is not where the standard looks for
suites, builds, or config — those are all **derived**, by walking upward from `source` to the
nearest manifest, never by scanning downward from `source`. The `typescript` call above scans
`packages/ts/src` for sources and unit tests, but installs, builds, and runs suites at
`packages/ts` — the derived package root, one level up. On a single-package repository the
drop-in carries no inputs — `source` defaults to `src`, which also happens to be the package
root's immediate child, so the distinction is invisible until a monorepo pulls the two apart.

## `source` vs. the package root

Two roots, two jobs, and confusing them is the single most common adoption mistake:

- **`source`** is a directory you name. It is scanned **recursively**: colocated-test and
  unit-lint take every matching file under it as a subject, and language detection looks under it
  for each language's file extensions. Nothing about a suite, a build, or a config file lives at
  `source` — it only ever holds first-party sources and their colocated unit tests.
- **The package root** is never named — it's **derived**, by walking upward from `source` through
  its ancestors until one holds a `package.json`, `pyproject.toml`, or `Cargo.toml` (stopping at
  the repository boundary). Every other gate — suites, install, build, packaging, e2e receipts,
  config discovery — reads from **fixed, non-configurable paths relative to this derived root**,
  never from `source` and never recursively.

Moving a file *into* `source` only ever helps the two things that scan `source`. It does nothing
for a gate that reads from the package root — if that gate isn't finding its subjects, the fix is
almost always a path at the package root being named wrong (singular instead of plural, or the
wrong directory entirely), not a file living in the wrong place relative to `source`.

## What each gate scans, and from where

| Gate | Subjects | Root it derives from | How it finds them |
| --- | --- | --- | --- |
| Language detection | file extensions per language | `source` | Recursive scan of `source` |
| `unit colocated-test` | source files paired with same-named unit tests | `source` | Recursive scan of `source`; `<package root>/tests/` is explicitly excluded |
| `unit lint` | colocated unit test files | `source` | Recursive scan of `source`; `<package root>/tests/` is explicitly excluded |
| `unit coverage` / `unit mutation` | source + colocated unit tests | `source` (scanned); package root (installed/run) | Recursive scan of `source` for subjects; toolchain provisioning and the suite run happen at the package root |
| `integration lint` | integration and e2e suite files | package root | **Fixed paths only** — `<package root>/tests/integration/` and `<package root>/tests/e2e/` (plural **`tests`**; Rust: the crate root's `tests/`). Never a recursive scan of `source`, and not configurable |
| Package manager | `packageManager` field, else lockfile | package root | Read from the manifest at the package root |
| Python environment | a `pyproject.toml` `[project]` table | package root | Read from the manifest at the package root |
| Native toolchain | a Rust-compiling build declaration (maturin backend, napi config, `Cargo.toml`) | package root | Read from the manifest at the package root |
| `packaging` | the built distribution | package root | Derives the build from the manifest and scans what it writes — `dist/` (Python/TypeScript) or `target/package/` (Rust), all at the package root |
| `e2e verify` | committed receipts | package root | **Fixed path** — `<package root>/e2e-attestations/`. Never a recursive scan |
| Config file | `testing-conventions.toml` | package root, falling back to repo root | Fixed filename, discovered upward from `source` |

A package whose suites live at `test/integration/` (singular) rather than `tests/integration/`
(plural) sits outside every fixed path above — `integration lint` finds nothing there and stays
silently green, no matter what `source` scans. Renaming `test/` to `tests/` at the package root is
the fix; moving files under `source` changes nothing, because `integration lint` never reads
`source` at all.

The config file's own `exempt` entries follow the same split: an entry's `path` resolves relative
to `source` for every gate except `integration lint`, whose suite subjects resolve relative to the
package root the suite tiers derive from — the same root as the row above, not `source`.

Two optional inputs refine a call: `languages` restricts the detected set explicitly, and
`config` names a config file somewhere other than the package root.

## Escape hatches

Two inputs and one config key cover what a manifest cannot express. They carry the same bar as
[exemptions](./guide/configure#exempt-a-file) — a reasoned last resort, with the manifest-level
fix preferred:

- **`build_command`** — a build step beyond the manifest's own hooks, declared per language in the
  package's own `testing-conventions.toml` and run at the package root. Each language's standard
  build is derived (a maturin/PEP 517 backend, Cargo's `build.rs` and `cargo package`, npm's
  `prepare` / `prepack`); `build_command` names only what an ecosystem structurally can't
  standardize — a PEP 517 backend's absent pre-build shell step, or a TypeScript compile in a
  `build` script npm doesn't run on `pack`. It supplies a necessary fact rather than waiving a
  check, so unlike the other escape hatches it needs no `reason`.
- **`rust_toolchain`** — forces cargo provisioning when no manifest declares the need.
- **`gates`** — restricts a call to named gates, for the rare package where one genuinely
  cannot run.

## Next

- [The testing model](./explanation/) — what each check enforces and why.
- [Configure the rules](./guide/configure) — per-package floors and reasoned exemptions.
- [Workflow reference](./reference/workflow) — every input, check, and run condition.
