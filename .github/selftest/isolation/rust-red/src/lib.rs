//! Violating: the inline `#[cfg(test)]` test opens a real socket. Effectful `std` must sit
//! behind an injected trait, so `unit lint` flags it and exits non-zero.

pub fn label() -> &'static str {
    "reader"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaches_the_network() {
        // VIOLATION: effectful std (the network) called directly in a unit test.
        let _ = std::net::TcpStream::connect("127.0.0.1:9");
        assert_eq!(label(), "reader");
    }
}
