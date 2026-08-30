//! Arc 216 Stone 7 — `Tuple` round-trip through `HolonAST::Bundle` of positional-Binds.
//!
//! Proves bidirectional round-trip: `value_to_atom` (forward, `Value::Tuple → HolonAST`)
//! on the wat-surface VSA path (arc 294.h removed the Rust-level tuple cascade;
//! see the note below).
//!
//! Per encoding doctrine (Stone 216.7): Tuple is collection-category — positional-Bind Bundle,
//! identical shape to Vec<T>. `Bundle([Bind(I64(0), t0_holon), Bind(I64(1), t1_holon), ...])`.
//! Keys are sequential i64 starting from 0. Reverse via `atom-value` returns Vec (same shape;
//! consumer-declared type is the discriminator — honest asymmetry per DESIGN Q9).
//!
//! ## The 12 probes (covers all EXPECTATIONS rows 6-11)
//!
//! Forward direction — WAT surface:
//!  1. `(:wat::holon::to-holon (:wat::core::Tuple 1 "hello"))` → Bundle with 2 Bind children
//!
//! Reverse direction — WAT surface:
//!  2. `atom-value` on Tuple-encoded Bundle → Vec (positional-Bind shape; honest asymmetry)
//!
//! Heterogeneous 3-tuple:
//!  3. `(bool, i64, String)` Bundle shape — 3 Bind children with I64 keys 0, 1, 2
//!
//! Nested + composition:
//!  4. Nested Tuple: `(:wat::core::Tuple (:wat::core::Tuple 1 2) "outer")` — Bundle of Bundles
//!  5. Tuple containing Vec: `(:wat::core::Tuple [1 2 3] "tag")` — outer Bind + inner Vec-shape
//!
//! Tuple containing HashSet:
//!  6. `(:wat::core::Tuple (:wat::core::HashSet :- [:wat::core::i64] 1 2) "label")` → Bundle 2 children
//!
//! is_atomizable predicate:
//!  7. Tuple<i64, String> admits; Tuple containing Fn rejects
//!
//! Arc 294.h: probes 8-12 are removed — every one is a Rust-side probe whose
//! body calls `.to_holon_ast()` / `::from_holon_ast(` directly, or (probe 10)
//! instantiates `pair::<(T1,T2)>()` for a tuple whose `EdnRepresentable` was
//! also only a `HolonRepresentable` delegating shim. `HolonRepresentable` had
//! zero production consumers and is deleted; probes 1-7 above are the
//! wat-surface VSA coverage and are untouched by that stone.
//! (Note: this stone's own disposition table names only 9-12 for removal;
//! measured against the body rule, probe 8 also calls `.to_holon_ast()` and
//! must go too — see the rider's report.)

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

// just-eval (rubric): each `:t::pN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside_value` — no inline wat driver.

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Extract element at `index` from a `Value::Tuple`, asserting it is i64.
fn tuple_element_i64(v: Value, index: usize, probe: &str) -> i64 {
    match v {
        Value::Tuple(items) => match items.get(index) {
            Some(Value::i64(n)) => *n,
            Some(other) => panic!("{}: tuple[{}] is {:?}, expected i64", probe, index, other),
            None => panic!("{}: tuple has fewer than {} elements", probe, index + 1),
        },
        other => panic!("{}: expected Tuple; got {:?}", probe, other),
    }
}

// ─── Probe 1 — Forward: 2-tuple → classifier-wrapped HolonAST ───────────────

