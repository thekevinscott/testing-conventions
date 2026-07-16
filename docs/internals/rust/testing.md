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
