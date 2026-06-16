//! Red: the unit test calls a *first-party* item in another module (`crate::store`).
//! A unit test should exercise only `super::`; reach a collaborator behind a trait.

pub fn label() -> &'static str {
    "widget"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_another_module() {
        // VIOLATION: first-party cross-module call.
        let _ = crate::store::load();
        assert_eq!(label(), "widget");
    }
}
