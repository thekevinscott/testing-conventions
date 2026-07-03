//! A unit whose colocated inline test exercises only the high-score arm, so one
//! of the branch's two outcomes is never taken — branch coverage reads 50%, the
//! number a `branch` floor gates (#267).

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
        // Only the `>= 60` arm is exercised; the other outcome stays untaken.
        assert_eq!(label(90), "pass");
    }
}
