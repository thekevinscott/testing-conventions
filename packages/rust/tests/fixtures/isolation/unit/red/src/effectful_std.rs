//! Red: the unit test performs real filesystem I/O (`std::fs`). Effectful `std`
//! must sit behind an injected trait, not be called directly in a unit test.

pub fn label() -> &'static str {
    "reader"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_filesystem() {
        // VIOLATION: effectful std (filesystem).
        let _ = std::fs::read("data.bin");
        assert_eq!(label(), "reader");
    }
}
