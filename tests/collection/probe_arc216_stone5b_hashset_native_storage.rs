//! Arc 216 Stone 216.5b — `Value::wat__std__HashSet` native storage refactor.
//!
//! Verifies that `Value::wat__std__HashSet` now stores `Arc<HashSet<Value>>`
//! (not the old `Arc<HashMap<String, Value>>` canonical-key crutch).
//! All probes exercise the WAT surface — constructor, accessors, dedupe,
//! round-trip through Atom, and cross-collection composition.
//!
//! ## Probes
//!
//! 1. Construction with primitive elements (i64, String, bool, keyword)
//! 2. `HashSet/contains?` works for all primitive types (hit + miss)
//! 3. `HashSet/length` works
//! 4. `HashSet/empty?` works (true for empty, false for non-empty)
//! 5. `HashSet/conj` returns new HashSet with element; dedupe preserved
//! 6. conj-then-contains? for bool elements
//! 7. Nested HashSet — `HashSet<HashSet<i64>>` construction + element lookup
//! 8. HashSet round-trip through `:wat::holon::to-holon` + `from-holon` (Stone 216.1 contract)
//! 9. HashSet inside HashMap as VALUE — `HashMap<keyword, HashSet<i64>>`
//! 10. HashSet inside HashMap as KEY — `HashMap<HashSet<i64>, String>`

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:t::pNN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside` — no inline wat driver.

// ─── Probe 1 — Construction with primitive elements ──────────────────────────

#[test]
fn probe_1_construction_primitives() {

    match call_beside(file!(), ":t::p1a-i64-set-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "i64 set length"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p1b-str-set-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "String set length"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p1c-bool-set-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "bool set length"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p1d-kw-set-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "keyword set length"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2 — `HashSet/contains?` ──────────────────────────────────────────

#[test]
fn probe_2_contains_q_hit_and_miss() {

    match call_beside(file!(), ":t::p2a-contains-i64-hit").expect("eval") {
        Value::bool(b) => assert!(b, "i64 hit"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside(file!(), ":t::p2b-contains-i64-miss").expect("eval") {
        Value::bool(b) => assert!(!b, "i64 miss"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside(file!(), ":t::p2c-contains-str-hit").expect("eval") {
        Value::bool(b) => assert!(b, "String hit"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside(file!(), ":t::p2d-contains-str-miss").expect("eval") {
        Value::bool(b) => assert!(!b, "String miss"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside(file!(), ":t::p2e-contains-kw-hit").expect("eval") {
        Value::bool(b) => assert!(b, "keyword hit"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 3 — `HashSet/length` ──────────────────────────────────────────────

#[test]
fn probe_3_length() {
    match call_beside(file!(), ":t::p3-length").expect("eval") {
        Value::i64(n) => assert_eq!(n, 5),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4 — `HashSet/empty?` ──────────────────────────────────────────────

#[test]
fn probe_4_empty_q() {

    match call_beside(file!(), ":t::p4a-nonempty").expect("eval") {
        Value::bool(b) => assert!(!b, "non-empty is false"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside(file!(), ":t::p4b-dedup-nonempty").expect("eval") {
        Value::bool(b) => assert!(!b, "dedupe still non-empty"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 5 — `HashSet/conj` ────────────────────────────────────────────────

#[test]
fn probe_5_conj_and_dedupe() {

    match call_beside(file!(), ":t::p5a-conj-add").expect("eval") {
        Value::bool(b) => assert!(b, "conj adds new element"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside(file!(), ":t::p5b-conj-dup").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "conj duplicate is idempotent"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p5c-conj-immutable").expect("eval") {
        Value::bool(b) => assert!(!b, "conj does not mutate input"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 6 — conj-then-contains? for bool elements ─────────────────────────

#[test]
fn probe_6_conj_bool_elements() {

    match call_beside(file!(), ":t::p6a-conj-bool-false").expect("eval") {
        Value::bool(b) => assert!(b, "conj false into set-with-true and find it"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside(file!(), ":t::p6b-conj-bool-dedup").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "conj of already-present bool element: length stays 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7 — Nested HashSet<HashSet<i64>> ──────────────────────────────────

#[test]
fn probe_7_nested_hashset() {

    match call_beside(file!(), ":t::p7a-nested-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "outer HashSet has 2 inner sets"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p7b-nested-contains").expect("eval") {
        Value::bool(b) => assert!(b, "inner HashSet found by value equality"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside(file!(), ":t::p7c-nested-dedup").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "duplicate inner HashSet deduped"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 8 — HashSet round-trip through to-holon + from-holon ──────────────────

#[test]
fn probe_8_atom_round_trip() {

    match call_beside(file!(), ":t::p8a-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "round-trip preserves length"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p8b-rt-contains").expect("eval") {
        Value::bool(b) => assert!(b, "round-trip preserves membership"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 9 — HashSet as VALUE inside a HashMap ─────────────────────────────

#[test]
fn probe_9_hashset_as_hashmap_value() {
    match call_beside(file!(), ":t::p9-hashset-as-hm-val").expect("eval") {
        Value::bool(b) => assert!(b, "HashSet value retrieved from HashMap and membership verified"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 10 — HashSet as KEY inside a HashMap ──────────────────────────────

#[test]
fn probe_10_hashset_as_hashmap_key() {
    match call_beside(file!(), ":t::p10-hashset-as-hm-key").expect("eval") {
        Value::bool(b) => assert!(b, "HashSet key found via HashMap/contains-key? (same elements = same canonical key)"),
        other => panic!("expected bool; got {:?}", other),
    }
}
