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
any violation.

### Inputs

| Input       | Default                     | Description                                                |
| ----------- | --------------------------- | ---------------------------------------------------------- |
| `languages` | `["python", "typescript"]`  | JSON array of languages to check (`python`, `typescript`). |
| `path`      | `src`                       | Directory scanned recursively for sources.                 |
| `version`   | latest                      | `testing-conventions` version to install (e.g. `0.1.0`).   |
| `config`    | `testing-conventions.toml`  | Config file with the coverage thresholds (`[python].coverage`). |

The Python **coverage** job runs when `python` is among `languages`: it runs your unit suite
under `coverage.py` (branch on, `*_test.py` excluded) and fails if the total is below the
`[python].coverage` floor in your `config`. It installs `coverage` + `pytest`, so it fits
suites with no third-party runtime imports; a project that needs its own dependencies should
drive the CLI directly (below) until #56 makes this config-driven.

## Roll your own

Prefer to wire it up by hand? The CLI is a single binary — install it (see
[Getting Started](../getting-started)) and call each rule as its own step, naming the language
with the required `--language` flag:

```yaml
- run: testing-conventions unit location --language python src/
- run: testing-conventions unit location --language typescript src/
- run: testing-conventions unit coverage --language python --config testing-conventions.toml src/
```

Either way, the non-zero exit on a violation is what fails the build.
