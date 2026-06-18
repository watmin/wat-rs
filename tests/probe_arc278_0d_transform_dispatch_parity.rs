//! Arc 278 stone 0d — disconfirming probe: transform-op CHECK-SIDE parity. RED at HEAD.
//!
//! 0c gave PersistentVector the transform/sequence ops at RUNTIME (the `eval_vec_*` arms dispatch on
//! PersistentVector). But the CHECKER never followed: `map`/`filter`/`foldl`/`foldr`/`reverse`/`take`/`drop`
//! are still monomorphic `Vector`-only static TypeSchemes (check.rs:17963-18073), `concat` checks via the
//! `Vector/concat` alias. So a TYPED body that folds/maps a PersistentVector is rejected at check time.
//!
//! This probe exercises the CHECKER (via `startup_from_source`, which type-checks at freeze) — NOT
//! `eval_in_frozen` (which bypasses the checker). Each op is wrapped in a typed `defn` returning `:i64`
//! (every container result is collapsed through `foldl`, so no container-return annotation is needed and
//! the ONLY thing under test is whether the op accepts a PersistentVector at check). RED at HEAD: at least
//! the first `(foldl …)` over a PersistentVector raises TypeMismatch (scheme expects Vector). GREEN when 0d's
//! 8 projective infer arms land.
//!
//! The guard (`wrong_element_still_rejected`) proves parity is not permissiveness: a String-fn folded over an
//! i64 PersistentVector must STAY rejected after the fix.
//!
//! Run: cargo test --release -p wat --test probe_arc278_0d_transform_dispatch_parity -- --include-ignored

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Type-check a whole program at freeze time. Ok(()) = checks clean; Err = a CheckError fired.
fn check(src: &str) -> Result<(), String> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

const MAIN: &str = "(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

// A fn-first reducer: (acc:i64, x:i64) -> i64. Used to collapse every container result to a scalar,
// so the probe needs no container-return annotation — the only thing under test is PersistentVector
// acceptance by each transform op.
const SUM: &str = "(:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 \
                     (:wat::core::i64::+ acc x))";
const DBL: &str = "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))";
const GT1: &str = "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 1))";
const PV: &str = "(:wat::core::PersistentVector 1 2 3)";

#[test]
fn transform_ops_typecheck_on_persistent_vector() {
    // Each defn returns :i64; every container-producing op is wrapped in (foldl SUM 0 …) so the body
    // type is a scalar. The 8 ops: foldl, foldr, map, filter, reverse, take, drop, concat — each over a
    // PersistentVector. RED at HEAD (the static Vec-only schemes reject PersistentVector).
    let src = format!(
        "(:wat::core::defn :user::p-foldl  [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 {PV}))\n\
         (:wat::core::defn :user::p-foldr  [] -> :wat::core::i64 (:wat::core::foldr {SUM} 0 {PV}))\n\
         (:wat::core::defn :user::p-map    [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 (:wat::core::map {DBL} {PV})))\n\
         (:wat::core::defn :user::p-filter [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 (:wat::core::filter {GT1} {PV})))\n\
         (:wat::core::defn :user::p-rev    [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 (:wat::core::reverse {PV})))\n\
         (:wat::core::defn :user::p-take   [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 (:wat::core::take {PV} 2)))\n\
         (:wat::core::defn :user::p-drop   [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 (:wat::core::drop {PV} 1)))\n\
         (:wat::core::defn :user::p-concat [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 (:wat::core::concat {PV} {PV})))\n\
         {MAIN}"
    );
    let r = check(&src);
    assert!(
        r.is_ok(),
        "all 8 transform ops must type-check on a PersistentVector after 0d. Got: {r:?}"
    );
}

#[test]
fn wrong_element_still_rejected() {
    // GUARD — parity is not permissiveness. A String reducer folded over an i64 PersistentVector must be
    // REJECTED (element type i64 ≠ String). Err today (PV rejected outright) AND after 0d (element mismatch).
    let str_sum = "(:wat::core::fn [acc <- :wat::core::String x <- :wat::core::String] -> :wat::core::String \
                     (:wat::core::string::concat acc x))";
    let src = format!(
        "(:wat::core::defn :user::bad [] -> :wat::core::String (:wat::core::foldl {str_sum} \"\" {PV}))\n{MAIN}"
    );
    let r = check(&src);
    assert!(
        r.is_err(),
        "folding a String reducer over an i64 PersistentVector must be rejected (parity != permissiveness). Got: {r:?}"
    );
}
