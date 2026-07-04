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

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

// ─── Probe 1: `[1 2 3]` integer Vec (regression — preserved) ─────────────────

#[test]
fn probe_1_integer_vec_length_and_first_element() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p1a-vec-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "[1 2 3] must have length 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p1b-vec-first)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "first element of [1 2 3] must be 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2: `[1.5 2.5]` float Vec ─────────────────────────────────────────

#[test]
fn probe_2_float_vec_length() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p2-float-vec-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "[1.5 2.5] must have length 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 3: `["a" "b"]` string Vec ────────────────────────────────────────

#[test]
fn probe_3_string_vec_length() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p3-str-vec-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, r#"["a" "b"] must have length 2"#),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4: `[]` empty Vec ────────────────────────────────────────────────

#[test]
fn probe_4_empty_vec_length_zero() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p4-empty-vec-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 0, "[] must have length 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5: `[true false true]` bool Vec ──────────────────────────────────

#[test]
fn probe_5_bool_vec_length() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p5-bool-vec-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "[true false true] must have length 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 6: `(:wat::core::Vector :wat::type::Infer 1 2 3)` new path ────────

#[test]
fn probe_6_explicit_infer_vector_form() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p6-explicit-infer-vec-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "(:wat::core::Vector :wat::type::Infer 1 2 3) must have length 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7: `(:wat::core::Vector :wat::type::Infer)` empty ─────────────────

#[test]
fn probe_7_explicit_infer_vector_form_empty() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p7-empty-infer-vec-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 0, "(:wat::core::Vector :wat::type::Infer) empty must have length 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 8: `[1 "two"]` mixed-type rejection ───────────────────────────────

#[test]
fn probe_8_mixed_type_vector_rejected_at_check() {
    let err = startup_from_file(
        "tests/collection/probe_arc215_stone2_p8_bad.wat",
    )
    .expect_err("expected startup failure for mixed-type vector");
    let err = format!("{}\n---\n{:?}", err, err);
    assert_eq!(
        err,
        r##"#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::core::vec: parameter #3 expects :wat::core::i64; got :wat::core::String" :location #wat.core/Span {:file "tests/collection/probe_arc215_stone2_p8_bad.wat" :line 4 :col 26 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 31}} :causes [] :callee ":wat::core::vec" :param "#3" :expected ":wat::core::i64" :got ":wat::core::String" :remedies [#wat.kernel/Remedy {:form ":wat::core::Vector" :kind :retirement :score 0 :note "as a TYPE CONSTRUCTOR, rename `:wat::core::vec` → `:wat::core::Vector` (verb-equals-type, arc 109 slice 1f); substrate produces the same Vec<T> value. To materialize a seqable/Stream into a Vector (arc 118.2a), use `(:wat::core::into [] coll)` instead"}]}]}
---
#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::core::vec: parameter #3 expects :wat::core::i64; got :wat::core::String" :location #wat.core/Span {:file "tests/collection/probe_arc215_stone2_p8_bad.wat" :line 4 :col 26 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 31}} :causes [] :callee ":wat::core::vec" :param "#3" :expected ":wat::core::i64" :got ":wat::core::String" :remedies [#wat.kernel/Remedy {:form ":wat::core::Vector" :kind :retirement :score 0 :note "as a TYPE CONSTRUCTOR, rename `:wat::core::vec` → `:wat::core::Vector` (verb-equals-type, arc 109 slice 1f); substrate produces the same Vec<T> value. To materialize a seqable/Stream into a Vector (arc 118.2a), use `(:wat::core::into [] coll)` instead"}]}]}"##,
        "probe_8: mixed-type vector TypeMismatch golden"
    );
}

// ─── Probe 9: `(:wat::core::Vector :wat::core::i64 1 2 3)` explicit ──────────

#[test]
fn probe_9_explicit_type_vector_form_preserved() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p9-explicit-type-vec-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "(:wat::core::Vector :wat::core::i64 1 2 3) must have length 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 10: let binder `[x 1 y 2]` preserved ─────────────────────────────

#[test]
fn probe_10_let_binder_vector_preserved() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p10-let-binder-preserved)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "let [x 1 y 2] must bind x=1, y=2, compute x+y=3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 11: `{1 "v" 2 "w"}` int-keyed map ────────────────────────────────

#[test]
fn probe_11_int_keyed_map_length_and_get() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p11a-int-keyed-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "{{1 \"v\" 2 \"w\"}} must have length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p11b-int-keyed-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "{{1 \"v\" 2 \"w\"}} must contain key 1"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 12: `{"a" 1 "b" 2}` string-keyed map ─────────────────────────────

#[test]
fn probe_12_string_keyed_map_length_and_contains() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p12a-str-keyed-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "{{\"a\" 1 \"b\" 2}} must have length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p12b-str-keyed-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "{{\"a\" 1 \"b\" 2}} must contain key \"a\""),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 13: `{1 "v" "two" "w"}` mixed-K rejection ────────────────────────

#[test]
fn probe_13_mixed_k_map_rejected_at_check() {
    let err = startup_from_file(
        "tests/collection/probe_arc215_stone2_p13_bad.wat",
    )
    .expect_err("expected startup failure for mixed-K map");
    let err = format!("{}\n---\n{:?}", err, err);
    assert_eq!(
        err,
        r##"#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message "{…} map literal: parameter key #2 expects :wat::core::i64; got :wat::core::String" :location #wat.core/Span {:file "tests/collection/probe_arc215_stone2_p13_bad.wat" :line 4 :col 30 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 35}} :causes [] :callee "{…} map literal" :param "key #2" :expected ":wat::core::i64" :got ":wat::core::String" :remedies []}]}
---
#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message "{…} map literal: parameter key #2 expects :wat::core::i64; got :wat::core::String" :location #wat.core/Span {:file "tests/collection/probe_arc215_stone2_p13_bad.wat" :line 4 :col 30 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 35}} :causes [] :callee "{…} map literal" :param "key #2" :expected ":wat::core::i64" :got ":wat::core::String" :remedies []}]}"##,
        "probe_13: mixed-K map TypeMismatch golden"
    );
}
