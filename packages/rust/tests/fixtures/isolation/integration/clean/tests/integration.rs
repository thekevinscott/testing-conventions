//! Clean: an integration test that runs first-party code (`gadget::compute`) for
//! real and doubles only an *external* crate (`rand`) — exactly what integration
//! tests may mock.

use gadget::compute;
use mockall_double::double;

// Allowed: doubling a third-party crate. Only first-party doubles are flagged.
#[double]
use rand::rngs::ThreadRng;

#[test]
fn runs_first_party_for_real() {
    assert_eq!(compute(), 7);
    let _double_in_scope: Option<ThreadRng> = None;
}
