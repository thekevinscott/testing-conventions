//! The binary target's entry point. It lives here rather than in `src/main.rs` so the binary
//! root stays declaration-only: Rust requires a `main` in that file, and a `fn main` with a body
//! is a unit-gate subject whose own `#[cfg(test)]` module the unit suite would have to run. A
//! re-export carries no function, so the gates read `main.rs` as declarations and the logic sits
//! here, under test.

use std::process::ExitCode;

/// Run the CLI over the process arguments and map the outcome onto an exit code.
///
/// `cfg(not(test))` keeps this out of the unit tier's coverage report. A test cannot execute it:
/// `cargo test` replaces a binary's `main` with the harness's own, and calling it directly would
/// run the real CLI over the harness's argv — `cargo test install` would write to the repo. It
/// carries no decision to cover either; [`report`] holds all of them and the e2e tier runs the
/// real binary. Only the process-boundary argv read lives here.
#[cfg(not(test))]
pub fn main() -> ExitCode {
    report(crate::run(std::env::args_os()))
}

/// The exit code a finished [`crate::run`] earns, after printing whatever the caller should see.
///
/// A clap failure renders itself — `--help` and `--version` are clap "errors" that belong on
/// stdout with code 0, a usage mistake on stderr with code 2 — so it prints itself and reports
/// its own code. `print` plus `exit_code` is what `clap::Error::exit` does either side of
/// `process::exit`; splitting them keeps this function callable from a test.
fn report(result: anyhow::Result<i32>) -> ExitCode {
    match result {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => match err.downcast_ref::<clap::Error>() {
            Some(clap_err) => {
                let _ = clap_err.print();
                ExitCode::from(clap_err.exit_code() as u8)
            }
            None => {
                // `{err:#}` prints the whole anyhow chain, so a wrapped failure keeps its context.
                eprintln!("error: {err:#}");
                ExitCode::from(1)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn a_success_code_becomes_that_exit_code() {
        assert_eq!(report(Ok(0)), ExitCode::from(0));
        assert_eq!(report(Ok(3)), ExitCode::from(3));
    }

    #[test]
    fn a_plain_failure_exits_one() {
        assert_eq!(
            report(Err(anyhow::anyhow!("the gate found violations"))),
            ExitCode::from(1)
        );
    }

    #[test]
    fn a_usage_mistake_carries_claps_own_exit_code() {
        let clap_err = clap::Error::new(ErrorKind::InvalidValue);
        let expected = clap_err.exit_code() as u8;

        assert_eq!(report(Err(clap_err.into())), ExitCode::from(expected));
        assert_eq!(expected, 2, "clap reports a usage mistake as 2");
    }

    #[test]
    fn a_help_request_exits_zero_rather_than_as_a_failure() {
        let help = clap::Error::new(ErrorKind::DisplayHelp);

        assert_eq!(report(Err(help.into())), ExitCode::from(0));
    }
}
