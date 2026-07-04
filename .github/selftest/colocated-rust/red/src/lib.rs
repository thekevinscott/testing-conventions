//! A bodied function with no inline `#[cfg(test)]` test module — the orphan the rust
//! colocated-test presence arm (#40) flags. The self-test drives the published
//! `unit colocated-test --language rust` over this crate and asserts the non-zero
//! exit that fails a consumer's build (#274).

pub fn orphan() -> u8 {
    9
}
