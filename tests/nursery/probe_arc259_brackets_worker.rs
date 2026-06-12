//! Arc 259 S3.4 — `:wat::bracket::map-worker` / `each-worker`: per-runner state via closures.
//!
//! The general pool engine: each runner i is built from `(worker-init i)` — a function
//! `i64 -> (I -> O)` whose OUTER call is the per-runner setup (run once, when the runner
//! is built — the place to allocate a resource reused across that runner's items) and
//! whose INNER result is the per-item work-fn. `worker-id` is the runner index, delivered
//! as the argument to `worker-init`. Per the four-questions decision (DESIGN-STONE-259.S3.4):
//! closures hold per-runner state — no ambient `bracket::Env`, no new substrate.
//!
//! `brackets/map` becomes a thin wrapper over `map-worker` (a constant `worker-init` that
//! ignores the id); `brackets/each` over `each-worker` the same way. So this stone also
//! re-expresses the shipped map/each — their probes (probe_arc259_brackets_map /
//! _each) must stay green, proving the wrappers preserve behavior.
//!
//! RED at HEAD: `:wat::bracket::map-worker` / `each-worker` do not exist.
//!
//! Run SERIALLY (spawns threads):
//!   `cargo test --release -p wat --test nursery probe_arc259_brackets_worker -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute_vec(body: &str) -> Vec<i64> {
    let src = format!("{body}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
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

fn run_compute_nil(body: &str) {
    let src = format!("{body}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned();
    assert_eq!(got, Value::Unit, "each-worker returns nil; got {got:?}");
}

/// `map-worker` with a `worker-init` that IGNORES the worker-id and doubles each item.
/// Result must equal plain `map`'s — `[2,4,…,100]` in input order — proving the general
/// engine is correct and the worker-init plumbing doesn't disturb the map semantics.
#[test]
fn map_worker_doubles_in_order_ignoring_worker_id() {
    let got = run_compute_vec(
        "(:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::core::i64> \
           (:wat::bracket::map-worker (:wat::spawn::thread) \
             (:wat::core::map (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ i 1)) \
                              (:wat::core::range 0 50)) \
             (:wat::core::fn [_wid <- :wat::core::i64] -> :wat::core::Fn(wat::core::i64)->wat::core::i64 \
               (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))))",
    );
    let expected: Vec<i64> = (1..=50).map(|n| n * 2).collect();
    assert_eq!(got, expected, "map-worker doubles 1..50 in input order; worker-id ignored");
}

/// `map-worker` with a `worker-init` whose per-item work-fn returns the WORKER-ID (ignoring
/// the item). Every primed runner produces at least its first result, so the distinct set of
/// worker-ids returned must be EXACTLY the runner indices `{0 .. N-1}`, N = min(cpu-count, 50).
/// This proves: worker-id is the runner index, delivered to the work-fn; every runner ran;
/// all 50 items were processed.
#[test]
fn map_worker_delivers_worker_id_as_runner_index() {
    let expected_n = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(50);
    let got = run_compute_vec(
        "(:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::core::i64> \
           (:wat::bracket::map-worker (:wat::spawn::thread) \
             (:wat::core::range 0 50) \
             (:wat::core::fn [wid <- :wat::core::i64] -> :wat::core::Fn(wat::core::i64)->wat::core::i64 \
               (:wat::core::fn [_item <- :wat::core::i64] -> :wat::core::i64 wid))))",
    );
    assert_eq!(got.len(), 50, "all 50 items processed");
    assert!(
        got.iter().all(|&w| w >= 0 && w < expected_n as i64),
        "every worker-id is a valid runner index in [0, {expected_n}); got {got:?}"
    );
    let distinct: std::collections::BTreeSet<i64> = got.iter().copied().collect();
    let expected: std::collections::BTreeSet<i64> = (0..expected_n as i64).collect();
    assert_eq!(
        distinct, expected,
        "worker-ids are EXACTLY the runner indices 0..{expected_n} (every primed runner produced a result)"
    );
}

/// `each-worker` over 50 items returns nil and drains the pool (completion proves drainage,
/// like the `each` probe), with the per-runner worker-init plumbing in place.
#[test]
fn each_worker_drains_and_returns_nil() {
    run_compute_nil(
        "(:wat::core::defn :user::compute [] -> :wat::core::nil \
           (:wat::bracket::each-worker (:wat::spawn::thread) \
             (:wat::core::range 0 50) \
             (:wat::core::fn [_wid <- :wat::core::i64] -> :wat::core::Fn(wat::core::i64)->wat::core::i64 \
               (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))))",
    );
}
