//! An always-compiled unit, fully exercised by its colocated inline test.

/// Double a value.
pub fn double(value: u8) -> u8 {
    value * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles_the_value() {
        assert_eq!(double(2), 4);
    }
}
