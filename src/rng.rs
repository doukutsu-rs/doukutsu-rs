use std::cell::Cell;

pub struct RNG(Cell<i32>);

impl RNG {
    pub fn new(seed: i32) -> Self {
        Self(Cell::new(seed))
    }

    pub fn next(&self) -> i32 {
        // MSVC LCG values
        self.0.replace(self.0.get().wrapping_mul(214013).wrapping_add(2531011));
        self.0.get()
    }

    pub fn next_u32(&self) -> u32 {
        self.next() as u32
    }

    pub fn range(&self, range: std::ops::Range<i32>) -> i32 {
        range.start.wrapping_add(self.next() % (range.end.wrapping_sub(range.start).wrapping_add(1)))
    }
}
