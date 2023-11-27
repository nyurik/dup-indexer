use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dup_indexer::{DupIndexer, DupIndexerRefs};

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

fn dup_indexer(c: &mut Criterion) {
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
        b.iter(|| {
            let mut di = DupIndexer::new();
            for _ in 0..100 {
                for val in 0..100 {
                    black_box(di.insert(val.to_string()));
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

fn dup_gen_indexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("DupGenIndexer");

    group.bench_function("String", |b| {
        b.iter(|| {
            let mut di = DupIndexerRefs::new();
            for _ in 0..100 {
                for val in 0..100 {
                    black_box(di.insert_owned(val.to_string()));
                }
            }
            black_box(di.into_vec())
        })
    });

    group.bench_function("str", |b| {
        let values: Vec<String> = (0..100).map(|i| i.to_string()).collect();
        b.iter(|| {
            let mut di: DupIndexerRefs<String> = DupIndexerRefs::new();
            for _ in 0..100 {
                for val in &values {
                    black_box(di.insert_ref(val));
                }
            }
            black_box(di.into_vec())
        })
    });
}

criterion_group!(benches, dup_indexer, dup_gen_indexer);
criterion_main!(benches);
