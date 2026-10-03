# Persistent Vector and List

## The ruling and the why

The builder (2026-10-02): "swapping vec and list to their persistent flavors is the
call here"; "long term we only use rpds". In wat-rs, `src/collection/eval.rs:280`
`vector_conj_inner` does `(**xs).clone(); out.push(item)` — every element cloned on
every `conj`, so building N elements is N²/2 clones. `src/value/value.rs:340` `List`
is `Arc<std::collections::LinkedList<Value>>`, and its `conj` (prepend) clones every
node.

## The contract — unobservable to wat programs

Same values, printing, equality, hashing, type names. Only the cost of `conj` changes.

## The rows

- **V — `Vector` becomes `PVec`.** `Value::Vec(Arc<Vec<Value>>)` →
  `Value::Vec(crate::value::pvec::PVec)`, the promoting vector `PersistentVector`
  already uses (`src/value/pvec.rs`: an array from a bulk build, an rpds RRB tree
  after persistent `conj` past 8; equality and hash by sequence). `conj` →
  `PVec::push_back`. ~238 `Value::Vec` sites in ~43 files (measured: 307 occurrences /
  61 files, grep-counted before the swap). List every site that needed a contiguous
  slice and what was done.
- **L — `List` becomes `rpds::ListSync`.** O(1) `push_front`; equality, hashing,
  printing by sequence as today. ~54 sites in ~16 files (measured: 48 occurrences /
  15 files).

## Tests

A wat-rs TEST may change only where it constructs or matches `Value::Vec` /
`Value::wat__core__List` in RUST (the Rust type changed). A change to what a test
EXPECTS a wat program to produce is a STOP: the swap would be observable.

## Gates

1. Baseline first, on the unmodified branch: `cargo build --release`, then the floor
   `NEXTEST_TEST_THREADS=4 cargo nextest run --release`.
2. A benchmark in `wat-scripts/bench/conj-build.wat`: build a Vector of N elements by
   `conj` and a List of N by `conj`, at N = 10,000 / 100,000 / 1,000,000, print a
   checksum. Time with the baseline binary and with the new one, pinned with
   `taskset -c 2`, three runs each.
3. After V, and again after L: the floor with the same result as the baseline, and
   the benchmark.
