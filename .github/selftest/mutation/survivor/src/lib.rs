//! A unit whose colocated inline test executes `is_positive` (so it has 100% line
//! coverage) but makes no assertion, so every mutant survives un-exempted — the gap
//! mutation testing exposes that coverage cannot, and the `unit mutation` gate trips.

/// Whether `n` is strictly positive.
pub fn is_positive(n: i32) -> bool {
    n > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_without_asserting() {
        // Executes the line (coverage is satisfied) but pins no behavior.
        let _ = is_positive(1);
    }
}
