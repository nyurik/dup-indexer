# Dup-Indexer

[![CI build](https://github.com/nyurik/dup-indexer/workflows/CI/badge.svg)](https://github.com/nyurik/dup-indexer/actions)
[![crates.io version](https://img.shields.io/crates/v/dup-indexer.svg)](https://crates.io/crates/dup-indexer)
[![docs.rs docs](https://docs.rs/dup-indexer/badge.svg)](https://docs.rs/dup-indexer)

Create a non-duplicated vector of values without extra memory allocations, even for ref values like `String`, `Vec`, and `Box`. The resulting vector is guaranteed to be in the same order as the original insertion order.

This approach is useful for creating a vector of unique values, such as a list of unique strings, or a list of unique objects, and then using the index of the value in the vector as a unique identifier, e.g. in a protobuf message.

## Example

```rust
use dup_indexer::DupIndexer;

fn main() {
    // Using `String` values
    let mut di = DupIndexer::new();
    assert_eq!(0, di.insert("hello".to_string()));
    assert_eq!(1, di.insert("world".to_string()));
    assert_eq!(0, di.insert("hello".to_string()));
    assert_eq!(di.into_vec(), vec!["hello".to_string(), "world".to_string()]);

    // Using i32 values
    let mut di = DupIndexer::new();
    assert_eq!(0, di.insert(42));
    assert_eq!(1, di.insert(13));
    assert_eq!(0, di.insert(42));
    assert_eq!(di.into_vec(), vec![42, 13]);
}
```

## Implementation
DupIndexer keeps inserted values in a vector in the order of insertion. It also tracks inserted values in a hash map. The hash map is a `HashMap<&T, usize>` where `T` is the type of the inserted value. This means that the inserted values must be `Hash` and `Eq` (i.e. `T: Hash + Eq`).

The value types like ints, floats, bools, chars and any references like `&str` cause no issues because they can be copied to both the vector and the hashmap. However, the non-copyable types with memory allocation like `String` and `Vec` cannot be owned by both at the same time. To solve this, DupIndexer uses a trick: it creates a shallow copy of the value wrapped as a `ManuallyDrop<T>` and inserts it into the hashmap, while the original value is inserted into the vector. This way, the hashmap owns the shallow copy, and the vector owns the original value. The hashmap is used to check for duplicates, and the vector is used to return the final result.
