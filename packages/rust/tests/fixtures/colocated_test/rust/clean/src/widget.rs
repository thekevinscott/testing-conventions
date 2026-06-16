//! A unit with behavior and a colocated inline `#[cfg(test)]` test module.

/// Add one, saturating — trivial, but it is behavior, so it is a unit-test subject.
pub fn make(n: u8) -> u8 {
    n.saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_one_more() {
        assert_eq!(make(1), 2);
    }
}
