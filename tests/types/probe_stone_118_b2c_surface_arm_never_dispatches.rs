//! Stone 118.B2c strike 2 — **a `defclause` arm typed with a SURFACE now dispatches.**
//!
//! The wat source is the co-located sibling fixture
//! `probe_stone_118_b2c_surface_arm_never_dispatches.wat`.
//! Design: `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/`
//! `DESIGN-STONE-118.B2c-a-surface-typed-clause-arm-never-dispatches.md`.
//!
//! ## THIS FILE INVERTED, EXACTLY AS ITS OWN HEADER PROMISED
//!
//! It was written as a WITNESS of the defect: four `clause_*` rows asserting that every container
//! is REFUSED by a `Seqable<T>`-typed clause arm, green on the broken substrate, with the header
//! stating *"when B2c lands, the four `clause_*` rows must go RED — that RED is the stone's
//! acceptance signal."* Strike 2 landed and all four went red. This is the mirror they were
//! replaced by.
//!
//! ## The defect it closes
//!
//! ```text
//! no clause of :wat::core::reductions matched (3 args);
//! clause 0 skipped (arg 2: expected :wat::core::Seqable<T>, got :wat::core::Vector)
//! ```
//!
//! B1a (`eab12e05`) taught the CHECKER that a concrete instantiation satisfies a parametric
//! surface. `value_matches_type_by_name` was a SECOND DOOR that never learned it, so a
//! surface-typed arm type-checked and then died at runtime. It now asks
//! `satisfies_bare_surface` — the checker's own answer, over the `extend-type` edges
//! `register_subtype` laid down. One question, one door.
//!
//! ## ★ The `control_*` rows are still load-bearing, in the opposite direction
//!
//! They call the SAME body with the SAME `Seqable<T>` parameter through a plain `defn`, which
//! always worked. Keeping them green proves strike 2 fixed the *dispatcher* and did not, say,
//! quietly widen `Seqable<T>` itself into something that accepts anything. Together the two halves
//! now say: the checker agrees with the runtime, through both doors, for all four containers.
//! `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

use wat::freeze::call_beside_value;

/// Every container, through a `defclause` arm declared `Seqable<T>`: it must now DISPATCH and
/// return the element count.
///
/// Before strike 2 each of these returned `Err(NoMatchingClause)` with the skip reason naming the
/// surface as `expected` and the concrete container as `got`.
fn assert_clause_arm_dispatches(entry: &str, expected_count: i64) {
    let v = call_beside_value(file!(), entry).unwrap_or_else(|e| {
        panic!(
            "a Seqable<T>-typed defclause ARM must dispatch — the checker accepts this call, so a \
             runtime refusal is the two doors disagreeing. Got: {e:?}"
        )
    });
    let wat::Value::i64(n) = v else {
        panic!("{entry}: expected an i64, got {v:?}");
    };
    assert_eq!(n, expected_count, "{entry}: wrong element count");
}

#[test]
fn clause_arm_dispatches_vector() {
    assert_clause_arm_dispatches(":my::clause-vector", 3);
}

#[test]
fn clause_arm_dispatches_list() {
    assert_clause_arm_dispatches(":my::clause-list", 3);
}

#[test]
fn clause_arm_dispatches_persistentvector() {
    assert_clause_arm_dispatches(":my::clause-persistentvector", 3);
}

#[test]
fn clause_arm_dispatches_stream() {
    assert_clause_arm_dispatches(":my::clause-stream", 2);
}

// ─── ★ THE CONTROL — the same Seqable<T> parameter on a plain `defn` MUST work ──────────────────

fn assert_defn_dispatches(entry: &str, expected_count: i64) {
    let v = call_beside_value(file!(), entry).unwrap_or_else(|e| {
        panic!("CONTROL BROKEN for {entry}: a Seqable<T> param on a plain defn must work — if this \
                fails, door 1 is NOT about defclause dispatch and the stone is mis-aimed. Got: {e:?}")
    });
    let wat::Value::i64(n) = v else {
        panic!("{entry}: expected an i64, got {v:?}");
    };
    assert_eq!(n, expected_count, "{entry}: wrong element count");
}

#[test]
fn control_defn_dispatches_vector() {
    assert_defn_dispatches(":my::defn-vector", 3);
}

#[test]
fn control_defn_dispatches_list() {
    assert_defn_dispatches(":my::defn-list", 3);
}

#[test]
fn control_defn_dispatches_persistentvector() {
    assert_defn_dispatches(":my::defn-persistentvector", 3);
}

#[test]
fn control_defn_dispatches_stream() {
    assert_defn_dispatches(":my::defn-stream", 2);
}
