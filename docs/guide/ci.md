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
      path: src
```

It installs the published `testing-conventions` binary and runs `testing-conventions check`:
the **config-driven umbrella**. `check` reads your config file and runs every rule its present
`[python]` / `[typescript]` / `[rust]` tables enable, failing the build — with the offending
files in the log — on any violation.

Because the rule set is driven by your config, not the workflow, a newly-shipped rule starts
enforcing as soon as you adopt a version that has it: there is no per-rule job to add.

### Inputs

| Input     | Default                    | Description                                                          |
| --------- | -------------------------- | -------------------------------------------------------------------- |
| `path`    | `src`                      | Directory scanned recursively for sources and tests.                 |
| `version` | latest                     | `testing-conventions` version to install (e.g. `0.1.0`).             |
| `config`  | `testing-conventions.toml` | Config file whose language tables decide which rules run.            |

### What runs, from your config

| Config table             | Rules `check` runs                                                            |
| ------------------------ | ----------------------------------------------------------------------------- |
| `[python]`               | `unit colocated-test`, `integration lint`, and — if `[python].coverage` is set — `unit coverage`. |
| `[typescript]`           | `unit colocated-test`.                                                         |
| `[rust]`                 | Nothing yet (Rust rules are still in progress).                               |

A configured threshold no rule covers yet (e.g. `[typescript].coverage`) is surfaced as a
`note:` in the log and skipped — never silently dropped — and a config that enables no checks
at all fails as a misconfiguration. The workflow sets up `coverage` + `pytest` for the Python
coverage rule, so it fits suites with no third-party runtime imports; a project that needs its
own dependencies should drive the CLI directly (below) until config-driven setup lands.

::: tip Migrating from the per-language form
Earlier versions took a `languages` input and ran each rule as its own matrix job. That input
is gone: drop it and let the config's `[python]` / `[typescript]` / `[rust]` tables decide what
runs. (`path`, `version`, and `config` are unchanged.)
:::

## Roll your own

Prefer to wire it up by hand? The CLI is a single binary — install it (see
[Getting Started](../getting-started)) and either run the umbrella in one step:

```yaml
- run: testing-conventions check --config testing-conventions.toml src/
```

or call each rule as its own step, naming the language with the required `--language` flag:

```yaml
- run: testing-conventions unit colocated-test --language python src/
- run: testing-conventions unit colocated-test --language typescript src/
- run: testing-conventions unit coverage --language python --config testing-conventions.toml src/
```

Either way, the non-zero exit on a violation is what fails the build.
