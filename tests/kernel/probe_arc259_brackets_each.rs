//! Arc 259 S3.3 — `:wat::bracket::each`: the side-effect pool (Ruby's `Parallel.each`).
//!
//! `each` is `map` that DISCARDS: it runs `work-fn` over every item through the same
//! bounded, dynamically-balanced pool, then returns `nil` (the results are dropped).
//! Built as a thin wrapper: `(do (brackets/map host items work-fn) nil)` — `map`
//! already blocks until all M results arrive (its collect-loop returns only when
//! `collected == M`), so by the time `map` returns, every work-fn has run. `each`
//! discards the Vector and returns nil.
//!
//! The delta this probe pins (over the already-proven `map`): `each` returns nil AND
//! drains every item. Completion IS the drainage proof — a single-shot or
//! under-draining pool would block forever on the M-th `select'` and HANG this test;
//! a non-hanging nil return proves all M items were processed.
//!
//! RED at HEAD: `:wat::bracket::each` does not exist (UnknownFunction).
//!
//! Run SERIALLY (spawns threads):
//!   `cargo nextest run --release -E 'binary(kernel)' -F probe_arc259_brackets_each`
//!
//! WAT fixtures: tests/kernel/probe_arc259_brackets_each_{50_items,small}.wat

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

/// Eval a fixture whose `compute` returns nil; assert the result is `Value::Unit`.
fn run_compute_nil(path: &str) {
    let world = startup_from_file(path).expect("startup should succeed");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {path:?}"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute eval");
    assert_eq!(got, Value::Unit, "brackets/each returns nil; got {got:?}");
}

/// `brackets/each` over 50 items returns nil — and, by completing at all (no hang),
/// proves the pool drained all 50 (the collect-loop only returns at collected==M).
#[test]
fn brackets_each_drains_50_and_returns_nil() {
    run_compute_nil("tests/kernel/probe_arc259_brackets_each_50_items.wat");
}

/// Small case: 3 items, returns nil.
#[test]
fn brackets_each_small_returns_nil() {
    run_compute_nil("tests/kernel/probe_arc259_brackets_each_small.wat");
}
