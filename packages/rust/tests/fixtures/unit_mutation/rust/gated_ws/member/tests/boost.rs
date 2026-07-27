//! An integration test over the feature-gated `boost` module: it names an item that is
//! compiled out without the `boost` feature, so this target fails to compile unless the
//! mutation run's feature selection reaches cargo's build phase. Its assertions kill
//! every mutant of `sub`.

use mut_gated_member::boost::sub;

#[test]
fn subtracts() {
    assert_eq!(sub(5, 3), 2);
    assert_eq!(sub(10, 1), 9);
}
