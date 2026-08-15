//! Arc 259 S3.2b — `:wat::bracket::map`: the coordinator-fed pool (Ruby's `Parallel.map`).
//!
//! A bounded pool of N runners (N defaults to `cpu-count`) draining a work list,
//! dynamically balanced, results in INPUT ORDER. Built over `spawn-program` + the
//! S3.2a runner-loop: each runner is a `spawn-program` peer; the coordinator feeds
//! `(idx, item)` to whichever runner is free (via `select'`) and collects `(idx,
//! result)`, so order round-trips through the index and the balance is dynamic (a
//! runner that finishes pulls the next item; spares idle when < N items remain).
//! The N peers drop at scope-exit → RAII drain + join. Coordinator touches runners
//! ONLY through the `Peer` (select'/send'/recv') — never a shared queue (the remote
//! axis, bought in advance).
//!
//! RED at HEAD: `:wat::bracket::map` does not exist.
//!
//! Run SERIALLY (spawns threads):
//!   `cargo nextest run --release -E 'binary(kernel)' -F probe_arc259_brackets_map`
//!
//! WAT fixtures: tests/kernel/probe_arc259_brackets_map_{doubles,small}.wat

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

/// Eval a fixture returning `Vector<i64>`; return it as a Rust Vec.
fn run_compute_vec(path: &str) -> Vec<i64> {
    let world = startup_from_file(path).expect("startup should succeed");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {path:?}"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute eval")
    {
        Value::Vec(v) => v
            .iter()
            .map(|tv| match tv {
                Value::i64(n) => *n,
                other => panic!("non-i64 element: {other:?}"),
            })
            .collect(),
        other => panic!("expected Vector; got {other:?}"),
    }
}

/// `brackets/map` over 50 items through a pool, doubling each. Result MUST be
/// `[2, 4, …, 100]` in INPUT ORDER (M=50 > N=cpu-count exercises the dynamic
/// balance: runners pull multiple items; the index round-trip preserves order
/// despite out-of-order completion).
#[test]
fn brackets_map_doubles_in_order() {
    let got = run_compute_vec("tests/kernel/probe_arc259_brackets_map_doubles.wat");
    let expected: Vec<i64> = (1..=50).map(|n| n * 2).collect();
    assert_eq!(got, expected, "brackets/map doubles 1..50 → 2..100, in input order");
}

/// Small sanity: `brackets/map` over [10, 20, 30] adding 1 → [11, 21, 31].
#[test]
fn brackets_map_small_in_order() {
    let got = run_compute_vec("tests/kernel/probe_arc259_brackets_map_small.wat");
    assert_eq!(got, vec![11, 21, 31], "brackets/map [10,20,30] +1 → [11,21,31] in order");
}
