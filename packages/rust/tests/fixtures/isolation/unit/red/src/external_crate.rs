//! Red: the unit test calls an *external crate* (`rand`) directly. External deps
//! must be doubled behind a trait, never reached from a unit test.

pub fn label() -> &'static str {
    "dice"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_an_external_crate() {
        // VIOLATION: external-crate call.
        let _ = rand::random::<u8>();
        assert_eq!(label(), "dice");
    }
}
