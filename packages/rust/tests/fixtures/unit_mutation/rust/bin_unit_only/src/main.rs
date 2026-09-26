fn main() {}

#[cfg(test)]
mod tests {
    #[test]
    fn checks_library_behavior() {
        assert_eq!(mut_bin_unit_only::triple(2), 6);
    }
}
