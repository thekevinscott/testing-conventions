---
description: The reusable GitHub Actions workflow — every input, every check and its run condition, and the @v0 versioning contract.
---

# Workflow

The reusable workflow is the adoption surface: one `uses:` call runs every check. The four static
source scans (colocated-test, its co-change variant, unit-lint, and integration-lint) run as steps
of one `Static checks (<language>)` job per language; the toolchain-heavy suites each run as their
own job. This page is the canonical record of its inputs, the checks it runs, and its versioning
contract.
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
| `rust_toolchain`     | `false`                    | An [escape hatch](../monorepo#escape-hatches): `true` forces a stable Rust toolchain, with build caching (the cargo registry, and `target/` at the derived workspace-aware location — the workspace root's `target/` for a crate that's a workspace member, else the package root's own, keyed off `Cargo.lock`), in the suite-executing jobs before the derived [`[python] build_command`](./config#build_command) runs. `unit coverage` / changed-line coverage / `unit mutation` already auto-provision it when the package root's manifest declares a Rust-compiling build (a `Cargo.toml`, a maturin backend, a napi key — detect's `provision_rust`); set this by hand only for a build the manifest doesn't express. The `rust` matrix arm always carries its own toolchain. |
| `packaging_artifact` | `''`                       | Name of an uploaded build artifact holding built distributions; when set, the packaging check downloads and inspects it as-is, building nothing. When empty, the packaging job derives the distribution build from the package's own manifest (`uv build` / `<pm> pack` / `cargo package`), runs it, and scans what it wrote — or, when the manifest can't state a build, scans a conventional `dist/` already committed at the derived package root (`.` for a single-package repo — see [Adopt on a monorepo](../monorepo)). An artifact holding no recognized distribution fails the job. |
| `run_e2e`            | `false`                    | Forces the `e2e verify` job on. It is already on when a committed `e2e-attestation.json` is present. The job is diff-scoped (`--base`), so it runs on `pull_request` only; it needs the attestation and full history. |
| `gates`              | `''` (all applicable)      | An [escape hatch](../monorepo#escape-hatches): a JSON array naming which checks run (`colocated-test`, `unit-lint`, `unit-coverage`, `mutation`, `integration-lint`, `packaging`, `e2e-verify`), for the rare package where one genuinely cannot run. Empty runs every applicable check. A named check's diff-scoped variant rides with it, and the allowlist is authoritative even when `run_e2e` / `packaging_artifact` is set. |

## The checks and when they run

Each check fails the build on a violation, with the offending files in the log. Each links to its
explanation page. The four static source scans (colocated-test, its co-change variant, unit-lint,
integration-lint) run as steps of one `Static checks (<language>)` job per language — each a
sub-second scan, so one job's setup covers all four; the toolchain-heavy suites (`unit coverage`,
its changed-line variant, `unit mutation`) each run as their own job. Every check keeps its own
`gates` membership and `--base` semantics — a gate left out of `gates` is skipped whether it runs as
a job or a step.

| Check | Runs | Notes |
| --- | --- | --- |
| [`unit colocated-test`](../explanation/colocated-test) | always | Python, TypeScript, and Rust (inline `#[cfg(test)]` presence). Runs as a step of the `Static checks (<language>)` job. Scans `path`, leaving `<package root>/tests/` to the suite tiers. Plus the diff-scoped co-change (`--base`) step on pull requests, for Python and TypeScript — Rust units are inline, so a sibling test can't go stale and co-change doesn't apply. |
| [`unit coverage`](../explanation/coverage) | always | The language's [default floor](./config#coverage), plus the changed-line (`--base`) job on pull requests. |
| [`unit lint`](../explanation/isolation) | always | Python, TypeScript, Rust. Runs as a step of the `Static checks (<language>)` job. Scans `path`, leaving `<package root>/tests/` to the suite tiers. |
| [`integration lint`](../explanation/isolation) | always | Python, TypeScript, Rust. Runs as a step of the `Static checks (<language>)` job. Subjects derive from the [package root](../monorepo#everything-derives-from-the-package): the `tests/integration/` and `tests/e2e/` suites (Rust: the crate root's `tests/`). A test file under `tests/` outside a standard tier is flagged (`unknown-tier`); a tree with no manifest is scanned at `path` directly. |
| [`unit mutation`](../explanation/mutation) | pull requests only | Diff-scoped to the `<base>...HEAD` changed lines; a binary gate — any un-exempted survivor on a changed line fails. Installs and runs from the derived package root: TypeScript picks `npm ci` or `pnpm install --frozen-lockfile` from the package's own manifest/lockfile; Python is provisioned with uv, identically to the coverage jobs (see below). |
| [`packaging`](../explanation/packaging) | when a build is derivable, an artifact is named, or a dist is committed | **Build-then-scan** (#335): derives the distribution build from the package's own manifest — `uv build` (Python `[project]`), `<pm> pack --pack-destination dist` (TypeScript), `cargo package` (Rust `[package]`) — provisions the toolchain, runs it at the derived package root, and scans the result (`dist/` for a wheel/sdist/tarball, `target/package/` for a crate). So a native monorepo adopts with `gates: ["packaging"]` and no bespoke build job. A named `packaging_artifact` is scanned as-is instead, and a committed `dist/` is scanned in place when the manifest can't state a build; **skipped, never failed** when none of the three holds. A `path`-scoped call builds and inspects only its own package. Needs a `testing-conventions` release whose `detect` derives `packaging_build` — a `version` pinned older falls back to locate-or-skip. |
| [`e2e verify`](../explanation/e2e) | when an attestation is present, on pull requests | Runs when a committed `e2e-attestation.json` sits at the [package root](../monorepo) (`path`'s nearest manifest directory, the repo root for a single-package repo); **skipped, never failed** otherwise. Diff-scoped like the changed-line coverage/mutation jobs, so it runs on `pull_request` only: freshness is measured over the scoped source this branch changed (`<base>..HEAD`), not the newest scoped commit in all history (#319) — a branch that changed none of it passes, so an unrelated PR stays green and a squash-merging repo can adopt the gate. The walk is also scoped to the caller's own `path`, not the (possibly broader) package root (#294) — a commit outside `path` but inside the package root doesn't trip a false-stale. A package whose e2e artifact is compiled from a shared source tree beside it (a native core bound into several bindings) declares that tree as an [extra freshness root](../explanation/e2e#a-shared-source-tree-beside-the-package) — `[e2e] extra_scope` / `exclude` in its own `testing-conventions.toml` — so `e2e verify` also accepts a repeatable `--extra-scope <dir>` / `--exclude <dir>` (#333), and the job appends the detected values as those flags — `detect` reads `[e2e] extra_scope` / `exclude` from the package's own config and renders them, so a package declaring neither is byte-identical to before. `run_e2e` forces it on. Needs a `testing-conventions` release carrying the `e2e verify <path> --scope <dir> --base <ref>` arguments (#281, #294, #319) — a `version` pinned older verifies the checkout root instead. |

The suite-executing jobs (`unit coverage`, changed-line coverage, `unit mutation`) install,
provision, and build at the [derived package root](../monorepo#everything-derives-from-the-package) — `.`
for a single-package repo, so an existing single-package call is unaffected. Python is provisioned
by **uv** in all three jobs, with identical steps: an installable package (a `pyproject.toml` with
a `[project]` table) is synced first — `uv sync` installs the project's own dependencies and
builds/installs the project itself — while a plain package gets a fresh `uv venv`; the suite
toolchain (`coverage`, `pytest`, and the `testing-conventions` adapter wheel) then installs into
that same `.venv`, which goes on the suite's `PATH` — so cosmic-ray's spawned pytest (for
`unit mutation`) and the coverage run import the project's dependencies and the adapter together.
uv reads its own index/network configuration (`UV_INDEX_URL`, `uv.toml`); a private index is
declared there. TypeScript runs under `vitest`
v8 coverage, installed with the package's own lockfile (`pnpm install --frozen-lockfile` or
`npm ci`, per its manifest); for `unit mutation` those project dependencies must include
`@stryker-mutator/core` and a runner plugin. Rust runs under `cargo llvm-cov --lib` (the unit
suite only) and needs
no install step of its own; a [`[rust].coverage` `branch` floor](./config#coverage) adds `--branch`,
which uses the nightly toolchain the crate pins in its own `rust-toolchain.toml` (with
`llvm-tools-preview`) — the coverage run reads that pin directly, so the job provisions nothing
extra for it.

A Python package whose suite imports a compiled module builds it first via a
[`[python] build_command`](./config#build_command) in its own `testing-conventions.toml`,
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
