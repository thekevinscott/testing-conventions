pub fn value() -> bool {
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn direct_call_is_caught() {
        let _ = crate::other::load();
    }

    #[test]
    fn asserted_call_is_caught() {
        assert!(crate::other::load());
    }

    #[test]
    fn asserted_external_call_is_caught() {
        assert_eq!(rand::random::<u8>(), 0, "{}", crate::other::load());
    }
}
