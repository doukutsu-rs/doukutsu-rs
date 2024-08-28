use std::{
    cell::UnsafeCell, fmt::{self, Debug, Display, Formatter}, hash::{Hash, Hasher}, sync::{Mutex, OnceLock}
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
        // let byte = bytes[i] as u32;
        // this is safe because we are guaranteed to have a valid index, avoids redundant bounds check
        let byte: u32 = unsafe { bytes.as_ptr().add(i).read() } as u32;
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
        debug_assert!(bucket < BUCKET_SIZE); // if this ever fails, the below is UB
        let bucket = unsafe { self.buckets.get_unchecked(bucket) };

        if let Some(atom) = bucket {
            for atom in atom.iter() {
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

        debug_assert!(bucket < BUCKET_SIZE); // if this ever fails, the below is UB
        let bucket = unsafe { self.buckets.get_unchecked_mut(bucket) };

        if let Some(atom) = bucket {
            let mut prev_atom = *atom;
            for atom in atom.iter() {
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
}

impl Debug for AtomStore {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("AtomStore")?;

        #[cfg(any(test, debug_assertions))]
        {
            struct AtomPrintList(&'static AtomData);
            impl Debug for AtomPrintList {
                fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                    f.debug_list().entries(self.0.iter().map(|atom| atom.data)).finish()
                }
            }

            let mut dbg = f.debug_map();
            for (i, bucket) in self.buckets.iter().enumerate() {
                if let Some(atom) = bucket {
                    dbg.entry(&i, &AtomPrintList(atom));
                }
            }
            dbg.finish()?;
        }

        Ok(())
    }
}

// todo: Rust 1.80 has LazyLock, which at time of writing is stable, but we'll wait a bit for everyone to catch up
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

impl Hash for Atom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data().hash.hash(state);
    }
}

impl Debug for Atom {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {

        f.write_str("Atom(")?;
        Debug::fmt(self.data().data, f)?;
        f.write_str(")")
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self.data().data, f)
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
