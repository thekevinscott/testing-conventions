//! A unit exercised only by the crate's integration test
//! (`tests/covers_shift.rs`) — the unit suite runs none of it. The unit
//! coverage floor measures the unit suite, so these regions and lines must
//! read uncovered.

/// Triple a value.
pub fn triple(value: u8) -> u8 {
    value * 3
}
