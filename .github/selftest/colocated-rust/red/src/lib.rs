//! A bodied function with no inline `#[cfg(test)]` test module — the orphan the rust
//! colocated-test presence arm flags, asserted through its non-zero exit.

pub fn orphan() -> u8 {
    9
}
