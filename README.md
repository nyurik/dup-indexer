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
    let mut di = DupIndexer::new();
    assert_eq!(0, di.insert_string("hello".to_string()));
    assert_eq!(1, di.insert_string("world".to_string()));
    assert_eq!(0, di.insert_string("hello".to_string())); // try inserting "hello" again
    assert_eq!(di.into_vec(), vec!["hello".to_string(), "world".to_string()]);
}
```
