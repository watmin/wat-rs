//! Arc 278 — fence-HOF: the 6a purity fence must accept higher-order fold fns (foldl/map/filter/foldr) and
//! `:wat::core::fn` literals. RED at HEAD: `pure?`/`deterministic?` return FALSE for a fold expr because
//! (a) the HOF combinators aren't recognized as pure∧det (foldl is native → classify_fn denies it before
//! intrinsic_meta), and (b) classify_expr has no `:wat::core::fn` lambda arm (a fn-literal is treated as an
//! unknown call head). This BLOCKS custom accumulators (8-custom) — every real fold fn uses both. GREEN when
//! the fence is extended (HOFs pure∧det with their fn-arg recursed; fn-literal classified by its body).
//!
//! Run: cargo test --release -p wat --test probe_arc278_fence_hof

use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};

/// Eval `(:wat::rete::<pred> (:wat::core::quote <expr>))` → bool.
fn classify(pred: &str, expr: &str) -> bool {
    let run = format!("(:wat::rete::{pred} (:wat::core::quote {expr}))");
    let w = startup_bare().expect("startup");
    let ast = wat::parse_one!(&run).expect("parse");
    match eval_in_frozen(&ast, &w, &Environment::new()).expect("eval").value_owned() {
        Value::bool(b) => b,
        other => panic!("expected bool; got {other:?}"),
    }
}

const PURE_FOLD: &str =
    "(:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64 \
     (:wat::core::i64::+ acc (:wat::core::i64::* x x))) 0 xs)";

const PURE_MAP: &str =
    "(:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x x)) xs)";

const IMPURE_FOLD: &str =
    "(:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64 \
     (:wat::core::do (:wat::kernel::println \"side effect\") acc)) 0 xs)";

/// 1 — a pure fold (foldl + a pure fn-literal) must classify PURE.
#[test]
fn pure_fold_is_pure() {
    assert!(classify("pure?", PURE_FOLD), "a pure foldl over a pure fn-literal must be pure");
}

/// 2 — a pure fold must classify DETERMINISTIC.
#[test]
fn pure_fold_is_deterministic() {
    assert!(classify("deterministic?", PURE_FOLD), "a pure foldl must be deterministic");
}

/// 3 — `map` with a pure fn-literal must classify PURE (the HOF family, not just foldl).
#[test]
fn pure_map_is_pure() {
    assert!(classify("pure?", PURE_MAP), "a pure map over a pure fn-literal must be pure");
}

/// 4 — GUARD: an IMPURE fold (fn body calls println) must STILL be rejected — the fix must NOT
/// blanket-allow HOFs; the impurity of the fn-arg must propagate (conditional purity).
#[test]
fn impure_fold_is_not_pure() {
    assert!(!classify("pure?", IMPURE_FOLD), "a foldl whose fn-literal is impure must NOT be pure");
}
