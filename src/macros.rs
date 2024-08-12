#[doc(hidden)]
pub use core::fmt;
#[doc(hidden)]
pub use core::mem::size_of;

#[macro_export(local_inner_macros)]
macro_rules! case_insensitive_hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(case_insensitive_hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { case_insensitive_hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = case_insensitive_hashmap!(@count $($key),*);
            let mut _map = ::case_insensitive_hashmap::CaseInsensitiveHashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}
