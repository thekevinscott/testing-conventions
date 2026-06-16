//! A thin launcher with no colocated test — its region and line are never
//! executed, so it drags the crate below 100 until a `coverage` exemption omits
//! it from the denominator.

/// Launcher shim: forwards to the real entry point. Never exercised by a unit.
pub fn launch(n: u8) -> u8 {
    crate::core::double(n)
}
