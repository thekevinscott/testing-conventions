//! A gate a test build can still satisfy, beside one it cannot.

use std::process::ExitCode;

#[cfg(not(test))]
pub fn main() -> ExitCode {
    report(std::env::args_os().count() as i32)
}

/// Compiled under `cargo test` as well, and reached by no test.
#[cfg(any(not(test), unix))]
pub fn untested(argc: i32) -> ExitCode {
    ExitCode::from(argc as u8)
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
