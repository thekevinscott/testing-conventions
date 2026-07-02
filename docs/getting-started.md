---
description: Get every rule running on your repository in about five minutes with the no-config drop-in workflow.
---

# Getting Started

`testing-conventions` enforces a library's testing standards deterministically in CI. This tutorial
gets every rule running on your repository in about five minutes — no local install, no config.

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
scans `src`, and runs every rule with strict defaults — each as its own job that fails the build on
a violation. A language with no sources is skipped, never failed, so the default is safe on any
library.

That's the whole setup: this one file opts your library into the full standard.

## Teach your agent

Run `npx testing-conventions install` to write the contract into your repo's `AGENTS.md`, so a
coding agent knows the rules before it writes code. Re-running refreshes the block; everything
outside its markers stays yours.

## Next

From here, the [How-to Guides](./guide/) cover the common tasks — configuring a floor, exempting a
file, extending the defaults to your local test runner, and scoping the workflow.
