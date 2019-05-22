use std::collections::{BTreeMap, BTreeSet};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, ParameterizedBenchmark};
use lazy_static::lazy_static;
use rand::Rng;

use rbtset::RBTreeSet;

fn make_data(size: usize) -> Vec<i64> {
    let mut rng = rand::thread_rng();
    let low = -1 * (size as i64);
    let high = size as i64;
    let mut data = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(rng.gen_range(low, high));
    }
    data
}

lazy_static! {
    static ref DATAS: BTreeMap<usize, Vec<i64>> = {
        let mut datas = BTreeMap::new();
        for i in &[10, 100, 500, 1_000] {
            datas.insert(*i, make_data(*i));
        }
        datas
    };
}

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

fn loads_of_values(c: &mut Criterion) {
    c.bench(
        "insert",
        ParameterizedBenchmark::new(
            "sorted vec",
            |b, s| {
                let mut sv = Vec::new();
                b.iter(|| sv_insert(&mut sv, &DATAS[s]));
            },
            DATAS.keys().map(|k| *k).collect::<Vec<usize>>(),
        )
        .with_function("btree set", |b, s| {
            let mut bts = BTreeSet::new();
            b.iter(|| bts_insert(&mut bts, &DATAS[s]));
        })
        .with_function("rbtree", |b, s| {
            let mut rbt = RBTreeSet::new();
            b.iter(|| rbt_insert(&mut rbt, &DATAS[s]));
        }),
    );
    c.bench(
        "contains",
        ParameterizedBenchmark::new(
            "sorted vec",
            |b, s| {
                let mut sv = Vec::new();
                sv_insert(&mut sv, &DATAS[s]);
                b.iter(|| sv_contains(&sv, &DATAS[s][..5]));
            },
            DATAS.keys().map(|k| *k).collect::<Vec<usize>>(),
        )
        .with_function("btree set", |b, s| {
            let mut bts = BTreeSet::new();
            bts_insert(&mut bts, &DATAS[s]);
            b.iter(|| bts_contains(&bts, &DATAS[s][..5]));
        })
        .with_function("rbtree", |b, s| {
            let mut rbt = RBTreeSet::new();
            rbt_insert(&mut rbt, &DATAS[s]);
            b.iter(|| rbt_contains(&rbt, &DATAS[s][..5]));
        }),
    );
    c.bench(
        "clone",
        ParameterizedBenchmark::new(
            "sorted vec",
            |b, s| {
                let mut sv = Vec::new();
                sv_insert(&mut sv, &DATAS[s]);
                b.iter(|| sv.clone());
            },
            DATAS.keys().map(|k| *k).collect::<Vec<usize>>(),
        )
        .with_function("btree set", |b, s| {
            let mut bts = BTreeSet::new();
            bts_insert(&mut bts, &DATAS[s]);
            b.iter(|| bts.clone());
        })
        .with_function("rbtree", |b, s| {
            let mut rbt = RBTreeSet::new();
            rbt_insert(&mut rbt, &DATAS[s]);
            b.iter(|| rbt.clone());
        }),
    );
    c.bench(
        "delete",
        ParameterizedBenchmark::new(
            "sorted vec",
            |b, s| {
                let mut sv = Vec::new();
                sv_insert(&mut sv, &DATAS[s]);
                b.iter_batched_ref(
                    || sv.clone(),
                    |sv| sv_delete(sv, &DATAS[s][5..10]),
                    BatchSize::SmallInput,
                );
            },
            DATAS.keys().map(|k| *k).collect::<Vec<usize>>(),
        )
        .with_function("btree set", |b, s| {
            let mut bts = BTreeSet::new();
            bts_insert(&mut bts, &DATAS[s]);
            b.iter_batched_ref(
                || bts.clone(),
                |bts| bts_delete(bts, &DATAS[s][5..10]),
                BatchSize::SmallInput,
            );
        })
        .with_function("rbtree", |b, s| {
            let mut rbt = RBTreeSet::new();
            rbt_insert(&mut rbt, &DATAS[s]);
            b.iter_batched_ref(
                || rbt.clone(),
                |rbt| rbt_delete(rbt, &DATAS[s][5..10]),
                BatchSize::SmallInput,
            );
        }),
    );
}

criterion_group!(benches, loads_of_values);
criterion_main!(benches);
