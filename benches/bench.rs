use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dup_indexer::DupIndexer;

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("DupIndexer");

    // group.bench_function("String", |b| {
    //     b.iter(|| {
    //         let mut di = DupIndexer::new();
    //         for _ in 0..100 {
    //             for val in 0..100 {
    //                 di.insert(val.to_string());
    //             }
    //         }
    //         black_box(di.into_vec())
    //     })
    // });

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
