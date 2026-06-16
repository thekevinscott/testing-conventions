//! An integration test (its own crate, under `tests/`) — not a unit source, so the
//! presence check must skip it rather than flag it for lacking an inline
//! `#[cfg(test)]` module.

#[test]
fn integration_smoke() {
    assert_eq!(2 + 2, 4);
}
