//! Arc 278 stone 0d — disconfirming probe: transform-op CHECK-SIDE parity. RED at HEAD.
//!
//! 0c gave PersistentVector the transform/sequence ops at RUNTIME (the `eval_vec_*` arms dispatch on
//! PersistentVector). But the CHECKER never followed: `map`/`filter`/`foldl`/`reverse`/`take`/`drop`
//! are still monomorphic `Vector`-only static TypeSchemes (check.rs:17963-18073), `concat` checks via the
//! `Vector/concat` alias. So a TYPED body that folds/maps a PersistentVector is rejected at check time.
//! (Arc 118.B6b: `foldr` retired — its slot below is now `reduce` over `reverse`.)
//!
//! This probe exercises the CHECKER (via startup, which type-checks at freeze) — NOT
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
//!
//! Wat source lives in the co-located fixture: probe_arc278_0d_transform_dispatch_parity.wat
//! (slurped via startup_beside(file!())).
//! Negative fixture: tests/collection/probe_arc278_0d_transform_dispatch_parity.wat.bad

use wat::freeze::{startup_beside, startup_from_file};

#[test]
fn transform_ops_typecheck_on_persistent_vector() {
    // Each defn returns :i64; every container-producing op is wrapped in (foldl SUM 0 …) so the body
    // type is a scalar. The 8 ops: foldl, reduce-over-reverse, map, filter, reverse, take, drop,
    // concat — each over a PersistentVector. RED at HEAD (the static Vec-only schemes reject
    // PersistentVector).
    let r = startup_beside(file!());
    assert!(
        r.is_ok(),
        "all 8 transform ops must type-check on a PersistentVector after 0d. Got: {r:?}"
    );
}

#[test]
fn wrong_element_still_rejected() {
    // GUARD — parity is not permissiveness. A String reducer folded over an i64 PersistentVector must be
    // REJECTED (element type i64 ≠ String). Err today (PV rejected outright) AND after 0d (element mismatch).
    let r = startup_from_file("tests/collection/probe_arc278_0d_transform_dispatch_parity.wat.bad");
    assert!(
        r.is_err(),
        "folding a String reducer over an i64 PersistentVector must be rejected (parity != permissiveness). Got: {r:?}"
    );
}

#[test]
fn bare_typed_containers_typecheck_through_hofs() {
    // 0d.1 REGRESSION GUARD — a BARE container annotation (no <T>) reduces to a `Path`, not a `Parametric`.
    // Parametric is the norm, but bare is valid (a heterogeneous field like Session.facts, or an
    // un-parameterized param) and must type-check through the HOFs. The 0d probe only tested the constructor
    // (parametric) form, so the HOF arms rejected bare containers — blocking anything that folds over one.
    // This asserts foldl/map type-check over a bare PersistentVector AND a bare Vector param (both failed
    // before 0d.1).
    // Shares the co-located fixture with transform_ops_typecheck_on_persistent_vector.
    let r = startup_beside(file!());
    assert!(
        r.is_ok(),
        "HOFs must type-check over BARE-typed (un-parameterized) containers — record fields + fn params use bare. Got: {r:?}"
    );
}
