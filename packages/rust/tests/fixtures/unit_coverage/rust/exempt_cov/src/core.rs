//! A fully-covered unit: its colocated inline test exercises every region and
//! line, so on its own this module is 100%.

/// Double a value, saturating at the type's max.
pub fn double(n: u8) -> u8 {
    n.saturating_mul(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles() {
        assert_eq!(double(3), 6);
    }

    #[test]
    fn saturates() {
        assert_eq!(double(u8::MAX), u8::MAX);
    }
}
