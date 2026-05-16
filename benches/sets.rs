use std::collections::BTreeSet;
use std::ops::Range;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::Rng;

use rbtset::{Consecutive, RBTreeSet};

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

const SAMPLE_SIZES: &[usize] = &[5, 10, 100, 500, 1_000];

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
        if let Some(index) = sv.iter().position(|x| x == value) {
            sv.remove(index);
        }
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
        if *size < 500 {
            group.bench_with_input(BenchmarkId::new("sorted vec", size), &data, |b, d| {
                b.iter_batched(
                    Vec::new,
                    |mut sv| sv_insert(&mut sv, d),
                    BatchSize::SmallInput,
                );
            });
        }
        group.bench_with_input(BenchmarkId::new("btree set", size), &data, |b, d| {
            b.iter_batched(
                BTreeSet::new,
                |mut bts| bts_insert(&mut bts, d),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("rbtree set", size), &data, |b, d| {
            b.iter_batched(
                RBTreeSet::new,
                |mut rbt| rbt_insert(&mut rbt, d),
                BatchSize::SmallInput,
            );
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
                |sv| sv.clone(),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("btree set", size), &data, |b, d| {
            b.iter_batched(
                || {
                    let mut bts = BTreeSet::new();
                    bts_insert(&mut bts, d);
                    bts
                },
                |bts| bts.clone(),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("rbtree set", size), &data, |b, d| {
            b.iter_batched(
                || {
                    let mut rbt = RBTreeSet::new();
                    rbt_insert(&mut rbt, d);
                    rbt
                },
                |rbt| rbt.clone(),
                BatchSize::SmallInput,
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
                |sv| sv_delete(sv, &d[1..5]),
                BatchSize::LargeInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("btree set", size), &data, |b, d| {
            let mut bts = BTreeSet::new();
            bts_insert(&mut bts, d);
            b.iter_batched_ref(
                || bts.clone(),
                |bts| bts_delete(bts, &d[1..5]),
                BatchSize::LargeInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("rbtree set", size), &data, |b, d| {
            let mut rbt = RBTreeSet::new();
            rbt_insert(&mut rbt, d);
            b.iter_batched_ref(
                || rbt.clone(),
                |rbt| rbt_delete(rbt, &d[1..5]),
                BatchSize::LargeInput,
            );
        });
    }
}

// Seq wraps a Range so consecutive ranges can be stored and merged via repack.
#[derive(Debug, Clone, Eq)]
struct Seq(Range<i64>);

impl PartialEq for Seq {
    fn eq(&self, other: &Seq) -> bool {
        other.0.start <= self.0.start && self.0.start < other.0.end
    }
}

impl Ord for Seq {
    fn cmp(&self, other: &Seq) -> std::cmp::Ordering {
        self.0.start.cmp(&other.0.start)
    }
}

impl PartialOrd for Seq {
    fn partial_cmp(&self, other: &Seq) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Consecutive for Seq {
    fn consecutive(&self, other: &Seq) -> bool {
        self.0.end == other.0.start
    }

    fn merged(&self, other: &Seq) -> Seq {
        Seq(self.0.start..other.0.end)
    }
}

fn make_seq_data(size: usize) -> Vec<Seq> {
    // Build a mix of consecutive and non-consecutive unit ranges so repack has real work.
    let mut rng = rand::thread_rng();
    let mut starts: Vec<i64> = (0..size as i64).collect();
    // Shuffle to randomise insertion order; repack merges them into longer runs.
    use rand::seq::SliceRandom;
    starts.shuffle(&mut rng);
    starts.iter().map(|&s| Seq(s..s + 1)).collect()
}

fn op_repack(c: &mut Criterion) {
    let mut group = c.benchmark_group("repack");
    for size in SAMPLE_SIZES {
        let data = make_seq_data(*size);
        group.bench_with_input(BenchmarkId::new("rbtree set", size), &data, |b, d| {
            b.iter_batched(
                || {
                    let mut rbt = RBTreeSet::new();
                    for seq in d {
                        rbt.insert(seq.clone());
                    }
                    rbt
                },
                |mut rbt| rbt.repack(),
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(benches, op_insert, op_contains, op_clone, op_delete, op_repack);
criterion_main!(benches);
