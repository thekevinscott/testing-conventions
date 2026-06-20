# Getting Started

`testing-conventions` enforces a library's testing standards deterministically in CI. It's
primarily useful for enforcing agent (LLM) behavior.

## The drop-in

Add one file to your repo — no inputs, no config:

```yaml
# .github/workflows/conventions.yml
name: Conventions
on: [pull_request]

jobs:
  conventions:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
```

On every pull request it **auto-detects the languages present** (Python, TypeScript, and
Rust), scans `src`, and runs every rule with sensible defaults — each as its own job that
fails the build on a violation. That's the whole setup: this one file opts a new library
into the full check set.

The one thing Rust needs before its coverage floor is enforced is a `[rust].coverage`
table — it has no default floor. Every other rule runs with no config. See
[Defaults](./reference/defaults) for every default the workflow applies and why.

## Going further

Everything below is optional — the drop-in above already works.

### Restrict or redirect the scan

`languages` is an optional restrictor and `path` defaults to `src`:

```yaml
    with:
      languages: '["python", "typescript"]'   # restrict to these (default: auto-detect every present language)
      path: packages/core/src                  # scan a different directory
```

A language with no sources under `path` is skipped, never failed, so the auto-detect
default is safe on any library.

### Customize with a config file

Adjust a floor or declare an exemption in a `testing-conventions.toml` at your repo root:

```toml
# Relax the Python floor below the strict default 100:
[python]
coverage = { branch = true, fail_under = 90 }

# Exempt a launcher shim; explicit, and a reason is required:
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test", "coverage"]
reason = "thin launcher; logic in run(), tested in run_test.py"
```

Anything you omit keeps its default. See [Configuration](./reference/#configuration) for
every key, [Defaults](./reference/defaults) for the baseline, and [Exempt a
file](./guide/exemptions) for the exemption rules.

## Install the CLI

The workflow runs a single binary, published to three registries under the same name.
Install it the way that matches your toolchain:

```sh
# Rust (crates.io)
cargo install testing-conventions

# Python (PyPI): the wheel bundles the binary
pip install testing-conventions

# Node (npm): a dev dependency, run via npx
npm install --save-dev testing-conventions
```

Confirm it's available (prefix `npx` if you installed it as an npm dev dependency):

```sh
testing-conventions --version
```

Then call any rule directly, naming the language with the required `--language` flag. For
example, the **colocated test** rule checks that every source file has a colocated unit
test named after it:

```sh
# Python: foo.py must have a sibling foo_test.py
testing-conventions unit colocated-test --language python src/

# TypeScript: foo-bar.ts must have a sibling foo-bar.test.ts
testing-conventions unit colocated-test --language typescript src/
```

When every source file is paired, the command prints nothing and exits `0`. When a file is
missing its twin, each orphan is listed on stderr and the command exits `1`:

```
missing colocated unit test: src/widget.ts
missing colocated unit test: src/pkg/orphan.ts
error: 2 source file(s) missing a colocated unit test
```

The non-zero exit fails CI, so an orphaned or missing test is caught. `unit coverage` runs
the same way, and its `--config` is optional too: omit it and the default floor applies.

## Next steps

- [Guides](./guide/): task-oriented recipes (enforce a rule, wire it into CI, exempt a file).
- [Reference](./reference/): every subcommand, flag, exit code, and config key.
- [Defaults](./reference/defaults): every default value, and why.
