---
description: Adopt the standard on a monorepo — one workflow call per package, scoped by `path`.
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
      path: packages/python/yourpkg

  typescript:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      path: packages/ts/src

  rust:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      path: packages/rust
```

## Everything derives from the package

`path` is the scan root and the only scoping mechanism. From it, each call derives:

- **The package root** — the nearest directory at or above `path` holding a `package.json`,
  `pyproject.toml`, or `Cargo.toml`. Dependency installs, builds, `dist/` discovery, and
  `e2e-attestation.json` discovery all happen there.
- **The languages** — every supported language with sources under `path` runs its gates.
- **The package manager** — the manifest's `packageManager` field, else the package's lockfile
  (npm and pnpm).
- **The Python environment** — a `pyproject.toml` with a `[project]` table is provisioned with
  uv, and the suite runs inside that environment.
- **The native toolchain** — a manifest that declares a Rust-compiling build (a maturin
  backend, a napi config, a `Cargo.toml` at the package root) provisions cargo, and the build
  runs through the manifest's own hooks: `uv sync` compiles a maturin module; the install runs
  an npm package's `prepare` script.
- **The config** — a `testing-conventions.toml` at the package root governs that package: its
  floors, and `exempt` entries whose `path` resolves relative to the call's scan root.

Two optional inputs refine a call: `languages` restricts the detected set explicitly, and
`config` names a config file somewhere other than the package root.

## Escape hatches

Three inputs cover what a manifest cannot express. They carry the same bar as
[exemptions](./guide/configure#exempt-a-file) — a reasoned last resort, with the manifest-level
fix preferred:

- **`build_command`** — a build step beyond the manifest's own hooks, run at the package root.
- **`rust_toolchain`** — forces cargo provisioning when no manifest declares the need.
- **`gates`** — restricts a call to named gates, for the rare package where one genuinely
  cannot run.

## Next

- [The testing model](./explanation/) — what each check enforces and why.
- [Configure the rules](./guide/configure) — per-package floors and reasoned exemptions.
- [Workflow reference](./reference/workflow) — every input, check, and run condition.
