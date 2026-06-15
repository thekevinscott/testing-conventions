# Getting Started

`testing-conventions` is a single CLI that enforces a library's testing standards as
deterministic, bright-line checks — the kind an agent (or a hurried human) can't quietly
cross while keeping CI green. One config file is meant to drive every rule, and each rule
is verified the same way in CI. This page takes you from install to a first check.

## Install

The tool is one Rust binary, published to three registries under the same name. Install it
whichever way matches your project's toolchain:

```sh
# Rust (crates.io)
cargo install testing-conventions

# Python (PyPI) — the wheel bundles the binary
pip install testing-conventions

# Node (npm) — as a dev dependency, run via npx
npm install --save-dev testing-conventions
```

Confirm it's available (prefix `npx` if you installed it as an npm dev dependency):

```sh
testing-conventions --version
```

## Your first check: unit-test location

The rule shipping today is **unit-test location & naming**: every source file must have a
colocated unit test named after it. Point the `unit-location` subcommand at the directory
you want to scan:

```sh
# Python is the default: foo.py must have a sibling foo_test.py
testing-conventions unit-location src/

# TypeScript: foo-bar.ts must have a sibling foo-bar.test.ts
testing-conventions unit-location --lang typescript src/
```

When every source file is paired, the command prints nothing and exits `0`. When a file is
missing its twin, each orphan is listed on stderr and the command exits `1`:

```
missing colocated unit test: src/widget.ts
missing colocated unit test: src/pkg/orphan.ts
error: 2 source file(s) missing a colocated unit test
```

That non-zero exit is the whole point: drop the same command into CI and an orphaned (or
missing) test can't slip through green.

## Next steps

- [Guides](./guide/) — task-oriented recipes (enforce a rule, wire it into CI).
- [Reference](./reference/) — every subcommand, flag, exit code, and config key.
