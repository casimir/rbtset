# rbtset - A set based on a RB-Tree for efficient operations.

![rbtset](https://docs.rs/rbtset/badge.svg)

## Keys features

* stay sorted
* efficient operations: for `n` items insert, delete and search are `O(log n)`
* partial iteration: iterate from a node reference instead of the full set
* repack: allow to optimize data organization on demand

See the documentation for more details and examples: https://docs.rs/rbtset