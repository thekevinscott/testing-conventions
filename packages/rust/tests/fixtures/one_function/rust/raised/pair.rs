pub fn alpha(value: u8) -> u8 {
    let scaled = value * 2;
    let total = scaled + 1;
    total
}

pub fn beta(value: u8) -> u8 {
    let scaled = value * 3;
    let total = scaled + 2;
    total
}
