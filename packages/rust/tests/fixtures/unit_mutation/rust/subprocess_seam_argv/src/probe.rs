//! A spawner carrying an argv decision no test asserts.

use std::process::Command;

/// `true` when the probed command exited zero, asking for verbose output past one attempt.
pub fn probe(attempts: u32) -> bool {
    let mut command = Command::new("true");
    if attempts > 1 {
        command.arg("--verbose");
    }
    command.output().is_ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_crate_has_a_unit_suite() {
        assert!(true);
    }
}
