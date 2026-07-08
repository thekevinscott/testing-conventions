//! Behavior plus a `#[cfg(not(test))]` module and NO inline `#[cfg(test)]` test
//! module. A `not(test)` gate compiles in *non-test* builds — it is production
//! code, not a test module — so this file has no inline test and is an orphan the
//! presence check must flag. (#390)

/// Behavior — a function with a body, so this file is a unit-test subject.
pub fn compute(x: u8) -> u8 {
    x.wrapping_mul(2)
}

#[cfg(not(test))]
mod platform {
    /// Production-only helper, gated out of test builds — not a test module.
    pub fn detect() -> &'static str {
        "prod"
    }
}
