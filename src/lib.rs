#![doc = include_str!("../README.md")]

use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
};
use std::ops::Index;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::{ops, ptr};

/// A value that can be used as a key in a [`DupIndexer`], which will copy its content
/// using the [`ptr::read`] function, while also owning it internally.
///
/// # Safety
/// Implementing this trait is unsafe because the implementation must guarantee that
/// the value can be copied by copying the bits of the value assuming that the value
/// itself is valid and readonly. All Copy types are `PtrRead`, but Box<T> is not.
pub unsafe trait PtrRead {}

macro_rules! impl_trait {
    ($($t:ty),*) => {
        $(
            unsafe impl PtrRead for $t {}
        )*
    };
}

impl_trait![(), &'static str];
impl_trait![f32, f64, bool, char, String, PathBuf];
impl_trait![SystemTime, Duration, Ipv4Addr, Ipv6Addr, IpAddr];
impl_trait![u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize];
impl_trait![NonZeroU8, NonZeroU16, NonZeroU32];
impl_trait![NonZeroU64, NonZeroU128, NonZeroUsize];
impl_trait![NonZeroI8, NonZeroI16, NonZeroI32];
impl_trait![NonZeroI64, NonZeroI128, NonZeroIsize];

unsafe impl<T: PtrRead> PtrRead for [T] {}
unsafe impl<T: PtrRead, const N: usize> PtrRead for [T; N] {}
unsafe impl<T: PtrRead> PtrRead for Wrapping<T> {}
unsafe impl<T: PtrRead> PtrRead for Option<T> {}
unsafe impl<T: PtrRead> PtrRead for Vec<T> {}
unsafe impl<T: PtrRead, V: PtrRead, S> PtrRead for HashMap<T, V, S> {}
unsafe impl<T: PtrRead, V: PtrRead> PtrRead for BTreeMap<T, V> {}
unsafe impl<T: PtrRead> PtrRead for BTreeSet<T> {}

pub struct DupIndexer<T> {
    values: Vec<T>,
    lookup: HashMap<ManuallyDrop<T>, usize>,
}

impl<T: PtrRead> DupIndexer<T> {
    /// Create a new instance of `DupIndexer<T>`, without requiring `T` to implement `Default`.
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    /// Constructs a new, empty `DupIndexer<T>` with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            lookup: HashMap::with_capacity(capacity),
        }
    }

    /// Returns the total number of elements the indexer can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Extracts a slice containing the entire indexer values.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// Get the number of values in the indexer.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Return true if the indexer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Converts the indexer into a vector.
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.values
    }
}

/// If `T` implements `Default`, create a new instance of `DupIndexer<T>`.
/// Note that [`DupIndexer::new`] does not require `T` to implement `Default`.
impl<T: PtrRead + Default> Default for DupIndexer<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash> DupIndexer<T> {
    /// Insert a value into the indexer if it doesn't already exist,
    /// and return the index of the value.
    pub fn insert(&mut self, value: T) -> usize {
        // This is safe because we own the value and will not drop it unless we consume the whole values vector,
        // nor would we access the values in the vector before then.
        // When dropping, index will be dropped without freeing the memory.
        let dup_value = ManuallyDrop::new(unsafe { ptr::read(&value) });
        match self.lookup.entry(dup_value) {
            Occupied(entry) => *entry.get(),
            Vacant(entry) => {
                let index = self.values.len();
                entry.insert(index);
                self.values.push(value);
                index
            }
        }
    }
}

impl<T> Index<usize> for DupIndexer<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl<T> IntoIterator for DupIndexer<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> std::vec::IntoIter<T> {
        self.values.into_iter()
    }
}

impl<T> ops::Deref for DupIndexer<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        &self.values
    }
}

