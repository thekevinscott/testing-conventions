//! A unit whose colocated inline test exercises only the high-score arm, so the
//! `else` branch — its region and its line — is never executed. `cargo llvm-cov`
//! reports under 100% regions and lines, so this crate fails a 100 floor.

/// Classify a score: `"pass"` at 60 or above, `"fail"` below.
pub fn label(score: u8) -> &'static str {
    if score >= 60 {
        "pass"
    } else {
        "fail"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_score_is_a_pass() {
        // Only the `>= 60` arm is exercised; the `else` arm stays uncovered.
        assert_eq!(label(90), "pass");
    }
}
