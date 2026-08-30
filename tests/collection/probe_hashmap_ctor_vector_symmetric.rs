//! Arc 214 P1 — HashMap constructor: Vector-symmetric shape probes.
//!
//! Verifies that the refactored `:wat::core::HashMap :- [K V] k0 v0 ...`
//! constructor shape (two type-keywords in one `:-`-marked bracket, per arc
//! 109 slice 1f / stone 3 THE WALL) is accepted by both the runtime
//! evaluator and the type-checker.
//!
//! ## The 9 probes
//!
//! 1. Empty literal — `(:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])` constructs empty HashMap
//! 2. Single pair — length 1; get returns the value
//! 3. Multi pair — three pairs; length + get per key
//! 4. String-keyed — K = String confirms K can be any hashable type
//! 5. HolonAST-keyed — K = HolonAST confirms structural keys
//! 6. Wrong-type rejection — value type mismatch at type-check
//! 7. Odd count rejection — type-check catches arity parity error
//! 8. Missing K type-arg — `(:wat::core::HashMap)` fails arity check
//! 9. Missing V type-arg — `(:wat::core::HashMap :- [:wat::core::keyword])` fails arity check

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

// just-eval (rubric): each `:t::pN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside_value` — no inline wat driver.

// ─── Probe 1: Empty literal ──────────────────────────────────────────────────

#[test]
fn probe_p1_empty_literal_constructs_empty_hashmap() {
    match call_beside_value(file!(), ":t::p1-empty-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "empty HashMap must have length 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2: Single pair ────────────────────────────────────────────────────

#[test]
fn probe_p2_single_pair_length_and_get() {
    match call_beside_value(file!(), ":t::p2-single-get").expect("eval") {
        Value::i64(n) => assert_eq!(n, 42, "get :foo should return 42"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 3: Multi pair ─────────────────────────────────────────────────────

#[test]
fn probe_p3_multi_pair_length_and_get() {

    match call_beside_value(file!(), ":t::p3a-multi-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "three pairs → length 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p3b-multi-get").expect("eval") {
        Value::i64(n) => assert_eq!(n, 20, "get :b from three-pair map → 20"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4: String-keyed ───────────────────────────────────────────────────

#[test]
fn probe_p4_string_keyed_constructs_correctly() {
    match call_beside_value(file!(), ":t::p4-str-keyed-get").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "String-keyed HashMap: get \"b\" → 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5: HolonAST-keyed ─────────────────────────────────────────────────

#[test]
fn probe_p5_holonast_keyed_length() {
    match call_beside_value(file!(), ":t::p5-holonast-keyed-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HolonAST-keyed HashMap with one pair → length 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 6: Wrong-type rejection ───────────────────────────────────────────

#[test]
fn probe_p6_wrong_value_type_rejected_at_type_check() {
    let err = startup_from_file(
        "tests/collection/probe_hashmap_ctor_vector_symmetric_p6.wat.bad",
    )
    .expect_err("expected startup failure for wrong value type");
    wat::assert_edn_matches_file!(format!("{err:?}"), "probe_hashmap_ctor_vector_symmetric__wrong_value_type.edn", "probe_p6: wrong-value type check-error golden");
}

// ─── Probe 7: Odd count rejection ────────────────────────────────────────────

#[test]
fn probe_p7_odd_pair_count_rejected() {
    let err = startup_from_file(
        "tests/collection/probe_hashmap_ctor_vector_symmetric_p7.wat.bad",
    )
    .expect_err("expected startup failure for odd pair count");
    wat::assert_edn_matches_file!(format!("{err:?}"), "probe_hashmap_ctor_vector_symmetric__odd_pair_count.edn", "probe_p7: odd pair count check-error golden");
}

// ─── Probe 8: Zero type-args (arity error) ───────────────────────────────────

#[test]
fn probe_p8_missing_both_type_args_rejected() {
    let err = startup_from_file(
        "tests/collection/probe_hashmap_ctor_vector_symmetric_p8.wat.bad",
    )
    .expect_err("expected startup failure for missing type args");
    wat::assert_edn_matches_file!(format!("{err:?}"), "probe_hashmap_ctor_vector_symmetric__missing_both_type_args.edn", "probe_p8: missing both type args arity-error golden");
}

// ─── Probe 9: Missing V type-arg ─────────────────────────────────────────────

#[test]
fn probe_p9_missing_v_type_arg_rejected() {
    let err = startup_from_file(
        "tests/collection/probe_hashmap_ctor_vector_symmetric_p9.wat.bad",
    )
    .expect_err("expected startup failure for missing V type arg");
    wat::assert_edn_matches_file!(format!("{err:?}"), "probe_hashmap_ctor_vector_symmetric__missing_v_type_arg.edn", "probe_p9: missing V type arg arity-error golden");
}
