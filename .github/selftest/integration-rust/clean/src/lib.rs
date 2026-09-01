//! The library under test. Its inline unit test covers `compute`, so the crate clears the
//! zero-config line floor on the unit suite alone, which is all the coverage arm measures.

pub fn compute() -> u8 {
    7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes() {
        assert_eq!(compute(), 7);
    }
}
