//! Red: a unit test glob-imports a first-party module. Only `use super::*;` is a
//! legal glob; pulling another module's surface into the test reaches out of it.

pub fn label() -> &'static str {
    "first-party glob"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::other::*; // VIOLATION: foreign glob import (first-party)

    #[test]
    fn t() {
        assert_eq!(label(), "first-party glob");
    }
}
