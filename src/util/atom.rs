use std::{
    cell::UnsafeCell,
    sync::{Mutex, OnceLock},
};

const BUCKET_BITS: usize = 10;
const BUCKET_SIZE: usize = 1 << BUCKET_BITS;
const BUCKET_MASK: usize = BUCKET_SIZE - 1;

type HashType = u32;

struct AtomData {
    data: &'static str,
    hash: HashType,
    next: UnsafeCell<Option<&'static AtomData>>,
}

impl AtomData {
    fn new(data: &'static str, hash: HashType) -> &'static Self {
        let data = Self { data, hash, next: UnsafeCell::new(None) };
        Box::leak(Box::new(data))
    }

    fn iter(&'static self) -> AtomIter {
        AtomIter { current: Some(self) }
    }
}

unsafe impl Sync for AtomData {}
unsafe impl Send for AtomData {}

struct AtomIter {
    current: Option<&'static AtomData>,
}

impl Iterator for AtomIter {
    type Item = &'static AtomData;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = unsafe { *current.next.get() };
        Some(current)
    }
}

struct AtomStore {
    buckets: [Option<&'static AtomData>; BUCKET_SIZE],
}

const fn string_hash(data: &str) -> HashType {
    let mut hash = 0x811c9dc5u32;
    let mut i = 0;
    let bytes = data.as_bytes();
    while i < bytes.len() {
        let byte = bytes[i] as u32;
        hash ^= byte;
        hash = hash.wrapping_mul(0x01000193u32);
        i += 1;
    }

    hash
}

impl AtomStore {
    fn new() -> Box<Self> {
        Box::new(Self { buckets: [None; BUCKET_SIZE] })
    }

    fn find(&self, data: &str) -> Option<Atom> {
        let hash = string_hash(data);
        let bucket = (hash as usize) & BUCKET_MASK;
        let bucket = unsafe { self.buckets.get_unchecked(bucket) };

        if let Some(atom) = bucket {
            let iter = atom.iter();
            for atom in iter {
                if atom.data == data {
                    return Some(Atom(atom));
                }
            }
        }

        None
    }

    fn find_or_insert(&mut self, data: &str) -> Atom {
        if let Some(atom) = self.find(data) {
            return atom;
        }

        let string: &'static str = Box::leak(data.to_string().into_boxed_str());
        self.insert(string)
    }

    fn insert(&mut self, data: &'static str) -> Atom {
        let hash = string_hash(data);
        let bucket = (hash as usize) & BUCKET_MASK;
        let bucket = unsafe { self.buckets.get_unchecked_mut(bucket) };

        if let Some(atom) = bucket {
            let iter = atom.iter();
            let mut prev_atom = *atom;
            for atom in iter {
                if atom.data == data {
                    return Atom(atom);
                }

                prev_atom = atom;
            }

            let new_atom = AtomData::new(data, hash);
            unsafe {
                *prev_atom.next.get() = Some(new_atom);
            }
            Atom(new_atom)
        } else {
            let atom = AtomData::new(data, hash);
            *bucket = Some(atom);
            Atom(atom)
        }
    }

    #[cfg(any(test, debug_assertions))]
    fn print_debug(&self) {
        for (i, bucket) in self.buckets.iter().enumerate() {
            if let Some(atom) = bucket {
                println!("Bucket {}: {:?}", i, atom.data);
                let iter = atom.iter();
                for atom in iter {
                    println!("  {:?}", atom.data);
                }
            }
        }
    }
}

impl std::fmt::Debug for AtomStore {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("AtomStore")?;

        #[cfg(any(test, debug_assertions))]
        {
            let mut dbg = f.debug_map();
            for (i, bucket) in self.buckets.iter().enumerate() {
                if let Some(atom) = bucket {
                    struct AtomPrintList(&'static AtomData);
                    impl std::fmt::Debug for AtomPrintList {
                        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                            let mut list = f.debug_list();
                            let iter = self.0.iter();
                            for atom in iter {
                                list.entry(&atom.data);
                            }
                            list.finish()
                        }
                    }

                    dbg.entry(&i, &AtomPrintList(atom));
                }
            }
            dbg.finish()
        }
    }
}

static ATOM_STORE: OnceLock<Mutex<Box<AtomStore>>> = OnceLock::new();

fn get_atom_store() -> &'static Mutex<Box<AtomStore>> {
    ATOM_STORE.get_or_init(|| Mutex::new(AtomStore::new()))
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Atom(*const AtomData);

impl Atom {
    pub fn new(data: &'static str) -> Self {
        get_atom_store().lock().unwrap().insert(data)
    }

    pub fn new_str(data: &str) -> Self {
        get_atom_store().lock().unwrap().find_or_insert(data)
    }

    fn data(&self) -> &'static AtomData {
        unsafe { &*self.0 }
    }

    pub fn as_str(&self) -> &'static str {
        self.data().data
    }
}

impl Eq for Atom {}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::hash::Hash for Atom {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { (*self.0).hash.hash(state) }
    }
}

impl std::fmt::Debug for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Atom(")?;
        self.data().data.fmt(f)?;
        f.write_str(")")
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.data().data.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom() {
        // basic tests
        let a = Atom::new("hello");
        let b = Atom::new("world");
        let c = Atom::new("hello");
        let d = Atom::new_str("hello");
        let e = Atom::new_str(&"hello".to_owned());

        assert_eq!(a, c);
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_eq!(a, d);
        assert_eq!(a, e);

        assert_eq!(a.as_str(), "hello");
        assert_eq!(b.as_str(), "world");

        // empty string tests
        let empty = Atom::new("");
        let empty2 = Atom::new_str("");
        let empty_str = Atom::new_str("");

        assert_eq!(empty, empty2);
        assert_eq!(empty, empty_str);
        assert_eq!(empty.as_str(), "");

        // fnv32a collision test
        let a = Atom::new("costarring");
        let b = Atom::new("liquid");

        assert_ne!(a, b);

        println!("{:#?}", *get_atom_store().lock().unwrap());
    }
}
