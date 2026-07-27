//! A unit compiled only under the `boost` feature, pinned by the crate's integration
//! test (`tests/boost.rs`) rather than a colocated one — so the feature has to reach the
//! integration test target for the crate to build at all.

/// Difference of two integers.
pub fn sub(a: i32, b: i32) -> i32 {
    a - b
}
