const CONSTANT: u8 = 3;

pub fn normalize(value: u8) -> u8 {
    let scaled = value * CONSTANT;
    scaled
}

pub fn double(value: u8) -> u8 { value * 2 }

pub fn triple(value: u8) -> u8 {
    value * 3
}
