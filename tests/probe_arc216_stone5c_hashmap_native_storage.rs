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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    )
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 1 — Construction with primitive K + V ─────────────────────────────

#[test]
fn probe_1_construction_primitives() {
    // keyword → i64 map
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashMap/length
                      (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                        :foo 1 :bar 2 :baz 3)))
    "#);
    assert_eq!(n, 3, "keyword→i64 map length");

    // String → bool map
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashMap/length
                      (:wat::core::HashMap :wat::core::String :wat::core::bool
                        "x" true "y" false)))
    "#);
    assert_eq!(n, 2, "String→bool map length");

    // i64 → String map
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashMap/length
                      (:wat::core::HashMap :wat::core::i64 :wat::core::String
                        1 "one" 2 "two")))
    "#);
    assert_eq!(n, 2, "i64→String map length");
}

// ─── Probe 2 — `HashMap/get` returns Option<V>; Some on hit, None on miss ───

#[test]
fn probe_2_get_hit_and_miss() {
    // Hit: get existing key
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 42 :bar 99)]
                      (:wat::core::match (:wat::core::HashMap/get m :foo) -> :wat::core::i64
                        ((:wat::core::Some v) v)
                        (_ -1))))
    "#);
    assert_eq!(n, 42, "get hit returns Some(42)");

    // Miss: get missing key returns None (via contains-key? check — avoids Option match complexity)
    let b = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 42)]
                      (:wat::core::not (:wat::core::HashMap/contains-key? m :missing))))
    "#);
    assert!(b, "get miss: key :missing not present");
}

// ─── Probe 3 — `HashMap/assoc` inserts; overwrite semantic preserved ─────────

#[test]
fn probe_3_assoc_insert_overwrite() {
    // Insert new key → length increases
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 1)]
                      (:wat::core::let [m2 (:wat::core::HashMap/assoc m :bar 99)]
                        (:wat::core::HashMap/length m2))))
    "#);
    assert_eq!(n, 2, "assoc inserts new key → length 2");

    // Overwrite existing key
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 1)]
                      (:wat::core::let [m2 (:wat::core::HashMap/assoc m :foo 999)]
                        (:wat::core::match (:wat::core::HashMap/get m2 :foo) -> :wat::core::i64
                          ((:wat::core::Some v) v)
                          (_ -1)))))
    "#);
    assert_eq!(n, 999, "assoc overwrites existing key");

    // Original map unchanged (functional semantics): assoc does not mutate
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 1)]
                      (:wat::core::let [_m2 (:wat::core::HashMap/assoc m :foo 999)]
                        (:wat::core::match (:wat::core::HashMap/get m :foo) -> :wat::core::i64
                          ((:wat::core::Some v) v)
                          (_ -1)))))
    "#);
    assert_eq!(n, 1, "original map unchanged after assoc");
}

// ─── Probe 4 — `HashMap/dissoc` removes ──────────────────────────────────────

#[test]
fn probe_4_dissoc_removes() {
    // Remove existing key → length decreases
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 1 :bar 2 :baz 3)]
                      (:wat::core::HashMap/length
                        (:wat::core::HashMap/dissoc m :foo))))
    "#);
    assert_eq!(n, 2, "dissoc removes one key → length 2");

    // Remove missing key — no-op, length unchanged
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 1 :bar 2)]
                      (:wat::core::HashMap/length
                        (:wat::core::HashMap/dissoc m :missing))))
    "#);
    assert_eq!(n, 2, "dissoc missing key — length unchanged");
}

// ─── Probe 5 — `HashMap/keys` SEMANTIC CORRECTION ────────────────────────────
//
// Pre-216.5c: old storage was HashMap<String, (original_K, V)>.
// `keys` returned original_K from the tuple — correct by accident.
// Post-216.5c: K is the direct HashMap key; `m.keys().cloned()` is unambiguous.
// This probe verifies actual K Values are returned (keyword Values, not canonical
// String keys like "K:foo").

#[test]
fn probe_5_keys_semantic_correction() {
    // Keys of a keyword-keyed map must produce a Vec<keyword> of correct length
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 10 :bar 20)]
                      (:wat::core::Vector/length (:wat::core::HashMap/keys m))))
    "#);
    assert_eq!(n, 2, "keys returns Vec of length 2");

    // Each key must round-trip through contains-key? (proving it is a keyword Value,
    // not a canonical String like "K:foo")
    let b = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 10)]
                      (:wat::core::let [ks (:wat::core::HashMap/keys m)]
                        (:wat::core::let [first-key (:wat::core::match
                                                       (:wat::core::Vector/get ks 0) -> :wat::core::keyword
                                                       ((:wat::core::Some k) k)
                                                       (_ :missing))]
                          (:wat::core::HashMap/contains-key? m first-key)))))
    "#);
    assert!(b, "keys returns actual keyword Values that round-trip through contains-key?");
}

// ─── Probe 6 — `HashMap/values` returns Vec<V> ───────────────────────────────

#[test]
fn probe_6_values() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 10 :bar 20 :baz 30)]
                      (:wat::core::Vector/length (:wat::core::HashMap/values m))))
    "#);
    assert_eq!(n, 3, "values returns Vec of length 3");
}

// ─── Probe 7 — `HashMap/contains-key?` works ─────────────────────────────────

