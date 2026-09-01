//! Violating: the inline `#[cfg(test)]` test performs real filesystem I/O. Effectful `std`
//! must sit behind an injected trait, so `unit lint` flags it and exits non-zero.

pub fn label() -> &'static str {
    "reader"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_filesystem() {
        // VIOLATION: effectful std (filesystem) called directly in a unit test.
        let _ = std::fs::read("data.bin");
        assert_eq!(label(), "reader");
    }
}
