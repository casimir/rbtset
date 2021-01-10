use std::collections::BTreeSet;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::Rng;

use rbtset::RBTreeSet;

fn make_data(size: usize) -> Vec<i64> {
    let mut rng = rand::thread_rng();
    let low = -1 * (size as i64);
    let high = size as i64;
    let mut data = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(rng.gen_range(low..high));
    }
    data
}

const SAMPLE_SIZES: &[usize] = &[10, 100, 500, 1_000];

fn sv_insert(sv: &mut Vec<i64>, data: &[i64]) {
    for v in data {
        sv.push(*v);
        sv.sort();
    }
}

fn sv_contains(sv: &Vec<i64>, values: &[i64]) {
    for value in values {
        assert!(sv.contains(value));
    }
}

fn sv_delete(sv: &mut Vec<i64>, values: &[i64]) {
    for value in values {
        let index = sv.iter().position(|x| x == value).unwrap();
        sv.remove(index);
    }
}

fn bts_insert(bts: &mut BTreeSet<i64>, data: &[i64]) {
    for v in data {
        bts.insert(*v);
    }
}

fn bts_contains(bts: &BTreeSet<i64>, values: &[i64]) {
    for value in values {
        assert!(bts.contains(value));
    }
}

fn bts_delete(bts: &mut BTreeSet<i64>, values: &[i64]) {
    for value in values {
        bts.remove(value);
    }
}

fn rbt_insert(rbt: &mut RBTreeSet<i64>, data: &[i64]) {
    for v in data {
        rbt.insert(*v);
    }
}

fn rbt_contains(rbt: &RBTreeSet<i64>, values: &[i64]) {
    for value in values {
        assert!(rbt.get_node(value).is_some());
    }
}

fn rbt_delete(rbt: &mut RBTreeSet<i64>, values: &[i64]) {
    for value in values {
        rbt.remove(value);
    }
}

fn op_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    for size in SAMPLE_SIZES {
        let data = make_data(*size);
        group.bench_with_input(BenchmarkId::new("sorted vec", size), &data, |b, d| {
            let mut sv = Vec::new();
            b.iter(|| sv_insert(&mut sv, d));
        });
        group.bench_with_input(BenchmarkId::new("btree set", size), &data, |b, d| {
            let mut bts = BTreeSet::new();
            b.iter(|| bts_insert(&mut bts, d));
        });
        group.bench_with_input(BenchmarkId::new("rbtree set", size), &data, |b, d| {
            let mut rbt = RBTreeSet::new();
            b.iter(|| rbt_insert(&mut rbt, &d));
        });
    }
}

fn op_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("contains");
    for size in SAMPLE_SIZES {
        let data = make_data(*size);
        group.bench_with_input(BenchmarkId::new("sorted vec", size), &data, |b, d| {
            let mut sv = Vec::new();
            sv_insert(&mut sv, d);
            b.iter(|| sv_contains(&mut sv, &d[..5]));
        });
        group.bench_with_input(BenchmarkId::new("btree set", size), &data, |b, d| {
            let mut bts = BTreeSet::new();
            bts_insert(&mut bts, d);
            b.iter(|| bts_contains(&mut bts, &d[..5]));
        });
        group.bench_with_input(BenchmarkId::new("rbtree set", size), &data, |b, d| {
            let mut rbt = RBTreeSet::new();
            rbt_insert(&mut rbt, d);
            b.iter(|| rbt_contains(&mut rbt, &d[..5]));
        });
    }
}

fn op_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone");
    for size in SAMPLE_SIZES {
        let data = make_data(*size);
        group.bench_with_input(BenchmarkId::new("sorted vec", size), &data, |b, d| {
            b.iter_batched(
                || {
                    let mut sv = Vec::new();
                    sv_insert(&mut sv, d);
                    sv
                },
                |sv| {
                    let cloned = sv.clone();
                    cloned
                },
                BatchSize::LargeInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("btree set", size), &data, |b, d| {
            b.iter_batched(
                || {
                    let mut bts = BTreeSet::new();
                    bts_insert(&mut bts, d);
                    bts
                },
                |bts| {
                    let cloned = bts.clone();
                    cloned
                },
                BatchSize::LargeInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("rbtree set", size), &data, |b, d| {
            b.iter_batched(
                || {
                    let mut rbt = RBTreeSet::new();
                    rbt_insert(&mut rbt, d);
                    rbt
                },
                |rbt| {
                    let cloned = rbt.clone();
                    cloned
                },
                BatchSize::LargeInput,
            );
        });
    }
}

fn op_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("delete");
    for size in SAMPLE_SIZES {
        let data = make_data(*size);
        group.bench_with_input(BenchmarkId::new("sorted vec", size), &data, |b, d| {
            let mut sv = Vec::new();
            sv_insert(&mut sv, d);
            b.iter_batched_ref(
                || sv.clone(),
                |sv| sv_delete(sv, &d[5..10]),
                BatchSize::LargeInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("btree set", size), &data, |b, d| {
            let mut bts = BTreeSet::new();
            bts_insert(&mut bts, d);
            b.iter_batched_ref(
                || bts.clone(),
                |bts| bts_delete(bts, &d[5..10]),
                BatchSize::LargeInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("rbtree set", size), &data, |b, d| {
            let mut rbt = RBTreeSet::new();
            rbt_insert(&mut rbt, d);
            b.iter_batched_ref(
                || rbt.clone(),
                |rbt| rbt_delete(rbt, &d[5..10]),
                BatchSize::LargeInput,
            );
        });
    }
}

// criterion_group!(benches, op_insert, op_contains, op_clone, op_delete);
criterion_group!(benches, op_insert, op_contains);
criterion_main!(benches);
