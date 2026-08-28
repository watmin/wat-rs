//! FM 2-bis probe — arc 237 Stone S-C.3: the base/holonic macro split.
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-C3.md`.
//!
//! `:wat::core::defrecord` → BASE (struct only; recordtype parent :wat::core::Record).
//! `:wat::holon::defrecord` → HOLONIC (struct + holon; parent :wat::holon::Record <: :wat::core::Record).
//! The recordtype parent IS the Liskov mechanism: a func wanting :wat::holon::Record rejects a
//! base-defined record at CHECK time; wanting :wat::core::Record accepts both.
//!
//! RED at the arc-237 strike: `:wat::holon::Record::def` did not exist, and `:wat::core::Record::def` still
//! built holonic (so base ops + to-holon-error + Liskov rejection were unmet). GREEN after the stone.
//! (Both macros were later renamed — arc 293.2 — to `:wat::core::defrecord` / `:wat::holon::defrecord`,
//! the names used in the design lines above.)
//!
//! Coverage (feedback_logic_coverage_mandate): base ops · holonic preserved · Liskov accept/reject
//! · cross-flavor.

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_beside, startup_from_file};
use wat::runtime::{RuntimeErrorKind, Value};

fn eval_bool(fn_name: &str) -> bool {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::bool(b) => b,
        other => panic!("expected bool from {}; got {:?}", fn_name, other),
    }
}

fn eval_i64(fn_name: &str) -> i64 {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::i64(n) => n,
        other => panic!("expected i64 from {}; got {:?}", fn_name, other),
    }
}

// ─── BASE flavor (:my::Pt via :wat::core::defrecord) ──────────────────────────────
#[test] fn base_construct_and_field() { assert_eq!(eval_i64(":user::base-construct-and-field"), 1); }
#[test] fn base_accessor() { assert_eq!(eval_i64(":user::base-accessor"), 2); }
#[test] fn base_predicate_true() { assert!(eval_bool(":user::base-predicate-true")); }
#[test] fn base_predicate_false() { assert!(!eval_bool(":user::base-predicate-false")); }
#[test] fn base_eq_equal() { assert!(eval_bool(":user::base-eq-equal")); }
#[test] fn base_eq_diff() { assert!(!eval_bool(":user::base-eq-diff")); }
#[test] fn base_same_data() { assert!(eval_bool(":user::base-same-data")); }
#[test] fn base_assoc_then_read() { assert_eq!(eval_i64(":user::base-assoc-then-read"), 9); }
#[test] fn base_to_holon_errors() {
    // base has NO holon flavor — to-holon must error (teaching error), not return Ok.
    let h = call_beside_value(file!(), ":user::base-to-holon-errors");
    assert!(
        matches!(&h, Err(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { head, reason }
            if head == ":wat::holon::to-holon"
            && reason == "base record `my::Pt` has no holon flavor; construct a holonic record \
                           (`:wat::holon::defrecord`) to use holon operations")),
        "to-holon on a BASE record must error; got {:?}",
        h
    );
}

// ─── HOLONIC flavor (:my::HPt via :wat::holon::defrecord) ────────────────────
#[test] fn holonic_construct_field() { assert_eq!(eval_i64(":user::holonic-construct-field"), 7); }
#[test] fn holonic_predicate_true() { assert!(eval_bool(":user::holonic-predicate-true")); }
#[test] fn holonic_to_holon_ok() {
    // holonic HAS a holon flavor — to-holon works.
    let t = call_beside_value(file!(), ":user::holonic-to-holon-ok");
    assert!(t.is_ok(), "to-holon on a HOLONIC record must work; got {:?}", t);
}

// ─── Liskov type-distinction (the static proof) ───────────────────────────────
// Positive cases: the shared .wat file includes :fb, :fh, :gh — their presence
// in the startup proves they type-check (startup_beside succeeds only if ALL pass).

#[test] fn liskov_base_into_base_ok() {
    startup_beside(file!()).expect("liskov: :fb [p <- :my::Pt] calling :wb [v <- :wat::core::Record] must type-check");
}
#[test] fn liskov_holonic_into_base_ok() {
    // holonic <: base — a func wanting base accepts a holonic-defined record.
    startup_beside(file!()).expect("liskov: :fh [p <- :my::HPt] calling :wb [v <- :wat::core::Record] must type-check");
}
#[test] fn liskov_holonic_into_holon_ok() {
    startup_beside(file!()).expect("liskov: :gh [p <- :my::HPt] calling :wh [v <- :wat::holon::Record] must type-check");
}
#[test] fn liskov_base_into_holon_rejected() {
    // THE static proof: a base-defined record is NOT a :wat::holon::Record → check error.
    let r = startup_from_file(
        "tests/types/probe_arc237_sC3_macro_split_liskov_base_into_holon.wat.bad",
    );
    wat::assert_startup_error!(r, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":my::wh"
            && param == "#1"
            && expected == ":wat::holon::Record"
            && got == ":my::Pt"
    );
}

// ─── Cross-flavor (needs both macros) ─────────────────────────────────────────
#[test] fn cross_flavor_same_data_true() {
    // base Pt[0,0] vs holonic HPt[0,0], same field names → type-blind same-data? true
    assert!(eval_bool(":user::cross-flavor-same-data-true"));
}
#[test] fn cross_flavor_eq_false() {
    // = is type-strict: different type/flavor → false
    assert!(!eval_bool(":user::cross-flavor-eq-false"));
}
