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

### Diff-scoped and opt-in checks

Some rules also run a **diff-scoped** variant on **pull requests** only (they check out full history and diff `<base>...HEAD`, with `base` defaulting to `origin/main`):

- **co-change** — `unit colocated-test --base`: a source changed in the PR whose colocated test didn't change with it fails (Python, TypeScript).
- **changed-line coverage** — `unit coverage --base`: a line changed in the PR that lands below the floor fails, no matter how small the diff (Python, TypeScript, Rust).
- **mutation** — `unit mutation --base`: a binary gate that fails the PR on any un-exempted surviving mutant on a changed line (Python, TypeScript, Rust). Diff-scoped because whole-tree mutation is too slow to gate, so there is no whole-tree mutation job — it runs on pull requests only. Each language installs its engine (cargo-mutants / Stryker / cosmic-ray).

The whole-tree colocated-test and coverage jobs run regardless; the diff-scoped jobs add the commit-scoped gate on top.

The **exemption-approval** gate — `exemptions --base` ([#229](https://github.com/thekevinscott/testing-conventions/issues/229)) — is the same diff-scoped shape: it fails the PR when the diff **adds** a `[[<language>.exempt]]` entry, so each new exemption costs a human greenlight (a reviewer applying the `tc:exemption-approved` label). The detection command ships now; wiring its reusable-workflow job is the remaining step (the command-first, workflow-next path `unit mutation` took). See [Configure — new exemptions need a greenlight](./configure#new-exemptions-need-a-greenlight).

The **`e2e verify`** job checks that your committed `e2e-attestation.json` names the latest code commit and fails (with a re-attest nudge) when the code has moved on. It never runs the e2e suite — CI only confirms someone attested against this code. It's **default-on, verify-if-present**: it runs whenever an `e2e-attestation.json` is committed at the repo root, and is skipped (never failed) otherwise. Set `run_e2e: true` to force it on regardless.

The **coverage** job runs once per requested language that has sources (a language with none is
skipped, not failed). Without a config file it enforces the language's default floor — a strict
100% (Python `fail_under = 100` with branch coverage on; TypeScript all four metrics at 100), and a
`[<language>].coverage` table lowers it. For `python` it runs your unit
suite under `coverage.py` (branch on, `*_test.py` excluded) and fails if the total is below the
floor, installing `coverage` + `pytest`. For `typescript` it runs the suite under `vitest` v8
coverage and fails below any of the four thresholds (`lines` / `branches` / `functions` /
`statements`), installing your project's deps with `pnpm` so `vitest` + `@vitest/coverage-v8` are
present. For `rust` it runs `cargo llvm-cov` against the `lines` floor (default `100`; `regions` is
opt-in via `[rust].coverage`, and branch coverage is experimental on stable, so there's no branch
component). A project on a different
toolchain (a non-`pnpm` package manager, or Python sources that need third-party runtime deps
installed) should drive the CLI directly (below) until #56 makes this config-driven.

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
      - uses: actions/upload-artifact@v4
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
