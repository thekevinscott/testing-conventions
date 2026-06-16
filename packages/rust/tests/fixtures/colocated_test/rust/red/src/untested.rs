//! Has behavior (a function) but no inline `#[cfg(test)]` module — the orphan the
//! presence check must flag.

pub fn compute(x: u8) -> u8 {
    x.wrapping_mul(2)
}
