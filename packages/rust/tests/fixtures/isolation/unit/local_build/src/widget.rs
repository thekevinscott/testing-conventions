//! The crate's own unit source: a well-isolated `#[cfg(test)]` module that stays in
//! `super::`. This is the only file the unit-isolation walk should scan for this crate
//! — `tests/` and `target/` are skipped — so the tree is clean.

pub fn label() -> &'static str {
    "widget"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_in_module() {
        assert_eq!(label(), "widget");
    }
}
