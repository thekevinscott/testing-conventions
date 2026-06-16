//! Red: the unit test reaches *past* the unit under test with `super::super::…`.
//! Only a single `super::` (the unit itself) is in-module.

pub fn label() -> &'static str {
    "leaf"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaches_past_the_unit() {
        // VIOLATION: ancestor-module reach.
        let _ = super::super::util::help();
        assert_eq!(label(), "leaf");
    }
}
