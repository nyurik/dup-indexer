#![doc = include_str!("../README.md")]

use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::ptr;

pub struct DupIndexer<T> {
    values: Vec<T>,
    lookup: HashMap<ManuallyDrop<T>, usize>,
}

impl<T> DupIndexer<T> {
    /// Create a new instance of `DupIndexer<T>`, without requiring `T` to implement `Default`.
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.values
    }
}

/// If `T` implements `Default`, create a new instance of `DupIndexer<T>`.
/// Note that [`DupIndexer::new`] does not require `T` to implement `Default`.
impl<T: Default> Default for DupIndexer<T> {
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

impl<T> IntoIterator for DupIndexer<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> std::vec::IntoIter<T> {
        self.values.into_iter()
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

    #[derive(Debug, Eq, PartialEq, Hash)]
    enum Value {
        Str(String),
        Int(i32),
    }

    #[test]
    fn test_custom_trait() {
        let mut di: DupIndexer<Value> = DupIndexer::new();
        assert_eq!(0, di.insert(Value::Str("foo".to_string())));
        assert_eq!(1, di.insert(Value::Int(42)));
        assert_eq!(0, di.insert(Value::Str("foo".to_string())));
        assert_eq!(
            di.into_vec(),
            vec![Value::Str("foo".to_string()), Value::Int(42)]
        );
    }

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

        let mut di: DupIndexer<i32> = DupIndexer::default();
        assert_eq!(0, di.insert(42));
        assert_eq!(1, di.insert(13));
        assert_eq!(0, di.insert(42));
        assert_eq!(di.into_iter().collect::<Vec::<_>>(), vec![42, 13]);
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
        assert_eq!(0, di.insert("foo".to_string()));
        assert_eq!(1, di.insert("bar".to_string()));
        assert_eq!(0, di.insert("foo".to_string()));
        assert_eq!(format!("{di:?}"), r#"{0: "foo", 1: "bar"}"#);
        assert_eq!(di.into_vec(), vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn test_vec() {
        let mut di: DupIndexer<Vec<i32>> = DupIndexer::default();
        assert_eq!(0, di.insert(vec![1, 2, 3]));
        assert_eq!(1, di.insert(vec![1, 2]));
        assert_eq!(0, di.insert(vec![1, 2, 3]));
        assert_eq!(di.into_vec(), vec![vec![1, 2, 3], vec![1, 2]]);
    }

    #[test]
    fn test_debug_fmt() {
        let mut di: DupIndexer<char> = DupIndexer::default();
        assert_eq!(0, di.insert('a'));
        assert_eq!(1, di.insert('b'));
        assert_eq!(2, di.insert('c'));
        assert_eq!(1, di.insert('b'));
        assert_eq!(format!("{di:?}"), "{0: 'a', 1: 'b', 2: 'c'}");
        assert_eq!(di.into_vec(), vec!['a', 'b', 'c']);
    }

    #[test]
    fn test_many_strings() {
        const ITERATIONS: usize = 100;
        let mut di: DupIndexer<String> = DupIndexer::new();
        for shift in &[0, ITERATIONS] {
            for _pass in 0..2 {
                for idx in 0..ITERATIONS {
                    assert_eq!(idx + shift, di.insert((idx + shift).to_string()));
                }
            }
        }
        assert_eq!(
            di.into_vec(),
            (0..ITERATIONS * 2)
                .into_iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_box() {
        let mut di: DupIndexer<Box<i32>> = DupIndexer::default();
        assert_eq!(0, di.insert(Box::new(42)));
        assert_eq!(1, di.insert(Box::new(13)));
        assert_eq!(0, di.insert(Box::new(42)));
        assert_eq!(di.into_vec(), vec![Box::new(42), Box::new(13)]);
    }
}
