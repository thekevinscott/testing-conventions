//! A unit whose colocated inline test pins `add`'s behavior, plus a declaration-only
//! sibling module (`settings`, no `fn`) whose const-init mutants are not mutation subjects.

pub mod settings;

/// Sum two integers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(10, 1), 11);
    }
}
