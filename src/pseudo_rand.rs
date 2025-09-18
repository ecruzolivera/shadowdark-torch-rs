#![no_std]

pub struct XorShift8 {
    state: i8,
}

impl XorShift8 {
    pub fn new(seed: i8) -> Self {
        Self { state: seed }
    }
    pub fn rand8(&mut self) -> i8 {
        self.state ^= self.state << 3;
        self.state ^= self.state >> 5;
        self.state ^= self.state << 1;
        self.state
    }
    pub fn random_between(&mut self, min: i8, max: i8) -> i8 {
        let r = self.rand8();
        let range = max - min + 1;
        min + ((r.abs() as i8) % range)
    }
}
