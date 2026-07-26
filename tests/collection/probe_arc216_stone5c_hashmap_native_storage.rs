//! Arc 216 Stone 216.5c — `Value::wat__std__HashMap` native storage refactor.
//!
//! Verifies that `Value::wat__std__HashMap` now stores `Arc<HashMap<Value, Value>>`
//! (not the old `Arc<HashMap<String, (Value, Value)>>` canonical-key crutch).
//! All probes exercise the WAT surface — constructor, accessors, overwrite semantic,
//! semantic correction for `keys`, round-trip through Atom, and cross-collection composition.
//!
//! ## Probes
//!
//! 1. Construction with primitive K + V — same observable behavior
//! 2. `HashMap/get` returns Option<V>; Some on hit, None on miss
//! 3. `HashMap/assoc` inserts; overwrite semantic preserved
//! 4. `HashMap/dissoc` removes; returns new HashMap without the key
//! 5. `HashMap/keys` returns Vec<K> with actual Values (SEMANTIC CORRECTION verified)
//! 6. `HashMap/values` returns Vec<V>
//! 7. `HashMap/contains-key?` works
//! 8. `HashMap/length` works
//! 9. `HashMap/empty?` works (true for empty, false for non-empty)
//! 10. Nested HashMap — `HashMap<keyword, HashMap<keyword, i64>>` construction + get
//! 11. HashMap with HashSet as K — `HashMap<HashSet<i64>, String>` (HashSet-as-K)
//! 12. HashMap round-trip through `:wat::holon::to-holon` + `from-holon` (Stone 216.3 contract)

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each `:t::pNN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside_value` — no inline wat driver.

// ─── Probe 1 — Construction with primitive K + V ─────────────────────────────

#[test]
fn probe_1_construction_primitives() {

    match call_beside_value(file!(), ":t::p1a-kw-i64-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "keyword→i64 map length"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p1b-str-bool-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "String→bool map length"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p1c-i64-str-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "i64→String map length"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2 — `HashMap/get` returns Option<V>; Some on hit, None on miss ───

#[test]
fn probe_2_get_hit_and_miss() {

    match call_beside_value(file!(), ":t::p2a-get-hit").expect("eval") {
        Value::i64(n) => assert_eq!(n, 42, "get hit returns Some(42)"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p2b-get-miss").expect("eval") {
        Value::bool(b) => assert!(b, "get miss: key :missing not present"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 3 — `HashMap/assoc` inserts; overwrite semantic preserved ─────────

#[test]
fn probe_3_assoc_insert_overwrite() {

    match call_beside_value(file!(), ":t::p3a-assoc-insert").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "assoc inserts new key → length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p3b-assoc-overwrite").expect("eval") {
        Value::i64(n) => assert_eq!(n, 999, "assoc overwrites existing key"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p3c-assoc-immutable").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "original map unchanged after assoc"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4 — `HashMap/dissoc` removes ──────────────────────────────────────

#[test]
fn probe_4_dissoc_removes() {

    match call_beside_value(file!(), ":t::p4a-dissoc-remove").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "dissoc removes one key → length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p4b-dissoc-noop").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "dissoc missing key — length unchanged"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5 — `HashMap/keys` SEMANTIC CORRECTION ────────────────────────────

#[test]
fn probe_5_keys_semantic_correction() {

    match call_beside_value(file!(), ":t::p5a-keys-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "keys returns Vec of length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p5b-keys-values").expect("eval") {
        Value::bool(b) => assert!(b, "keys returns actual keyword Values that round-trip through contains-key?"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 6 — `HashMap/values` returns Vec<V> ───────────────────────────────

#[test]
fn probe_6_values() {
    match call_beside_value(file!(), ":t::p6-values-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "values returns Vec of length 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7 — `HashMap/contains-key?` works ─────────────────────────────────

#[test]
fn probe_7_contains_key() {

    match call_beside_value(file!(), ":t::p7a-contains-hit").expect("eval") {
        Value::bool(b) => assert!(b, "contains-key? hit"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p7b-contains-miss").expect("eval") {
        Value::bool(b) => assert!(!b, "contains-key? miss"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 8 — `HashMap/length` works ────────────────────────────────────────

#[test]
fn probe_8_length() {

    match call_beside_value(file!(), ":t::p8a-length-four").expect("eval") {
        Value::i64(n) => assert_eq!(n, 4, "length of 4-entry map"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p8b-length-empty").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "length of empty map"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 9 — `HashMap/empty?` works ────────────────────────────────────────

#[test]
fn probe_9_empty_q() {

    match call_beside_value(file!(), ":t::p9a-empty-true").expect("eval") {
        Value::bool(b) => assert!(b, "empty? true for empty map"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p9b-empty-false").expect("eval") {
        Value::bool(b) => assert!(!b, "empty? false for non-empty map"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 10 — Nested HashMap ───────────────────────────────────────────────

#[test]
fn probe_10_nested_hashmap() {

    match call_beside_value(file!(), ":t::p10a-nested-contains").expect("eval") {
        Value::bool(b) => assert!(b, "nested HashMap: outer contains :inner key"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p10b-nested-get").expect("eval") {
        Value::i64(n) => assert_eq!(n, 42, "nested HashMap: get :inner then get :x → 42"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 11 — HashMap with HashSet as K ────────────────────────────────────

#[test]
fn probe_11_hashset_as_key() {

    match call_beside_value(file!(), ":t::p11a-hashset-key-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<HashSet<i64>, String> length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p11b-hashset-key-contains").expect("eval") {
        Value::bool(b) => assert!(b, "HashSet-as-K: same elements different construction → same key (hash equality)"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 12 — HashMap round-trip through to-holon + from-holon ─────────────────

#[test]
fn probe_12_atom_roundtrip() {

    match call_beside_value(file!(), ":t::p12a-rt-forward").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "forward (arc-228 classifier-wrap): to-holon→from-holon preserves 2 entries"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p12b-rt-contains").expect("eval") {
        Value::bool(b) => assert!(b, "reverse: from-holon recovers HashMap; contains-key? :foo = true"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p12c-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "round-trip length preserved"),
        other => panic!("expected i64; got {:?}", other),
    }
}
