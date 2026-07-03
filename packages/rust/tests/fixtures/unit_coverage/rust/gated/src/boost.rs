//! A unit compiled only under the `boost` feature, fully exercised by its
//! colocated inline test. The coverage run enables the feature via
//! `[rust] features = ["boost"]`, so this module is measured (#266).

/// Triple a value.
pub fn triple(value: u8) -> u8 {
    value * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triples_the_value() {
        assert_eq!(triple(2), 6);
    }
}
