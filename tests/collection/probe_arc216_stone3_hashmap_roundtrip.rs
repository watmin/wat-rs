//! Arc 216 Stone 3 — `HashMap<K, V>` round-trip through `HolonAST::Bundle` of arbitrary-K Binds.
//!
//! ## The 14 probes
//!
//! Forward direction:
//!  1. `(:wat::holon::to-holon{:foo 42 :bar 99})` → `HolonAST::Bundle` of 2 Bind children
//!
//! Reverse direction:
//!  2. `(:wat::holon::from-holon<bundle>)` → HashMap; length = 2; :foo key present
//!
//! Edge cases:
//!  3. Empty map `{}` + consumer declares HashMap → empty HashMap
//!
//! Multi-K types:
//!  4. HashMap<keyword,V>, HashMap<String,V>, HashMap<i64,V>, HashMap<bool,V> all round-trip
//!
//! Multi-V types:
//!  5. HashMap<K,i64>, HashMap<K,String>, HashMap<K,bool>, HashMap<K,keyword> all round-trip
//!
//! Non-keyword keys:
//!  6. HashMap<i64, String> round-trips (arbitrary K via atom-value)
//!
//! Nested map:
//!  7. HashMap<keyword, HashMap<keyword, i64>> round-trips
//!
//! Mixed nesting (Vec):
//!  8. HashMap<keyword, Vec<i64>> round-trips (composes with Stone 216.2)
//!
//! Mixed nesting (HashSet):
//!  9. HashMap<keyword, HashSet<i64>> round-trips (composes with Stone 216.1)
//!
//! Check-level atomizable predicate:
//! 10. `(:wat::holon::to-holon m)` for atomizable K+V type-checks cleanly
//! 11. `(:wat::holon::to-holon fn-value)` — non-atomizable type fails at check (TypeMismatch)
//!
//! Arc 294.h: probe 12 (a Rust-side `HolonRepresentable` cascade) is removed
//! wholesale — `HolonRepresentable` had zero production consumers and is
//! deleted. Probe 13 loses only its Rust-side steps 1-2 (which asserted
//! against the deleted trait directly); its wat-surface step 3 survives
//! below as the whole probe body.
//!
//! Shape disambiguation:
//! 13. Bundle with non-sequential i64 keys [Bind(0,v), Bind(5,v)] → HashMap (not Vec) — wat-surface only
//!
//! Empty Bundle disambiguation via consumer-declared HashMap type:
//! 14. `(atom-value empty-bundle -> :wat::core::HashMap<K,V>)` → empty HashMap

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

// just-eval (rubric): each `:t::pNN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside_value` — no inline wat driver.

// ─── Probe 1 — Forward: HashMap → classifier-wrapped HolonAST ────────────────

