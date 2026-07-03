//! Integration test covering `shift`, which the unit suite leaves unexecuted.
//! The unit coverage floor measures the unit suite, so the coverage this test
//! produces must stay out of the floor's numbers (#265).

#[test]
fn triples_the_value() {
    assert_eq!(cov_rust_padded::shift::triple(2), 6);
}
