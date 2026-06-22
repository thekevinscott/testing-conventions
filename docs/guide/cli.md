---
description: Install and run the rules yourself — outside GitHub Actions, in another CI system, or as a one-off locally.
---

# Use the CLI directly

Most projects never install the CLI — the [drop-in workflow](./ci) runs it for you. Reach for it
directly only when you want to run a rule yourself: outside GitHub Actions, in another CI system, or
as a one-off locally.

## Install it

The CLI is a single binary, published to three registries under the same name. Install whichever
matches your toolchain:

```sh
# Rust (crates.io)
cargo install testing-conventions

# Python (PyPI) — the wheel bundles the binary
pip install testing-conventions

# Node (npm) — a dev dependency, run via npx
npm install --save-dev testing-conventions
```

Confirm it's available (prefix `npx` if you installed it via npm):

```sh
testing-conventions --version
```

## Run a rule

Call any rule directly, naming the language with the required `--language` flag. For example, the
colocated-test rule checks that every source file has a matching unit test:

```sh
testing-conventions unit colocated-test --language python src/
testing-conventions unit colocated-test --language typescript src/
```

A clean run prints nothing and exits `0`; a violation is listed on stderr and exits non-zero, which
fails CI.

## See also

- [Reference](../reference/): every subcommand, flag, and exit code.
- [Enforce conventions in CI — roll your own](./ci#roll-your-own): wire the CLI into CI by hand.
