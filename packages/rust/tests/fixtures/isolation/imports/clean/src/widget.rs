//! Clean: a unit test that imports only what's in-module — `super::*` (the unit
//! under test) and pure `std` (`collections`, `io::Cursor`). Nothing foreign is
//! brought into scope, so nothing is flagged.

pub struct Counter;

impl Counter {
    pub fn tally() -> usize {
        2
    }
}

pub fn label() -> &'static str {
    "clean"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Cursor;

    #[test]
    fn t() {
        // All in-module: `Counter`/`label` via `super::*`, pure-std `HashMap` and
        // `Cursor`. No foreign import, no out-of-module call.
        let mut seen: HashMap<&str, usize> = HashMap::new();
        seen.insert(label(), Counter::tally());
        let _ = Cursor::new(b"buffer");
        assert_eq!(seen.get("clean"), Some(&2));
    }
}
