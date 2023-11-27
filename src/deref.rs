use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ops::{Deref, Index};

/// A value that can be stably dereferenced with [`Deref`] trait.
/// A stable dereference means that a reference to the value will be valid
/// even if the reference is moved to a different memory location.
///
/// See <https://stackoverflow.com/q/77548941/177275> for more details.
///
/// # Safety
/// Implementing this trait is unsafe because the implementation must guarantee that
/// the [`Deref`] is stable per above.
pub unsafe trait StableDerefKey: Deref + Eq + Hash {}

unsafe impl StableDerefKey for String {}

pub struct DupIndexerRefs<T: StableDerefKey>
where
    <T as Deref>::Target: 'static,
{
    values: Vec<T>,
    lookup: HashMap<&'static T::Target, usize>,
}

impl<T> Default for DupIndexerRefs<T>
where
    T: StableDerefKey,
    T::Target: Eq + Hash + ToOwned<Owned = T>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DupIndexerRefs<T>
where
    T: StableDerefKey,
    T::Target: Eq + Hash + ToOwned<Owned = T>,
{
    /// Constructs a new, empty `DupGenIndexer`
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    /// Constructs a new, empty `DupGenIndexer` with at least the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            lookup: HashMap::with_capacity(capacity),
        }
    }

    /// Returns the total number of elements the indexer can hold without reallocating.
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Extracts a slice containing the entire indexer values.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// Get the number of values in the indexer.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Return true if the indexer is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Converts the indexer into a vector.
    #[inline]
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        self.values
    }

    /// Insert a string value into the indexer if it doesn't already exist,
    /// and return the index of the value.
    ///
    /// ```
    /// # use dup_indexer::DupIndexerRefs;
    /// # fn main() {
    /// let mut di = DupIndexerRefs::<String>::new();
    /// assert_eq!(di.insert_owned("hello".to_string()), 0);
    /// assert_eq!(di.insert_owned("world".to_string()), 1);
    /// assert_eq!(di.insert_owned("hello".to_string()), 0);
    /// assert_eq!(di.into_vec(), vec!["hello", "world"]);
    /// # }
    /// ```
    pub fn insert_owned(&mut self, value: T) -> usize {
        // This is safe because we own the value and will not modify or drop it,
        // unless we consume the whole values vector,
        // nor would we access the values in the vector before then.
        // When dropping, index will be dropped without freeing the memory.
        // create a static reference to the string, which will live as long as the program
        let value_ref =
            unsafe { std::mem::transmute::<&T::Target, &'static T::Target>(value.deref()) };

        match self.lookup.entry(value_ref) {
            Occupied(entry) => *entry.get(),
            Vacant(entry) => {
                let index = self.values.len();
                entry.insert(index);
                self.values.push(value);
                index
            }
        }
    }

    /// Insert a cloneable value into the indexer if it doesn't already exist,
    /// and return the index of the value. Slightly slower than [`DupIndexerRefs::insert_owned`],
    /// but allows value to be a reference that does not need to be cloned if it was already added.
    ///
    /// ```
    /// # use dup_indexer::DupIndexerRefs;
    /// # fn main() {
    /// let mut di = DupIndexerRefs::<String>::new();
    /// assert_eq!(di.insert_ref("hello"), 0);
    /// assert_eq!(di.insert_ref("world"), 1);
    /// assert_eq!(di.insert_ref("hello"), 0);
    /// assert_eq!(di.into_vec(), vec!["hello", "world"]);
    /// # }
    /// ```
    pub fn insert_ref(&mut self, value: &T::Target) -> usize {
        match self.lookup.get(value) {
            Some(index) => *index,
            None => self.insert_owned(value.to_owned()),
        }
    }
}

impl<T: StableDerefKey> Index<usize> for DupIndexerRefs<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl<T: StableDerefKey> IntoIterator for DupIndexerRefs<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> std::vec::IntoIter<T> {
        self.values.into_iter()
    }
}

impl<T: StableDerefKey> Deref for DupIndexerRefs<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        &self.values
    }
}

impl<T: StableDerefKey + Debug> Debug for DupIndexerRefs<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.values.iter().enumerate())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let mut di: DupIndexerRefs<String> = DupIndexerRefs::with_capacity(5);
        assert!(di.is_empty());
        assert!(di.capacity() >= 5);
        assert_eq!(di.insert_owned("foo".to_string()), 0);
        assert_eq!(di.insert_owned("bar".to_string()), 1);
        assert_eq!(di.insert_owned("foo".to_string()), 0);
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
    fn test_string_own() {
        let mut di: DupIndexerRefs<String> = DupIndexerRefs::with_capacity(5);
        assert_eq!(di.insert_owned("foo".to_string()), 0);
        assert_eq!(di.insert_ref("bar"), 1);
        assert_eq!(di.insert_ref("foo"), 0);
        assert_eq!(di.into_vec(), vec!["foo", "bar"]);
    }

    #[test]
    fn test_many_strings() {
        const ITERATIONS: usize = 50;
        let mut di: DupIndexerRefs<String> = DupIndexerRefs::with_capacity(1);
        let mut old_capacity = 0;
        let mut capacity_has_grown = false;
        for shift in &[0, ITERATIONS] {
            for _pass in 0..2 {
                for idx in 0..ITERATIONS {
                    assert_eq!(di.insert_owned((idx + shift).to_string()), idx + shift);
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
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
        );
    }
}
