//! The library under test (first-party code an integration test must run for
//! real). The inline unit test covers `compute` — the coverage arm measures the
//! unit suite only (#265), so the crate clears the zero-config line floor on its
//! inline tests while the integration test exercises the same code for the lint.

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
