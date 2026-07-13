---
description: "The packaging check — the built artifact ships no test files: what each registry's artifact is scanned for, the derived build per language, and the three ways the job gets an artifact."
---

# `packaging`

The built artifact — the wheel, tarball, or crate you actually publish — ships no test files.
This page is the complete record of the check: why it reads the artifact instead of the source
tree, what it scans for, how the job gets an artifact, and its configuration surface.

## Why this check exists

Colocated unit tests are the standard's core structural move, and they have one cost: the tests
live in `src/`, exactly where a naive build scoops them up. This check settles that cost — and it
reads the **built artifact** because that's the only honest place: whether tests ship is a
property of the build configuration (the wheel's include list, the npm `files` field, the Cargo
`exclude`), and the artifact is where a build-config regression becomes visible. A source-tree
check would pass forever while a changed manifest quietly started shipping `*_test.py` to every
consumer.

The **build-then-scan** design (#335) makes the check run without a bespoke build job: the job
derives the build from the package's own manifest, runs it, and scans what it wrote.

## What it enforces

<!--@include: ../../explanation/packaging.md#enforces-->

## How the job gets an artifact

In order:

1. **A named artifact.** A built artifact you upload and name via the
   [`packaging_artifact` input](/reference/workflow#inputs) is downloaded and scanned as-is —
   the job builds nothing. An artifact holding no recognized distribution fails the job.
2. **The derived build.** Otherwise the job derives the distribution build from the package's own
   manifest, provisions the toolchain, runs it at the derived
   [package root](/monorepo#source-vs-the-package-root), and scans the result:
   - **Python** (a `pyproject.toml` with a `[project]` table) — `uv build`, scanning `dist/`.
   - **TypeScript** (a `package.json`) — `<pnpm|npm> pack --pack-destination dist`, which runs
     the package's own `prepare` / `prepack` lifecycle.
   - **Rust** (a `Cargo.toml` with a `[package]` table) — `cargo package`, scanning
     `target/package/`, redirected with `--target-dir target` for a workspace member so the
     crate lands at the package's own `target/package/`.

   The primary language is derived from the manifest — a PyO3 binding publishes a Python wheel, a
   napi binding an npm tarball — so a binding's second, private manifest doesn't misroute the
   build.
3. **A committed `dist/`.** When the manifest structurally can't state a build (a workspace-only
   `Cargo.toml`, a non-`[project]` pyproject), a conventional `dist/` already committed at the
   package root is scanned in place.

## When it runs

When any of the three sources above holds; **skipped, never failed** otherwise, so the drop-in is
safe on a repository that hasn't built anything yet. The
[`gates` input](/reference/workflow#inputs) names it `packaging`. It needs a
`testing-conventions` release whose `detect` derives the build — a `version` pinned older falls
back to locate-or-skip.

## Configuration

- [`packaging_artifact`](/reference/workflow#inputs) — the workflow input naming an uploaded
  artifact to scan as-is.
- [`build_command`](/reference/config#build-command) — the one-line declaration for a
  compile-before-pack step npm doesn't standardize: npm runs `prepare` / `prepack` on `pack`, but
  the build script's *name* (`build` in one package, `compile` in the next) is yours to state.

The check honors no exemption rules — a test file in the artifact is always a violation.

## Learn more

- [Explanation — Packaging](/explanation/packaging): the cost colocation imposes, in full.
