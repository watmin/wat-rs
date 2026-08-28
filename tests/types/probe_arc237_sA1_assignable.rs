//! FM 2-bis probe — arc 237 Stone S-A1: `assignable` choke point (subtyping at the arg boundary).
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-A1-assignable.md`
//! (§ "POST-B.2 SCOPE CORRECTION (grounded)").
//!
//! S-A shipped `is_subtype` + the `typesub` registry; S-B.2 made every `defrecord`
//! emit `(recordtype :Name :wat::core::Record)`, registering the edge `:my::Circle typesub
//! :wat::core::Record`. But the type CHECKER does not consult that hierarchy: a value typed
//! `:my::Circle` is REJECTED at a `[v <- :wat::core::Record]` parameter (proven: pre-stone,
//! `unify(:my::Circle, :wat::core::Record)` → Err, because records are nominal and
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
//!   1. subtype accepted, single-arg call — `:my::Circle` into `[v <- :wat::core::Record]` (site 6386).
//!   2. subtype accepted, multi-arg call — `:my::Circle` into 2nd param `[_ <- :wat::core::Record]` (7025/7079/12044).
//!   3. directional rejection — `:wat::core::Record` into `[c <- :my::Circle]` slot → type ERROR (guard; pre+post).
//!   4. exact-match unchanged — `:wat::core::Record` into `[v <- :wat::core::Record]` → Ok (regression).
//!   5. transitive — `:my::Special typesub :my::Circle typesub :wat::core::Record`; `:my::Special`
//!      into `[v <- :wat::core::Record]` accepted (is_subtype transitivity at the boundary).
//!   6. no-edge rejection — unrelated `:my::Square` into `[c <- :my::Circle]` → type ERROR
//!      (no edge → assignable == unify; regression guard).
//!
//! Pre-stone: 1, 2, 5 FAIL (TypeMismatch); 3, 4, 6 PASS (guards/regression).
//! Post-stone S-A1: 6/6 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before the BRIEF.
//! Wat fixtures: probe_arc237_sA1_assignable_probe{01,02,04,05}.wat (positive),
//!   probe_arc237_sA1_assignable_probe{03,06}.wat.bad (negative).

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

// ─── Probe 1: subtype accepted at a single-arg boundary (site 6386) ───────────
#[test]
fn probe_01_subtype_accepted_single_arg() {
    let r = startup_from_file("tests/types/probe_arc237_sA1_assignable_probe01.wat")
        .map(|_| ())
        .map_err(|e| format!("{:?}", e));
    assert!(r.is_ok(), "subtype into base param must type-check: {:?}", r);
}

// ─── Probe 2: subtype accepted at a multi-arg boundary, 2nd param (7025/7079/12044)
#[test]
fn probe_02_subtype_accepted_multi_arg() {
    let r = startup_from_file("tests/types/probe_arc237_sA1_assignable_probe02.wat")
        .map(|_| ())
        .map_err(|e| format!("{:?}", e));
    assert!(r.is_ok(), "subtype into 2nd (base) param must type-check: {:?}", r);
}

// ─── Probe 3: directional rejection — supertype into subtype slot (guard) ─────
#[test]
fn probe_03_directional_rejection() {
    let r = startup_from_file("tests/types/probe_arc237_sA1_assignable_probe03.wat.bad");
    wat::assert_startup_error!(r, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":my::needs-circle"
            && param == "#1"
            && expected == ":my::Circle"
            && got == ":wat::core::Record"
    );
}

// ─── Probe 4: exact-match unchanged (regression) ──────────────────────────────
#[test]
fn probe_04_exact_match_ok() {
    let r = startup_from_file("tests/types/probe_arc237_sA1_assignable_probe04.wat")
        .map(|_| ())
        .map_err(|e| format!("{:?}", e));
    assert!(r.is_ok(), "exact :wat::core::Record into :wat::core::Record must type-check: {:?}", r);
}

// Probe 5 (transitive :my::Special <: :my::Circle) DELETED — arc 293 inheritance annihilation:
// a recordtype parent must be a nature-root; :my::Circle is a user type, so
// (:wat::core::recordtype :my::Special :my::Circle []) is now rejected at registration.

// ─── Probe 6: no-edge rejection — unrelated record (guard) ────────────────────
#[test]
fn probe_06_no_edge_rejected() {
    let r = startup_from_file("tests/types/probe_arc237_sA1_assignable_probe06.wat.bad");
    wat::assert_startup_error!(r, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":my::needs-circle"
            && param == "#1"
            && expected == ":my::Circle"
            && got == ":my::Square"
    );
}
