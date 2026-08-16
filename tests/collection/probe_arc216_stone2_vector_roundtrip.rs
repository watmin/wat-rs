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
//! Arc 294.h: probes 11-12 (a Rust-side `HolonRepresentable` cascade + a
//! reverse-shape validation exercised only through that trait) are removed —
//! `HolonRepresentable` had zero production consumers and is deleted. Probes
//! 1-10 above are the wat-surface VSA coverage and are untouched by that stone.

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

// just-eval (rubric): each `:t::pNN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside_value` — no inline wat driver.

// ─── Probe 1 — Forward: `[1 2 3]` → classifier-wrapped HolonAST ─────────────

#[test]
fn probe_1_forward_vec_to_bundle() {
    match call_beside_value(file!(), ":t::p1-forward-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "classifier-wrapped Vector encoding must preserve 3 elements in round-trip"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2 — Reverse: Bundle → Vec round-trip ──────────────────────────────

#[test]
fn probe_2_reverse_bundle_to_vec_roundtrip() {

    match call_beside_value(file!(), ":t::p2a-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "round-trip must preserve length 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p2b-rt-first").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "round-trip must preserve first element = 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 3 — Empty vec round-trip ──────────────────────────────────────────

#[test]
fn probe_3_empty_vec_forward() {
    match call_beside_value(file!(), ":t::p3-empty-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "empty vec classifier-wrapped encoding must round-trip to Vec length 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4 — Single element round-trip ─────────────────────────────────────

#[test]
fn probe_4_single_element_roundtrip() {

    match call_beside_value(file!(), ":t::p4a-single-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "single-element round-trip must have length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p4b-single-rt-elem").expect("eval") {
        Value::i64(n) => assert_eq!(n, 42, "single-element round-trip must retrieve 42 at index 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5 — Multi-T types ─────────────────────────────────────────────────

#[test]
fn probe_5_multi_t_types() {

    match call_beside_value(file!(), ":t::p5a-i64-elem1").expect("eval") {
        Value::i64(n) => assert_eq!(n, 20, "Vec<i64> round-trip: element at index 1 must be 20"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p5b-str-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "Vec<String> round-trip: length must be 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p5c-bool-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "Vec<bool> round-trip: length must be 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 6 — Order preservation ────────────────────────────────────────────

#[test]
fn probe_6_order_preservation() {

    match call_beside_value(file!(), ":t::p6a-order-idx0").expect("eval") {
        Value::i64(n) => assert_eq!(n, 10, "order preservation: index 0 must be 10"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p6b-order-idx2").expect("eval") {
        Value::i64(n) => assert_eq!(n, 30, "order preservation: index 2 must be 30"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7 — Nested vector round-trip ──────────────────────────────────────

#[test]
fn probe_7_nested_vector_roundtrip() {

    match call_beside_value(file!(), ":t::p7a-nested-outer-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "nested Vec round-trip: outer length must be 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p7b-nested-arc228").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "nested Vec arc 228: classifier-wrapped encoding outer length = 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p7c-nested-inner-elem").expect("eval") {
        Value::i64(n) => assert_eq!(n, 4, "nested Vec round-trip: inner vec at index 1, element at index 0 must be 4"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 8 — Mixed nesting: Vec<HashSet<i64>> ──────────────────────────────

#[test]
fn probe_8_mixed_nesting_vec_of_hashset() {

    match call_beside_value(file!(), ":t::p8a-mixed-outer-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "Vec<HashSet<i64>> round-trip: outer length must be 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p8b-mixed-arc228").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "Vec<HashSet<i64>> arc 228: classifier-wrapped outer Vec length = 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 9 — Check passes for atomizable T ─────────────────────────────────

#[test]
fn probe_9_check_passes_for_atomizable_t() {

    match call_beside_value(file!(), ":t::p9a-atomizable-passes").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "Atom on Vec<i64> must pass check and run"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p9b-nested-atomizable").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "Atom on Vec<Vec<i64>> must pass check and run (recursive atomizable)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 10 — Check fails for non-atomizable T ─────────────────────────────

#[test]
fn probe_10_check_fails_for_non_atomizable_t() {
    let err = startup_from_file(
        "tests/collection/probe_arc216_stone2_vector_roundtrip_p10.wat.bad",
    )
    .expect_err("expected startup failure for non-atomizable Fn type");
    wat::assert_edn_matches_file!(format!("{err}"), "probe_arc216_stone2_vector_roundtrip__non_atomizable_fn.edn", "probe_10: non-atomizable Fn type check-error golden (Display)");
    wat::assert_edn_matches_file!(format!("{err:?}"), "probe_arc216_stone2_vector_roundtrip__non_atomizable_fn.edn", "probe_10: non-atomizable Fn type check-error golden (Debug)");
}
