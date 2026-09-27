//! Red: the unit test spawns a child process. `std::process::id` is carved out of the
//! effectful-`std` rule; the rest of `std::process` spawns, controls, or terminates, and is not.

pub fn program() -> &'static str {
    "true"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_the_program() {
        // VIOLATION: effectful std (subprocess).
        let _ = std::process::Command::new(program()).status();
        assert_eq!(program(), "true");
    }
}
