//! A unit with behavior, fully exercised by its colocated inline test — every
//! region and line runs, so `cargo llvm-cov` reports 100% on both metrics.

/// Add one, saturating at the type's max.
pub fn make(n: u8) -> u8 {
    n.saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_one_more() {
        assert_eq!(make(1), 2);
    }

    #[test]
    fn saturates_at_the_max() {
        assert_eq!(make(u8::MAX), u8::MAX);
    }
}
