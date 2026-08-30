/// Return the value unchanged.
pub fn described(value: u8) -> u8 {
    // The identity is the whole contract.
    //
    // A blank line follows this comment block.

    value
}

pub fn compute(value: u8) -> u8 {
    let scaled = value * 2;
    scaled + 1
}
