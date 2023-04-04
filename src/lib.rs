#![doc = include_str!("../README.md")]

use std::collections::HashMap;
use std::hash::Hash;
use std::mem;

pub struct DupIndexer<T> {
    values: Vec<T>,
    index: LeakyKeyHashMap<T, usize>,
}

impl<T> DupIndexer<T> {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            index: LeakyKeyHashMap::new(),
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.values
    }
}

impl<T: Default> Default for DupIndexer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + Eq + Hash> DupIndexer<T> {
    pub fn insert(&mut self, value: T) -> usize {
        if let Some(index) = self.index.0.get(&value) {
            *index
        } else {
            let index = self.values.len();
            self.values.push(value);
            self.index.0.insert(value, index);
            index
        }
    }
}

impl DupIndexer<String> {
    pub fn insert_string(&mut self, value: String) -> usize {
        if let Some(index) = self.index.0.get(&value) {
            *index
        } else {
            let index = self.values.len();
            let (ptr, len, cap) = (value.as_ptr(), value.len(), value.capacity());
            self.values.push(value);
            let val_dup = unsafe { String::from_raw_parts(ptr as *mut u8, len, cap) };
            self.index.0.insert(val_dup, index);
            index
        }
    }
}

impl<T: Eq + Hash> DupIndexer<Vec<T>> {
    pub fn insert_vec(&mut self, value: Vec<T>) -> usize {
        if let Some(index) = self.index.0.get(&value) {
            *index
        } else {
            let index = self.values.len();
            let (ptr, len, cap) = (value.as_ptr(), value.len(), value.capacity());
            self.values.push(value);
            let val_dup = unsafe { Vec::from_raw_parts(ptr as *mut T, len, cap) };
            self.index.0.insert(val_dup, index);
            index
        }
    }
}

impl<T: Eq + Hash> DupIndexer<Box<T>> {
    /// # Safety
    /// Make sure you only pass in a Box that was allocated using the default allocator.
    /// Once https://github.com/rust-lang/rust/issues/32838 is released,
    /// this can be changed to use [`Box::from_raw_in`] instead, and it would become safer.
    pub unsafe fn insert_box(&mut self, value: Box<T>) -> usize {
        if let Some(index) = self.index.0.get(&value) {
            *index
        } else {
            let index = self.values.len();
            // This is not ideal - we need to get the
            let raw = Box::into_raw(value);
            let value = unsafe { Box::from_raw(raw) };
            self.values.push(value);
            let val_dup = unsafe { Box::from_raw(raw) };
            self.index.0.insert(val_dup, index);
            index
        }
    }
}

struct LeakyKeyHashMap<K, V>(pub HashMap<K, V>);

impl<K, V> LeakyKeyHashMap<K, V> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl<K, V> Drop for LeakyKeyHashMap<K, V> {
    fn drop(&mut self) {
        for k in mem::take(&mut self.0).into_keys() {
            mem::forget(k);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str() {
        let mut di: DupIndexer<&str> = DupIndexer::default();
        assert_eq!(0, di.insert("foo"));
        assert_eq!(1, di.insert("bar"));
        assert_eq!(0, di.insert("foo"));
        assert_eq!(di.into_vec(), vec!["foo", "bar"]);
    }

    #[test]
    fn test_copyable_value() {
        let mut di: DupIndexer<i32> = DupIndexer::default();
        assert_eq!(0, di.insert(42));
        assert_eq!(1, di.insert(13));
        assert_eq!(0, di.insert(42));
        assert_eq!(di.into_vec(), vec![42, 13]);
    }

    #[test]
    fn test_copyable_struct() {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        struct Foo(pub i32);

        let mut di: DupIndexer<Foo> = DupIndexer::new();
        assert_eq!(0, di.insert(Foo(42)));
        assert_eq!(1, di.insert(Foo(13)));
        assert_eq!(0, di.insert(Foo(42)));
        assert_eq!(di.into_vec(), vec![Foo(42), Foo(13)]);
    }

    #[test]
    fn test_string() {
        let mut di: DupIndexer<String> = DupIndexer::default();
        assert_eq!(0, di.insert_string("foo".to_string()));
        assert_eq!(1, di.insert_string("bar".to_string()));
        assert_eq!(0, di.insert_string("foo".to_string()));
        assert_eq!(di.into_vec(), vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn test_vec() {
        let mut di: DupIndexer<Vec<i32>> = DupIndexer::default();
        assert_eq!(0, di.insert_vec(vec![1, 2, 3]));
        assert_eq!(1, di.insert_vec(vec![1, 2, 4]));
        assert_eq!(0, di.insert_vec(vec![1, 2, 3]));
        assert_eq!(di.into_vec(), vec![vec![1, 2, 3], vec![1, 2, 4]]);
    }

    #[test]
    fn test_box() {
        let mut di: DupIndexer<Box<i32>> = DupIndexer::default();
        assert_eq!(0, unsafe { di.insert_box(Box::new(42)) });
        unsafe { di.insert_box(Box::new(13)) };
        assert_eq!(0, unsafe { di.insert_box(Box::new(42)) });
        assert_eq!(di.into_vec(), vec![Box::new(42), Box::new(13)]);
    }
}
