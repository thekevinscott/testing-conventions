---
description: Why test files never ship in the built artifact — the cost colocation imposes, and how the packaging check settles it per registry.
---

# Packaging

Colocated unit tests are the standard's core structural move — and they have one cost: the tests
live in `src/`, exactly where a naive build scoops them up. The `packaging` check settles that
cost: it inspects the **built artifact** — the thing you actually publish — and fails if any test
file shipped.

## Why the built artifact, not the source tree

Every other check reads the source tree; this one reads the wheel, tarball, or crate. That's the
only honest place to check: whether tests ship is a property of the *build configuration* (the
wheel's include list, the npm `files` field, the Cargo `exclude`), and the artifact is where a
build-config regression becomes visible. A source-tree check would pass forever while a changed
manifest quietly started shipping `*_test.py` to every consumer.

## What it enforces

<!-- #region enforces -->
The check unpacks each distribution and scans for the language's test pattern:

- **Python** — no `*_test.py` in the wheel (`.whl`) or sdist (`.tar.gz`).
- **TypeScript** — no `*.test.*` in the `npm pack` tarball (`.tgz`).
- **Rust** — no crate-root `tests/` directory in the `.crate`. Inline `#[cfg(test)]` units compile
  out of the consumer artifact for free; only the integration `tests/` needs a Cargo `exclude`.
<!-- #endregion enforces -->

## How the workflow gets an artifact to scan

In the [workflow](../reference/workflow) the check is **build-then-scan**: the packaging job
derives the distribution build from the package's own manifest, runs it, and scans what it wrote.
The build the tool derives, from `source` and the manifest alone:

- **Python** (a `pyproject.toml` with a `[project]` table) → `uv build`, writing `dist/*.whl` and
  `*.tar.gz`. The PEP 517 build resolves its own build dependencies and compiles a maturin/PyO3
  core along the way.
- **TypeScript** (a `package.json`) → `<pnpm|npm> pack --pack-destination dist`, which runs the
  package's own `prepare` / `prepack` lifecycle. A compile that lives in a bare `build` script —
  a name npm doesn't standardize — is named once in `[typescript].build_command` and runs first.
- **Rust** (a `Cargo.toml` with a `[package]` table) → `cargo package`, writing
  `target/package/*.crate`. When the crate is a member of an ancestor Cargo workspace, Cargo
  always resolves the target directory — and so `cargo package`'s output — at the *workspace*
  root, regardless of the invoking working directory. The derived command detects that from the
  manifests alone and redirects with `--target-dir target`, so the crate lands at the package's
  own `target/package/`, exactly where the job already scans, rather than at the workspace
  root where the scan would never see it.

So a native monorepo adopts the gate with `gates: ["packaging"]` and no bespoke build job: the job
provisions the toolchain, builds, and scans on its own. The primary language is derived from the
manifest — a PyO3 binding (`pyproject.toml` + `Cargo.toml`) publishes a Python wheel, a napi
binding (`package.json` + `Cargo.toml`) publishes an npm tarball — so a binding's second, private
manifest doesn't misroute the build.

The job still runs when you supply a prebuilt distribution instead: a built artifact you upload and
name via the `packaging_artifact` input is scanned as-is, and a conventional `dist/` already
committed in the checkout is scanned in place. When the manifest structurally can't state a build
(a workspace-only `Cargo.toml`, a non-`[project]` pyproject), the job falls back to that committed
`dist/`. It is skipped — never failed — when none of the three holds, so the drop-in is safe on a
repository that hasn't built anything yet. A named artifact holding no recognized distribution is a
misconfigured upload, and that fails.
