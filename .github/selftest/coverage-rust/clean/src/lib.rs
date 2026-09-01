//! A unit fully exercised by its colocated inline test, which reaches only its own module —
//! so `cargo llvm-cov` reports 100% and the crate is clean for `unit lint` too.

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
