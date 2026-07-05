# Reusable-workflow self-test fixtures

Fixtures for [`../workflows/testing-conventions-selftest.yml`](../workflows/testing-conventions-selftest.yml),
which smoke-tests the reusable workflow
([`../workflows/testing-conventions.yml`](../workflows/testing-conventions.yml))
end to end.

- `clean/` — a fully-covered, colocated, mock-free Python suite. Every rule the
  reusable workflow runs over it (colocated-test, coverage, integration-lint)
  passes, so the `uses:` call must succeed. Also reused, under the zero-config
  default `["python", "typescript"]`, to prove that the source-free TypeScript
  jobs are skipped while the Python jobs still run (#94).
- `below-floor/` — a Python suite whose coverage lands under the floor, so
  `unit coverage` exits non-zero — the build-failing behavior the workflow
  promises.
- `no-sources/` — a directory with no language sources at all. Called with the
  zero-config default `["python", "typescript"]`, the workflow must skip every
  language's jobs and still pass (#94) — the dogfooding case of our own
  `packages/python`, which ships a wheel but carries no `.py`.
- `integration-waiver/` — a clean Python suite whose one integration test trips
  `no-constant-patch`, lifted by a `[[python.exempt]]` entry (#102) in a
  *non-default* config path. Run through the reusable workflow, the call passes only
  because the integration-lint job forwards `--config` (#126); the colocated-test and
  coverage jobs pass alongside it.
- `integration-rust/` — `clean/` and `red/` Rust crates for the rust integration-lint
  arm (#126). `clean/` doubles only an external crate (passes); `red/` doubles the
  crate under test with `#[double]` (`no-first-party-double`, fails). The `clean/`
  crate is driven through the reusable workflow to prove the rust arm runs and that
  rust stays out of the coverage matrix; `red/` is driven directly for the fail path.
- `integration-typescript/` — `clean/` and `red/` TypeScript integration suites for
  the typescript arm (#126). `clean/` mocks only third-party / Node built-ins
  (passes); `red/` mocks a first-party module (`no-first-party-mock`, fails). Both
  are driven directly (a TypeScript `uses:` call would also pull in the vitest
  coverage job).
- `isolation/` — fixtures for the `unit lint` job (#125):
  - `rust-clean/` — a minimal crate whose inline `#[cfg(test)]` unit reaches only
    `super::`. Driven through the reusable workflow under `["rust"]`, it proves
    `detect` recognizes a crate (a `Cargo.toml` / `*.rs`) and fans the isolation job
    over `rust`, then passes the well-isolated unit.
  - `rust-red/` — the same shape but its unit test performs real filesystem I/O
    (`std::fs`), an out-of-module effectful-`std` call, so `unit lint` exits
    non-zero. (The fail path drives the published command directly, since a failing
    `uses:` call would fail the whole run.) The workflow's isolation job covers
    Python, TypeScript, and Rust; the Python arm shipped to npm and now rides the
    `clean` / `absent-language-skipped` jobs
    ([#146](https://github.com/thekevinscott/testing-conventions/issues/146)).

`clean/`, `below-floor/`, and `integration-waiver/` each carry their own
`testing-conventions.toml` with the `[python].coverage` floor for that run. The
self-test drives the *published* `testing-conventions` binary (what consumers get via
`npx`), so these fixtures track the released surface rather than this branch's source.

- `monorepo/` — a per-package-lockfile monorepo with **no manifest or lockfile at its own
  root** ([#277](https://github.com/thekevinscott/testing-conventions/issues/277)),
  mirroring the shape a real consumer hit (dirsql PR #410):
  - `packages/ts/` — an npm package (its own `package.json` + `package-lock.json`, no
    `packageManager` field) with a colocated vitest suite.
  - `packages/py/` — a uv-managed package (`pyproject.toml` with a `[project]` table and
    a real third-party dependency, plus `uv.lock`) with a colocated pytest suite.

  Each package also carries its own `testing-conventions.toml` at its package root.
  `detect-package-root-ts` / `detect-package-root-py` drive the *local* detect action
  (not the `@v0`-pinned one the reusable workflow's `detect` job uses) to prove
  `package_root` / `ts_package_manager` / `python_env` / `provision_rust` / `config`
  resolve correctly for each package — including that each package's own config file
  is discovered with no `config` input — so, like `detect-routes-python`, they aren't
  blocked by the `@v0` rolling-release lag. The gate fixes that consume these outputs
  (#278–#281) drive this same fixture through real per-package `uses:` calls.
