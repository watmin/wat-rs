//! Arc 216 Stone 2 — `Vec<T>` (`:wat::core::Vector<T>`) round-trip through
//! `HolonAST::Bundle` of positional-Binds.
//!
//! ## The 12 probes
//!
//! Forward direction:
//!  1. `(:wat::holon::to-holon [1 2 3])` → `HolonAST::Bundle` containing 3 Bind children
//!
//! Reverse direction:
//!  2. `(:wat::holon::from-holon<bundle>)` on a round-tripped Vec → reconstructs Vec
//!
//! Edge cases:
//!  3. Empty vec `[]` → `Bundle([])` → reconstructs (edge: empty bundle)
//!  4. Single element `[42]` → `Bundle([Bind(0, I64(42))])` → `[42]`
//!
//! Multi-T types:
//!  5. Works for `Vec<i64>`, `Vec<String>`, `Vec<bool>`, `Vec<keyword>`
//!
//! Order preservation:
//!  6. Round-trip preserves element order via i64 key sequence
//!
//! Nested vector:
//!  7. `Vec<Vec<i64>>` — outer Bundle of positional Binds whose values are inner Bundles
//!
//! Mixed nesting:
//!  8. `Vec<HashSet<i64>>` — composes with Stone 216.1 (inner Bundles are bare-atom set-shape)
//!
//! Check-level atomizable predicate:
//!  9. `(:wat::holon::to-holon [1 2 3])` for atomizable T type-checks cleanly
//! 10. `(:wat::holon::to-holonvec-of-fns)` fails at check (non-atomizable T)
//!
//! HolonRepresentable Rust-side:
//! 11. `Vec<String>` satisfies `HolonRepresentable` at compile time; roundtrip correct
//!
//! Reverse-shape validation:
//! 12. Bundle with non-sequential i64 keys → `from_holon_ast` error (positional invariant)

use wat::comms::HolonRepresentable;
use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

// ─── Probe 1 — Forward: `[1 2 3]` → classifier-wrapped HolonAST ─────────────

#[test]
fn probe_1_forward_vec_to_bundle() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p1-forward-rt-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "classifier-wrapped Vector encoding must preserve 3 elements in round-trip"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2 — Reverse: Bundle → Vec round-trip ──────────────────────────────

#[test]
fn probe_2_reverse_bundle_to_vec_roundtrip() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p2a-rt-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "round-trip must preserve length 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p2b-rt-first)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "round-trip must preserve first element = 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 3 — Empty vec round-trip ──────────────────────────────────────────

#[test]
fn probe_3_empty_vec_forward() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p3-empty-rt-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 0, "empty vec classifier-wrapped encoding must round-trip to Vec length 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4 — Single element round-trip ─────────────────────────────────────

#[test]
fn probe_4_single_element_roundtrip() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p4a-single-rt-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "single-element round-trip must have length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p4b-single-rt-elem)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 42, "single-element round-trip must retrieve 42 at index 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5 — Multi-T types ─────────────────────────────────────────────────

#[test]
fn probe_5_multi_t_types() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p5a-i64-elem1)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 20, "Vec<i64> round-trip: element at index 1 must be 20"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p5b-str-rt-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "Vec<String> round-trip: length must be 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p5c-bool-rt-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "Vec<bool> round-trip: length must be 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 6 — Order preservation ────────────────────────────────────────────

#[test]
fn probe_6_order_preservation() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p6a-order-idx0)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 10, "order preservation: index 0 must be 10"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p6b-order-idx2)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 30, "order preservation: index 2 must be 30"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7 — Nested vector round-trip ──────────────────────────────────────

#[test]
fn probe_7_nested_vector_roundtrip() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p7a-nested-outer-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "nested Vec round-trip: outer length must be 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p7b-nested-arc228)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "nested Vec arc 228: classifier-wrapped encoding outer length = 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p7c-nested-inner-elem)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 4, "nested Vec round-trip: inner vec at index 1, element at index 0 must be 4"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 8 — Mixed nesting: Vec<HashSet<i64>> ──────────────────────────────

#[test]
fn probe_8_mixed_nesting_vec_of_hashset() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p8a-mixed-outer-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "Vec<HashSet<i64>> round-trip: outer length must be 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p8b-mixed-arc228)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "Vec<HashSet<i64>> arc 228: classifier-wrapped outer Vec length = 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 9 — Check passes for atomizable T ─────────────────────────────────

#[test]
fn probe_9_check_passes_for_atomizable_t() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p9a-atomizable-passes)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "Atom on Vec<i64> must pass check and run"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p9b-nested-atomizable)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "Atom on Vec<Vec<i64>> must pass check and run (recursive atomizable)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 10 — Check fails for non-atomizable T ─────────────────────────────

