//! Arc 216 Stone 4 — Atomizable predicate composite verification.
//!
//! This stone is the VERIFICATION stone. The predicate (`fn is_atomizable`) was
//! pre-landed in Stone 216.1 (Delta 6) and confirmed individually in Stones 216.2
//! (Vector) and 216.3 (HashMap). Stone 4 exercises the predicate via composite
//! probes that nest all three collection types recursively.
//!
//! The predicate (DESIGN Q6, `src/check.rs:3623`):
//! ```text
//! atomizable(T) :=
//!   T ∈ {i64, f64, bool, String, keyword, HolonAST, WatAST, Uuid}  -- primitives
//!   OR T = HashSet<T'>  ∧ atomizable(T')                             -- Stone 1 (shipped)
//!   OR T = Vector<T'>   ∧ atomizable(T')                             -- Stone 2 (shipped)
//!   OR T = HashMap<K,V> ∧ atomizable(K) ∧ atomizable(V)             -- Stone 3 (shipped)
//! ```
//!
//! ## The 6 probes
//!
//! Positive composition (type-checks and runs):
//!  1. HashMap<keyword, Vector<i64>>       — HashMap-of-Vector; Stone 3 + Stone 2 composed
//!  2. Vector<HashSet<i64>>                — Vector-of-HashSet; Stone 2 + Stone 1 composed
//!  3. HashSet<Vector<i64>>                — HashSet-of-Vector; Stone 1 + Stone 2 composed
//!  4. HashMap<keyword, Vector<HashSet<i64>>> — all three collections nested; triple composition
//!
//! Negative (type-check fails with TypeMismatch naming non-atomizable position):
//!  5. Vector<Fn(...)>                     — non-atomizable element T
//!  6. non-atomizable argument to Atom     — non-atomizable K (via fn value)

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

// just-eval (rubric): each `:t::probeN…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside_value` — no inline wat driver.

// ─── Probe 1 — Composite HashMap-of-Vector ────────────────────────────────────

/// HashMap<keyword, Vector<i64>> round-trip length = 2.
/// Verifies Stone 3 + Stone 2 composition: HashMap-of-Vector.
#[test]
fn probe_1_composite_hashmap_of_vector() {
    match call_beside_value(file!(), ":t::probe1-hashmap-of-vector").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashMap<keyword, Vector<i64>> round-trip must preserve 2 entries"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2 — Composite Vector-of-HashSet ────────────────────────────────────

/// Vector<HashSet<i64>> round-trip length = 2.
/// Verifies Stone 2 + Stone 1 composition: Vector-of-HashSet.
#[test]
fn probe_2_composite_vector_of_hashset() {
    match call_beside_value(file!(), ":t::probe2-vector-of-hashset").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "Vector<HashSet<i64>> round-trip must preserve 2 inner sets"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 3 — Composite HashSet-of-Vector ───────────────────────────────────

/// HashSet<Vector<i64>> round-trip length = 2.
/// Stone 216.4 SCORE Delta 2 substituted HashSet<HashSet<i64>> because hashmap_key
/// did not handle Value::Vec. Stone 216.5 fixed the gap. Relanded to original type.
#[test]
fn probe_3_composite_hashset_of_vector() {
    match call_beside_value(file!(), ":t::probe3-hashset-of-vector").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "HashSet<Vector<i64>> round-trip must preserve 2 inner vectors"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 4 — Triple-nested: HashMap<keyword, Vector<HashSet<i64>>> ─────────

/// All three collections nested; round-trip length = 1.
/// Stone 3 + Stone 2 + Stone 1 composition.
#[test]
fn probe_4_triple_nested_hashmap_vector_hashset() {
    match call_beside_value(file!(), ":t::probe4-triple-nested").expect("eval") {
        Value::i64(n) => assert_eq!(n, 1, "HashMap<keyword, Vector<HashSet<i64>>> round-trip must preserve 1 entry"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5 — Negative: non-atomizable element in Vector ─────────────────────

/// `to-holon` on Fn type fails at check with TypeMismatch.
/// is_atomizable(Fn([i64])->i64) = false → TypeMismatch naming :wat::holon::to-holon.
#[test]
fn probe_5_negative_non_atomizable_element() {
    let err = startup_from_file(
        "tests/collection/probe_arc216_stone4_predicate_composition_p5.wat.bad",
    )
    .expect_err("expected startup failure for non-atomizable Fn type");
    let golden = include_str!("probe_arc216_stone4_predicate_composition__non_atomizable_element.edn");
    wat::assert_edn_eq!(format!("{err}"), golden, "probe_5: non-atomizable element check-error golden (Display)");
    wat::assert_edn_eq!(format!("{err:?}"), golden, "probe_5: non-atomizable element check-error golden (Debug)");
}

// ─── Probe 6 — Negative: non-atomizable argument via nested function ──────────

/// Second non-atomizable negative probe — distinct Fn value.
/// is_atomizable(Fn{...}) = false → TypeMismatch naming :wat::holon::to-holon.
#[test]
fn probe_6_negative_non_atomizable_nested_fn() {
    let err = startup_from_file(
        "tests/collection/probe_arc216_stone4_predicate_composition_p6.wat.bad",
    )
    .expect_err("expected startup failure for non-atomizable Fn type");
    let golden = include_str!("probe_arc216_stone4_predicate_composition__non_atomizable_nested_fn.edn");
    wat::assert_edn_eq!(format!("{err}"), golden, "probe_6: non-atomizable Fn type check-error golden (Display)");
    wat::assert_edn_eq!(format!("{err:?}"), golden, "probe_6: non-atomizable Fn type check-error golden (Debug)");
}
