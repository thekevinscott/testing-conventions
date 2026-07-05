---
description: The reusable GitHub Actions workflow — every input, every check and its run condition, and the @v0 versioning contract.
---

# Workflow

The reusable workflow is the adoption surface: one `uses:` call runs every check as its own job.
This page is the canonical record of its inputs, the checks it runs, and its versioning contract.
To adopt it, start with [Getting Started](../getting-started) or [Adopt on a
monorepo](../monorepo).

```yaml
jobs:
  conventions:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
```

## Inputs

| Input                | Default                    | Description |
| -------------------- | -------------------------- | ----------- |
| `languages`          | `''` (auto-detect)         | Empty auto-detects every supported language present under `path`. A JSON array (`python`, `typescript`, `rust`) restricts the run to those named. Rust is detected as a crate: a `Cargo.toml` **with** `.rs` sources under `path`. |
| `path`               | `src`                      | The scan root — the directory scanned recursively for sources, and the only scoping mechanism (see [Scoping and exemptions](../explanation/scoping)). |
| `config`             | `testing-conventions.toml` | The [config file](./config) supplying floors and exemptions. Per-call; absent means every check runs on its default. |
| `base`               | `origin/main`              | Base ref for the diff-scoped `--base` jobs, diffed as `<base>...HEAD`. The diff-scoped jobs run on `pull_request` only. |
| `version`            | latest                     | `testing-conventions` version to install (e.g. `0.1.0`). |
| `build_command`      | `''` (no build step)       | An escape hatch: a shell command run at the [derived package root](../monorepo#what-the-suite-jobs-expect), after toolchain and dependency setup and **before** the suite, in the suite-executing jobs only (`unit coverage`, changed-line coverage, `unit mutation`) — for a build the package manifest can't express. `unit coverage` / changed-line coverage auto-build a maturin (`uv sync`) or napi (`prepare` script) package from its manifest with no input set; set this only when that auto-derivation doesn't cover your build. The static checks parse source and never run it. |
| `rust_toolchain`     | `false`                    | A manual override: `true` provisions a stable Rust toolchain, with build caching (the cargo registry and `target/`, keyed off `Cargo.lock`), in the suite-executing jobs before `build_command` runs. `unit coverage` / changed-line coverage already auto-provision it when the package root's manifest declares a Rust-compiling build (a `Cargo.toml`, a maturin backend, a napi key); set this by hand only for a build the manifest doesn't express, or for `unit mutation`, which doesn't yet auto-derive it. The `rust` matrix arm always carries its own toolchain. |
| `packaging_artifact` | `''`                       | Name of an uploaded build artifact holding built distributions; when set, the packaging check downloads and inspects it. When empty, packaging runs over a conventional `dist/` in the checkout. An artifact holding no recognized distribution fails the job. |
| `run_e2e`            | `false`                    | Forces the `e2e verify` job on. It is already on when a committed `e2e-attestation.json` is present. Needs the attestation and full history. |
| `gates`              | `''` (all applicable)      | A JSON array naming which checks run (`colocated-test`, `unit-lint`, `unit-coverage`, `mutation`, `integration-lint`, `packaging`, `e2e-verify`). Empty runs every applicable check. A named check's diff-scoped variant rides with it, and the allowlist is authoritative even when `run_e2e` / `packaging_artifact` is set. |

## The checks and when they run

Each check runs as its own job per language present and fails the build on a violation, with the
offending files in the log. Each links to its explanation page.

| Check | Runs | Notes |
| --- | --- | --- |
| [`unit colocated-test`](../explanation/colocated-test) | always | Python, TypeScript, and Rust (inline `#[cfg(test)]` presence). Plus the diff-scoped co-change (`--base`) job on pull requests, for Python and TypeScript — Rust units are inline, so a sibling test can't go stale and co-change doesn't apply. |
| [`unit coverage`](../explanation/coverage) | always | The language's [default floor](./config#coverage), plus the changed-line (`--base`) job on pull requests. |
| [`unit lint`](../explanation/isolation) | always | Python, TypeScript, Rust. |
| [`integration lint`](../explanation/isolation) | always | Python, TypeScript, Rust. |
| [`unit mutation`](../explanation/mutation) | pull requests only | Diff-scoped to the `<base>...HEAD` changed lines; a binary gate — any un-exempted survivor on a changed line fails. |
| [`packaging`](../explanation/packaging) | when a built dist is discoverable | Inspects a `dist/` in the checkout or a named `packaging_artifact`; **skipped, never failed** when neither exists. |
| [`e2e verify`](../explanation/e2e) | when an attestation is present | Runs when a committed `e2e-attestation.json` sits at the repo root; **skipped, never failed** otherwise. `run_e2e` forces it on. |

`unit coverage` and changed-line coverage install, provision, and build at the [derived package
root](../monorepo#what-the-suite-jobs-expect) — `.` for a single-package repo, so an existing
single-package call is unaffected. Python runs under `coverage.py`: `coverage` + `pytest` for a
plain package, or `uv sync` (installing the project's own dependencies and building/installing the
project itself) plus `coverage` + `pytest` for a package with its own `[project]` table. TypeScript
runs under `vitest` v8 coverage, installed with the package's own lockfile (`pnpm install
--frozen-lockfile` or `npm ci`, per its manifest). Rust runs under `cargo llvm-cov --lib` (the unit
suite only) and needs no install step of its own.

## Versioning: `@v0` is a moving tag

`@v0` is a **moving major tag**, not a frozen release. It always points at the latest released
`main`, so pinning `@v0` opts you into a rolling release: fixes and new checks reach your CI on the
next run, with no tag bump on your side. Breaking changes are coordinated across consumers rather
than held back by a semver pin.

The workflow *file* is pinned at `@v0`; the `testing-conventions` **binary** it runs is pulled
fresh from npm each run (the latest published version), and `@v0` only advances once that binary is
published — so the workflow and the binary it calls always match. To freeze the binary, set the
`version` input; the workflow file still tracks `@v0`.
