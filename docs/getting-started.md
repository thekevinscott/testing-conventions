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

On every pull request it **auto-detects the languages present** (Python, TypeScript, and Rust),
scans `src`, and runs every rule with strict defaults — each as its own job that fails the build
on a violation. A language with no sources is skipped, never failed, so the default is safe on any
library.

That's the whole setup. To restrict languages, scan a different path, or check a built artifact,
see [Enforce conventions in CI](./guide/ci).

## Install the CLI

The workflow runs a single binary, published to three registries under the same name. Install it
to run any rule locally:

```sh
# Rust (crates.io)
cargo install testing-conventions

# Python (PyPI): the wheel bundles the binary
pip install testing-conventions

# Node (npm): a dev dependency, run via npx
npm install --save-dev testing-conventions
```

Confirm it's available (prefix `npx` if you installed it via npm):

```sh
testing-conventions --version
```

## Going further

Everything beyond the drop-in is optional and lives in a focused guide:

- [Isolate tests](./guide/isolation) — the unit/integration boundary and the mocking rules per language.
- [Extend the defaults](./guide/extending) — relax a floor, exempt a file, or reuse our shared test config.
- [Exempt a file](./guide/exemptions) — the explicit, reason-required escape hatch.
- [Enforce conventions in CI](./guide/ci) — the reusable workflow's inputs, diff-scoped checks, and rolling your own steps.
- [Mutation testing](./guide/mutation) — verify the lines a change touches, not just execute them.
- [Reference](./reference/) and [Defaults](./reference/defaults) — every subcommand, flag, exit code, config key, and default value.
