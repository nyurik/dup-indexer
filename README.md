# Dup-Indexer

[![GitHub](https://img.shields.io/badge/github-nyurik/dup--indexer-8da0cb?logo=github)](https://github.com/nyurik/dup-indexer)
[![crates.io version](https://img.shields.io/crates/v/dup-indexer.svg)](https://crates.io/crates/dup-indexer)
[![docs.rs docs](https://docs.rs/dup-indexer/badge.svg)](https://docs.rs/dup-indexer)
[![crates.io version](https://img.shields.io/crates/l/dup-indexer.svg)](https://github.com/nyurik/dup-indexer/blob/main/LICENSE-APACHE)
[![CI build](https://github.com/nyurik/dup-indexer/workflows/CI/badge.svg)](https://github.com/nyurik/dup-indexer/actions)

Create a non-duplicated vector of values without extra memory allocations, even for ref values like `String` and `Vec`. Each insertion returns the `usize` index of the inserted value. When done, the entire vector can be used.

This approach is useful for creating a vector of unique values, such as a list of unique strings, or a list of unique objects, and then using the index of the value in the vector as a unique identifier, e.g. in a protobuf message.

## Example

```rust
use dup_indexer::DupIndexer;

fn main() {
    // with strings
    let mut di = DupIndexer::new();
    assert_eq!(di.insert("hello".to_string()), 0);
    assert_eq!(di.insert("world".to_string()), 1);
    assert_eq!(di.insert("hello".to_string()), 0);
    assert_eq!(di.into_vec(), vec!["hello", "world"]);

    // with i32
    let mut di = DupIndexer::with_capacity(10);
    assert_eq!(di.insert(42), 0);
    assert_eq!(di.insert(13), 1);
    assert_eq!(di.insert(42), 0);
    assert_eq!(di[1], 13); // use it as a read-only vector
    assert_eq!(di.into_iter().collect::<Vec<_>>(), vec![42, 13]);

    //
    // with custom enum
    //
    #[derive(Debug, Eq, PartialEq, Hash, Clone)]
    enum Value {
        Str(String),
        Int(i32),
    }
    
    // All values inside the Value enum implement PtrRead
    unsafe impl dup_indexer::PtrRead for Value {}
    
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
```

## Implementation
DupIndexer keeps inserted values in a vector in the order of insertion. It also tracks inserted values in a lookup `HashMap<T, usize>` where `T` is the type of the inserted value. This means that the inserted values must implement `Hash` and `Eq`.

The value types like ints, floats, bools, chars and any references like `&str` cause no issues because they can be copied to both the vector and the lookup map containers. However, the non-copyable types with memory allocation like `String` and `Vec` cannot be owned by both containers at the same time. To solve this, DupIndexer creates a shallow non-droppable copy of the value, and stores it in the hashmap, whereas the original value goes into the vector:

```rust
use std::collections::hash_map::{Entry, HashMap};
use std::mem::ManuallyDrop;
use std::hash::Hash;

pub unsafe trait PtrRead {}

pub struct DupIndexer<T> {
  values: Vec<T>,
  lookup: HashMap<ManuallyDrop<T>, usize>,
}

impl<T: Hash + Eq + PtrRead> DupIndexer<T> {
  pub fn insert(&mut self, value: T) -> usize {
    let dup_value = ManuallyDrop::new(unsafe { std::ptr::read(&value) });
    match self.lookup.entry(dup_value) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let index = self.values.len();
        entry.insert(index);
        self.values.push(value);
        index
      }
    }
  }
}
```

This way, the hashmap owns the shallow copy, and the vector owns the original value. On subsequent calls, the new value is checked against the hashmap for duplicates. Once finished, the vector with the keys is consumed by the user with `.into_vec()`, and the hashmap is dropped without dropping the actual keys.

### Safety
I believe the above code is safe because the hashmap only keeps the `ptr:read`-created copy of the original value while we own it, and the value is never modified.  Some types like `Box` might be a bit trickier, so for safety /  to keep Miri happy this lib has an unsafe `PtrRead` marker trait that most basic types implement.

Miri passes all tests for which there is a PtrRead, but fails if `T` is a `Box<i32>`. When the test tries to insert a duplicate value, I see the following Miri warning. Do let me know if you know if this is really an issue and how to fix this.

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

## Development
* This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`. Install it with `cargo install just`.
* To get a list of available commands, run `just`.
* To run tests, use `just test`.
* On `git push`, it will run a few validations, including `cargo fmt`, `cargo clippy`, and `cargo test`.  Use `git push --no-verify` to skip these checks.
* To run benchmarks, use `just bench`.
* To test with Miri, use `just miri` (note that one of the tests is disabled due to the above issue).

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
