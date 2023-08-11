use std::borrow::Borrow;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dup_indexer::{DupIndexer, PtrRead, ShallowCopy};

#[derive(Default)]
pub struct DupIndexerRaw {
    values: Vec<i32>,
    lookup: HashMap<i32, usize>,
}

impl DupIndexerRaw {
    pub fn insert(&mut self, value: i32) -> usize {
        match self.lookup.entry(value) {
            Occupied(entry) => *entry.get(),
            Vacant(entry) => {
                let index = self.values.len();
                entry.insert(index);
                self.values.push(value);
                index
            }
        }
    }

    pub fn into_vec(self) -> Vec<i32> {
        self.values
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
enum EnumValue {
    Str(String),
    Int(i32),
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum EnumValueRef<'a> {
    Str(&'a str),
    Int(i32),
}

impl ToOwned for dyn Key + '_ {
    type Owned = EnumValue;

    fn to_owned(&self) -> Self::Owned {
        match self.to_key() {
            EnumValueRef::Str(s) => EnumValue::Str(s.to_owned()),
            EnumValueRef::Int(i) => EnumValue::Int(i),
        }
    }
}

unsafe impl PtrRead for EnumValue {}

trait Key {
    fn to_key(&self) -> EnumValueRef;
}

impl EnumValue {
    fn as_ref(&self) -> EnumValueRef {
        match self {
            EnumValue::Str(v) => EnumValueRef::Str(v),
            EnumValue::Int(v) => EnumValueRef::Int(*v),
        }
    }
}

impl Hash for dyn Key + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_key().hash(state);
    }
}

impl PartialEq for dyn Key + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.to_key() == other.to_key()
    }
}

impl Eq for dyn Key + '_ {}

impl Key for EnumValue {
    fn to_key(&self) -> EnumValueRef {
        self.as_ref()
    }
}

impl<'a> Borrow<dyn Key + 'a> for ShallowCopy<EnumValue> {
    fn borrow(&self) -> &(dyn Key + 'a) {
        self.deref().borrow()
    }
}

impl<'a> Borrow<dyn Key + 'a> for EnumValue {
    fn borrow(&self) -> &(dyn Key + 'a) {
        self
    }
}

impl<'a> Key for &'a str {
    fn to_key(&self) -> EnumValueRef {
        EnumValueRef::Str(self)
    }
}

impl<'a> Borrow<dyn Key + 'a> for &'a str {
    fn borrow(&self) -> &(dyn Key + 'a) {
        self
    }
}

impl Key for i32 {
    fn to_key(&self) -> EnumValueRef {
        EnumValueRef::Int(*self)
    }
}

impl<'a> Borrow<dyn Key + 'a> for i32 {
    fn borrow(&self) -> &(dyn Key + 'a) {
        self
    }
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("DupIndexer");

    group.bench_function("i32-baseline", |b| {
        b.iter(|| {
            let mut di = DupIndexerRaw::default();
            for _ in 0..100 {
                for val in 0..100 {
                    black_box(di.insert(val));
                }
            }
            black_box(di.into_vec())
        })
    });

    group.bench_function("String", |b| {
        let values = (0..100)
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        b.iter(|| {
            let mut di = DupIndexer::<String>::new();
            for _ in 0..100 {
                for val in &values {
                    black_box(di.insert(val.to_string()));
                }
            }
            black_box(di.into_vec())
        })
    });

    group.bench_function("String-cloneable", |b| {
        let values = (0..100)
            .into_iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>();
        b.iter(|| {
            let mut di = DupIndexer::<String>::new();
            for _ in 0..100 {
                for val in &values {
                    black_box(di.insert_ref(val.as_str()));
                }
            }
            black_box(di.into_vec())
        })
    });

    group.bench_function("EnumValue-String", |b| {
        let values = (0..100)
            .into_iter()
            .map(|i| EnumValue::Str(i.to_string()))
            .collect::<Vec<_>>();
        b.iter(|| {
            let mut di = DupIndexer::<EnumValue>::new();
            for _ in 0..100 {
                for val in &values {
                    black_box(di.insert(val.clone()));
                }
            }
            black_box(di.into_vec())
        })
    });

    group.bench_function("EnumValue-String-clone", |b| {
        let values = (0..100)
            .into_iter()
            .map(|i| EnumValue::Str(i.to_string()))
            .collect::<Vec<_>>();
        b.iter(|| {
            let mut di = DupIndexer::<EnumValue>::new();
            for _ in 0..100 {
                for val in &values {
                    black_box(di.insert_ref(val as &dyn Key));
                }
            }
            black_box(di.into_vec())
        })
    });

    group.bench_function("i32", |b| {
        b.iter(|| {
            let mut di = DupIndexer::new();
            for _ in 0..100 {
                for val in 0..100 {
                    black_box(di.insert(val));
                }
            }
            black_box(di.into_vec())
        })
    });

    group.bench_function("u8", |b| {
        b.iter(|| {
            let mut di = DupIndexer::new();
            for _ in 0..100 {
                for val in 0_u8..100 {
                    black_box(di.insert(val));
                }
            }
            black_box(di.into_vec())
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
