# Enforce conventions in CI

`testing-conventions` ships a **reusable GitHub Actions workflow**, so a consumer repo can
enforce its conventions in one job.

## Use the reusable workflow

Call it from a workflow in your repo, pinned to a tag:

```yaml
# .github/workflows/conventions.yml
name: Conventions
on: [pull_request]

jobs:
  conventions:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      languages: '["python", "typescript"]'
      path: src
```

It installs the published `testing-conventions` binary and runs every requested rule for each
language as its own matrix job, failing the build on any violation (with the offending files in
the log). No config file is required: every rule runs with defaults, so this one job opts a new
library into the full check set. Add a `testing-conventions.toml` only to tighten a floor or
declare exemptions.

A requested language the repo has no sources for is **skipped, not run**: the workflow scans
`path` for each language first. So the zero-config default `["python", "typescript"]` is safe to
keep on a single-language library, and the absent language's jobs don't fire.

The **unit lint** rule runs for Python, TypeScript, and Rust. Rust isn't part of the default
`languages`, so the scan detects it separately for this job: request `rust` and the workflow
looks for a crate (a `Cargo.toml` / `*.rs`) under `path`. Keeping Rust on its own set holds it
out of the coverage matrix, which has no Rust toolchain.

### Inputs

| Input                | Default                     | Description                                                |
| -------------------- | --------------------------- | ---------------------------------------------------------- |
| `languages`          | `["python", "typescript"]`  | JSON array of languages to check (`python`, `typescript`; also `rust`, for the unit-lint, integration-lint, and coverage rules). |
| `path`               | `src`                       | Directory scanned recursively for sources.                 |
| `version`            | latest                      | `testing-conventions` version to install (e.g. `0.1.0`).   |
| `config`             | `testing-conventions.toml`  | Optional config file to refine the checks (coverage thresholds, exemptions). Absent → every check runs with defaults. |
| `base`               | `origin/main`               | Base ref for the diff-scoped `--base` jobs, diffed as `<base>...HEAD`. The `*-changed` jobs run on `pull_request` only. |
| `run_e2e`            | `false`                     | Opt in to the `e2e verify` job: fail if the committed `e2e-attestation.json` doesn't name the latest code commit. Needs a committed attestation and full history. |
| `packaging_artifact` | `''` (skipped)              | Name of an uploaded build artifact holding your built distributions; when set, the packaging rule downloads and inspects it. See [Check the built distribution](#check-the-built-distribution-packaging). |

### Diff-scoped and opt-in checks

Two rules also run a **diff-scoped** variant as its own `*-changed` job, on **pull requests** only (they check out full history and diff `<base>...HEAD`, with `base` defaulting to `origin/main`):

- **co-change** — `unit colocated-test --base`: a source changed in the PR whose colocated test didn't change with it fails (Python, TypeScript).
- **changed-line coverage** — `unit coverage --base`: a line changed in the PR that lands below the floor fails, no matter how small the diff (Python, TypeScript, Rust).

The whole-tree colocated-test and coverage jobs run regardless; the `*-changed` jobs add the commit-scoped gate on top.

Set `run_e2e: true` to add an **`e2e verify`** job: it checks that your committed `e2e-attestation.json` names the latest code commit and fails (with a re-attest nudge) when the code has moved on. It never runs the e2e suite — CI only confirms someone attested against this code. Off by default, since it needs a committed attestation and full git history.

The **coverage** job runs once per requested language that has sources (a language with none is
skipped, not failed). Without a config file it enforces the language's default floor (Python
`fail_under = 85` with branch coverage on; TypeScript `lines` / `functions` / `statements` 80 and
`branches` 75), and a `[<language>].coverage` table overrides it. For `python` it runs your unit
suite under `coverage.py` (branch on, `*_test.py` excluded) and fails if the total is below the
floor, installing `coverage` + `pytest`. For `typescript` it runs the suite under `vitest` v8
coverage and fails below any of the four thresholds (`lines` / `branches` / `functions` /
`statements`), installing your project's deps with `pnpm` so `vitest` + `@vitest/coverage-v8` are
present. For `rust` it runs `cargo llvm-cov` against the `regions` / `lines` floor; Rust has no
default floor, so the crate must declare a `[rust].coverage` table. A project on a different
toolchain (a non-`pnpm` package manager, or Python sources that need third-party runtime deps
installed) should drive the CLI directly (below) until #56 makes this config-driven.

### Check the built distribution (packaging)

The other rules read your **source tree**; the **packaging** rule reads your **built
distribution**. It verifies that no test file slipped into the artifact you publish. A built
artifact only exists after a build, so this rule doesn't ride the source matrix. Build your
dists in your own job, upload them with `actions/upload-artifact`, and pass that artifact's name
as `packaging_artifact`:

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

The packaging job downloads the artifact and inspects every distribution it finds, inferring the
language from each file's extension: `.whl` / `.tar.gz` (Python wheel / sdist), `.tgz`
(`npm pack` tarball, TypeScript), `.crate` (Cargo crate, Rust). It fails (naming the offending
path) if any of them ships a test file, and also fails if the artifact held no recognized
distribution at all, which signals a misconfigured upload. Leave `packaging_artifact` unset and
the packaging job is skipped, never failed.

## Roll your own

The CLI is a single binary. Install it (see [Getting Started](../getting-started)) and call each
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
