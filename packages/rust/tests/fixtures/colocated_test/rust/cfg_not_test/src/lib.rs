//! Library root: only module declarations — not a subject. `gated` has behavior
//! but its only cfg-gated module is `#[cfg(not(test))]` (production code, not a
//! test), so `gated` is the orphan the presence check must flag. (#390)

pub mod gated;
