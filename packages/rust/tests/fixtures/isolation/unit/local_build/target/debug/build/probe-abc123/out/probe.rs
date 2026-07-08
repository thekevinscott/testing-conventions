// A built artifact under `target/` (as a locally-built crate has). Its `#[cfg(test)]`
// module reaches out of its module, so walking `target/` would false-flag it — but the
// unit-isolation walk skips `target/`, exactly like the colocated-test presence walk.

#[cfg(test)]
mod tests {
    #[test]
    fn generated() {
        let _ = crate::store::load();
        let _ = std::fs::read("x");
    }
}
