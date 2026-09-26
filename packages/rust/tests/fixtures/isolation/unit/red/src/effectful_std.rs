//! Red: the unit test opens a socket (`std::net`). Effectful `std` must sit behind an
//! injected trait, not be called directly in a unit test. The filesystem is the one
//! exception, and the network is not it.

pub fn label() -> &'static str {
    "reader"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaches_the_network() {
        // VIOLATION: effectful std (network).
        let _ = std::net::TcpStream::connect("127.0.0.1:9");
        assert_eq!(label(), "reader");
    }
}