#[test]
fn probe_1_forward_hashmap_to_bundle() {
    match call_beside_value(file!(), ":t::p1-forward-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "classifier-wrapped Map encoding must preserve 2 entries in round-trip"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2 — Reverse: Bundle → HashMap round-trip ──────────────────────────

#[test]
fn probe_2_reverse_bundle_to_hashmap_roundtrip() {

    match call_beside_value(file!(), ":t::p2a-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "round-trip must preserve length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p2b-rt-foo").expect("eval") {
        Value::bool(b) => assert!(b, "round-trip must preserve :foo key"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p2c-rt-bar").expect("eval") {
        Value::bool(b) => assert!(b, "round-trip must preserve :bar key"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 3 — Empty map round-trip via consumer-declared HashMap type ────────

#[test]
fn probe_3_empty_map_roundtrip_consumer_declared() {

    match call_beside_value(file!(), ":t::p3a-empty-rt-forward").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "empty HashMap classifier-wrapped encoding must round-trip to HashMap length 0"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p3b-empty-rt-reverse").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "empty Map classifier-wrapped + consumer hint: empty HashMap (length 0)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4 — Multi-K types ─────────────────────────────────────────────────

#[test]
fn probe_4_multi_k_types() {

    match call_beside_value(file!(), ":t::p4a-kw-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<keyword,i64> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p4b-str-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<String,i64> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p4c-i64k-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<i64,String> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p4d-bool-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<bool,i64> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5 — Multi-V types ─────────────────────────────────────────────────

#[test]
fn probe_5_multi_v_types() {

    match call_beside_value(file!(), ":t::p5a-v-i64").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,i64> V=i64 round-trip: length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p5b-v-str").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<keyword,String> V=String round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p5c-v-bool").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<keyword,bool> V=bool round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p5d-v-kw").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<keyword,keyword> V=keyword round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 6 — Non-keyword keys: HashMap<i64, String> ────────────────────────

#[test]
fn probe_6_non_keyword_keys_i64_string() {

    match call_beside_value(file!(), ":t::p6a-i64k-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<i64,String> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p6b-i64k-rt-contains").expect("eval") {
        Value::bool(b) => assert!(b, "HashMap<i64,String> round-trip must preserve key 100"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 7 — Nested map: HashMap<keyword, HashMap<keyword, i64>> ───────────

#[test]
fn probe_7_nested_map_roundtrip() {

    match call_beside_value(file!(), ":t::p7a-nested-outer-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "nested map outer length = 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p7b-nested-arc228").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "nested map arc 228: classifier-wrapped outer HashMap length = 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 8 — Mixed nesting: HashMap<keyword, Vec<i64>> ─────────────────────

#[test]
fn probe_8_mixed_nesting_hashmap_of_vec() {

    match call_beside_value(file!(), ":t::p8a-hashmap-of-vec-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,Vec<i64>> round-trip: outer length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p8b-hashmap-of-vec-arc228").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,Vec<i64>> arc 228: classifier-wrapped outer length = 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 9 — Mixed nesting: HashMap<keyword, HashSet<i64>> ─────────────────

#[test]
fn probe_9_mixed_nesting_hashmap_of_hashset() {

    match call_beside_value(file!(), ":t::p9a-hashmap-of-set-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,HashSet<i64>> round-trip: outer length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p9b-hashmap-of-set-arc228").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,HashSet<i64>> arc 228: classifier-wrapped outer length = 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 10 — Check passes for atomizable K+V types ───────────────────────

#[test]
fn probe_10_check_passes_atomizable_k_v() {

    match call_beside_value(file!(), ":t::p10a-atomizable-passes").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "Atom on HashMap<keyword,i64> must pass check and run"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p10b-nested-atomizable").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "Atom on HashMap<keyword,HashMap<keyword,i64>> must pass check and run"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 11 — Check fails for non-atomizable type ──────────────────────────

#[test]
fn probe_11_check_fails_non_atomizable() {
    let err = startup_from_file(
        "tests/collection/probe_arc216_stone3_hashmap_roundtrip_p11.wat.bad",
    )
    .expect_err("expected startup failure for non-atomizable Fn type");
    wat::assert_edn_matches_file!(format!("{err}"), "probe_arc216_stone3_hashmap_roundtrip__non_atomizable_fn.edn", "probe_11: non-atomizable type check-error golden (Display)");
    wat::assert_edn_matches_file!(format!("{err:?}"), "probe_arc216_stone3_hashmap_roundtrip__non_atomizable_fn.edn", "probe_11: non-atomizable type check-error golden (Debug)");
}

// ─── Probe 13 — Shape disambiguation: non-sequential i64 keys → HashMap ──────
//
// Arc 294.h: this probe's original steps 1-2 asserted directly against
// `HolonRepresentable::from_holon_ast` (deleted). Its step 3 — the
// wat-surface assertion — is untouched by that stone and is this probe's
// entire body now.

#[test]
fn probe_13_shape_disambiguation_non_sequential_i64() {
    // Via WAT surface — HashMap<i64, String> with keys 0+5 round-trips as HashMap.
    match call_beside_value(file!(), ":t::p13-non-seq-i64-keys").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<i64,String> with keys 0+5 must round-trip as HashMap (not Vec)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 14 — Empty Bundle disambiguation via consumer-declared HashMap ─────

#[test]
fn probe_14_empty_bundle_disambiguation_consumer_declares_hashmap() {

    match call_beside_value(file!(), ":t::p14a-empty-classifier-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "arc 228: empty HashMap classifier-wrapped encoding returns HashMap (not HashSet)"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p14b-empty-classifier-annotated").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "annotated form still works: empty Map classifier + consumer hint → empty HashMap (length 0)"),
        other => panic!("expected i64; got {:?}", other),
    }
}
