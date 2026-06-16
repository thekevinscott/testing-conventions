//! Red: a unit test imports an external crate (`rand`). External deps belong behind
//! an injected trait, never imported for real into a unit test.

pub fn label() -> &'static str {
    "external"
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng; // VIOLATION: external-crate import

    #[test]
    fn t() {
        assert_eq!(label(), "external");
    }
}
