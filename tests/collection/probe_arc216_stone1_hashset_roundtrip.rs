//! Arc 216 Stone 1 — `HashSet<T>` round-trip through `HolonAST::Bundle`.
//!
//! Verifies bidirectional round-trip: `value_to_atom` (forward, `Value → HolonAST`)
//! and `atom-value` (reverse, `HolonAST → Value`) for `HashSet<T>`.
//!
//! ## The 10 probes
//!
//! Forward direction:
//!  1. `(:wat::holon::to-holon #{1 2 3})` → classifier-wrapped HolonAST (arc 228)
//!
//! Reverse direction:
//!  2. `(:wat::holon::from-holon<bundle>)` on a round-tripped HashSet → reconstructs set
//!
//! Edge cases:
//!  3. Empty set `#{}` → `Bundle([])` → `#{}`; length preserved
//!  4. Single element `#{42}` → `Bundle([I64(42)])` → `#{42}`
//!
//! Multi-T types:
//!  5. Works for `HashSet<i64>`, `HashSet<String>`, `HashSet<bool>`, `HashSet<keyword>`
//!
//! Dedupe semantic:
//!  6. Reverse trip with duplicate atoms in Bundle deduplicates naturally via HashSet insert
//!
//! Nested set:
//!  7. `HashSet<HashSet<i64>>` — outer Bundle of inner Bundles; recursive atomization
//!
//! Check-level atomizable predicate:
//!  8. `(:wat::holon::to-holonmy-hashset)` for atomizable T type-checks cleanly
//!  9. `(:wat::holon::to-holonfn-value)` where T is Fn — fails at check (TypeMismatch)
//!
//! Arc 294.h: probe 10 (a Rust-side `HolonRepresentable` cascade) is removed —
//! `HolonRepresentable` had zero production consumers and is deleted. Probes
//! 1-9 above are the wat-surface VSA coverage and are untouched by that stone.

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

// just-eval (rubric): each `:t::pNN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside_value` — no inline wat driver.

// ─── Probe 1 — Forward: `#{1 2 3}` → classifier-wrapped HolonAST ────────────

#[test]
fn probe_1_forward_hashset_to_bundle() {
    match call_beside_value(file!(), ":t::p1-forward-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "classifier-wrapped Set encoding must preserve 3 elements in round-trip"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2 — Reverse: Bundle → HashSet round-trip ─────────────────────────

#[test]
fn probe_2_reverse_bundle_to_hashset_roundtrip() {

    match call_beside_value(file!(), ":t::p2a-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "round-trip must preserve length 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p2b-rt-contains").expect("eval") {
        Value::bool(b) => assert!(b, "round-trip must preserve element 2"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 3 — Empty set round-trip ──────────────────────────────────────────

#[test]
fn probe_3_empty_set_roundtrip() {
    match call_beside_value(file!(), ":t::p3-empty-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "empty set round-trip must preserve length 0"),
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

    match call_beside_value(file!(), ":t::p4b-single-rt-contains").expect("eval") {
        Value::bool(b) => assert!(b, "single-element round-trip must contain 42"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 5 — Multi-T types ─────────────────────────────────────────────────

#[test]
fn probe_5_multi_t_types() {

    match call_beside_value(file!(), ":t::p5a-i64-rt-contains").expect("eval") {
        Value::bool(b) => assert!(b, "HashSet<i64> round-trip must contain 20"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p5b-str-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "HashSet<String> round-trip: length must be 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p5c-bool-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashSet<bool> round-trip: length must be 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 6 — Dedupe semantic ────────────────────────────────────────────────

#[test]
fn probe_6_dedupe_semantic() {
    match call_beside_value(file!(), ":t::p6-dedupe-rt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "deduplicated set round-trip must yield length 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7 — Nested set round-trip ─────────────────────────────────────────

#[test]
fn probe_7_nested_set_roundtrip() {

    match call_beside_value(file!(), ":t::p7a-nested-rt-outer-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "nested set round-trip: outer length must be 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p7b-nested-rt-arc228").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "nested set: round-trip outer HashSet length must be 2 (arc 228 classifier-wrap verified)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 8 — Check passes for atomizable T ─────────────────────────────────

#[test]
fn probe_8_check_passes_for_atomizable_t() {

    match call_beside_value(file!(), ":t::p8a-atomizable-passes").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "Atom on HashSet<i64> must pass check and run"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::p8b-nested-atomizable").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "Atom on HashSet<HashSet<i64>> must pass check and run (recursive atomizable)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 9 — Check fails for non-atomizable T ──────────────────────────────

#[test]
fn probe_9_check_fails_for_non_atomizable_t() {
    let err = startup_from_file(
        "tests/collection/probe_arc216_stone1_hashset_roundtrip_p9.wat.bad",
    )
    .expect_err("expected startup failure for non-atomizable Fn type");
    wat::assert_edn_matches_file!(format!("{err}"), "probe_arc216_stone1_hashset_roundtrip__non_atomizable_fn.edn", "probe_9: non-atomizable Fn type check-error golden (Display)");
    wat::assert_edn_matches_file!(format!("{err:?}"), "probe_arc216_stone1_hashset_roundtrip__non_atomizable_fn.edn", "probe_9: non-atomizable Fn type check-error golden (Debug)");
}
