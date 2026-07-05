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
| `languages`          | `''` (auto-detect)         | A restriction on auto-detection: empty runs every supported language with sources present under `path`; a JSON array (`python`, `typescript`, `rust`) narrows the run to those named. Rust is detected as a crate: a `Cargo.toml` **with** `.rs` sources under `path`. |
| `path`               | `src`                      | The scan root — the directory scanned recursively for sources, and the only scoping mechanism (see [Scoping and exemptions](../explanation/scoping)). |
| `config`             | `testing-conventions.toml` | The [config file](./config) supplying floors and exemptions. Discovery order: an explicit non-default value is used verbatim; otherwise a `testing-conventions.toml` at the derived [package root](../monorepo) wins if present, else the repo-root default. Absent (no file at any of those locations) means every check runs on its default. |
| `base`               | `origin/main`              | Base ref for the diff-scoped `--base` jobs, diffed as `<base>...HEAD`. The diff-scoped jobs run on `pull_request` only. |
| `version`            | latest                     | `testing-conventions` version to install (e.g. `0.1.0`). |
| `rust_toolchain`     | `false`                    | An [escape hatch](../monorepo#escape-hatches): `true` forces a stable Rust toolchain, with build caching (the cargo registry and `target/` under the package root, keyed off `Cargo.lock`), in the suite-executing jobs before the derived [`[python] build_command`](./config#python-build_command) runs. `unit coverage` / changed-line coverage / `unit mutation` already auto-provision it when the package root's manifest declares a Rust-compiling build (a `Cargo.toml`, a maturin backend, a napi key — detect's `provision_rust`); set this by hand only for a build the manifest doesn't express. The `rust` matrix arm always carries its own toolchain. |
| `packaging_artifact` | `''`                       | Name of an uploaded build artifact holding built distributions; when set, the packaging check downloads and inspects it. When empty, packaging runs over a conventional `dist/` at the derived package root (`.` for a single-package repo — see [Adopt on a monorepo](../monorepo)). An artifact holding no recognized distribution fails the job. |
| `run_e2e`            | `false`                    | Forces the `e2e verify` job on. It is already on when a committed `e2e-attestation.json` is present. Needs the attestation and full history. |
| `gates`              | `''` (all applicable)      | An [escape hatch](../monorepo#escape-hatches): a JSON array naming which checks run (`colocated-test`, `unit-lint`, `unit-coverage`, `mutation`, `integration-lint`, `packaging`, `e2e-verify`), for the rare package where one genuinely cannot run. Empty runs every applicable check. A named check's diff-scoped variant rides with it, and the allowlist is authoritative even when `run_e2e` / `packaging_artifact` is set. |

## The checks and when they run

Each check runs as its own job per language present and fails the build on a violation, with the
offending files in the log. Each links to its explanation page.

| Check | Runs | Notes |
| --- | --- | --- |
| [`unit colocated-test`](../explanation/colocated-test) | always | Python, TypeScript, and Rust (inline `#[cfg(test)]` presence). Plus the diff-scoped co-change (`--base`) job on pull requests, for Python and TypeScript — Rust units are inline, so a sibling test can't go stale and co-change doesn't apply. |
| [`unit coverage`](../explanation/coverage) | always | The language's [default floor](./config#coverage), plus the changed-line (`--base`) job on pull requests. |
| [`unit lint`](../explanation/isolation) | always | Python, TypeScript, Rust. |
| [`integration lint`](../explanation/isolation) | always | Python, TypeScript, Rust. |
| [`unit mutation`](../explanation/mutation) | pull requests only | Diff-scoped to the `<base>...HEAD` changed lines; a binary gate — any un-exempted survivor on a changed line fails. Installs and runs from the derived package root: TypeScript picks `npm ci` or `pnpm install --frozen-lockfile` from the package's own manifest/lockfile; Python runs `uv sync` plus an adapter/pytest install into the project's own venv for a `uv`-managed package, or the existing global `pytest` + `testing-conventions` wheel install otherwise. |
| [`packaging`](../explanation/packaging) | when a built dist is discoverable | Inspects a `dist/` at the call's derived package root, or a named `packaging_artifact`; **skipped, never failed** when neither exists. A `path`-scoped call inspects only its own package's `dist/` — a repo-root `dist/` counts only for a call whose derived package root IS the repo root. |
| [`e2e verify`](../explanation/e2e) | when an attestation is present | Runs when a committed `e2e-attestation.json` sits at the [package root](../monorepo) (`path`'s nearest manifest directory, the repo root for a single-package repo); **skipped, never failed** otherwise. `run_e2e` forces it on. Needs a `testing-conventions` release carrying the `e2e verify <path>` argument (#281) — a `version` pinned older verifies the checkout root instead. The CLI also accepts `--scope <dir>` (#294) to narrow the freshness walk independently of `path`; the workflow doesn't wire it yet. |

The suite-executing jobs (`unit coverage`, changed-line coverage, `unit mutation`) install,
provision, and build at the [derived package root](../monorepo#everything-derives-from-the-package) — `.`
for a single-package repo, so an existing single-package call is unaffected. Python runs under
`coverage.py` (for the coverage jobs): `coverage` + `pytest` for a plain package, or `uv sync`
(installing the project's own dependencies and building/installing the project itself) plus
`coverage` + `pytest` for a package with its own `[project]` table. TypeScript runs under `vitest`
v8 coverage, installed with the package's own lockfile (`pnpm install --frozen-lockfile` or
`npm ci`, per its manifest). Rust runs under `cargo llvm-cov --lib` (the unit suite only) and needs
no install step of its own. `unit mutation` installs the same way: TypeScript's project
dependencies (must include `@stryker-mutator/core` and a runner plugin) install with `npm ci` or
`pnpm install --frozen-lockfile`, whichever the package's own manifest/lockfile names; Python
installs `pytest` + the `testing-conventions` wheel globally for a `pip`-only project, or runs
`uv sync` and installs both into the project's own `.venv` for a `uv`-managed one, so cosmic-ray's
spawned pytest can import the project's dependencies and the adapter together.

A Python package whose suite imports a compiled module builds it first via a
[`[python] build_command`](./config#python-build_command) in its own `testing-conventions.toml`,
discovered at the package root like the config file itself. The suite-executing jobs run that
command after toolchain and dependency setup and before the suite; the static checks parse source,
so they run no build. TypeScript and Rust need no key — an npm `prepare` / `postinstall` script and
Cargo's `build.rs` build their compiled modules during dependency install.

## Versioning: `@v0` is a moving tag

`@v0` is a **moving major tag**, not a frozen release. It always points at the latest released
`main`, so pinning `@v0` opts you into a rolling release: fixes and new checks reach your CI on the
next run, with no tag bump on your side. Breaking changes are coordinated across consumers rather
than held back by a semver pin.

The workflow *file* is pinned at `@v0`; the `testing-conventions` **binary** it runs is pulled
fresh from npm each run (the latest published version), and `@v0` only advances once that binary is
published — so the workflow and the binary it calls always match. To freeze the binary, set the
`version` input; the workflow file still tracks `@v0`.
