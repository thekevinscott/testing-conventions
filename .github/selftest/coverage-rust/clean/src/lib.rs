//! A unit fully exercised by its colocated inline test — every region and line
//! runs, so `cargo llvm-cov` reports 100% on both metrics. The test reaches only
//! its own module (`super::add`), so the crate is also clean for the `unit lint`
//! and `integration lint` jobs that fan out alongside coverage under `["rust"]`.

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
