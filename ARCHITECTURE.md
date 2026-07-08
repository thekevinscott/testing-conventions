# Architecture

A single Rust binary is the source of truth. Python and Node wrappers
exist only to put that binary on `PATH` under their respective package
manager.

## Packages

```
packages/
  rust/      crate — the CLI + library. clap for parsing.
  python/    maturin-built wheel that bundles the rust binary.
  node/      thin wrapper, resolves a per-platform optional dep
             whose payload is the rust binary.
internals/   contributor + agent conventions (not published).
docs/        VitePress site (published to GitHub Pages).
```

## Release flow

`putitoutthere.toml` declares the three artifacts and their dependency
cascade. The `Release` workflow (`.github/workflows/release.yml`) calls
the reusable workflow at `thekevinscott/putitoutthere`. Edits under
`packages/rust/**` retrigger PyPI and npm builds via the cascade.

## CI gates

- Per-language workflow (`rust.yml`, `python.yml`, `node.yml`) runs lint + test + build with path filters.
- `changelog.yml` enforces `CHANGELOG.md` + `MIGRATIONS.md` updates on PRs that touch package code.
- `docs.yml` builds + deploys the VitePress site (which also emits the generated `llms.txt` / `llms-full.txt` agent digest, per [llmstxt.org](https://llmstxt.org)).
- `testing-conventions-selftest.yml` smoke-tests the reusable `testing-conventions.yml` against fixtures in `.github/selftest/` (a clean suite passes; a below-floor suite trips the coverage gate).
- `pr-monitor.yml` gates merge on the aggregate CI status, with `timeout: '20'` (minutes) so the gate outlasts the full per-language + selftest fan-out instead of the action's 10-minute default.

## Public-API surface

Defined in `internals/repo.md`: every exported value/type, every CLI
flag, every config key, every observable artifact. Changes to that
surface require `CHANGELOG.md` + `MIGRATIONS.md` updates.
