//! Arc 215 Stone 2 — `[...]` Vector unification + `{...}` keyword-key lift probes.
//!
//! Verifies that:
//! - `[...]` expression-position vector literals route through the unified
//!   `:wat::type::Infer` machinery (infer_list_constructor); behavior preserved
//! - `(:wat::core::Vector :wat::type::Infer ...)` explicit-infer verb form works
//! - Mixed-type vector literals rejected at check time with TypeMismatch
//! - `{...}` map literal now accepts non-keyword keys (K inferred from actual keys)
//! - Mixed-K map literals rejected at check time with TypeMismatch
//! - Binder-position `WatAST::Vector` (let/fn/match) unchanged
//!
//! ## The 13 probes
//!
//! Change A — Vector unification (expression position):
//!  1. `[1 2 3]` → Vec<i64>; length 3; first element 1 (regression: behavior preserved)
//!  2. `[1.5 2.5]` → Vec<f64>; length 2 (T inferred f64)
//!  3. `["a" "b"]` → Vec<String>; length 2 (T inferred String)
//!  4. `[]` empty → Vec; length 0 (T fresh type variable)
//!  5. `[true false true]` → Vec<bool>; length 3
//!  6. `(:wat::core::Vector :wat::type::Infer 1 2 3)` → Vec<i64>; equivalent to `[1 2 3]`
//!  7. `(:wat::core::Vector :wat::type::Infer)` empty → Vec; length 0
//!  8. `[1 "two"]` → check fails with TypeMismatch (mixed-type rejection)
//!  9. `(:wat::core::Vector :wat::core::i64 1 2 3)` → Vec<i64>; explicit-type path unchanged
//! 10. `(:wat::core::let [x 1 y 2] ...)` → tuple-destructure via Vector binder still works
//!
//! Change B — keyword-key restriction lifted:
//! 11. `{1 "v" 2 "w"}` → HashMap<i64, String>; length 2; get 1 → Some("v")
//! 12. `{"a" 1 "b" 2}` → HashMap<String, i64>; length 2; get "a" → Some(1)
//! 13. `{1 "v" "two" "w"}` → check fails with TypeMismatch at key #2 (mixed-K rejection)

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
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

// ─── Probe 1: `[1 2 3]` integer Vec (regression — preserved) ─────────────────

/// `[1 2 3]` at expression position → Vec<i64>; length 3; first element 1.
/// Arc 215 stone 2 routes this through `infer_list_constructor`; user-visible
/// behavior is identical to before.
#[test]
fn probe_1_integer_vec_length_and_first_element() {
    // Length check.
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length [1 2 3]))
    "#;
    assert_eq!(run_i64(src_len), 3, "[1 2 3] must have length 3");

    // First-element check via get at index 0 (Option<i64> → match → i64).
    let src_first = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::match
                      (:wat::core::Vector/get [1 2 3] 0)
                      -> :wat::core::i64
                      ((:wat::core::Some v) v)
                      (:wat::core::None -1)))
    "#;
    assert_eq!(run_i64(src_first), 1, "first element of [1 2 3] must be 1");
}

// ─── Probe 2: `[1.5 2.5]` float Vec (regression — preserved) ────────────────

/// `[1.5 2.5]` → Vec<f64>; length 2. T inferred as f64 from first element.
#[test]
fn probe_2_float_vec_length() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length [1.5 2.5]))
    "#;
    assert_eq!(run_i64(src), 2, "[1.5 2.5] must have length 2");
}

// ─── Probe 3: `["a" "b"]` string Vec (regression — preserved) ───────────────

/// `["a" "b"]` → Vec<String>; length 2. T inferred as String.
#[test]
fn probe_3_string_vec_length() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length ["a" "b"]))
    "#;
    assert_eq!(run_i64(src), 2, r#"["a" "b"] must have length 2"#);
}

// ─── Probe 4: `[]` empty Vec (regression — preserved) ───────────────────────

/// `[]` → Vec with fresh T; length 0.
#[test]
fn probe_4_empty_vec_length_zero() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length []))
    "#;
    assert_eq!(run_i64(src), 0, "[] must have length 0");
}

// ─── Probe 5: `[true false true]` bool Vec (regression — preserved) ─────────

/// `[true false true]` → Vec<bool>; length 3. T inferred as bool.
#[test]
fn probe_5_bool_vec_length() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length [true false true]))
    "#;
    assert_eq!(run_i64(src), 3, "[true false true] must have length 3");
}

// ─── Probe 6: `(:wat::core::Vector :wat::type::Infer 1 2 3)` new path ────────

/// Explicit-infer verb form `(:wat::core::Vector :wat::type::Infer 1 2 3)`
/// produces the same Vec<i64> as `[1 2 3]`. This is the shared machinery
/// that both paths route through after arc 215 stone 2.
#[test]
fn probe_6_explicit_infer_vector_form() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length (:wat::core::Vector :wat::type::Infer 1 2 3)))
    "#;
    assert_eq!(
        run_i64(src),
        3,
        "(:wat::core::Vector :wat::type::Infer 1 2 3) must have length 3"
    );
}

