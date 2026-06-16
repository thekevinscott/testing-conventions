//! Violating: the inline `#[cfg(test)]` test reaches out of its module by performing
//! real filesystem I/O (`std::fs`). Effectful `std` must sit behind an injected
//! trait, so the `unit isolation` rule flags it and the command exits non-zero.

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
