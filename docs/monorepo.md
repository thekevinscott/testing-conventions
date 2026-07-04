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

A package whose unit suite imports a compiled module — a PyO3 extension, a napi-rs addon — needs
that module built before the suite-executing checks run. Two inputs on that package's call handle
it:

```yaml
  python:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      languages: '["python"]'
      path: packages/python/yourpkg
      config: packages/python/testing-conventions.toml
      rust_toolchain: true                      # the build compiles a Rust core
      build_command: uv run maturin develop     # the suite imports the compiled module
```

`build_command` runs after toolchain and dependency setup and before the suite, in the same job;
`rust_toolchain: true` provisions a cached stable Rust toolchain for it. The binding crate itself
(the Rust source of a PyO3/napi package) is a package like any other: give it its own
`languages: '["rust"]'` call with inline `#[cfg(test)]` tests, or keep it outside every scan root.

## What the suite jobs expect

The suite-executing TypeScript jobs (`unit coverage`, `unit mutation`) install dependencies with
`pnpm install --frozen-lockfile` from the repository root — a pnpm workspace with a root lockfile
satisfies this as-is.

## Next

- [The testing model](./explanation/) — what each check enforces and why.
- [Configure the rules](./guide/configure) — per-package floors and reasoned exemptions.
- [Workflow reference](./reference/workflow) — every input, check, and run condition.
