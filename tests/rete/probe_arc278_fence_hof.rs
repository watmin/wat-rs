//! Arc 278 — fence-HOF: the 6a purity fence must accept higher-order fold fns (foldl/map/filter) and
//! `:wat::core::fn` literals. RED at HEAD: `pure?`/`deterministic?` return FALSE for a fold expr because
//! (a) the HOF combinators aren't recognized as pure∧det (foldl is native → classify_fn denies it before
//! intrinsic_meta), and (b) classify_expr has no `:wat::core::fn` lambda arm (a fn-literal is treated as an
//! unknown call head). This BLOCKS custom accumulators (8-custom) — every real fold fn uses both. GREEN when
//! the fence is extended (HOFs pure∧det with their fn-arg recursed; fn-literal classified by its body).
//!
//! Run: cargo test --release -p wat --test probe_arc278_fence_hof

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// Invoke a co-located zero-arg entry (each quotes its own expr under test and hands it to the
/// fence predicate) and return its bool result.
fn classify(fn_name: &str) -> bool {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {other:?}"),
    }
}

/// 1 — a pure fold (foldl + a pure fn-literal) must classify PURE.
#[test]
fn pure_fold_is_pure() {
    assert!(classify(":user::pure-fold-is-pure"), "a pure foldl over a pure fn-literal must be pure");
}

/// 2 — a pure fold must classify DETERMINISTIC.
#[test]
fn pure_fold_is_deterministic() {
    assert!(classify(":user::pure-fold-is-deterministic"), "a pure foldl must be deterministic");
}

/// 3 — `map` with a pure fn-literal must classify PURE (the HOF family, not just foldl).
#[test]
fn pure_map_is_pure() {
    assert!(classify(":user::pure-map-is-pure"), "a pure map over a pure fn-literal must be pure");
}

/// 4 — GUARD: an IMPURE fold (fn body calls println) must STILL be rejected — the fix must NOT
/// blanket-allow HOFs; the impurity of the fn-arg must propagate (conditional purity).
#[test]
fn impure_fold_is_not_pure() {
    assert!(!classify(":user::impure-fold-is-not-pure"), "a foldl whose fn-literal is impure must NOT be pure");
}
