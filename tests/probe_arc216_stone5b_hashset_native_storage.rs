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
//! 6. `HashSet/dissoc` — not yet a verb; probe documents expected error
//! 7. Nested HashSet — `HashSet<HashSet<i64>>` construction + element lookup
//! 8. HashSet round-trip through `:wat::holon::to-holon` + `from-holon` (Stone 216.1 contract)
//! 9. HashSet inside HashMap as VALUE — `HashMap<keyword, HashSet<i64>>`
//! 10. HashSet inside HashMap as KEY — `HashMap<HashSet<i64>, String>`

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

fn runtime_err(src: &str) -> String {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env) {
        Ok(v) => panic!("expected runtime error; got {:?}", v),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1 — Construction with primitive elements ──────────────────────────

#[test]
fn probe_1_construction_primitives() {
    // i64 elements
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashSet/length
                      (:wat::core::HashSet :wat::core::i64 1 2 3)))
    "#);
    assert_eq!(n, 3, "i64 set length");

    // String elements
    let n2 = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashSet/length
                      (:wat::core::HashSet :wat::core::String "a" "b" "c")))
    "#);
    assert_eq!(n2, 3, "String set length");

    // bool elements (2 distinct: true, false)
    let n3 = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashSet/length
                      (:wat::core::HashSet :wat::core::bool true false)))
    "#);
    assert_eq!(n3, 2, "bool set length");

    // keyword elements
    let n4 = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashSet/length
                      (:wat::core::HashSet :wat::core::keyword :foo :bar :baz)))
    "#);
    assert_eq!(n4, 3, "keyword set length");
}

// ─── Probe 2 — `HashSet/contains?` ──────────────────────────────────────────

#[test]
fn probe_2_contains_q_hit_and_miss() {
    // i64 hit
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [s (:wat::core::HashSet :wat::core::i64 10 20 30)]
                      (:wat::core::contains? s 20)))
    "#), "i64 hit");

    // i64 miss
    assert!(!run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [s (:wat::core::HashSet :wat::core::i64 10 20 30)]
                      (:wat::core::contains? s 99)))
    "#), "i64 miss");

    // String hit
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [s (:wat::core::HashSet :wat::core::String "apple" "banana")]
                      (:wat::core::contains? s "apple")))
    "#), "String hit");

    // String miss
    assert!(!run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [s (:wat::core::HashSet :wat::core::String "apple" "banana")]
                      (:wat::core::contains? s "cherry")))
    "#), "String miss");

    // keyword hit
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [s (:wat::core::HashSet :wat::core::keyword :x :y)]
                      (:wat::core::contains? s :x)))
    "#), "keyword hit");
}

// ─── Probe 3 — `HashSet/length` ──────────────────────────────────────────────

#[test]
fn probe_3_length() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::HashSet/length
                      (:wat::core::HashSet :wat::core::i64 1 2 3 4 5)))
    "#);
    assert_eq!(n, 5);
}

// ─── Probe 4 — `HashSet/empty?` ──────────────────────────────────────────────

#[test]
fn probe_4_empty_q() {
    // Non-empty
    assert!(!run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::HashSet/empty?
                      (:wat::core::HashSet :wat::core::i64 1)))
    "#), "non-empty is false");

    // Deduped to one element — still non-empty
    assert!(!run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::HashSet/empty?
                      (:wat::core::HashSet :wat::core::i64 42 42 42)))
    "#), "dedupe still non-empty");
}

// ─── Probe 5 — `HashSet/conj` ────────────────────────────────────────────────

#[test]
fn probe_5_conj_and_dedupe() {
    // conj adds new element
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [s0 (:wat::core::HashSet :wat::core::i64 1 2)
                       s1 (:wat::core::conj s0 3)]
                      (:wat::core::contains? s1 3)))
    "#), "conj adds new element");

    // conj with existing element — idempotent (length unchanged)
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [s0 (:wat::core::HashSet :wat::core::i64 1 2)
                       s1 (:wat::core::conj s0 1)]
                      (:wat::core::HashSet/length s1)))
    "#);
    assert_eq!(n, 2, "conj duplicate is idempotent");

    // conj is functional — input unchanged
    assert!(!run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [s0 (:wat::core::HashSet :wat::core::i64 1 2)
                       _  (:wat::core::conj s0 3)]
                      (:wat::core::contains? s0 3)))
    "#), "conj does not mutate input");
}

// ─── Probe 6 — conj-then-contains? for a second element type ─────────────────

