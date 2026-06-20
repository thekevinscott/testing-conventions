//! A unit whose colocated inline test pins its behavior, so every mutant of `add`
//! (`a - b`, `a * b`, replace with 0/1, …) changes the asserted output and is caught.

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
