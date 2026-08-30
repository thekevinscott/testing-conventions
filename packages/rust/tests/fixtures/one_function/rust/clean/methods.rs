pub struct Widget {
    size: u8,
}

impl Widget {
    pub fn grow(&mut self, amount: u8) -> u8 {
        self.size += amount;
        self.size
    }

    pub fn shrink(&mut self, amount: u8) -> u8 {
        self.size -= amount;
        self.size
    }
}