/// `(:wat::holon::to-holon (:wat::core::Tuple 1 "hello"))` produces a classifier-wrapped HolonAST.
/// Arc 228 Stone 228.1: the output is `Bind(Atom("Tuple"), Bundle(positional Binds))`.
/// Arc 216 Stone 7 forward direction — forward-corrected per typed-entities doctrine.
/// Verified via round-trip: to-holon → from-holon → Tuple → first element = 1.
///
/// Type-checker note: `from-holon` returns `?T` (fresh type var). To allow the return
/// without calling `first`/`second` (which require statically-known tuple/Vec type),
/// we declare the return type explicitly and extract the element at Rust level.
#[test]
fn probe_1_forward_2tuple_to_bundle() {
    let v = call_beside_value(file!(), ":t::p1-rt-pair").expect("eval");
    assert_eq!(
        tuple_element_i64(v, 0, "probe_1"),
        1,
        "classifier-wrapped Tuple round-trip: first element must be 1"
    );
}

// ─── Probe 2 — Reverse: Tuple-classified → Tuple (arc 228 no honest asymmetry) ─

/// Arc 228 Stone 228.1: from-holon now returns Tuple (not Vec) for Tuple-encoded forms.
///
/// Pre-arc-228 (arc 216 "honest asymmetry"): Tuple and Vec had identical bare-Bundle
/// encoding; from-holon always returned Vec for positional-Bind Bundles; consumer-declared
/// type was the only discriminator.
///
/// Post-arc-228: the classifier Atom("Tuple") vs Atom("Vector") is the discriminator.
/// from-holon on Tuple-classified form returns Tuple; from-holon on Vector returns Vec.
/// The honest asymmetry is resolved: the substrate type is recoverable from data alone.
///
/// Type-checker note: return Tuple directly with explicit annotation; element access at Rust level.
#[test]
fn probe_2_reverse_bundle_to_vec_honest_asymmetry() {
    // Arc 228: from-holon returns Tuple (not Vec). Verify first element = 1 (i64) at Rust level.
    let v = call_beside_value(file!(), ":t::p2-rt-pair").expect("eval");
    assert_eq!(
        tuple_element_i64(v, 0, "probe_2"),
        1,
        "arc 228: from-holon Tuple round-trip: first element = 1 (Tuple, not Vec)"
    );
}

// ─── Probe 3 — 3-tuple primitives → round-trip element verification ───────────

/// `(bool, i64, String)` 3-tuple forward: classifier-wrapped Bind with 3-element inner Bundle.
/// Arc 228: Bundle/children no longer works on classifier-wrapped top-level Bind.
/// Verify via round-trip: to-holon → from-holon → Tuple; second element = 42.
///
/// Type-checker note: from-holon returns ?T; declare explicit return type; element at Rust level.
#[test]
fn probe_3_three_tuple_primitives_bundle_shape() {
    let v = call_beside_value(file!(), ":t::p3-rt-triple").expect("eval");
    assert_eq!(
        tuple_element_i64(v, 1, "probe_3"),
        42,
        "3-tuple round-trip: element at index 1 must be 42"
    );
}

// ─── Probe 4 — Nested Tuple: ((i64, i64), String) ────────────────────────────

/// `(:wat::core::Tuple (:wat::core::Tuple 1 2) "outer")` — nested Tuple.
/// Arc 228: outer is classifier-wrapped Bind; Bundle/children no longer applies.
/// Verify via round-trip: from-holon → outer Tuple; first element is inner Tuple.
/// Inner Tuple's first element = 1 and second element = 2.
///
/// Type-checker note: from-holon returns ?T; return the outer Tuple directly with
/// explicit type annotation; extract nested elements at Rust level.
#[test]
fn probe_4_nested_tuple_roundtrip() {
    let v = call_beside_value(file!(), ":t::p4-rt-nested").expect("eval");

    // Single nested match: outer Tuple → inner Tuple → verify length + elements.
    match v {
        Value::Tuple(outer_items) => match outer_items.first() {
            Some(Value::Tuple(inner_items)) => {
                assert_eq!(
                    inner_items.len(),
                    2,
                    "nested Tuple: inner Tuple (element 0 of outer) must have length 2"
                );
                assert_eq!(inner_items.first(), Some(&Value::i64(1)), "nested Tuple: inner[0] = 1");
                assert_eq!(inner_items.get(1), Some(&Value::i64(2)), "nested Tuple: inner[1] = 2");
            }
            other => panic!("probe_4: outer[0] should be Tuple; got {:?}", other),
        },
        other => panic!("probe_4: expected outer Tuple; got {:?}", other),
    }
}

