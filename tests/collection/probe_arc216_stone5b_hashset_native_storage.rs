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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// ─── Probe 1 — Construction with primitive elements ──────────────────────────

#[test]
fn probe_1_construction_primitives() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p1a-i64-set-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "i64 set length"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p1b-str-set-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "String set length"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p1c-bool-set-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "bool set length"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p1d-kw-set-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "keyword set length"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2 — `HashSet/contains?` ──────────────────────────────────────────

#[test]
fn probe_2_contains_q_hit_and_miss() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p2a-contains-i64-hit)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "i64 hit"),
        other => panic!("expected bool; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p2b-contains-i64-miss)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(!b, "i64 miss"),
        other => panic!("expected bool; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p2c-contains-str-hit)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "String hit"),
        other => panic!("expected bool; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p2d-contains-str-miss)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(!b, "String miss"),
        other => panic!("expected bool; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p2e-contains-kw-hit)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "keyword hit"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 3 — `HashSet/length` ──────────────────────────────────────────────

#[test]
fn probe_3_length() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p3-length)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 5),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4 — `HashSet/empty?` ──────────────────────────────────────────────

#[test]
fn probe_4_empty_q() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p4a-nonempty)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(!b, "non-empty is false"),
        other => panic!("expected bool; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p4b-dedup-nonempty)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(!b, "dedupe still non-empty"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 5 — `HashSet/conj` ────────────────────────────────────────────────

#[test]
fn probe_5_conj_and_dedupe() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p5a-conj-add)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "conj adds new element"),
        other => panic!("expected bool; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p5b-conj-dup)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "conj duplicate is idempotent"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p5c-conj-immutable)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(!b, "conj does not mutate input"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 6 — conj-then-contains? for bool elements ─────────────────────────

#[test]
fn probe_6_conj_bool_elements() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p6a-conj-bool-false)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "conj false into set-with-true and find it"),
        other => panic!("expected bool; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p6b-conj-bool-dedup)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "conj of already-present bool element: length stays 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7 — Nested HashSet<HashSet<i64>> ──────────────────────────────────

#[test]
fn probe_7_nested_hashset() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p7a-nested-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "outer HashSet has 2 inner sets"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p7b-nested-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "inner HashSet found by value equality"),
        other => panic!("expected bool; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p7c-nested-dedup)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "duplicate inner HashSet deduped"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 8 — HashSet round-trip through to-holon + from-holon ──────────────────

#[test]
fn probe_8_atom_round_trip() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p8a-rt-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "round-trip preserves length"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p8b-rt-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "round-trip preserves membership"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 9 — HashSet as VALUE inside a HashMap ─────────────────────────────

#[test]
fn probe_9_hashset_as_hashmap_value() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p9-hashset-as-hm-val)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "HashSet value retrieved from HashMap and membership verified"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 10 — HashSet as KEY inside a HashMap ──────────────────────────────

#[test]
fn probe_10_hashset_as_hashmap_key() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p10-hashset-as-hm-key)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "HashSet key found via HashMap/contains-key? (same elements = same canonical key)"),
        other => panic!("expected bool; got {:?}", other),
    }
}
