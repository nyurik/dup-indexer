#![doc = include_str!("../README.md")]

use std::collections::HashMap;
use std::hash::Hash;
use std::mem::ManuallyDrop;

pub struct DupIndexer<T> {
    values: Vec<T>,
    index: HashMap<ManuallyDrop<T>, usize>,
}

impl<T> DupIndexer<T> {
    /// Create a new instance of `DupIndexer<T>`, without requiring `T` to implement `Default`.
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            index: HashMap::new(),
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

impl<T: UnsafeCopy + Eq + Hash> DupIndexer<T> {
    /// Insert a value into the indexer if it doesn't already exist,
    /// and return the index of the value.
    pub fn insert(&mut self, value: T) -> usize {
        let value = ManuallyDrop::new(value);
        let index = self.index.get(&value);
        let value = ManuallyDrop::into_inner(value);

        if let Some(index) = index {
            *index
        } else {
            let index = self.values.len();
            // Index stores `val_dup` because the duplicate is "less sound" than the original.
            // For example, the dup might not have the vector capacity value set, which we don't need for the read access.
            self.index.insert(unsafe { value.duplicate() }, index);
            self.values.push(value);
            index
        }
    }
}

impl<T: Copy + Eq + Hash> DupIndexer<T> {
    /// Insert a value into the indexer if it doesn't already exist,
    /// and return the index of the value.
    /// Same as [`DupIndexer::insert`] but for custom types that implement the [`Copy`] trait.
    pub fn insert_copy(&mut self, value: T) -> usize {
        let value = ManuallyDrop::new(value);
        let index = self.index.get(&value);
        let value = ManuallyDrop::into_inner(value);

        if let Some(index) = index {
            *index
        } else {
            let index = self.values.len();
            self.index.insert(ManuallyDrop::new(value), index);
            self.values.push(value);
            index
        }
    }
}

/// Trait for non-[Copy]-able types to create a shallow copy of an object so it can be used by [`DupIndexer`].
/// The usage and implementation of this trait is inherently unsafe.
pub unsafe trait UnsafeCopy {
    /// Creates a shallow copy of an object without allocating new memory.
    /// # Safety
    /// The duplicate can be less sound than the original,
    /// but it must be safe to use as a `HashMap` key for read-only lookups.
    /// The duplicate will not be dropped.
    /// Object duplication is unsafe, and should only be called by the [`DupIndexer`] implementation.
    unsafe fn duplicate(&self) -> ManuallyDrop<Self>
    where
        ManuallyDrop<Self>: Sized;
}

unsafe impl UnsafeCopy for String {
    unsafe fn duplicate(&self) -> ManuallyDrop<Self> {
        let (ptr, len, cap) = (self.as_ptr(), self.len(), self.capacity());
        ManuallyDrop::new(unsafe { String::from_raw_parts(ptr as *mut u8, len, cap) })
    }
}

unsafe impl<T> UnsafeCopy for Vec<T> {
    unsafe fn duplicate(&self) -> ManuallyDrop<Self> {
        let (ptr, len, cap) = (self.as_ptr(), self.len(), self.capacity());
        ManuallyDrop::new(unsafe { Vec::from_raw_parts(ptr as *mut T, len, cap) })
    }
}

impl<T: Eq + Hash> DupIndexer<Box<T>> {
    /// Inserts a boxed value into the indexer, and returns the index of the inserted or existing value.
    ///
    /// # Safety
    /// Make sure you only pass in a Box that was allocated using the default allocator.
    /// Once <https://github.com/rust-lang/rust/issues/32838> is released,
    /// this can be changed to use [`Box::from_raw_in`] instead, and it would become safer.
    pub unsafe fn insert_box(&mut self, value: Box<T>) -> usize {
        let value = ManuallyDrop::new(value);
        if let Some(index) = self.index.get(&value) {
            *index
        } else {
            let index = self.values.len();
            let raw = Box::into_raw(ManuallyDrop::into_inner(value));

            self.index
                .insert(ManuallyDrop::new(unsafe { Box::from_raw(raw) }), index);

            // Bad: first we destroyed the box, and now reconstructing it again using default allocator
            let value = unsafe { Box::from_raw(raw) };
            self.values.push(value);

            index
        }
    }
}

macro_rules! impl_unsafe_copy {
    ($($t:ty),* $(,)?) => {
        $(
            unsafe impl UnsafeCopy for $t {
                #[inline(always)]
                unsafe fn duplicate(&self) -> ManuallyDrop<Self> {
                    ManuallyDrop::new(*self)
                }
            }
        )*
    };
}

// Value types are always safe to duplicate because they implement `Copy` trait
impl_unsafe_copy!(
    &str, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool,
);

#[cfg(test)]
mod tests {
    use super::*;

    unsafe impl UnsafeCopy for Value {
        unsafe fn duplicate(&self) -> ManuallyDrop<Self> {
            ManuallyDrop::new(match self {
                Value::Str(v) => unsafe { Value::Str(ManuallyDrop::into_inner(v.duplicate())) },
                Value::Int(v) => Value::Int(*v),
            })
        }
    }

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
    }

    #[test]
    fn test_copyable_struct() {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        struct Foo(pub i32);

        let mut di: DupIndexer<Foo> = DupIndexer::new();
        assert_eq!(0, di.insert_copy(Foo(42)));
        assert_eq!(1, di.insert_copy(Foo(13)));
        assert_eq!(0, di.insert_copy(Foo(42)));
        assert_eq!(di.into_vec(), vec![Foo(42), Foo(13)]);
    }

    #[test]
    fn test_string() {
        let mut di: DupIndexer<String> = DupIndexer::default();
        assert_eq!(0, di.insert("foo".to_string()));
        assert_eq!(1, di.insert("bar".to_string()));
        assert_eq!(0, di.insert("foo".to_string()));
        assert_eq!(di.into_vec(), vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn test_vec() {
        let mut di: DupIndexer<Vec<i32>> = DupIndexer::default();
        assert_eq!(0, di.insert(vec![1, 2, 3]));
        assert_eq!(1, di.insert(vec![1, 2, 4]));
        assert_eq!(0, di.insert(vec![1, 2, 3]));
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