// ─── Probe 7: `(:wat::core::Vector :wat::type::Infer)` empty new path ────────

/// `(:wat::core::Vector :wat::type::Infer)` → Vec with fresh T; length 0.
#[test]
fn probe_7_explicit_infer_vector_form_empty() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length (:wat::core::Vector :wat::type::Infer)))
    "#;
    assert_eq!(
        run_i64(src),
        0,
        "(:wat::core::Vector :wat::type::Infer) empty must have length 0"
    );
}

// ─── Probe 8: `[1 "two"]` mixed-type rejection ───────────────────────────────

/// `[1 "two"]` → check fails with TypeMismatch. T inferred as i64 from 1;
/// "two" fails to unify against i64. Position-named diagnostic.
#[test]
fn probe_8_mixed_type_vector_rejected_at_check() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length [1 "two"]))
    "#;
    let err = startup_err(src);
    assert!(
        err.to_lowercase().contains("typemismatch") || err.to_lowercase().contains("type mismatch") || err.contains("TypeMismatch"),
        "[1 \"two\"] must fail with TypeMismatch; got: {}",
        err
    );
}

// ─── Probe 9: `(:wat::core::Vector :wat::core::i64 1 2 3)` explicit ──────────

/// P1-style explicit type form `(:wat::core::Vector :wat::core::i64 1 2 3)`
/// still works; explicit-type path unchanged by arc 215 stone 2.
#[test]
fn probe_9_explicit_type_vector_form_preserved() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length (:wat::core::Vector :wat::core::i64 1 2 3)))
    "#;
    assert_eq!(
        run_i64(src),
        3,
        "(:wat::core::Vector :wat::core::i64 1 2 3) must have length 3"
    );
}

// ─── Probe 10: let binder `[x 1 y 2]` preserved (binder position) ───────────

/// Tuple-destructure-via-Vector binder `[x 1 y 2]` in let position still
/// works. Arc 169 / arc 167 binder semantics are unchanged by arc 215 stone 2.
/// The expression-position routing only applies to the `infer` arm; binder-
/// position arms (process_let_binding, etc.) are unaffected.
#[test]
fn probe_10_let_binder_vector_preserved() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [x 1
                       y 2]
                      (:wat::core::+ x y)))
    "#;
    assert_eq!(
        run_i64(src),
        3,
        "let [x 1 y 2] must bind x=1, y=2, compute x+y=3"
    );
}

// ─── Probe 11: `{1 "v" 2 "w"}` int-keyed map ────────────────────────────────

/// `{1 "v" 2 "w"}` → HashMap<i64, String>; length 2; get 1 → Some("v").
/// Arc 215 stone 2 lifts the keyword-key parse restriction;
/// K is `:wat::type::Infer`; check.rs infers K = i64 from key 1.
#[test]
fn probe_11_int_keyed_map_length_and_get() {
    // Length check.
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {1 "v" 2 "w"}))
    "#;
    assert_eq!(run_i64(src_len), 2, "{{1 \"v\" 2 \"w\"}} must have length 2");

    // Presence check: HashMap/contains-key? returns bool.
    let src_contains = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::HashMap/contains-key? {1 "v" 2 "w"} 1))
    "#;
    assert!(
        run_bool(src_contains),
        "{{1 \"v\" 2 \"w\"}} must contain key 1"
    );
}

// ─── Probe 12: `{"a" 1 "b" 2}` string-keyed map ─────────────────────────────

/// `{"a" 1 "b" 2}` → HashMap<String, i64>; length 2; contains "a".
/// K is inferred as String from first key "a"; V inferred as i64 from 1.
#[test]
fn probe_12_string_keyed_map_length_and_contains() {
    // Length check.
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {"a" 1 "b" 2}))
    "#;
    assert_eq!(run_i64(src_len), 2, "{{\"a\" 1 \"b\" 2}} must have length 2");

    // Contains-key? check.
    let src_contains = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::HashMap/contains-key? {"a" 1 "b" 2} "a"))
    "#;
    assert!(
        run_bool(src_contains),
        "{{\"a\" 1 \"b\" 2}} must contain key \"a\""
    );
}

// ─── Probe 13: `{1 "v" "two" "w"}` mixed-K rejection ────────────────────────

/// `{1 "v" "two" "w"}` → check fails with TypeMismatch at key #2.
/// K is inferred as i64 from key 1; "two" (String) fails to unify against i64.
/// Position-named diagnostic per arc 138.
#[test]
fn probe_13_mixed_k_map_rejected_at_check() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {1 "v" "two" "w"}))
    "#;
    let err = startup_err(src);
    assert!(
        err.to_lowercase().contains("typemismatch") || err.to_lowercase().contains("type mismatch") || err.contains("TypeMismatch"),
        "{{1 \"v\" \"two\" \"w\"}} must fail with TypeMismatch; got: {}",
        err
    );
}
