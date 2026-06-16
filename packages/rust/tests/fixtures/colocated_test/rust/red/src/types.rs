//! A pure data type with no behavior — not a unit-test subject, so it must not be
//! flagged even though it carries no inline test.

pub struct Point {
    pub x: u8,
    pub y: u8,
}
