//! A unit compiled only under the `boost` feature, whose colocated inline test
//! pins its behavior. The mutation run enables the feature via
//! `[rust] features = ["boost"]`, so this module's mutants are exercised and
//! every one is caught (#266).

/// Difference of two integers.
pub fn sub(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtracts() {
        assert_eq!(sub(5, 3), 2);
        assert_eq!(sub(10, 1), 9);
    }
}
