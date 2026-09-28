//! A seam that spawns, a seam that delegates to it, and the pure verdict a caller reads.

use std::process::Command;

/// `true` when the probed command exists and exited zero.
pub fn probe() -> bool {
    Command::new("true").output().is_ok()
}

/// Probe again, delegating rather than repeating the spawn.
pub fn probe_again() -> bool {
    probe()
}

/// What a probe's result is called in a report.
pub fn verdict(ok: bool) -> &'static str {
    if ok {
        "reachable"
    } else {
        "unreachable"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_successful_probe_reads_as_reachable() {
        assert_eq!(verdict(true), "reachable");
    }

    #[test]
    fn a_failed_probe_reads_as_unreachable() {
        assert_eq!(verdict(false), "unreachable");
    }
}
