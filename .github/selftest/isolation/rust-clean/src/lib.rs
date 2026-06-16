//! A well-isolated Rust unit: the inline `#[cfg(test)]` module reaches only its own
//! module (`super::add`), so nothing leaves it and the `unit lint` rule passes.

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds() {
        assert_eq!(add(2, 2), 4);
    }
}
