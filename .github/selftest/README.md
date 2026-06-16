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
    TypeScript + Rust today; Python isolation is merged but unreleased, so it's
    deferred to [#146](https://github.com/thekevinscott/testing-conventions/issues/146)
    until a release ships it.

`clean/`, `below-floor/`, and `integration-waiver/` each carry their own
`testing-conventions.toml` with the `[python].coverage` floor for that run. The
self-test drives the *published* `testing-conventions` binary (what consumers get via
`npx`), so these fixtures track the released surface rather than this branch's source.
