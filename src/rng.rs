use std::cell::Cell;

/// Deterministic XorShift-based random number generator
pub struct RNG(Cell<(u64, u64, u64, u64)>);

impl RNG {
    pub fn new(seed: i32) -> Self {
        Self(Cell::new((seed as u64,
                        (seed as u64).wrapping_add(0x9e3779b97f4a7c15),
                        (seed as u64).wrapping_add(0xbdd3944475a73cf0),
                        0
        )))
    }

    pub fn next_u64(&self) -> i32 {
        let mut state = self.0.get();
        let result = state.1.wrapping_mul(5).rotate_left(5).wrapping_mul(9);
        let t = state.1 << 17;

        state.2 ^= state.0;
        state.3 ^= state.1;
        state.1 ^= state.2;
        state.0 ^= state.3;

        state.2 ^= t;
        state.3 = state.3.rotate_left(45);

        self.0.replace(state);
        result as i32
    }

    #[inline]
    pub fn next(&self) -> i32 {
        self.next_u64() as i32
    }

    #[inline]
    pub fn next_u32(&self) -> u32 {
        self.next_u64() as u32
    }

    pub fn dump_state(&self) -> (u64, u64, u64, u64) {
        self.0.get()
    }

    pub fn load_state(&mut self, saved_state: (u64, u64, u64, u64)) {
        self.0.replace(saved_state);
    }

    pub fn range(&self, range: std::ops::Range<i32>) -> i32 {
        range.start.wrapping_add((self.next_u32() >> 2) as i32 % (range.end.wrapping_sub(range.start).wrapping_add(1)))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Xoroshiro32PlusPlus(u16, u16);

impl Xoroshiro32PlusPlus {
    pub fn new(seed: u32) -> Xoroshiro32PlusPlus {
        Xoroshiro32PlusPlus(
            (seed & 0xffff) as u16,
            (seed >> 16 & 0xffff) as u16
        )
    }

    pub fn next_u16(&mut self) -> u16 {
        let mut result = (self.0.wrapping_add(self.1)).rotate_left(9).wrapping_add(self.0);

        self.1 ^= self.0;
        self.0 = self.0.rotate_left(13) ^ self.1 ^ (self.1 << 5);
        self.1 = self.1.rotate_left(10);

        result
    }

    pub fn dump_state(&self) -> u32 {
        (self.0 as u32) | (self.1 as u32) << 16
    }

    pub fn load_state(&mut self, state: u32) {
        self.0 = (state & 0xffff) as u16;
        self.1 = ((state >> 16) & 0xffff) as u16;
    }

    pub fn range(&mut self, range: std::ops::Range<i32>) -> i32 {
        let num = ((self.next_u16() as u32) << 16 | self.next_u16() as u32) >> 2;
        range.start.wrapping_add(num as i32 % (range.end.wrapping_sub(range.start).wrapping_add(1)))
    }
}
