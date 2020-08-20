/// Stateful RNG
pub struct RNG {
    pub seed: u32,
}

impl RNG {
    pub fn new() -> Self {
        Self {
            seed: 0,
        }
    }

    pub fn next(&mut self) -> u32 {
        // MSVC LCG values
        self.seed = self.seed.wrapping_mul(214013).wrapping_add(2531011);
        self.seed
    }

    pub fn range(&mut self, start: i32, end: i32) -> i32 {
        start + (self.next() % (end - start) as u32) as i32
    }
}
