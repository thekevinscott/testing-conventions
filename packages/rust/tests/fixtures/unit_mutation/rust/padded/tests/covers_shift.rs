//! Integration test covering `shift`, which the unit suite leaves unjudged.
//! Mutation stays scoped to the unit suite, so this test must not decide
//! whether `triple`'s mutants are caught.

#[test]
fn triples_the_value() {
    assert_eq!(mut_padded::shift::triple(2), 6);
}
