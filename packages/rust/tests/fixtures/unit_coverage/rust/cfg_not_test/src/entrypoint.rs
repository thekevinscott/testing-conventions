//! A gated entry point beside the tested function it delegates to.

use std::process::ExitCode;

#[cfg(not(test))]
pub fn main() -> ExitCode {
    report(std::env::args_os().count() as i32)
}

/// The exit code a run with `argc` arguments earns.
pub fn report(argc: i32) -> ExitCode {
    ExitCode::from(argc as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_argument_count_becomes_the_exit_code() {
        assert_eq!(report(3), ExitCode::from(3));
    }
}
