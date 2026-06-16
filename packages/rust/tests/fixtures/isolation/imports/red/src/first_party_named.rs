//! Red: a unit test imports a specific first-party item from another module. A
//! collaborator brought into scope this way would be called unqualified, hiding
//! the out-of-module reach from the call detector — so the import itself is flagged.

pub fn label() -> &'static str {
    "first-party named"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::other::Thing; // VIOLATION: foreign import (first-party)

    #[test]
    fn t() {
        assert_eq!(label(), "first-party named");
    }
}
