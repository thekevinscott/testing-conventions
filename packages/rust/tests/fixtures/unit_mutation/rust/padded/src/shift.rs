//! A unit exercised only by the crate's integration test
//! (`tests/covers_shift.rs`) — no colocated unit test judges its mutants.

/// Triple a value.
pub fn triple(value: u8) -> u8 {
    value * 3
}
