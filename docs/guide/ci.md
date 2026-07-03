---
description: Enforce your conventions with the reusable GitHub Actions workflow — its inputs, diff-scoped jobs, and rolling your own steps.
---

# Enforce conventions in CI

`testing-conventions` ships a **reusable GitHub Actions workflow**, so a consumer repo can
enforce its conventions in one job.

## Use the reusable workflow

Call it from a workflow in your repo, pinned to a tag — no inputs required:

```yaml
# .github/workflows/conventions.yml
name: Conventions
on: [pull_request]

jobs:
  conventions:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
```

It installs the published `testing-conventions` binary and runs every applicable rule for each
language as its own matrix job, failing the build on any violation (with the offending files in
the log). No config file is required: every rule runs with defaults, so this one job opts a new
library into the full check set. Add a `testing-conventions.toml` only to tighten a floor or
declare exemptions.

With no `languages` input, the workflow **auto-detects every supported language present** under
`path` (Python, TypeScript, and Rust) and runs those. A language with no sources is **skipped,
not run**, so the auto-detect default is safe on any library. Pass `languages` (a JSON array) to
**restrict** the run to specific languages — e.g. `'["python"]'` to check Python alone.

The **unit lint** and **integration lint** rules run for Python, TypeScript, and Rust; Rust is
detected as a crate (a `Cargo.toml` **with** `.rs` source) under `path` — a manifest alone, with no
source to measure, is not treated as a crate. Rust is in the coverage matrix too — it
defaults to a `lines = 100` floor (`regions` opt-in; no branch component).

### Versioning — `@v0` is a moving tag

`@v0` is a **moving major tag**, not a frozen release. It always points at the latest released
`main`, so pinning `@v0` opts you into a **rolling release**: bug fixes and new rules reach your CI
on the next run, with no tag bump on your side. (Breaking changes are coordinated across consumers
rather than held back by a semver pin.)

The workflow *file* is pinned at `@v0`; the `testing-conventions` **binary** it runs is pulled
fresh from npm each run (the latest published version), and `@v0` only advances once that binary is
published — so the workflow and the binary it calls always match. To **freeze** the binary, set the
`version` input (e.g. `version: 0.1.0`); the workflow file still tracks `@v0`.

### Inputs

