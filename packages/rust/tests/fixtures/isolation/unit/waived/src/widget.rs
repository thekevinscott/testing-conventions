//! Waived: the unit test reaches into another module (`crate::store`), which would
//! be `no-out-of-module-call`, but the file is lifted by a `[[rust.exempt]]` entry
//! in testing-conventions.toml.

pub fn label() -> &'static str {
    "widget"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_another_module() {
        let _ = crate::store::load();
        assert_eq!(label(), "widget");
    }
}
