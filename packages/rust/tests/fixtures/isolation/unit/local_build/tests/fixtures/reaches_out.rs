// A parseable `.rs` under `tests/fixtures/` whose `#[cfg(test)]` module reaches out of
// its module (`crate::store::load()`). If the walk recursed `tests/`, this would be
// false-flagged `no-out-of-module-call` — but `tests/` is skipped, so it is not.

pub fn helper() -> u8 {
    0
}

#[cfg(test)]
mod tests {
    #[test]
    fn reaches_out() {
        let _ = crate::store::load();
    }
}
