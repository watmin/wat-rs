//! Arc 214 P1 — HashMap constructor: Vector-symmetric shape probes.
//!
//! Verifies that the refactored `:wat::core::HashMap :K :V k0 v0 ...`
//! constructor shape (two separate type-keywords, per arc 109 slice 1f)
//! is accepted by both the runtime evaluator and the type-checker.
//!
//! ## The 9 probes
//!
//! 1. Empty literal — `(:wat::core::HashMap :wat::core::keyword :wat::core::i64)` constructs empty HashMap
//! 2. Single pair — length 1; get returns the value
//! 3. Multi pair — three pairs; length + get per key
//! 4. String-keyed — K = String confirms K can be any hashable type
//! 5. HolonAST-keyed — K = HolonAST confirms structural keys
//! 6. Wrong-type rejection — value type mismatch at type-check
//! 7. Odd count rejection — type-check catches arity parity error
//! 8. Missing K type-arg — `(:wat::core::HashMap)` fails arity check
//! 9. Missing V type-arg — `(:wat::core::HashMap :wat::core::keyword)` fails arity check

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// ─── Probe 1: Empty literal ──────────────────────────────────────────────────

#[test]
fn probe_p1_empty_literal_constructs_empty_hashmap() {
    // `(:wat::core::HashMap :wat::core::keyword :wat::core::i64)` — two type-args, zero pairs.
    // Must produce an empty HashMap (length 0).
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::length
            (:wat::core::HashMap :wat::core::keyword :wat::core::i64)))
    "#;
    assert_eq!(run_i64(src), 0, "empty HashMap must have length 0");
}

// ─── Probe 2: Single pair ────────────────────────────────────────────────────

#[test]
fn probe_p2_single_pair_length_and_get() {
    // `(:wat::core::HashMap :wat::core::keyword :wat::core::i64 :foo 42)` —
    // one key/value pair; length 1; get :foo returns Some(42).
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :foo 42)]
            (:wat::core::match (:wat::core::get m :foo) -> :wat::core::i64
              ((:wat::core::Some v) v)
              (:wat::core::None -1))))
    "#;
    assert_eq!(run_i64(src), 42, "get :foo should return 42");
}

// ─── Probe 3: Multi pair ─────────────────────────────────────────────────────

#[test]
fn probe_p3_multi_pair_length_and_get() {
    // Three key/value pairs — length 3 and get :b returns 20.
    // Length check first (uses length); then get from same map.
    let src_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::length
            (:wat::core::HashMap :wat::core::keyword :wat::core::i64
              :a 1
              :b 2
              :c 3)))
    "#;
    assert_eq!(run_i64(src_len), 3, "three pairs → length 3");
    let src_get = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                  :a 10
                  :b 20
                  :c 30)]
            (:wat::core::match (:wat::core::get m :b) -> :wat::core::i64
              ((:wat::core::Some v) v)
              (:wat::core::None -1))))
    "#;
    assert_eq!(run_i64(src_get), 20, "get :b from three-pair map → 20");
}

// ─── Probe 4: String-keyed ───────────────────────────────────────────────────

#[test]
fn probe_p4_string_keyed_constructs_correctly() {
    // K = :wat::core::String confirms K can be any hashable type, not just Keyword.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m (:wat::core::HashMap :wat::core::String :wat::core::i64
                  "a" 1
                  "b" 2)]
            (:wat::core::match (:wat::core::get m "b") -> :wat::core::i64
              ((:wat::core::Some v) v)
              (:wat::core::None -1))))
    "#;
    assert_eq!(run_i64(src), 2, "String-keyed HashMap: get \"b\" → 2");
}

// ─── Probe 5: HolonAST-keyed ─────────────────────────────────────────────────

#[test]
fn probe_p5_holonast_keyed_length() {
    // K = :wat::holon::HolonAST — structural values as keys.
    // Uses length to avoid complex match on HolonAST values.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::length
            (:wat::core::HashMap :wat::holon::HolonAST :wat::holon::HolonAST
              (:wat::holon::to-holon 42) (:wat::holon::to-holon "answer"))))
    "#;
    assert_eq!(run_i64(src), 1, "HolonAST-keyed HashMap with one pair → length 1");
}

// ─── Probe 6: Wrong-type rejection ───────────────────────────────────────────

#[test]
fn probe_p6_wrong_value_type_rejected_at_type_check() {
    // `(:wat::core::HashMap :wat::core::keyword :wat::core::i64 :foo "not-an-i64")` —
    // "not-an-i64" is a String, not i64 — must fail type-check (startup_from_source fails).
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                  :foo "not-an-i64")]
            (:wat::core::match (:wat::core::get m :foo) -> :wat::core::i64
              ((:wat::core::Some v) v)
              (:wat::core::None -1))))
    "#;
    let err = startup_err(src);
    assert!(
        err.to_lowercase().contains("mismatch") || err.to_lowercase().contains("type"),
        "wrong-value type must produce a type-check error; got: {}",
        err
    );
}

// ─── Probe 7: Odd count rejection ────────────────────────────────────────────

#[test]
fn probe_p7_odd_pair_count_rejected() {
    // `(:wat::core::HashMap :wat::core::keyword :wat::core::i64 :foo)` —
    // one value arg after the two type-args: length 1 is odd. Must fail type-check.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :foo)]
            0))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("even") || err.contains("MalformedForm"),
        "odd pair count must produce the 'even' arity error; got: {}",
        err
    );
}

// ─── Probe 8: Zero type-args (arity error) ───────────────────────────────────

#[test]
fn probe_p8_missing_both_type_args_rejected() {
    // `(:wat::core::HashMap)` — zero args, must fail arity check.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m (:wat::core::HashMap)]
            0))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("ArityMismatch") || err.contains("arity") || err.contains("2"),
        "missing type args must produce an arity error; got: {}",
        err
    );
}

// ─── Probe 9: Missing V type-arg ─────────────────────────────────────────────

#[test]
fn probe_p9_missing_v_type_arg_rejected() {
    // `(:wat::core::HashMap :wat::core::keyword)` — only K given, V missing.
    // Must fail arity check (args.len() == 1 < 2).
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [m (:wat::core::HashMap :wat::core::keyword)]
            0))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("ArityMismatch") || err.contains("arity") || err.contains("2"),
        "missing V type arg must produce an arity error; got: {}",
        err
    );
}