| Input                | Default                    | Description                                                |
| -------------------- | -------------------------- | ---------------------------------------------------------- |
| `languages`          | `''` (auto-detect)         | Empty auto-detects every supported language present under `path`. A JSON array (`python`, `typescript`, `rust`) restricts the run to those named. |
| `path`               | `src`                      | Directory scanned recursively for sources.                 |
| `version`            | latest                     | `testing-conventions` version to install (e.g. `0.1.0`).   |
| `config`             | `testing-conventions.toml` | Optional config file to refine the checks (coverage thresholds, exemptions). Absent → every check runs with defaults. |
| `base`               | `origin/main`              | Base ref for the diff-scoped `--base` jobs, diffed as `<base>...HEAD`. The `*-changed` jobs run on `pull_request` only. |
| `run_e2e`            | `false`                    | Forces the `e2e verify` job on. It is already default-on when a committed `e2e-attestation.json` is present; this runs it regardless. Needs the attestation and full history. |
| `packaging_artifact` | `''`                       | Name of an uploaded build artifact holding your built distributions; when set, the packaging rule downloads and inspects it. When empty, packaging still runs over a conventional `dist/` in the checkout. See [Check the built distribution](#check-the-built-distribution-packaging). |
| `build_command`      | `''`                       | A shell command run after toolchain + dependency setup and **before** the suite, in the same job, for the suite-executing jobs only (`unit coverage`, changed-line `coverage`, `unit mutation`). Use it to build a native module the suite imports. Empty (default) ⇒ no build step. See [Build a native module before the suite](#build-a-native-module-before-the-suite-build-command). |
| `gates`              | `''` (all applicable)      | A JSON array naming which gates run — e.g. `'["colocated-test", "unit-lint", "integration-lint"]'`. Empty runs every applicable gate. A named gate's diff-scoped variant rides with it. See [Run a subset of gates](#run-a-subset-of-gates-gates). |

### Diff-scoped and opt-in checks

Some rules also run a **diff-scoped** variant on **pull requests** only (they check out full history and diff `<base>...HEAD`, with `base` defaulting to `origin/main`):

- **co-change** — `unit colocated-test --base`: a source changed in the PR whose colocated test didn't change with it fails (Python, TypeScript).
- **changed-line coverage** — `unit coverage --base`: a line changed in the PR that lands below the floor fails, no matter how small the diff (Python, TypeScript, Rust).
- **mutation** — `unit mutation --base`: a binary gate that fails the PR on any un-exempted surviving mutant on a changed line (Python, TypeScript, Rust). Diff-scoped because whole-tree mutation is too slow to gate, so there is no whole-tree mutation job — it runs on pull requests only. The tool drives all three engines itself — Stryker, cosmic-ray, and cargo-mutants, the last provisioned on first use (a pinned `cargo install` into the tool's cache).

The whole-tree colocated-test and coverage jobs run regardless; the diff-scoped jobs add the commit-scoped gate on top.

The **`e2e verify`** job checks that your committed `e2e-attestation.json` names the latest code commit and fails (with a re-attest nudge) when the code has moved on. It never runs the e2e suite — CI only confirms someone attested against this code. It's **default-on, verify-if-present**: it runs whenever an `e2e-attestation.json` is committed at the repo root, and is skipped (never failed) otherwise. Set `run_e2e: true` to force it on regardless.

The **coverage** job runs once per requested language that has sources (a language with none is
skipped, not failed). Without a config file it enforces the language's default floor — a strict
100% (Python `fail_under = 100` with branch coverage on; TypeScript all four metrics at 100), and a
`[<language>].coverage` table lowers it. For `python` it runs your unit
suite under `coverage.py` (branch on, `*_test.py` excluded) and fails if the total is below the
floor, installing `coverage` + `pytest`. For `typescript` it runs the suite under `vitest` v8
coverage and fails below any of the four thresholds (`lines` / `branches` / `functions` /
`statements`), installing your project's deps with `pnpm` so `vitest` + `@vitest/coverage-v8` are
present. For `rust` it runs `cargo llvm-cov --lib` — the unit suite only — against the `lines` floor
(default `100`; `regions` is opt-in via `[rust].coverage`, and branch coverage is experimental on
stable, so there's no branch component). A project on a different
toolchain (a non-`pnpm` package manager, or Python sources that need third-party runtime deps
installed) should drive the CLI directly (below) until #56 makes this config-driven.

### Build a native module before the suite (`build_command`)

Most libraries are pure-language, so the suite-executing jobs install the toolchain and run
the suite directly. A project whose unit suite **imports a compiled native module** — a Python
SDK importing a Rust/PyO3 extension built with `maturin develop`, or a TypeScript SDK importing
a napi-rs addon built with `napi build` — needs that module **built first**, or the suite fails
at import before any rule runs.

Set `build_command` to your build step. The workflow runs it after toolchain and dependency
setup and **before the suite**, in the **same job** (so the freshly built module is importable):

```yaml
jobs:
  conventions:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      build_command: uv run maturin develop      # Python / maturin
      # build_command: pnpm build                # TypeScript / napi
```

It runs from the repository root — the same place your own CI runs the build — so the value
mirrors the command you already use. Empty (the default) means no build step, so pure-language
callers are unaffected.

`build_command` is wired only into the jobs that **actually execute the suite** and would import
the module: `unit coverage` (whole-tree), changed-line `coverage --base`, and `unit mutation`.
The static rules — `colocated-test`, `unit lint`, `integration lint` — only parse source and never
import it, and `e2e verify` checks the committed attestation rather than running the e2e suite, so
those jobs neither need nor run the build step.

### Run a subset of gates (`gates`)

By default the workflow runs **every applicable gate**. Pass `gates` — a JSON array of gate
names — to run exactly the gates it names:

```yaml
jobs:
  conventions:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      gates: '["colocated-test", "unit-lint", "integration-lint"]'
```

There are seven gates: `colocated-test`, `unit-lint`, `unit-coverage`, `mutation`,
`integration-lint`, `packaging`, and `e2e-verify`. A named gate still applies its own
conditions — language presence, the pull-request event for the diff-scoped jobs, a discoverable
distribution or attestation — and a gate's **diff-scoped variant rides with it**:
`colocated-test` covers the whole-tree job and the co-change (`--base`) job, and
`unit-coverage` covers the whole-tree and changed-line jobs. The allowlist is authoritative:
it decides which gates run even when `run_e2e` or `packaging_artifact` is set, so a scoped
call stays scoped.

Scoping serves split setups. The static gates — `colocated-test`, `unit-lint`,
`integration-lint` — only parse source, so they run anywhere; the suite-executing gates
(`unit-coverage`, `mutation`) run your test suite, which a native-binding project may need to
build in jobs of its own (say, a cross-language core whose toolchain sits beyond
[`build_command`](#build-a-native-module-before-the-suite-build-command)). Such a repo adopts
the reusable workflow for the static gates and keeps the suite-executing gates in its
build-capable jobs.

### Check the built distribution (packaging)

The other rules read your **source tree**; the **packaging** rule reads your **built
distribution**. It verifies that no test file slipped into the artifact you publish. It's
**default-on, locate-or-skip**: it runs over a conventional `dist/` directory found in the
checkout, or over a built artifact you upload — and is skipped, never failed, when neither
exists. To check a freshly built dist, build it in your own job, upload it with
`actions/upload-artifact`, and pass that artifact's name as `packaging_artifact`:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - run: python -m build            # → dist/*.whl, dist/*.tar.gz
      - uses: actions/upload-artifact@v7
        with:
          name: dist
          path: dist/

  conventions:
    needs: build
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      packaging_artifact: dist
```

The packaging job inspects every distribution it finds, inferring the language from each file's
extension: `.whl` / `.tar.gz` (Python wheel / sdist), `.tgz` (`npm pack` tarball, TypeScript),
`.crate` (Cargo crate, Rust). It fails (naming the offending path) if any of them ships a test
file. When `packaging_artifact` is set but holds no recognized distribution, that's a
misconfigured upload and the job fails; when neither an artifact nor a `dist/` exists, the
packaging job is skipped, never failed.

## Roll your own

The CLI is a single binary. Install it (see [Use the CLI directly](./cli)) and call each
rule as its own step, naming the language with the required `--language` flag:

```yaml
- run: testing-conventions unit colocated-test --language python src/
- run: testing-conventions unit colocated-test --language typescript src/
- run: testing-conventions unit lint --language python --config testing-conventions.toml src/
- run: testing-conventions unit lint --language typescript --config testing-conventions.toml src/
- run: testing-conventions unit lint --language rust --config testing-conventions.toml .   # a crate root
- run: testing-conventions unit coverage --language python --config testing-conventions.toml src/
- run: testing-conventions unit coverage --language typescript --config testing-conventions.toml src/
- run: testing-conventions unit colocated-test --language python --base origin/main src/   # commit-scoped co-change: needs the PR base ref fetched
- run: testing-conventions integration lint --language python --config testing-conventions.toml src/
- run: testing-conventions integration lint --language typescript --config testing-conventions.toml src/
- run: testing-conventions integration lint --language rust --config testing-conventions.toml .   # crate root (scans tests/)
- run: testing-conventions packaging dist/my_pkg-0.1.0-py3-none-any.whl --language python  # built dist, not src/
```

Either way, the non-zero exit on a violation is what fails the build.
