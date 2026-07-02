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

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

// ─── Probe 1: Single pair with inferred V ─────────────────────────────────────

#[test]
fn probe_1_single_pair_inferred_v_i64() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p1a-map-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "single-pair inferred map must have length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p1b-map-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "single-pair inferred map must contain :foo"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 2: Multi pair with inferred V ─────────────────────────────────────

#[test]
fn probe_2_multi_pair_inferred_v_i64() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p2a-map-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "three-pair inferred map must have length 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p2b-map-get-b)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "get :b from {{:a 1 :b 2 :c 3}} must return 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 3: String-valued map ───────────────────────────────────────────────

#[test]
fn probe_3_string_valued_map_inferred_v() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p3-string-map-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "string-valued inferred map must have length 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4: Nested map (arc 215 resolves P2 Probe 5 class) ─────────────────

#[test]
fn probe_4_nested_map_literal_resolved() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p4a-nested-map-outer-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "outer map of nested literal must have length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p4b-nested-map-inner-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "inner map retrieved from nested literal must have length 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5: Mixed-value-type rejection ─────────────────────────────────────

#[test]
fn probe_5_mixed_value_types_rejected_at_check() {
    let err = startup_from_file(
        "tests/collection/probe_arc215_collection_literal_inference_p5_bad.wat",
    )
    .expect_err("expected startup failure for mixed-value-type map");
    let err = format!("{}\n---\n{:?}", err, err);
    assert_eq!(
        err,
        // Stone B (arc 296): Display+Debug now emit EDN. Golden recaptured.
        r##"#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message "{…} map literal: parameter value #2 expects :wat::core::i64; got :wat::core::String" :location #wat.core/Span {:file "tests/collection/probe_arc215_collection_literal_inference_p5_bad.wat" :line 4 :col 32 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 37}} :causes [] :callee "{…} map literal" :param "value #2" :expected ":wat::core::i64" :got ":wat::core::String" :remedies []}]}
---
#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message "{…} map literal: parameter value #2 expects :wat::core::i64; got :wat::core::String" :location #wat.core/Span {:file "tests/collection/probe_arc215_collection_literal_inference_p5_bad.wat" :line 4 :col 32 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 37}} :causes [] :callee "{…} map literal" :param "value #2" :expected ":wat::core::i64" :got ":wat::core::String" :remedies []}]}"##,
        "probe_5: mixed-value-type map TypeMismatch golden"
    );
}

// ─── Probe 6: Empty `{}` ─────────────────────────────────────────────────────

#[test]
fn probe_6_empty_map_literal_length_zero() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p6-empty-map-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 0, "empty map literal must have length 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7: Empty `#{}` ────────────────────────────────────────────────────

#[test]
fn probe_7_empty_set_literal_length_zero() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p7-empty-set-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 0, "empty set literal must have length 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 8: Single element `#{42}` ─────────────────────────────────────────

#[test]
fn probe_8_single_element_set() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p8a-single-set-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "single-element set literal must have length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p8b-single-set-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "single-element set literal must contain 42"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 9: Multi element `#{1 2 3}` ───────────────────────────────────────

#[test]
fn probe_9_multi_element_set() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p9a-multi-set-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "three-element set literal must have length 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p9b-multi-set-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "three-element set must contain 2"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 10: Dedup at construction `#{1 1 2 2 3}` ─────────────────────────

#[test]
fn probe_10_set_literal_dedup_at_construction() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p10-set-dedup-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "duplicate elements must collapse at set construction; length must be 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 11: Mixed-element-type rejection ───────────────────────────────────

#[test]
fn probe_11_mixed_element_types_rejected_at_check() {
    let err = startup_from_file(
        "tests/collection/probe_arc215_collection_literal_inference_p11_bad.wat",
    )
    .expect_err("expected startup failure for mixed-element-type set");
    let err = format!("{}\n---\n{:?}", err, err);
    assert_eq!(
        err,
        r##"#wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message "#{…} set literal: parameter element #2 expects :wat::core::i64; got :wat::core::keyword" :location #wat.core/Span {:file "tests/collection/probe_arc215_collection_literal_inference_p11_bad.wat" :line 4 :col 27 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 31}} :causes [] :callee "#{…} set literal" :param "element #2" :expected ":wat::core::i64" :got ":wat::core::keyword" :remedies []} #wat.check/TypeMismatch {:message "#{…} set literal: parameter element #3 expects :wat::core::i64; got :wat::core::String" :location #wat.core/Span {:file "tests/collection/probe_arc215_collection_literal_inference_p11_bad.wat" :line 4 :col 32 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 35}} :causes [] :callee "#{…} set literal" :param "element #3" :expected ":wat::core::i64" :got ":wat::core::String" :remedies []}]}
---
#wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message "#{…} set literal: parameter element #2 expects :wat::core::i64; got :wat::core::keyword" :location #wat.core/Span {:file "tests/collection/probe_arc215_collection_literal_inference_p11_bad.wat" :line 4 :col 27 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 31}} :causes [] :callee "#{…} set literal" :param "element #2" :expected ":wat::core::i64" :got ":wat::core::keyword" :remedies []} #wat.check/TypeMismatch {:message "#{…} set literal: parameter element #3 expects :wat::core::i64; got :wat::core::String" :location #wat.core/Span {:file "tests/collection/probe_arc215_collection_literal_inference_p11_bad.wat" :line 4 :col 32 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 35}} :causes [] :callee "#{…} set literal" :param "element #3" :expected ":wat::core::i64" :got ":wat::core::String" :remedies []}]}"##,
        "probe_11: mixed-element-type set TypeMismatch golden"
    );
}

// ─── Probe 12: Map of sets ────────────────────────────────────────────────────

#[test]
fn probe_12_map_of_sets() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p12a-map-of-sets-outer-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "map of sets must have outer length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p12b-map-of-sets-inner-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "inner set #{{1 2}} retrieved from map must have length 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}
