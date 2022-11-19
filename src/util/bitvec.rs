pub struct BitVec {
    bits: Vec<u8>,
    len: usize,
}

impl BitVec {
    pub fn new() -> BitVec {
        BitVec { bits: Vec::new(), len: 0 }
    }

    pub fn with_capacity(capacity: usize) -> BitVec {
        BitVec { bits: Vec::with_capacity(capacity), len: 0 }
    }

    pub fn with_size(size: usize) -> BitVec {
        let mut bits = Vec::with_capacity(size / 8 + 1);
        for _ in 0..bits.capacity() {
            bits.push(0);
        }

        BitVec { bits, len: size }
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.len {
            None
        } else {
            unsafe { Some(*self.bits.get_unchecked(index / 8) & (1 << (index % 8)) != 0) }
        }
    }

    pub fn set(&mut self, index: usize, bit: bool) {
        assert!(index < self.len);
        if bit {
            self.bits[index / 8] |= 1 << (index % 8);
        } else {
            self.bits[index / 8] &= !(1 << (index % 8));
        }
    }

    pub fn resize(&mut self, new_size: usize) {
        if new_size > self.len {
            let new_capacity = new_size / 8 + 1;
            if new_capacity > self.bits.capacity() {
                self.bits.reserve(new_capacity - self.bits.capacity());
            }
            for _ in 0..new_capacity - self.bits.len() {
                self.bits.push(0);
            }
        }
        self.len = new_size;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.bits.capacity() * 8
    }

    pub fn iter(&self) -> BitVecIter {
        BitVecIter { bitvec: self, index: 0 }
    }

    pub fn copy_to_slice(&self, slice: &mut [u8]) -> usize {
        let count = std::cmp::min(self.len / 8, slice.len());
        for i in 0..count {
            slice[i] = self.bits[i];
        }

        count
    }
}

pub struct BitVecIter<'a> {
    bitvec: &'a BitVec,
    index: usize,
}

impl<'a> Iterator for BitVecIter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.index >= self.bitvec.len {
            None
        } else {
            let bit = self.bitvec.get(self.index).unwrap();
            self.index += 1;
            Some(bit)
        }
    }
}
