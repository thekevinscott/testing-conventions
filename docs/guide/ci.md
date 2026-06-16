# Enforce conventions in CI

A convention is only worth something if it runs on every change. `testing-conventions` ships
a **reusable GitHub Actions workflow**, so a consumer repo can enforce its conventions in one
drop-in job.

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
language as its own matrix job, failing the build — with the offending files in the log — on
any violation. No config file is required: every rule runs with sane defaults, so this one job
opts a new library into the full check set. Add a `testing-conventions.toml` only to tighten a
floor or declare exemptions.

A requested language the repo has no sources for is **skipped, not run** — the workflow scans
`path` for each language first — so the zero-config default `["python", "typescript"]` is safe
to keep on a single-language library: the absent language's jobs simply don't fire.

### Inputs

| Input                | Default                     | Description                                                |
| -------------------- | --------------------------- | ---------------------------------------------------------- |
| `languages`          | `["python", "typescript"]`  | JSON array of languages to check (`python`, `typescript`). |
| `path`               | `src`                       | Directory scanned recursively for sources.                 |
| `version`            | latest                      | `testing-conventions` version to install (e.g. `0.1.0`).   |
| `config`             | `testing-conventions.toml`  | Optional config file to refine the checks (coverage thresholds, exemptions). Absent → every check runs with sane defaults. |
| `packaging_artifact` | `''` (skipped)              | Name of an uploaded build artifact holding your built distributions; when set, the packaging rule downloads and inspects it. See [Check the built distribution](#check-the-built-distribution-packaging). |

The **coverage** job runs once per requested language that has sources (a language with none is
skipped, not failed). Without a config file it enforces the
language's default floor — Python `fail_under = 85` with branch coverage on; TypeScript `lines`
/ `functions` / `statements` 80 and `branches` 75 — and a `[<language>].coverage` table
overrides it. For `python` it runs your unit suite under `coverage.py` (branch on, `*_test.py`
excluded) and fails if the total is below the floor, installing `coverage` + `pytest`. For
`typescript` it runs the suite under `vitest` v8 coverage and fails below any of the four
thresholds (`lines` / `branches` / `functions` / `statements`), installing your project's deps with `pnpm`
so `vitest` + `@vitest/coverage-v8` are present. A project on a different toolchain — a non-`pnpm`
package manager, or Python sources that need third-party runtime deps installed — should drive
the CLI directly (below) until #56 makes this config-driven.

### Check the built distribution (packaging)

The other rules read your **source tree**; the **packaging** rule reads your **built
distribution** — it verifies that no test file slipped into the artifact you publish. A built
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
(`npm pack` tarball, TypeScript), `.crate` (Cargo crate, Rust). It fails — naming the offending
path — if any of them ships a test file, and also fails if the artifact held no recognized
distribution at all (a sign the upload was misconfigured). Leave `packaging_artifact` unset and
the packaging job is skipped, never failed.

## Roll your own

Prefer to wire it up by hand? The CLI is a single binary — install it (see
[Getting Started](../getting-started)) and call each rule as its own step, naming the language
with the required `--language` flag:

```yaml
- run: testing-conventions unit colocated-test --language python src/
- run: testing-conventions unit colocated-test --language typescript src/
- run: testing-conventions unit coverage --language python --config testing-conventions.toml src/
- run: testing-conventions unit coverage --language typescript --config testing-conventions.toml src/
- run: testing-conventions integration lint --language python src/   # python only for now
- run: testing-conventions packaging dist/my_pkg-0.1.0-py3-none-any.whl --language python  # built dist, not src/
```

Either way, the non-zero exit on a violation is what fails the build.
