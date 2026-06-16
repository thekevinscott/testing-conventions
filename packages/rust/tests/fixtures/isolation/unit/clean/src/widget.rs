//! Clean: a well-isolated unit. Logic is exercised via `super::`, the collaborator
//! is a hand-rolled trait double (injected, not reached out of module), and I/O is
//! in-memory via `Cursor`. Nothing leaves the module, so nothing is flagged.

use std::io::Read;

/// A collaborator the unit depends on — injected as a trait so a unit test can
/// double it without reaching out of the module.
pub trait Clock {
    fn now_secs(&self) -> u64;
}

pub fn stamp<C: Clock>(clock: &C) -> String {
    format!("t={}", clock.now_secs())
}

pub fn read_all(mut input: impl Read) -> std::io::Result<String> {
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Hand-rolled double of `Clock` — the in-module way to isolate the unit.
    struct FrozenClock;

    impl Clock for FrozenClock {
        fn now_secs(&self) -> u64 {
            42
        }
    }

    #[test]
    fn stamps_with_an_injected_clock() {
        // `super::stamp` (via glob) + an injected double — all in-module.
        assert_eq!(stamp(&FrozenClock), "t=42");
    }

    #[test]
    fn reads_from_an_in_memory_cursor() {
        // `Cursor` is the idiomatic in-memory I/O double — pure std, allowed.
        let got = read_all(Cursor::new(b"hello")).expect("cursor read never fails");
        assert_eq!(got, "hello");
    }
}
