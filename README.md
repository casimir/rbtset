# rbtset

[![crates.io](https://meritbadge.herokuapp.com/rbtset)](https://crates.io/crates/rbtset) ![rbtset](https://docs.rs/rbtset/badge.svg)

A set based on a RB-Tree for efficient operations.

## Keys features

* stay sorted
* efficient operations: for `n` items insert, delete and search are `O(log n)`
* partial iteration: iterate from a node reference instead of the full set
* repack: allow to optimize data organization on demand

## Details

See the documentation for more details and examples: https://docs.rs/rbtset