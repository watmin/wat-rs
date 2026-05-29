//! Arc 215 Stone 1 — `_infer` placeholder + literal completion probes.
//!
//! Verifies that:
//! - `{...}` map literals use `:wat::type::Infer` for V; type inferred from values
//! - `#{...}` set literals desugar to `(:wat::core::HashSet :wat::type::Infer ...)`
//! - The type-checker correctly infers concrete types from first element/value
//! - Mixed-type literals are rejected at check time with TypeMismatch
//! - Nested collections work without Atom auto-wrap (resolves P2 Probe 5 class)
//!
//! ## The 12 probes
//!
//! `{...}` probes (extending P2 coverage with inferred types):
//! 1. Single pair with inferred V: `{:foo 42}` → length 1; V inferred as i64
//! 2. Multi pair with inferred V: `{:a 1 :b 2 :c 3}` → length 3; contains :b
//! 3. String-valued map: `{:a "hello" :b "world"}` → length 2; V inferred as String
//! 4. Nested map (Probe 5 resolution): `{:outer {:inner 42}}` → outer length 1;
//!    get :outer returns inner-map; inner length 1; succeeds at runtime
//! 5. Mixed-value-type rejection: `{:a 1 :b "two"}` → TypeMismatch at check
//! 6. Empty literal: `{}` → length 0; type-check passes with fresh K, V
//!
//! `#{...}` probes (new parser dispatch):
//! 7. Empty set: `#{}` → length 0
//! 8. Single element: `#{42}` → length 1; contains 42
//! 9. Multi element: `#{1 2 3}` → length 3; contains 2
//! 10. Dedup at construction: `#{1 1 2 2 3}` → length 3
//! 11. Mixed-type rejection: `#{1 :foo "x"}` → TypeMismatch at check
//!
//! Cross-literal:
//! 12. Map of sets: `{:a #{1 2} :b #{3 4}}` → outer V = HashSet<i64>;
//!     both inner sets have length 2

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

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1: Single pair with inferred V ─────────────────────────────────────

/// `{:foo 42}` desugars to `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer :foo 42)`.
/// V is inferred as :wat::core::i64 from the literal 42.
/// Values are NOT Atom-wrapped; the i64 is stored directly.
/// Length = 1; contains :foo.
#[test]
fn probe_1_single_pair_inferred_v_i64() {
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:foo 42}))
    "#;
    assert_eq!(run_i64(src_len), 1, "single-pair inferred map must have length 1");

    let src_contains = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::HashMap/contains-key? {:foo 42} :foo))
    "#;
    assert!(
        run_bool(src_contains),
        "single-pair inferred map must contain :foo"
    );
}

// ─── Probe 2: Multi pair with inferred V ─────────────────────────────────────

/// `{:a 1 :b 2 :c 3}` → length 3; all three keys present. V = i64 inferred from 1.
/// Subsequent values 2 and 3 unify against i64.
#[test]
fn probe_2_multi_pair_inferred_v_i64() {
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:a 1 :b 2 :c 3}))
    "#;
    assert_eq!(run_i64(src_len), 3, "three-pair inferred map must have length 3");

    let src_get = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [m {:a 1 :b 2 :c 3}]
                      (:wat::core::match (:wat::core::get m :b) -> :wat::core::i64
                        ((:wat::core::Some v) v)
                        (:wat::core::None -1))))
    "#;
    assert_eq!(run_i64(src_get), 2, "get :b from {{:a 1 :b 2 :c 3}} must return 2");
}

// ─── Probe 3: String-valued map ───────────────────────────────────────────────

/// `{:a "hello" :b "world"}` → length 2; V inferred as :wat::core::String.
/// Values are stored directly as String (no Atom wrap).
#[test]
fn probe_3_string_valued_map_inferred_v() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:a "hello" :b "world"}))
    "#;
    assert_eq!(run_i64(src), 2, "string-valued inferred map must have length 2");
}

// ─── Probe 4: Nested map (arc 215 resolves P2 Probe 5 class) ─────────────────

/// `{:outer {:inner 42}}` — the inner `{:inner 42}` evaluates to a
/// `HashMap<keyword, i64>` value (V inferred as i64).
/// The outer V is inferred as `HashMap<keyword, i64>`.
/// No Atom wrap at any level — Probe 5's runtime failure class eliminated.
///
/// Outer length = 1. Get :outer returns the inner map; inner length = 1.
#[test]
fn probe_4_nested_map_literal_resolved() {
    // Outer length = 1.
    let src_outer_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:outer {:inner 42}}))
    "#;
    assert_eq!(
        run_i64(src_outer_len),
        1,
        "outer map of nested literal must have length 1"
    );

    // Get :outer from outer map; call length on inner map.
    let src_inner_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [outer {:outer {:inner 42}}]
                      (:wat::core::match (:wat::core::get outer :outer) -> :wat::core::i64
                        ((:wat::core::Some inner-map) (:wat::core::length inner-map))
                        (:wat::core::None -1))))
    "#;
    assert_eq!(
        run_i64(src_inner_len),
        1,
        "inner map retrieved from nested literal must have length 1"
    );
}