impl<T: Debug> Debug for DupIndexer<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.values.iter().enumerate())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    #[test]
    fn test_str() {
        let mut di: DupIndexer<&str> = DupIndexer::default();
        assert!(di.is_empty());
        assert_eq!(di.capacity(), 0);
        assert_eq!(di.insert("foo"), 0);
        assert_eq!(di.insert("bar"), 1);
        assert_eq!(di.insert("foo"), 0);
        assert_eq!(di[1], "bar");
        assert!(!di.is_empty());
        assert_eq!(di.len(), 2);
        assert!(di.capacity() >= 2);
        assert_eq!(di.deref(), &["foo", "bar"]);
        assert_eq!(di.as_slice(), &["foo", "bar"]);
        assert_eq!(format!("{di:?}"), r#"{0: "foo", 1: "bar"}"#);
        assert_eq!(di.into_vec(), vec!["foo", "bar"]);
    }

    #[test]
    fn test_string() {
        let mut di: DupIndexer<String> = DupIndexer::with_capacity(5);
        assert!(di.is_empty());
        assert!(di.capacity() >= 5);
        assert_eq!(di.insert("foo".to_string()), 0);
        assert_eq!(di.insert("bar".to_string()), 1);
        assert_eq!(di.insert("foo".to_string()), 0);
        assert_eq!(di[1], "bar");
        assert_eq!(di[1], "bar".to_string());
        assert!(!di.is_empty());
        assert_eq!(di.len(), 2);
        assert!(di.capacity() >= 5);
        assert_eq!(di.deref(), &["foo", "bar"]);
        assert_eq!(di.as_slice(), &["foo", "bar"]);
        assert_eq!(format!("{di:?}"), r#"{0: "foo", 1: "bar"}"#);
        assert_eq!(di.into_vec(), vec!["foo", "bar"]);
    }

    #[test]
    fn test_copyable_value() {
        let mut di: DupIndexer<i32> = DupIndexer::default();
        assert_eq!(di.insert(42), 0);
        assert_eq!(di.insert(13), 1);
        assert_eq!(di.insert(42), 0);
        assert_eq!(di[1], 13);
        assert_eq!(di.into_iter().collect::<Vec::<_>>(), vec![42, 13]);
    }

    #[test]
    fn test_copyable_struct() {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        struct Foo(pub i32);

        unsafe impl PtrRead for Foo {}

        let mut di: DupIndexer<Foo> = DupIndexer::new();
        assert_eq!(di.insert(Foo(42)), 0);
        assert_eq!(di.insert(Foo(13)), 1);
        assert_eq!(di.insert(Foo(42)), 0);
        assert_eq!(di[1], Foo(13));
        assert_eq!(di.into_vec(), vec![Foo(42), Foo(13)]);
    }

    #[test]
    fn test_vec() {
        let mut di: DupIndexer<Vec<i32>> = DupIndexer::default();
        assert_eq!(di.insert(vec![1, 2, 3]), 0);
        assert_eq!(di.insert(vec![1, 2]), 1);
        assert_eq!(di.insert(vec![1, 2, 3]), 0);
        assert_eq!(di[1], vec![1, 2]);
        assert_eq!(di.into_vec(), vec![vec![1, 2, 3], vec![1, 2]]);
    }

    #[test]
    fn test_debug_fmt() {
        let mut di: DupIndexer<char> = DupIndexer::default();
        assert_eq!(di.insert('a'), 0);
        assert_eq!(di.insert('b'), 1);
        assert_eq!(di.insert('c'), 2);
        assert_eq!(di.insert('b'), 1);
        assert_eq!(di[2], 'c');
        assert_eq!(format!("{di:?}"), "{0: 'a', 1: 'b', 2: 'c'}");
        assert_eq!(di.into_vec(), vec!['a', 'b', 'c']);
    }

    #[test]
    fn test_many_strings() {
        const ITERATIONS: usize = 50;
        let mut di: DupIndexer<String> = DupIndexer::with_capacity(1);
        let mut old_capacity = 0;
        let mut capacity_has_grown = false;
        for shift in &[0, ITERATIONS] {
            for _pass in 0..2 {
                for idx in 0..ITERATIONS {
                    assert_eq!(di.insert((idx + shift).to_string()), idx + shift);
                    if old_capacity == 0 {
                        old_capacity = di.capacity();
                    } else if di.capacity() > old_capacity {
                        capacity_has_grown = true;
                    }
                }
            }
        }
        // Ensure that capacity has grown at least once
        assert!(capacity_has_grown);
        assert_eq!(
            di.into_vec(),
            (0..ITERATIONS * 2)
                .into_iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_custom_trait() {
        #[derive(Debug, Eq, PartialEq, Hash)]
        enum Value {
            Str(String),
            Int(i32),
        }

        unsafe impl PtrRead for Value {}

        let mut di: DupIndexer<Value> = DupIndexer::new();
        assert_eq!(di.insert(Value::Str("foo".to_string())), 0);
        assert_eq!(di.insert(Value::Int(42)), 1);
        assert_eq!(di.insert(Value::Str("foo".to_string())), 0);
        assert_eq!(di[1], Value::Int(42));
        assert_eq!(
            di.into_vec(),
            vec![Value::Str("foo".to_string()), Value::Int(42)]
        );
    }

    // // This test is ignored on Miri because it fails without any good explanation at the moment.
    // // See issue https://github.com/nyurik/dup-indexer/issues/1
    // #[test]
    // #[cfg_attr(miri, ignore)]
    // fn test_box() {
    //     let mut di: DupIndexer<Box<i32>> = DupIndexer::default();
    //     assert_eq!(di.insert(Box::new(42)), 0);
    //     assert_eq!(di.insert(Box::new(13)), 1);
    //     assert_eq!(di.insert(Box::new(42)), 0);
    //     assert_eq!(di[1], Box::new(13));
    //     assert_eq!(di.into_vec(), vec![Box::new(42), Box::new(13)]);
    // }
}
