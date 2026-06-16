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

**E2E attestation** — e2e tests aren't run in CI. Run them locally and attest:
`testing-conventions e2e attest 'cargo test --test e2e'` commits a receipt naming the
commit they ran against; in CI, `e2e verify` checks that receipt is current (re-run
`attest` when it goes stale). CI never runs the e2e suite.
