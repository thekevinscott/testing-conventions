//! A gated entry point beside a tested decision. `main` is absent from the `--lib --bins` build,
//! so no test can kill its mutants; `report` holds the decision and its test pins it.

/// Run and map the outcome onto an exit code.
#[cfg(not(test))]
pub fn main() -> u8 {
    report(Ok(0))
}

/// The exit code a finished run earns.
pub fn report(result: Result<u8, u8>) -> u8 {
    match result {
        Ok(code) => code,
        Err(code) => code + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_success_carries_its_own_code() {
        assert_eq!(report(Ok(0)), 0);
        assert_eq!(report(Ok(3)), 3);
    }

    #[test]
    fn a_failure_reports_one_past_its_code() {
        assert_eq!(report(Err(0)), 1);
        assert_eq!(report(Err(4)), 5);
    }
}