#[test]
fn probe_10_check_fails_for_non_atomizable_t() {
    let err = startup_from_file(
        "tests/collection/probe_arc216_stone2_vector_roundtrip_p10_bad.wat",
    )
    .expect_err("expected startup failure for non-atomizable Fn type");
    let err = format!("{}\n---\n{:?}", err, err);
    assert_eq!(
        err,
        r##"check:
2 type-check error(s):
  - tests/collection/probe_arc216_stone2_vector_roundtrip_p10_bad.wat:6:28: :wat::holon::to-holon: parameter #1 expects atomizable type (primitive | HolonAST | WatAST | HashSet<T> | Vector<T> | HashMap<K,V> for atomizable T); got :wat::core::Fn(wat::core::i64)->wat::core::i64
  - tests/collection/probe_arc216_stone2_vector_roundtrip_p10_bad.wat:4:3: :user::compute: body produces :wat::holon::HolonAST; signature declares :()

---
Check(CheckErrors([CheckError { span: Span { file: "tests/collection/probe_arc216_stone2_vector_roundtrip_p10_bad.wat", line: 6, col: 28, end_line: 6, end_col: 29 }, kind: TypeMismatch { callee: ":wat::holon::to-holon", param: "#1", expected: "atomizable type (primitive | HolonAST | WatAST | HashSet<T> | Vector<T> | HashMap<K,V> for atomizable T)", got: ":wat::core::Fn(wat::core::i64)->wat::core::i64" } }, CheckError { span: Span { file: "tests/collection/probe_arc216_stone2_vector_roundtrip_p10_bad.wat", line: 4, col: 3, end_line: 6, end_col: 31 }, kind: ReturnTypeMismatch { function: ":user::compute", expected: ":()", got: ":wat::holon::HolonAST", remedies: [] } }]))"##,
        "probe_10: non-atomizable Fn type check-error golden"
    );
}

// ─── Probe 11 — HolonRepresentable cascade (compile-time + runtime) ──────────

fn assert_holon_representable<T: HolonRepresentable>() {}

#[test]
fn probe_11_holon_representable_cascade() {
    // Compile-time: if this call compiles, Vec<String>: HolonRepresentable.
    assert_holon_representable::<Vec<String>>();

    // Runtime roundtrip: ["hello", "world", "foo"].
    let v: Vec<String> = vec!["hello".into(), "world".into(), "foo".into()];
    let ast = v.to_holon_ast();

    // to_holon_ast produces a Bundle of 3 Bind children.
    match &ast {
        holon::HolonAST::Bundle(items) => {
            assert_eq!(items.len(), 3, "Bundle must have 3 children");
            for (i, item) in items.iter().enumerate() {
                match item {
                    holon::HolonAST::Bind(k, val_ast) => {
                        assert!(
                            matches!(k.as_ref(), holon::HolonAST::I64(n) if *n == i as i64),
                            "Bind key at position {} must be I64({})",
                            i,
                            i
                        );
                        assert!(
                            matches!(val_ast.as_ref(), holon::HolonAST::String(_)),
                            "Bind value at position {} must be HolonAST::String leaf",
                            i
                        );
                    }
                    other => panic!(
                        "element {} must be HolonAST::Bind, got {:?}",
                        i, other
                    ),
                }
            }
        }
        other => panic!("expected HolonAST::Bundle, got {:?}", other),
    }

    // from_holon_ast reconstructs the vec in original order.
    let reconstructed: Vec<String> =
        HolonRepresentable::from_holon_ast(&ast).expect("roundtrip");
    assert_eq!(
        reconstructed,
        v,
        "roundtrip must reproduce original Vec<String> in original order"
    );

    // Nested: Vec<Vec<String>> — bounds compose.
    assert_holon_representable::<Vec<Vec<String>>>();
    let nested: Vec<Vec<String>> = vec![
        vec!["a".into(), "b".into()],
        vec!["c".into()],
    ];
    let nested_ast = nested.to_holon_ast();
    let nested_back: Vec<Vec<String>> =
        HolonRepresentable::from_holon_ast(&nested_ast).expect("nested roundtrip");
    assert_eq!(nested_back, nested, "nested Vec<Vec<String>> roundtrip");
}

// ─── Probe 12 — Reverse-shape validation ─────────────────────────────────────

#[test]
fn probe_12_reverse_shape_validation_non_sequential_keys() {
    // Construct a malformed Bundle: Bind(0, String("a")), Bind(2, String("b")) — key 1 missing.
    let bind0 =
        holon::HolonAST::bind(holon::HolonAST::i64(0), holon::HolonAST::string("a"));
    let bind2 =
        holon::HolonAST::bind(holon::HolonAST::i64(2), holon::HolonAST::string("b"));
    let malformed_bundle = holon::HolonAST::bundle(vec![bind0, bind2]);

    // from_holon_ast must return Err (positional invariant violated).
    let result = <Vec<String> as HolonRepresentable>::from_holon_ast(&malformed_bundle);
    assert!(
        result.is_err(),
        "from_holon_ast on non-sequential Bundle must return Err; got Ok({:?})",
        result.ok()
    );
    let err_msg = result.unwrap_err();
    assert_eq!(
        err_msg.message(),
        "Vec positional invariant violated: expected key 1 at position 1, got 2",
        "probe_12: non-sequential Bundle error golden"
    );

    // Bundle with reversed keys [Bind(1, "second"), Bind(0, "first")] should succeed.
    let bind1 = holon::HolonAST::bind(holon::HolonAST::i64(1), holon::HolonAST::string("second"));
    let bind0_str =
        holon::HolonAST::bind(holon::HolonAST::i64(0), holon::HolonAST::string("first"));
    let reordered_bundle = holon::HolonAST::bundle(vec![bind1, bind0_str]);
    let reordered: Vec<String> =
        HolonRepresentable::from_holon_ast(&reordered_bundle).expect("reordered bundle ok");
    assert_eq!(
        reordered,
        vec!["first".to_string(), "second".to_string()],
        "reversed-key Bundle must sort by key and reconstruct in key order"
    );
}
