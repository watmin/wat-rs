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
//! HolonRepresentable Rust-side:
//! 12. `HashMap<String, String>` satisfies `HolonRepresentable` at compile time; roundtrip correct
//!
//! Shape disambiguation:
//! 13. Bundle with non-sequential i64 keys [Bind(0,v), Bind(5,v)] → HashMap (not Vec)
//!
//! Empty Bundle disambiguation via consumer-declared HashMap type:
//! 14. `(atom-value empty-bundle -> :wat::core::HashMap<K,V>)` → empty HashMap

use std::collections::HashMap;
use wat::comms::HolonRepresentable;
use wat::freeze::{call_beside, startup_from_file};
use wat::runtime::Value;

// just-eval (rubric): each `:t::pNN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside` — no inline wat driver.

// ─── Probe 1 — Forward: HashMap → classifier-wrapped HolonAST ────────────────

#[test]
fn probe_1_forward_hashmap_to_bundle() {
    match call_beside(file!(), ":t::p1-forward-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "classifier-wrapped Map encoding must preserve 2 entries in round-trip"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2 — Reverse: Bundle → HashMap round-trip ──────────────────────────

#[test]
fn probe_2_reverse_bundle_to_hashmap_roundtrip() {

    match call_beside(file!(), ":t::p2a-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "round-trip must preserve length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p2b-rt-foo").expect("eval") {
        Value::bool(b) => assert!(b, "round-trip must preserve :foo key"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside(file!(), ":t::p2c-rt-bar").expect("eval") {
        Value::bool(b) => assert!(b, "round-trip must preserve :bar key"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 3 — Empty map round-trip via consumer-declared HashMap type ────────

#[test]
fn probe_3_empty_map_roundtrip_consumer_declared() {

    match call_beside(file!(), ":t::p3a-empty-rt-forward").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "empty HashMap classifier-wrapped encoding must round-trip to HashMap length 0"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p3b-empty-rt-reverse").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "empty Map classifier-wrapped + consumer hint: empty HashMap (length 0)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4 — Multi-K types ─────────────────────────────────────────────────

#[test]
fn probe_4_multi_k_types() {

    match call_beside(file!(), ":t::p4a-kw-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<keyword,i64> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p4b-str-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<String,i64> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p4c-i64k-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<i64,String> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p4d-bool-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<bool,i64> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5 — Multi-V types ─────────────────────────────────────────────────

#[test]
fn probe_5_multi_v_types() {

    match call_beside(file!(), ":t::p5a-v-i64").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,i64> V=i64 round-trip: length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p5b-v-str").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<keyword,String> V=String round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p5c-v-bool").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<keyword,bool> V=bool round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p5d-v-kw").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<keyword,keyword> V=keyword round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 6 — Non-keyword keys: HashMap<i64, String> ────────────────────────

#[test]
fn probe_6_non_keyword_keys_i64_string() {

    match call_beside(file!(), ":t::p6a-i64k-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<i64,String> round-trip: length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p6b-i64k-rt-contains").expect("eval") {
        Value::bool(b) => assert!(b, "HashMap<i64,String> round-trip must preserve key 100"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 7 — Nested map: HashMap<keyword, HashMap<keyword, i64>> ───────────

#[test]
fn probe_7_nested_map_roundtrip() {

    match call_beside(file!(), ":t::p7a-nested-outer-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "nested map outer length = 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p7b-nested-arc228").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "nested map arc 228: classifier-wrapped outer HashMap length = 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 8 — Mixed nesting: HashMap<keyword, Vec<i64>> ─────────────────────

#[test]
fn probe_8_mixed_nesting_hashmap_of_vec() {

    match call_beside(file!(), ":t::p8a-hashmap-of-vec-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,Vec<i64>> round-trip: outer length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p8b-hashmap-of-vec-arc228").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,Vec<i64>> arc 228: classifier-wrapped outer length = 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 9 — Mixed nesting: HashMap<keyword, HashSet<i64>> ─────────────────

#[test]
fn probe_9_mixed_nesting_hashmap_of_hashset() {

    match call_beside(file!(), ":t::p9a-hashmap-of-set-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,HashSet<i64>> round-trip: outer length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p9b-hashmap-of-set-arc228").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword,HashSet<i64>> arc 228: classifier-wrapped outer length = 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 10 — Check passes for atomizable K+V types ───────────────────────

#[test]
fn probe_10_check_passes_atomizable_k_v() {

    match call_beside(file!(), ":t::p10a-atomizable-passes").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "Atom on HashMap<keyword,i64> must pass check and run"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p10b-nested-atomizable").expect("eval") {
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
    let err = format!("{}\n---\n{:?}", err, err);
    assert_eq!(
        err,
        r##"#wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::holon::to-holon: parameter #1 expects atomizable type (primitive | HolonAST | WatAST | HashSet<T> | Vector<T> | HashMap<K,V> for atomizable T); got :wat::core::Fn(wat::core::i64)->wat::core::i64" :location #wat.core/Span {:file "tests/collection/probe_arc216_stone3_hashmap_roundtrip_p11.wat.bad" :line 6 :col 28 :end #wat.core.Option/Some #wat.core/Pos {:line 6 :col 29}} :causes [] :callee ":wat::holon::to-holon" :param "#1" :expected "atomizable type (primitive | HolonAST | WatAST | HashSet<T> | Vector<T> | HashMap<K,V> for atomizable T)" :got ":wat::core::Fn(wat::core::i64)->wat::core::i64" :remedies []} #wat.check/ReturnTypeMismatch {:message ":user::compute: body produces :wat::holon::HolonAST; signature declares :()" :location #wat.core/Span {:file "tests/collection/probe_arc216_stone3_hashmap_roundtrip_p11.wat.bad" :line 4 :col 3 :end #wat.core.Option/Some #wat.core/Pos {:line 6 :col 31}} :causes [] :function ":user::compute" :expected ":()" :got ":wat::holon::HolonAST" :remedies []}]}
---
#wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::holon::to-holon: parameter #1 expects atomizable type (primitive | HolonAST | WatAST | HashSet<T> | Vector<T> | HashMap<K,V> for atomizable T); got :wat::core::Fn(wat::core::i64)->wat::core::i64" :location #wat.core/Span {:file "tests/collection/probe_arc216_stone3_hashmap_roundtrip_p11.wat.bad" :line 6 :col 28 :end #wat.core.Option/Some #wat.core/Pos {:line 6 :col 29}} :causes [] :callee ":wat::holon::to-holon" :param "#1" :expected "atomizable type (primitive | HolonAST | WatAST | HashSet<T> | Vector<T> | HashMap<K,V> for atomizable T)" :got ":wat::core::Fn(wat::core::i64)->wat::core::i64" :remedies []} #wat.check/ReturnTypeMismatch {:message ":user::compute: body produces :wat::holon::HolonAST; signature declares :()" :location #wat.core/Span {:file "tests/collection/probe_arc216_stone3_hashmap_roundtrip_p11.wat.bad" :line 4 :col 3 :end #wat.core.Option/Some #wat.core/Pos {:line 6 :col 31}} :causes [] :function ":user::compute" :expected ":()" :got ":wat::holon::HolonAST" :remedies []}]}"##,
        "probe_11: non-atomizable type check-error golden"
    );
}

// ─── Probe 12 — HolonRepresentable cascade (compile-time + runtime) ──────────

fn assert_holon_representable<T: HolonRepresentable>() {}

#[test]
fn probe_12_holon_representable_cascade() {
    // Compile-time: if this call compiles, HashMap<String, String>: HolonRepresentable.
    assert_holon_representable::<HashMap<String, String>>();

    // Runtime roundtrip: {"foo" -> "bar", "baz" -> "qux"}.
    let mut original: HashMap<String, String> = HashMap::new();
    original.insert("foo".into(), "bar".into());
    original.insert("baz".into(), "qux".into());
    let ast = original.to_holon_ast();

    // to_holon_ast produces a Bundle of 2 Bind children.
    match &ast {
        holon::HolonAST::Bundle(items) => {
            assert_eq!(items.len(), 2, "Bundle must have 2 children");
            for item in items.iter() {
                assert!(
                    matches!(item, holon::HolonAST::Bind(_, _)),
                    "each child must be HolonAST::Bind; got {:?}",
                    item
                );
            }
        }
        other => panic!("expected HolonAST::Bundle, got {:?}", other),
    }

    // from_holon_ast reconstructs the HashMap with same entries.
    let reconstructed: HashMap<String, String> =
        HolonRepresentable::from_holon_ast(&ast).expect("roundtrip");
    assert_eq!(reconstructed.len(), 2, "roundtrip must preserve entry count");
    assert_eq!(
        reconstructed.get("foo").map(String::as_str),
        Some("bar"),
        "roundtrip must preserve foo -> bar"
    );
    assert_eq!(
        reconstructed.get("baz").map(String::as_str),
        Some("qux"),
        "roundtrip must preserve baz -> qux"
    );

    // Nested: HashMap<String, Vec<String>> — bounds compose.
    assert_holon_representable::<HashMap<String, Vec<String>>>();
    let mut nested: HashMap<String, Vec<String>> = HashMap::new();
    nested.insert("first".into(), vec!["a".into(), "b".into()]);
    nested.insert("second".into(), vec!["c".into()]);
    let nested_ast = nested.to_holon_ast();
    let nested_back: HashMap<String, Vec<String>> =
        HolonRepresentable::from_holon_ast(&nested_ast).expect("nested roundtrip");
    assert_eq!(nested_back.len(), 2, "nested roundtrip must preserve entry count");
    assert_eq!(
        nested_back.get("first").map(|v| v.len()),
        Some(2),
        "nested roundtrip must preserve first -> [a, b]"
    );
}

// ─── Probe 13 — Shape disambiguation: non-sequential i64 keys → HashMap ──────

#[test]
fn probe_13_shape_disambiguation_non_sequential_i64() {
    // Step 1: Verify Vec<String>::from_holon_ast rejects non-sequential bundle.
    let bind0 = holon::HolonAST::bind(holon::HolonAST::i64(0), holon::HolonAST::string("a"));
    let bind5 = holon::HolonAST::bind(holon::HolonAST::i64(5), holon::HolonAST::string("b"));
    let non_seq_bundle = holon::HolonAST::bundle(vec![bind0, bind5]);

    let vec_result = <Vec<String> as HolonRepresentable>::from_holon_ast(&non_seq_bundle);
    assert!(
        vec_result.is_err(),
        "Vec<String>::from_holon_ast on non-sequential i64-keyed Bundle must return Err"
    );

    // Step 2: Verify the Bundle has exactly 2 Bind children (shape is all-Bind).
    match &non_seq_bundle {
        holon::HolonAST::Bundle(items) => {
            assert_eq!(items.len(), 2, "Bundle must have 2 Bind children");
            for item in items.iter() {
                assert!(
                    matches!(item, holon::HolonAST::Bind(_, _)),
                    "each child must be Bind"
                );
            }
        }
        other => panic!("expected Bundle; got {:?}", other),
    }

    // Step 3: Via WAT surface — HashMap<i64, String> with keys 0+5 round-trips as HashMap.
    match call_beside(file!(), ":t::p13-non-seq-i64-keys").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<i64,String> with keys 0+5 must round-trip as HashMap (not Vec)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 14 — Empty Bundle disambiguation via consumer-declared HashMap ─────

#[test]
fn probe_14_empty_bundle_disambiguation_consumer_declares_hashmap() {

    match call_beside(file!(), ":t::p14a-empty-classifier-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "arc 228: empty HashMap classifier-wrapped encoding returns HashMap (not HashSet)"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside(file!(), ":t::p14b-empty-classifier-annotated").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "annotated form still works: empty Map classifier + consumer hint → empty HashMap (length 0)"),
        other => panic!("expected i64; got {:?}", other),
    }
}