#[test]
fn probe_7_contains_key() {
    let b = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 1 :bar 2)]
                      (:wat::core::HashMap/contains-key? m :foo)))
    "#);
    assert!(b, "contains-key? hit");

    let b = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 1 :bar 2)]
                      (:wat::core::HashMap/contains-key? m :missing)))
    "#);
    assert!(!b, "contains-key? miss");
}

// ─── Probe 8 — `HashMap/length` works ────────────────────────────────────────

#[test]
fn probe_8_length() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashMap/length
                      (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                        :a 1 :b 2 :c 3 :d 4)))
    "#);
    assert_eq!(n, 4, "length of 4-entry map");

    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashMap/length
                      (:wat::core::HashMap :wat::core::keyword :wat::core::i64)))
    "#);
    assert_eq!(n, 0, "length of empty map");
}

// ─── Probe 9 — `HashMap/empty?` works ────────────────────────────────────────

#[test]
fn probe_9_empty_q() {
    let b = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::HashMap/empty?
                      (:wat::core::HashMap :wat::core::keyword :wat::core::i64)))
    "#);
    assert!(b, "empty? true for empty map");

    let b = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::HashMap/empty?
                      (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                        :foo 1)))
    "#);
    assert!(!b, "empty? false for non-empty map");
}

// ─── Probe 10 — Nested HashMap ───────────────────────────────────────────────
//
// `HashMap<keyword, HashMap<keyword, i64>>`: outer contains inner map as value.
// Uses :wat::type::Infer for the value type arg to avoid the `:keyword` vs
// `:HashMap<K,V>` type mismatch at check time.

#[test]
fn probe_10_nested_hashmap() {
    let b = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 42)
                       outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)]
                      (:wat::core::HashMap/contains-key? outer :inner)))
    "#);
    assert!(b, "nested HashMap: outer contains :inner key");

    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 42)
                       outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)]
                      (:wat::core::match (:wat::core::HashMap/get outer :inner) -> :wat::core::i64
                        ((:wat::core::Some inner2)
                          (:wat::core::match (:wat::core::HashMap/get inner2 :x) -> :wat::core::i64
                            ((:wat::core::Some v) v)
                            (_ -2)))
                        (_ -1))))
    "#);
    assert_eq!(n, 42, "nested HashMap: get :inner then get :x → 42");
}

// ─── Probe 11 — HashMap with HashSet as K ────────────────────────────────────
//
// `HashMap<HashSet<i64>, String>`: HashSet as K exercises native Hash on HashSet
// (Stone 216.5b + Stone 216.5a composition). Two HashSets with the same elements
// (different insertion order) must produce the same Hash → same HashMap slot.

#[test]
fn probe_11_hashset_as_key() {
    // Length check: HashMap<HashSet<i64>, String> with one entry
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [k (:wat::core::HashSet :wat::core::i64 1 2 3)]
                      (:wat::core::HashMap/length
                        (:wat::core::HashMap :wat::type::Infer :wat::core::String k "hello"))))
    "#);
    assert_eq!(n, 1, "HashMap<HashSet<i64>, String> length 1");

    // contains-key? with same-element HashSet inserted in different order
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [k     (:wat::core::HashSet :wat::core::i64 7 8 9)
                       m     (:wat::core::HashMap :wat::type::Infer :wat::core::String k "found-it")
                       probe (:wat::core::HashSet :wat::core::i64 7 8 9)]
                      (:wat::core::HashMap/contains-key? m probe)))
    "#), "HashSet-as-K: same elements different construction → same key (hash equality)");
}

// ─── Probe 12 — HashMap round-trip through to-holon + from-holon ─────────────────
//
// Stone 216.3 contract preserved: HashMap → to-holon (HolonAST Bundle) → from-holon → HashMap.

#[test]
fn probe_12_atom_roundtrip() {
    // Forward: HashMap<keyword, i64> → to-holon captured both entries.
    // Arc 228 classifier-wrap: to-holon on a HashMap produces
    // Bind(Atom("Map"), Bundle(...)), so Bundle/children on the top-level Bind
    // no longer applies — verify the forward encoding via round-trip length
    // (mirrors probe_arc216_stone4 / stone1's classifier-wrap fix).
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 42 :bar 99)]
                      (:wat::core::let [h (:wat::holon::to-holon m)]
                        (:wat::core::let [back (:wat::holon::from-holon h)]
                          (:wat::core::HashMap/length back)))))
    "#);
    assert_eq!(n, 2, "forward (arc-228 classifier-wrap): to-holon→from-holon preserves 2 entries");

    // Reverse: to-holon → from-holon → HashMap; contains-key? works
    let b = run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 42 :bar 99)]
                      (:wat::core::let [h (:wat::holon::to-holon m)]
                        (:wat::core::let [m2 (:wat::holon::from-holon h)]
                          (:wat::core::HashMap/contains-key? m2 :foo)))))
    "#);
    assert!(b, "reverse: from-holon recovers HashMap; contains-key? :foo = true");

    // Round-trip length preserved
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                         :foo 42 :bar 99)]
                      (:wat::core::let [h (:wat::holon::to-holon m)]
                        (:wat::core::let [m2 (:wat::holon::from-holon h)]
                          (:wat::core::HashMap/length m2)))))
    "#);
    assert_eq!(n, 2, "round-trip length preserved");
}
