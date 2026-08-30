pub fn encode(value: u8) -> u8 {
    let total = value + 1;
    total
}

pub fn decode(value: u8) -> u8 {
    let total = value - 1;
    total
}
