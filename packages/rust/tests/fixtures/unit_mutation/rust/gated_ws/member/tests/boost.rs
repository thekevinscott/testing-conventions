//! An integration test over the feature-gated `boost` module, compiled out without the
//! `boost` feature — proving the feature reaches the build phase. Mutation scopes to the
//! unit suite, so this test's kills never count toward the gate.

use mut_gated_member::boost::sub;

#[test]
fn subtracts() {
    assert_eq!(sub(5, 3), 2);
    assert_eq!(sub(10, 1), 9);
}
