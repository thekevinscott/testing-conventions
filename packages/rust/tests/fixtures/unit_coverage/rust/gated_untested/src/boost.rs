//! A unit compiled only under the `boost` feature, with no tests at all. Under
//! `[rust] features = ["boost"]` its uncovered regions and lines are measured
//! and drag the crate below a 100 floor (#266).

/// Triple a value.
pub fn triple(value: u8) -> u8 {
    value * 3
}
