//! Stone 118.B2c — DISCONFIRMING PROBE for **door 1**: a `defclause` ARM typed with a SURFACE never
//! dispatches at runtime, even though the checker accepts the call.
//!
//! The wat source is the co-located sibling fixture
//! `probe_stone_118_b2c_surface_arm_never_dispatches.wat`. It LOADS and TYPE-CHECKS cleanly — that
//! is the defect: the program is legal and dies when called.
//!
//! Found while migrating the six walkers (118.B2b, `d4c6f3a5`); PRE-EXISTING since B1 (`488eacd0`).
//! Design: `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-STONE-118.B2c-a-surface-typed-clause-arm-never-dispatches.md`
//!
//! ## ⚠ THIS FILE IS A WITNESS, AND IT INVERTS WHEN THE FIX LANDS
//!
//! It asserts the DEFECT, so it is GREEN on the broken substrate. **When B2c lands, the four
//! `clause_*` rows must go RED** — that RED is the stone's acceptance signal, and the fix's job is
//! to delete them and replace them with the mirror of `control_*` (every container dispatching).
//! A witness is committed rather than described in prose because a negative control that CAN be
//! kept MUST be kept (`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`), and it is
//! NOT `#[ignore]`d, because "commit RED probes ignored" is precisely the convention that grew this
//! repo's ignore pile (`[[feedback_a_house_convention_can_be_the_mechanism_that_built_the_pile]]`).
//!
//! ## ★ The control is the load-bearing half
//!
//! `control_*` calls the SAME body with the SAME `Seqable<T>` parameter through a plain `defn`.
//! Those four must SUCCEED. Without them, the four failing rows are equally explained by
//! "`Seqable<T>` parameters are broken everywhere" — which is false, and would send the fix at the
//! wrong door entirely. The pair together says: the type is fine, the CHECKER is fine, and the
//! defect is exactly `defclause` dispatch.
//! `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

use wat::freeze::call_beside_value;
use wat::value::signal::RuntimeErrorKind;
use wat::value::value::ClauseFailureReason;

/// Every container, through a `defclause` arm declared `Seqable<T>`: `NoMatchingClause`, with the
/// skip reason naming the surface as `expected` and the concrete container as `got`.
///
/// Asserting the ARM (not merely "it failed") is what makes this a characterization and not a
/// smoke test — each `ClauseFailureReason` predicts a different mechanism, and only
/// `ArgTypeMismatch` at position 0 means *this* one.
fn assert_clause_arm_refused(entry: &str, expected_got: &str) {
    let err = call_beside_value(file!(), entry)
        .expect_err("a Seqable<T>-typed defclause arm must fail to dispatch at HEAD");

    let RuntimeErrorKind::NoMatchingClause {
        name,
        attempted_clauses,
        ..
    } = err.kind()
    else {
        panic!("expected NoMatchingClause for {entry}, got {:?}", err.kind());
    };
    assert_eq!(name, ":my::count-via-clause", "wrong defclause for {entry}");

    let attempt = attempted_clauses
        .first()
        .unwrap_or_else(|| panic!("no clause attempt recorded for {entry}"));

    let ClauseFailureReason::ArgTypeMismatch {
        position,
        expected,
        got,
    } = &attempt.failure_reason
    else {
        panic!(
            "{entry}: expected the arm to be skipped on ARG TYPE (that is the door-1 mechanism); \
             got {:?}",
            attempt.failure_reason
        );
    };

    assert_eq!(*position, 0, "{entry}: the receiver is arg 0");
    assert_eq!(
        expected, ":wat::core::Seqable<T>",
        "{entry}: the declared type must be the surface"
    );
    assert_eq!(got, expected_got, "{entry}: the value's resolved container");
}

#[test]
fn clause_arm_refuses_vector() {
    assert_clause_arm_refused(":my::clause-vector", ":wat::core::Vector");
}

#[test]
fn clause_arm_refuses_list() {
    assert_clause_arm_refused(":my::clause-list", ":wat::core::List");
}

#[test]
fn clause_arm_refuses_persistentvector() {
    assert_clause_arm_refused(":my::clause-persistentvector", ":wat::core::PersistentVector");
}

#[test]
fn clause_arm_refuses_stream() {
    assert_clause_arm_refused(":my::clause-stream", ":wat::stream::Stream");
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
