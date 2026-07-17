# Rust — testing

- Inline unit tests at the bottom of the same file:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use std::io::Cursor;

      #[test]
      fn it_works() {
          assert_eq!(count_lines(Cursor::new("a\nb\n")).unwrap(), 2);
      }
  }
  ```
- `Cursor::new(...)` for in-memory I/O testing — pairs beautifully with generic `<R: BufRead>` signatures.
- **Doc tests** in `///` comments for public APIs — they get run by `cargo test` and keep docs verified-correct.
- Integration tests in top-level `tests/` directory (each file is a separate crate, only sees the public API).

Inline `#[cfg(test)] mod tests` is the Rust default — tests only in `tests/` when an inline module would work is a sign of treating Rust like Python.

**No mechanism-hygiene integration lint (by design).** Python's `integration lint` carries
three mechanism lints — `no-monkeypatch`, `no-inline-patch`, `no-environ-mutation` — that
police *how* a pytest test mocks. Rust has none, deliberately: there is no `monkeypatch`
fixture, no string-based `patch`, and no in-place `os.environ` idiom — collaborators are
injected as trait doubles the compiler checks against the real trait. The Rust `integration
lint` is the first-party *direction* check alone — `no-first-party-double` (don't `#[double]`
a first-party item).

**E2E attestation** — e2e tests aren't run in CI. Run them locally and attest:
`testing-conventions e2e attest 'cargo test --test e2e'` commits a receipt naming the
commit they ran against; in CI, `e2e verify` checks that receipt is current (re-run
`attest` when it goes stale). CI never runs the e2e suite.

## Gate fixture layout

The suite-executing gates (mutation, coverage, and their diff-scoped `--base` variants) run
a real engine over a fixture codebase, so a fixture's **layout** is part of the contract under
test. The default fixture shape is the prescribed consumer package layout — a package root with
a manifest (`package.json` / `pyproject.toml` / `Cargo.toml`), sources under `src/`, and suite
tiers under `tests/` — scanned at `src/`. The gate is pointed at `<package-root>/src`, so the
run roots the engine at the package root (where an upward `../package.json` import or a
package-root config resolves) while discovery and measurement stay scoped to the scan path. This
is the shape a consumer actually runs; a fixture built this way can exhibit the layout-dependent
behavior — sandbox roots, config discovery, upward imports, suite-tier separation — that a flat
tree hides, because in a flat tree the scan path and the package root are the same directory.

The flat, no-manifest shape (loose scripts at the scanned root, e.g. a bare `index.ts` +
`stryker.conf.json`) is the explicitly-named special case: the mutation fixtures carry it as
`loose_killed` / `loose_survivors` and stage it through `Staged::loose` / `Staged::python_loose`,
and the coverage fixtures keep it in the feature-named flat cases (`exempt_cov`,
`full_with_config`, `conftest_omit`). Line-scoped exemption tests pin their `lines` to a fixed
flat file, so they run against the loose fixtures on purpose.

Each suite-executing TS/Python gate carries at least one fixture that **distinguishes the package
root from the scan path**, so a regression that confuses the two goes red rather than vacuously
green: a source under `src/` that imports a package-level file (`../package.json`) or a
package-root config the run depends on, plus a `tests/` tier that fails loudly if the gate ever
collects it (`tests/integration/tiers.*` asserts it is never reached). Rust's crate layout forces
the package shape already; the parity bar is met by giving Python and TypeScript the same default.