// ─── Probe 5 — Tuple containing Vec: (Vec<i64>, String) ──────────────────────

/// `(:wat::core::Tuple [1 2 3] "tag")` — Tuple whose first element is a Vec<i64>.
/// Arc 228: outer is classifier-wrapped Bind; Bundle/children no longer applies.
/// Verify via round-trip: to-holon → from-holon → outer Tuple; first element = inner Vec.
/// Inner Vec (Vector-classified) decodes to Vec; Vector/length = 3.
#[test]
fn probe_5_tuple_containing_vec_roundtrip() {
    let v = call_beside_value(file!(), ":t::p5-rt-with-vec").expect("eval");

    // Single nested match: outer Tuple → inner Vec → verify length + first element.
    match v {
        Value::Tuple(outer_items) => match outer_items.first() {
            Some(Value::Vec(inner_v)) => {
                assert_eq!(
                    inner_v.len(),
                    3,
                    "Tuple containing Vec: inner Vec (element 0) must have length 3"
                );
                assert_eq!(inner_v.first(), Some(&Value::i64(1)), "Tuple containing Vec: inner Vec[0] = 1");
            }
            other => panic!("probe_5: outer[0] should be Vec; got {:?}", other),
        },
        other => panic!("probe_5: expected Tuple; got {:?}", other),
    }
}

// ─── Probe 6 — Tuple containing HashSet ───────────────────────────────────────

/// `(:wat::core::Tuple (:wat::core::HashSet :- [:wat::core::i64] 1 2) "label")` — composition
/// with Stone 216.1. Arc 228: outer is classifier-wrapped Bind; Bundle/children no longer applies.
/// Verify via round-trip: to-holon → from-holon → outer Tuple; first element = inner HashSet.
/// HashSet/length = 2.
#[test]
fn probe_6_tuple_containing_hashset() {
    let v = call_beside_value(file!(), ":t::p6-rt-with-set").expect("eval");

    match v {
        Value::Tuple(outer_items) => match outer_items.first() {
            Some(Value::wat__std__HashSet(hs)) => {
                assert_eq!(hs.len(), 2, "Tuple containing HashSet: inner HashSet must have length 2");
            }
            other => panic!("probe_6: outer[0] should be HashSet; got {:?}", other),
        },
        other => panic!("probe_6: expected Tuple; got {:?}", other),
    }
}

// ─── Probe 7 — is_atomizable predicate ────────────────────────────────────────

/// Tuple<i64, String> admits (all elements atomizable).
/// Tuple containing Fn rejects (Fn not in atomizable set).
#[test]
fn probe_7_is_atomizable_tuple() {
    // Admits: (:wat::core::Tuple 1 "hello") — i64 and String are atomizable.
    match call_beside_value(file!(), ":t::p7-admits").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "Tuple<i64, String> must pass is_atomizable check"),
        other => panic!("expected i64; got {:?}", other),
    }

    // Rejects: Tuple containing a Fn — Fn types are not atomizable.
    let err = startup_from_file(
        "tests/collection/probe_arc216_stone7_tuple_roundtrip_p7.wat.bad",
    )
    .expect_err("expected startup failure for Tuple containing Fn");
    wat::assert_edn_matches_file!(format!("{err}"), "probe_arc216_stone7_tuple_roundtrip__tuple_with_fn.edn", "probe_7: Tuple-with-Fn non-atomizable check-error golden (Display)");
    wat::assert_edn_matches_file!(format!("{err:?}"), "probe_arc216_stone7_tuple_roundtrip__tuple_with_fn.edn", "probe_7: Tuple-with-Fn non-atomizable check-error golden (Debug)");
}

