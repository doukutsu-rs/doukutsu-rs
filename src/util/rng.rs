use std::cell::Cell;
use std::ops::Range;

pub trait RNG {
    fn next(&self) -> i32;

    fn range(&self, range: Range<i32>) -> i32 {
        range.start + ((self.next() & 0x7fffffff) % (range.len() as i32 + 1))
    }
}

/// Deterministic XorShift-based random number generator
pub struct XorShift(Cell<u64>);

impl XorShift {
    pub fn new(seed: i32) -> Self {
        Self(Cell::new(seed as u64))
    }

    pub fn next_u64(&self) -> u64 {
        let mut state = self.0.get();

        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;

        self.0.replace(state);

        state.wrapping_mul(0x2545F4914F6CDD1Du64)
    }

    #[inline]
    pub fn next_u32(&self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    pub fn dump_state(&self) -> u64 {
        self.0.get()
    }

    pub fn load_state(&mut self, saved_state: u64) {
        self.0.replace(saved_state);
    }
}

impl RNG for XorShift {
    #[inline]
    fn next(&self) -> i32 {
        self.next_u32() as i32
    }
}

#[derive(Debug, Clone)]
pub struct Xoroshiro32PlusPlus(Cell<(u16, u16)>);

impl Xoroshiro32PlusPlus {
    pub fn new(seed: u32) -> Xoroshiro32PlusPlus {
        Xoroshiro32PlusPlus(Cell::new((
            (seed & 0xffff) as u16,
            (seed >> 16 & 0xffff) as u16
        )))
    }

    pub fn next_u16(&self) -> u16 {
        let mut state = self.0.get();
        let result = (state.0.wrapping_add(state.1)).rotate_left(9).wrapping_add(state.0);

        state.1 ^= state.0;
        state.0 = state.0.rotate_left(13) ^ state.1 ^ (state.1 << 5);
        state.1 = state.1.rotate_left(10);

        self.0.replace(state);

        result
    }

    pub fn dump_state(&self) -> u32 {
        let state = self.0.get();

        (state.0 as u32) | (state.1 as u32) << 16
    }

    pub fn load_state(&mut self, state: u32) {
        self.0.replace((
            (state & 0xffff) as u16,
            ((state >> 16) & 0xffff) as u16
        ));
    }
}

impl RNG for Xoroshiro32PlusPlus {
    fn next(&self) -> i32 {
        ((self.next_u16() as u32) << 16 | self.next_u16() as u32) as i32
    }
}
