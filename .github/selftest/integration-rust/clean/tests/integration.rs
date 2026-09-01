//! Clean: an integration test that runs first-party code for real and doubles nothing, so
//! `integration lint` passes. The "external double is allowed" case lives in the rule's own
//! syn-only tests, `packages/rust/tests/rust_integration_lint.rs`.

use gadget::compute;

#[test]
fn runs_first_party_for_real() {
    assert_eq!(compute(), 7);
}
