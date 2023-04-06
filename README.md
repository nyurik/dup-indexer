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
    assert_eq!(di.into_iter().collect::<Vec<_>>(), vec![42, 13]);
}
```

## Implementation
DupIndexer keeps inserted values in a vector in the order of insertion. It also tracks inserted values in a lookup `HashMap<T, usize>` where `T` is the type of the inserted value. This means that the inserted values must implement `Hash` and `Eq`.

The value types like ints, floats, bools, chars and any references like `&str` cause no issues because they can be copied to both the vector and the lookup map containers. However, the non-copyable types with memory allocation like `String` and `Vec` cannot be owned by both containers at the same time. To solve this, DupIndexer creates a shallow non-droppable copy of the value, and stores it in the hashmap, whereas the original value goes into the vector:

```rust,ignore
pub struct DupIndexer<T> {
    values: Vec<T>,
    lookup: HashMap<ManuallyDrop<T>, usize>,
}

pub fn insert(&mut self, value: T) -> usize {
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
```

This way, the hashmap owns the shallow copy, and the vector owns the original value. On subsequent calls, the new value is checked against the hashmap for duplicates. Once finished, the vector with the keys is consumed by the user with `.into_vec()`, and the hashmap is dropped without dropping the actual keys.

### Safety
I believe the above code is safe because the hashmap only keeps the `ptr:read`-created copy of the original value while we own it, and the value is never modified.  Miri **mostly** agrees with this, passing all tests except the one where `T` is a `Box<i32>`. When the test tries to insert a duplicate value, I see the following Miri warning. Do let me know if you know if this is really an issue and how to fix this.

```text
❯ cargo +nightly miri test

    --> .../.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:1328:23
     |
1328 |         PartialEq::eq(&**self, &**other)
     |                       ^^^^^^^
     |                       |
     |                       trying to retag from <170684> for SharedReadOnly permission at alloc64636[0x0], but that tag does not exist in the borrow stack for this location
     |                       this error occurs as part of retag at alloc64636[0x0..0x4]
     |
     = help: this indicates a potential bug in the program: it performed an invalid operation, but the Stacked Borrows rules it violated are still experimental
     = help: see https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md for further information
help: <170684> was created by a Unique retag at offsets [0x0..0x4]
    --> src/lib.rs:49:17
     |
49   |                 entry.insert(index);
     |                 ^^^^^^^^^^^^^^^^^^^
help: <170684> was later invalidated at offsets [0x0..0x4] by a Unique retag
    --> src/lib.rs:50:34
     |
50   |                 self.values.push(value);
     |                                  ^^^^^
```
