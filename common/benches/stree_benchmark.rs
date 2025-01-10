use criterion::{criterion_group, criterion_main, Criterion};
use common::stree::SPlusTree;
use rand::Rng;

fn bench_splus_tree(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let size = 100_000;
    let mut vals: Vec<u32> = (0..size).map(|_| rng.gen()).collect();
    vals.sort_unstable();

    // Build S+ tree
    const B: usize = 16;
    const N: usize = 16;
    let stree = SPlusTree::<B, N>::new_left_tree(&vals);

    // Prepare queries
    let queries: Vec<u32> = (0..1024).map(|_| rng.gen()).collect();

    // Benchmark sequential queries
    c.bench_function("splus_tree_query_one", |b| {
        b.iter(|| {
            for &q in &queries {
                criterion::black_box(stree.query_one(q));
            }
        })
    });

    // Benchmark batched queries with different batch sizes
    for &batch_size in &[8, 16, 32] {
        let name = format!("splus_tree_query_batch_{}", batch_size);
        c.bench_function(&name, |b| {
            b.iter(|| {
                let mut idx = 0;
                while idx + batch_size <= queries.len() {
                    let chunk = &queries[idx..idx + batch_size];
                    match batch_size {
                        8 => {
                            let arr: [u32; 8] = chunk.try_into().unwrap();
                            criterion::black_box(stree.query_batch(&arr));
                        }
                        16 => {
                            let arr: [u32; 16] = chunk.try_into().unwrap();
                            criterion::black_box(stree.query_batch(&arr));
                        }
                        32 => {
                            let arr: [u32; 32] = chunk.try_into().unwrap();
                            criterion::black_box(stree.query_batch(&arr));
                        }
                        _ => unreachable!(),
                    }
                    idx += batch_size;
                }
            })
        });
    }

    // Benchmark cache effects
    let mut clustered_queries = Vec::with_capacity(1024);
    for base in (0..1024).step_by(32) {
        for i in 0..32 {
            clustered_queries.push(base as u32 + i as u32);
        }
    }

    c.bench_function("splus_tree_clustered_queries", |b| {
        b.iter(|| {
            for chunk in clustered_queries.chunks(32) {
                let arr: [u32; 32] = chunk.try_into().unwrap();
                criterion::black_box(stree.query_batch(&arr));
            }
        })
    });

    let scattered_queries: Vec<u32> = (0..1024).map(|_| rng.gen()).collect();
    c.bench_function("splus_tree_scattered_queries", |b| {
        b.iter(|| {
            for chunk in scattered_queries.chunks(32) {
                let arr: [u32; 32] = chunk.try_into().unwrap();
                criterion::black_box(stree.query_batch(&arr));
            }
        })
    });
}

criterion_group!(benches, bench_splus_tree);
criterion_main!(benches);
