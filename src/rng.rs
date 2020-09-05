use std::cell::Cell;

pub struct RNG(Cell<(u64, u64, u64, u64)>);

fn rol64(x: u64, shift: u64) -> u64
{
    if shift == 0 || shift == 64 {
        x
    } else {
        (x << shift) | (x >> (64 - shift))
    }
}

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
        let result = rol64(state.1.wrapping_mul(5), 7).wrapping_mul(9);
        let t = state.1 << 17;

        state.2 ^= state.0;
        state.3 ^= state.1;
        state.1 ^= state.2;
        state.0 ^= state.3;

        state.2 ^= t;
        state.3 = rol64(state.3, 45);

        self.0.replace(state);
        result as i32
    }

    pub fn next(&self) -> i32 {
        self.next_u64() as i32
    }

    pub fn next_u32(&self) -> u32 {
        self.next_u64() as u32
    }

    pub fn range(&self, range: std::ops::Range<i32>) -> i32 {
        range.start.wrapping_add((self.next_u32() >> 2) as i32 % (range.end.wrapping_sub(range.start).wrapping_add(1)))
    }
}
