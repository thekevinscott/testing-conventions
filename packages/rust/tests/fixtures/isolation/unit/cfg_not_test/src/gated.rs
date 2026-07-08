//! Production code inside a `#[cfg(not(test))]` module makes a first-party
//! cross-module call. A `not(test)` gate compiles in *non-test* builds — this is
//! shipping code, not a unit test — so the out-of-module call is correct and must
//! NOT be flagged by the isolation lint. (#390)

pub fn label() -> &'static str {
    "widget"
}

#[cfg(not(test))]
mod platform {
    /// Production wiring, gated out of test builds. Reaching a sibling module here
    /// is legitimate — this is not a unit test.
    pub fn boot() {
        let _ = crate::other::load();
    }
}
