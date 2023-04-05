use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dup_indexer::DupIndexer;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

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

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("DupIndexer");

    group.bench_function("i32-baseline", |b| {
        b.iter(|| {
            let mut di = DupIndexerRaw::default();
            for _ in 0..100 {
                for val in 0..100 {
                    di.insert(val);
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
                    di.insert(val.to_string());
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
                    di.insert(val);
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
                    di.insert(val);
                }
            }
            black_box(di.into_vec())
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
