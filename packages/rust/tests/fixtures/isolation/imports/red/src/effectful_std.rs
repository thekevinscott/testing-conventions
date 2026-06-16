//! Red: a unit test imports effectful `std` (`std::fs`). The filesystem is an
//! external dependency; import it behind an injected trait, not directly.

pub fn label() -> &'static str {
    "effectful std"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs; // VIOLATION: effectful-std import

    #[test]
    fn t() {
        assert_eq!(label(), "effectful std");
    }
}
