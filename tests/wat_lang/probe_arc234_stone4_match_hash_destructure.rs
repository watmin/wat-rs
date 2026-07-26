//! Diagnostic probe — match-arm hash-destructure (arc 234 Stone 234.4.match).
//!
//! Verifies the Clojure-style `{var :field var2 :field2 ...}` brace-form
//! in match-arm pattern position. Receiver-polymorphic over
//! record / struct / HashMap. Mirror of Stone 234.4 let-binding probe shape.
//!
//! Probe contracts (6):
//!   1. Match record with single {var :field} — extracts field; body uses var
//!   2. Match record with multi {var1 :f1 var2 :f2} — multi-field bind
//!   3. Match HashMap with {var :field} — Option<V> bind per key (Some)
//!   4. Match HashMap multi-key — multiple Option<V> bindings
//!   5. Match-arm fall-through: scrutinee is i64 → hash-destructure arm
//!      does not match → falls to next arm (wildcard)
//!   6. Mixed match: one arm hash-destructure; another a wildcard — selection
//!      is correct per scrutinee type
//!
//! Initial state: 6/6 FAIL (StructPattern in match-arm position returned Err).
//! Post-stone: 6/6 PASS.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// ─── Probe 1 ────────────────────────────────────────────────────────────────
// Match record with single {var :field} — extracts field; body uses var.
#[test]
fn probe_1_match_record_single_field() {
    match call_beside_value(file!(), ":t::probe1-match-record-single").expect("eval") {
        Value::f64(f) => assert!((f - 7.5).abs() < 1e-9, "got {}", f),
        other => panic!("Probe 1: expected f64; got {:?}", other),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
// Match record with multi {var1 :f1 var2 :f2} — multi-field bind.
#[test]
fn probe_2_match_record_multi_field() {
    match call_beside_value(file!(), ":t::probe2-match-record-multi").expect("eval") {
        Value::i64(n) => assert_eq!(n, 7, "got {}", n),
        other => panic!("Probe 2: expected i64; got {:?}", other),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
// Match HashMap with {var :field} — Option<V> bind per key (present key → Some).
#[test]
fn probe_3_match_hashmap_single_key_some() {
    match call_beside_value(file!(), ":t::probe3-hashmap-single-some").expect("eval") {
        Value::i64(n) => assert_eq!(n, 9000, "got {}", n),
        other => panic!("Probe 3: expected i64; got {:?}", other),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
// Match HashMap multi-key — multiple Option<V> bindings; body uses both.
// Uses a homogeneous String-valued HashMap to satisfy the type checker.
#[test]
fn probe_4_match_hashmap_multi_key() {
    // h = :host → Some("localhost"), mv = :missing → None
    // → h arm matches Some → check mv → None → true
    match call_beside_value(file!(), ":t::probe4-hashmap-multi").expect("eval") {
        Value::bool(b) => assert!(b, "Probe 4: expected true (h=Some, mv=None)"),
        other => panic!("Probe 4: expected bool; got {:?}", other),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
// Match-arm fall-through: scrutinee is i64 → hash-destructure arm does not
// match → falls to next wildcard arm which returns the integer.
#[test]
fn probe_5_fall_through_on_non_receiver() {
    match call_beside_value(file!(), ":t::probe5-fall-through").expect("eval") {
        Value::i64(n) => assert_eq!(
            n, 99,
            "expected fall-through to wildcard arm (99); got {}",
            n
        ),
        other => panic!("Probe 5: expected i64; got {:?}", other),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
// Mixed match: one arm hash-destructure on record; another wildcard.
// Selection is correct per scrutinee type.
#[test]
fn probe_6_mixed_match_arm_selection() {
    match call_beside_value(file!(), ":t::probe6-mixed").expect("eval") {
        Value::String(s) => assert_eq!(
            s.as_str(),
            "record-matched",
            "Probe 6: hash-destructure arm should have matched the record"
        ),
        other => panic!("Probe 6: expected String; got {:?}", other),
    }
}
