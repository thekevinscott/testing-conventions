pub fn build(values: &[u8]) -> Vec<u8> {
    fn inner(value: u8) -> u8 {
        let doubled = value * 2;
        doubled
    }

    let mut mapped: Vec<u8> = values.iter().copied().map(inner).collect();
    mapped.sort_unstable();
    mapped
}
