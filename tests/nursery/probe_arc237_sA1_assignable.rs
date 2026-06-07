//! FM 2-bis probe — arc 237 Stone S-A1: `assignable` choke point (subtyping at the arg boundary).
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-A1-assignable.md`
//! (§ "POST-B.2 SCOPE CORRECTION (grounded)").
//!
//! S-A shipped `is_subtype` + the `typesub` registry; S-B.2 made every `defrecord`
//! emit `(recordtype :Name :wat::Record)`, registering the edge `:my::Circle typesub
//! :wat::Record`. But the type CHECKER does not consult that hierarchy: a value typed
//! `:my::Circle` is REJECTED at a `[v <- :wat::Record]` parameter (proven: pre-stone,
//! `unify(:my::Circle, :wat::Record)` → Err, because records are nominal and
//! `expand_alias` does not expand `TypeDef::Record`).
//!
//! S-A1 mints `fn assignable(actual, expected, subst, types)` — directional-subtype-
//! FIRST (mutation-free `is_subtype`), then ordinary `unify` — and routes the call-arg
//! boundary sites in `infer_list` (6386 / 7025 / 7079 / 7213 / 10256 / 10365 / 12044)
//! plus the defclause clause-match (6867) through it. Liskov: a subtype is accepted
//! where its supertype is wanted. NO constructor-return flip is needed — a param
//! annotation `[c <- :my::Circle]` already binds a subtype-typed value.
//!
//! Probe contracts (6):
//!   1. subtype accepted, single-arg call — `:my::Circle` into `[v <- :wat::Record]` (site 6386).
//!   2. subtype accepted, multi-arg call — `:my::Circle` into 2nd param `[_ <- :wat::Record]` (7025/7079/12044).
//!   3. directional rejection — `:wat::Record` into `[c <- :my::Circle]` slot → type ERROR (guard; pre+post).
//!   4. exact-match unchanged — `:wat::Record` into `[v <- :wat::Record]` → Ok (regression).
//!   5. transitive — `:my::Special typesub :my::Circle typesub :wat::Record`; `:my::Special`
//!      into `[v <- :wat::Record]` accepted (is_subtype transitivity at the boundary).
//!   6. no-edge rejection — unrelated `:my::Square` into `[c <- :my::Circle]` → type ERROR
//!      (no edge → assignable == unify; regression guard).
//!
//! Pre-stone: 1, 2, 5 FAIL (TypeMismatch); 3, 4, 6 PASS (guards/regression).
//! Post-stone S-A1: 6/6 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before the BRIEF.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Type-check a whole program; `Ok(())` iff it startups clean (no check errors).
fn check(src: &str) -> Result<(), String> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

/// Two record classes via the everyday `defrecord` surface (S-B.2 emits recordtype).
const PRELUDE: &str = "\
(:wat::Record::def :my::Circle [radius <- :wat::core::f64])\n\
(:wat::Record::def :my::Square [side <- :wat::core::f64])\n";

const MAIN: &str = "\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

// ─── Probe 1: subtype accepted at a single-arg boundary (site 6386) ───────────
#[test]
fn probe_01_subtype_accepted_single_arg() {
    let src = format!(
        "{PRELUDE}\
         (:wat::core::defn :needs-record [v <- :wat::Record] -> :wat::core::f64 1.0)\n\
         (:wat::core::defn :force [c <- :my::Circle] -> :wat::core::f64 (:needs-record c)){MAIN}"
    );
    assert!(check(&src).is_ok(), "subtype into base param must type-check: {:?}", check(&src));
}

// ─── Probe 2: subtype accepted at a multi-arg boundary, 2nd param (7025/7079/12044)
#[test]
fn probe_02_subtype_accepted_multi_arg() {
    let src = format!(
        "{PRELUDE}\
         (:wat::core::defn :two [a <- :wat::core::f64 b <- :wat::Record] -> :wat::core::f64 a)\n\
         (:wat::core::defn :force2 [c <- :my::Circle] -> :wat::core::f64 (:two 2.0 c)){MAIN}"
    );
    assert!(check(&src).is_ok(), "subtype into 2nd (base) param must type-check: {:?}", check(&src));
}

// ─── Probe 3: directional rejection — supertype into subtype slot (guard) ─────
#[test]
fn probe_03_directional_rejection() {
    let src = format!(
        "{PRELUDE}\
         (:wat::core::defn :needs-circle [c <- :my::Circle] -> :wat::core::f64 1.0)\n\
         (:wat::core::defn :feed [r <- :wat::Record] -> :wat::core::f64 (:needs-circle r)){MAIN}"
    );
    assert!(check(&src).is_err(), "supertype into subtype slot must remain a type error");
}

// ─── Probe 4: exact-match unchanged (regression) ──────────────────────────────
#[test]
fn probe_04_exact_match_ok() {
    let src = format!(
        "{PRELUDE}\
         (:wat::core::defn :needs-record [v <- :wat::Record] -> :wat::core::f64 1.0)\n\
         (:wat::core::defn :passthru [v <- :wat::Record] -> :wat::core::f64 (:needs-record v)){MAIN}"
    );
    assert!(check(&src).is_ok(), "exact :wat::Record into :wat::Record must type-check: {:?}", check(&src));
}

// ─── Probe 5: transitive — :my::Special <: :my::Circle <: :wat::Record ────────
#[test]
fn probe_05_transitive() {
    let src = format!(
        "{PRELUDE}\
         (:wat::core::recordtype :my::Special :my::Circle [])\n\
         (:wat::core::defn :needs-record [v <- :wat::Record] -> :wat::core::f64 1.0)\n\
         (:wat::core::defn :force3 [s <- :my::Special] -> :wat::core::f64 (:needs-record s)){MAIN}"
    );
    assert!(check(&src).is_ok(), "transitive subtype into base must type-check: {:?}", check(&src));
}

// ─── Probe 6: no-edge rejection — unrelated record (guard) ────────────────────
#[test]
fn probe_06_no_edge_rejected() {
    let src = format!(
        "{PRELUDE}\
         (:wat::core::defn :needs-circle [c <- :my::Circle] -> :wat::core::f64 1.0)\n\
         (:wat::core::defn :feed-sq [s <- :my::Square] -> :wat::core::f64 (:needs-circle s)){MAIN}"
    );
    assert!(check(&src).is_err(), "unrelated record into subtype slot must be a type error (no edge)");
}
