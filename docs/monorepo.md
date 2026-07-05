---
description: Adopt the standard on a monorepo — one workflow call per package, each scoped to that package's source directory, language, and config.
---

# Adopt on a Monorepo

The [drop-in](./getting-started)'s defaults — scan `src`, auto-detect every language — fit a
single-package repository. A monorepo holds several packages, each with its own source root and
its own conventions surface, so it adopts the standard **one workflow call per package**: each
`uses:` job scopes the scan to that package's real source directory. This repository consumes
itself the same way (its dogfood workflow points `path` at each package's source dir).

## One call per package

Give each package its own job, restricted to its language, scoped to its source directory, and
carrying its own config:

```yaml
# .github/workflows/conventions.yml
name: Conventions
on: [pull_request]

jobs:
  python:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      languages: '["python"]'
      path: packages/python/yourpkg
      config: packages/python/testing-conventions.toml

  typescript:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      languages: '["typescript"]'
      path: packages/ts/src
      config: packages/ts/testing-conventions.toml

  rust:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      languages: '["rust"]'
      path: packages/rust
```

Three inputs do the scoping (the [workflow reference](./reference/workflow) lists them all):

- **`path`** is the scan root, and it is the *only* scoping mechanism: point it at each package's
  real source directory and everything else — virtualenvs, tooling, sibling packages — sits outside
  every scan, untouched. A deliberate omission *inside* a scan root is a named, reason-required
  [exemption](./guide/configure#exempt-a-file), never a glob; [Scoping and
  exemptions](./explanation/scoping) explains why.
- **`languages`** restricts each call to its package's language, so the Python call runs the Python
  rules and nothing else. (Rust is detected as a crate: a `Cargo.toml` with `.rs` sources under
  `path`.)
- **`config`** names each package's own `testing-conventions.toml`. The config is per-call, and an
  `exempt` entry's `path` resolves **relative to that call's scan root** — each package's floors
  and exemptions live next to the package they govern.

## Packages that build a native module

The `unit coverage` jobs (whole-tree and changed-line) derive their install, provision, and build
step from the package root's own manifest, so a package whose unit suite imports a compiled
module — a PyO3/maturin extension, a napi-rs addon — needs no configuration on its call:

```yaml
  python:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      languages: '["python"]'
      path: packages/python/yourpkg
      config: packages/python/testing-conventions.toml
```

A `pyproject.toml` with a maturin `build-system.build-backend` gets `uv sync`, which builds and
installs the project itself — the native module compiles right there, with cargo already
provisioned from the same manifest. A `package.json` with a `napi` key or an `@napi-rs/cli`
devDependency gets its Rust core built during the npm/pnpm `prepare` script, for the same reason.

The binding crate itself (the Rust source of a PyO3/napi package) is a package like any other:
give it its own `languages: '["rust"]'` call with inline `#[cfg(test)]` tests, or keep it outside
every scan root.

`build_command` and `rust_toolchain` remain as escape hatches for a build the manifest can't
express — set them by hand when auto-derivation doesn't cover your build. `unit mutation` doesn't
yet derive from the package manifest this way, so a mutation call over a native-binding package
still names `build_command` / `rust_toolchain` explicitly.

## What the suite jobs expect

The `unit coverage` jobs install, provision, and build at the **derived package root** — the
nearest directory at-or-above `path`, down to the checkout root, holding a `package.json` /
`pyproject.toml` / `Cargo.toml` (`.` for a single-package repo, so an existing single-package call
is unaffected). A per-package call needs only `path`:

- **TypeScript** installs with the package's own lockfile — `pnpm install --frozen-lockfile` or
  `npm ci`, chosen from the manifest's `packageManager` field or the lockfile present. Both
  managers run the package's `prepare` script during install.
- **Python** installs `coverage` + `pytest` for a plain package; a package whose `pyproject.toml`
  carries a `[project]` table gets `uv sync` instead, installing the project's own dependencies and
  the project itself, with the venv's `coverage` / `pytest` resolved for the suite.

## Next

- [The testing model](./explanation/) — what each check enforces and why.
- [Configure the rules](./guide/configure) — per-package floors and reasoned exemptions.
- [Workflow reference](./reference/workflow) — every input, check, and run condition.