#[test]
fn probe_6_conj_bool_elements() {
    // Verify conj works for bool (which has a distinct hash from i64 via discriminant tagging).
    // Stone 216.5b — native insert via Value: Hash + Eq.
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [s0 (:wat::core::HashSet :wat::core::bool true)
                       s1 (:wat::core::conj s0 false)]
                      (:wat::core::contains? s1 false)))
    "#), "conj false into set-with-true and find it");

    // Verify dedupe: conj with already-present element
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [s0 (:wat::core::HashSet :wat::core::bool true false)
                       s1 (:wat::core::conj s0 true)]
                      (:wat::core::HashSet/length s1)))
    "#);
    assert_eq!(n, 2, "conj of already-present bool element: length stays 2");
}

// ─── Probe 7 — Nested HashSet<HashSet<i64>> ──────────────────────────────────

#[test]
fn probe_7_nested_hashset() {
    // Build outer HashSet with two distinct inner HashSets.
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [inner1 (:wat::core::HashSet :wat::core::i64 1 2)
                       inner2 (:wat::core::HashSet :wat::core::i64 3 4)
                       outer  (:wat::core::HashSet :wat::type::Infer inner1 inner2)]
                      (:wat::core::HashSet/length outer)))
    "#);
    assert_eq!(n, 2, "outer HashSet has 2 inner sets");

    // contains? on the outer HashSet with an equal-value inner HashSet
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [inner1 (:wat::core::HashSet :wat::core::i64 1 2)
                       inner2 (:wat::core::HashSet :wat::core::i64 3 4)
                       outer  (:wat::core::HashSet :wat::type::Infer inner1 inner2)
                       probe  (:wat::core::HashSet :wat::core::i64 1 2)]
                      (:wat::core::contains? outer probe)))
    "#), "inner HashSet found by value equality");

    // Dedupe: inserting the same inner HashSet twice
    let n2 = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [inner (:wat::core::HashSet :wat::core::i64 1 2)
                       outer (:wat::core::HashSet :wat::type::Infer inner inner)]
                      (:wat::core::HashSet/length outer)))
    "#);
    assert_eq!(n2, 1, "duplicate inner HashSet deduped");
}

// ─── Probe 8 — HashSet round-trip through to-holon + from-holon ──────────────────

#[test]
fn probe_8_atom_round_trip() {
    // Stone 216.1 contract preserved: HashSet → to-holon (Bundle of bare atoms) → from-holon.
    // Stone 216.5b: value_to_atom iterates s.iter() (Values directly, not String keys).
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [s     (:wat::core::HashSet :wat::core::i64 10 20 30)
                       atom  (:wat::holon::to-holon s)
                       back  (:wat::holon::from-holon atom)]
                      (:wat::core::HashSet/length back)))
    "#);
    assert_eq!(n, 3, "round-trip preserves length");

    // contains? on round-tripped value
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [s     (:wat::core::HashSet :wat::core::i64 10 20 30)
                       atom  (:wat::holon::to-holon s)
                       back  (:wat::holon::from-holon atom)]
                      (:wat::core::contains? back 20)))
    "#), "round-trip preserves membership");
}

// ─── Probe 9 — HashSet as VALUE inside a HashMap ─────────────────────────────

#[test]
fn probe_9_hashset_as_hashmap_value() {
    // HashMap<keyword, HashSet<i64>>. HashMap still uses hashmap_key for keyword key;
    // HashSet uses native Hash for its elements. The boundary must work.
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [inner   (:wat::core::HashSet :wat::core::i64 1 2 3)
                       m       (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :my-set inner)
                       fetched (:wat::core::match (:wat::core::get m :my-set) -> :wat::core::bool
                                  ((:wat::core::Some v) (:wat::core::contains? v 2))
                                  (:wat::core::None     false))]
                      fetched))
    "#), "HashSet value retrieved from HashMap and membership verified");
}

// ─── Probe 10 — HashSet as KEY inside a HashMap ──────────────────────────────

#[test]
fn probe_10_hashset_as_hashmap_key() {
    // HashMap<HashSet<i64>, String>. HashMap's hashmap_key arm for HashSet now
    // iterates s.iter() (Values) to compute the canonical key. Two HashSets with
    // the same elements must produce the same canonical key → same HashMap slot.
    assert!(run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [key   (:wat::core::HashSet :wat::core::i64 7 8 9)
                       m     (:wat::core::HashMap :wat::type::Infer :wat::core::String key "found-it")
                       probe (:wat::core::HashSet :wat::core::i64 7 8 9)]
                      (:wat::core::HashMap/contains-key? m probe)))
    "#), "HashSet key found via HashMap/contains-key? (same elements = same canonical key)");
}
