//! Two units: `double` is exercised by the colocated inline test, `triple` is
//! defined and never called — an uncovered *function* whose lines barely dent a
//! low line floor, so only a `functions` floor catches it (#267).

/// Double a value.
pub fn double(value: u8) -> u8 {
    value * 2
}

/// Triple a value — never exercised.
pub fn triple(value: u8) -> u8 {
    value * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles_the_value() {
        assert_eq!(double(2), 4);
    }
}
