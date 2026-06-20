//! Clean: an integration test that runs first-party code (`gadget::compute`) for
//! real and doubles nothing — so it passes `integration lint` (no first-party
//! double to flag), and, compiled under `cargo llvm-cov`, exercises every line of
//! `gadget` so the crate also clears the zero-config Rust coverage floor (#206).
//!
//! (A real mockall_double `#[double]` always resolves to a first-party mock under
//! `#[cfg(test)]`, which the lint flags — and an external concrete type has no mock
//! to resolve to, so it can't compile under coverage. A genuinely clean integration
//! test therefore doubles nothing; the "external double is allowed" lint case lives
//! in the rule's own syn-only tests, `packages/rust/tests/rust_integration_lint.rs`.)

use gadget::compute;

#[test]
fn runs_first_party_for_real() {
    assert_eq!(compute(), 7);
}