// ─── Probe 5: Mixed-value-type rejection ─────────────────────────────────────

/// `{:a 1 :b "two"}` — first value 1 is i64; second value "two" is String.
/// Type-checker infers V = i64 from 1, then fails to unify String against it.
/// Must fail at CHECK time (startup fails) with TypeMismatch.
#[test]
fn probe_5_mixed_value_types_rejected_at_check() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:a 1 :b "two"}))
    "#;
    let err = startup_err(src);
    assert!(
        err.to_lowercase().contains("mismatch") || err.to_lowercase().contains("type"),
        "mixed-value-type map must produce TypeMismatch at check; got: {}",
        err
    );
}

// ─── Probe 6: Empty `{}` ─────────────────────────────────────────────────────

/// `{}` desugars to `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer)`.
/// V stays as a fresh type variable (no values to infer from).
/// Type-check passes; runtime produces a length-0 HashMap.
#[test]
fn probe_6_empty_map_literal_length_zero() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {}))
    "#;
    assert_eq!(run_i64(src), 0, "empty map literal must have length 0");
}

// ─── Probe 7: Empty `#{}` ────────────────────────────────────────────────────

/// `#{}` desugars to `(:wat::core::HashSet :wat::type::Infer)`.
/// T stays as a fresh type variable. Type-check passes; length = 0.
#[test]
fn probe_7_empty_set_literal_length_zero() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length #{}))
    "#;
    assert_eq!(run_i64(src), 0, "empty set literal must have length 0");
}

// ─── Probe 8: Single element `#{42}` ─────────────────────────────────────────

/// `#{42}` desugars to `(:wat::core::HashSet :wat::type::Infer 42)`.
/// T inferred as i64. Length = 1; contains 42.
#[test]
fn probe_8_single_element_set() {
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length #{42}))
    "#;
    assert_eq!(run_i64(src_len), 1, "single-element set literal must have length 1");

    let src_contains = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::contains? #{42} 42))
    "#;
    assert!(
        run_bool(src_contains),
        "single-element set literal must contain 42"
    );
}

// ─── Probe 9: Multi element `#{1 2 3}` ───────────────────────────────────────

/// `#{1 2 3}` — T inferred as i64. Length = 3; contains 2.
#[test]
fn probe_9_multi_element_set() {
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length #{1 2 3}))
    "#;
    assert_eq!(run_i64(src_len), 3, "three-element set literal must have length 3");

    let src_contains = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::contains? #{1 2 3} 2))
    "#;
    assert!(
        run_bool(src_contains),
        "three-element set must contain 2"
    );
}

// ─── Probe 10: Dedup at construction `#{1 1 2 2 3}` ─────────────────────────

/// `#{1 1 2 2 3}` — duplicate elements collapse at construction.
/// Length = 3 (three distinct values: 1, 2, 3).
/// T inferred as i64 from the first element.
#[test]
fn probe_10_set_literal_dedup_at_construction() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length #{1 1 2 2 3}))
    "#;
    assert_eq!(
        run_i64(src),
        3,
        "duplicate elements must collapse at set construction; length must be 3"
    );
}

// ─── Probe 11: Mixed-element-type rejection ───────────────────────────────────

/// `#{1 :foo "x"}` — first element 1 is i64; second :foo is keyword; third "x" is String.
/// T inferred as i64 from 1; :foo fails to unify against i64.
/// Must fail at CHECK time with TypeMismatch.
#[test]
fn probe_11_mixed_element_types_rejected_at_check() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length #{1 :foo "x"}))
    "#;
    let err = startup_err(src);
    assert!(
        err.to_lowercase().contains("mismatch") || err.to_lowercase().contains("type"),
        "mixed-element-type set must produce TypeMismatch at check; got: {}",
        err
    );
}

// ─── Probe 12: Map of sets ────────────────────────────────────────────────────

/// `{:a #{1 2} :b #{3 4}}` — outer V inferred as HashSet<i64>.
/// Both inner sets have length 2.
/// Outer map has length 2.
#[test]
fn probe_12_map_of_sets() {
    // Outer map length = 2.
    let src_outer_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:a #{1 2} :b #{3 4}}))
    "#;
    assert_eq!(
        run_i64(src_outer_len),
        2,
        "map of sets must have outer length 2"
    );

    // Get :a; inner set length = 2.
    let src_inner_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [m {:a #{1 2} :b #{3 4}}]
                      (:wat::core::match (:wat::core::get m :a) -> :wat::core::i64
                        ((:wat::core::Some s) (:wat::core::length s))
                        (:wat::core::None -1))))
    "#;
    assert_eq!(
        run_i64(src_inner_len),
        2,
        "inner set #{{1 2}} retrieved from map must have length 2"
    );
}
