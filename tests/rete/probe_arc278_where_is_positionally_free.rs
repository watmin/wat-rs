//! Arc 278 — a `:where` guard is POSITIONALLY FREE.
//!
//! THE BUG THIS GATES, filed 2026-08-24 as a SILENT WRONG ANSWER: a guard followed
//! by TWO OR MORE fact conditions made the rule match NOTHING. It compiled, ran,
//! exited 0, and returned an empty result set. For a corpus search that is
//! indistinguishable from "nothing matched" — and it was only caught because a
//! positive control with a deliberate duplicate ALSO returned 0. Without that
//! control it would have shipped as a finding ABOUT the codebase.
//!
//! BOTH REFERENCES DISAGREED WITH NATIVE, measured rather than assumed:
//!
//!     native   [E: guard then 2 facts] -> 0      the $oracle -> 1     Clara 0.24.0 -> 1
//!
//! So the report's "two acceptable outcomes" — it is a bug, OR `where`s must be
//! trailing and compile must refuse — resolved to the first. Refusing mid-guard
//! rules would have made the fence contradict the engine's own definition of
//! correct, and Clara independently agrees with the oracle.
//!
//! THE TRIGGER IS NOT WHAT THE REPORT THOUGHT, and narrowing it mattered. It is
//! exactly "two or more FACT conditions after the guard" — not absolute position
//! (a guard at slot 3 of 5 failed while the same guard at slot 4 of 5 worked), and
//! not the join shape (the report's unjoined `Source` was a red herring; a
//! fully-joined third fact failed identically).
//!
//! MECHANISM: `filter_after_join` walks its frontier through FILTER children only,
//! so `:where -> HashJoin(a) -> HashJoin(b)` stalled at (a) — (b) is not a filter,
//! nothing left-activated it, production read an empty `d_beta`, `next_delta` came
//! back empty and the fixpoint exited. See `pass::left_activate_join`.
//!
//! WHY FOUR ROWS. `one` and `trailing` already worked — they are here so a
//! regression on the working path surfaces in the same place rather than somewhere
//! far away. `two` is the reported bug. `four` is DEPTH: the frontier must LOOP, not
//! take one extra step. A fix that special-cased two would pass `two` and leave
//! `four` silently wrong — the same class of defect all over again.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_where_is_positionally_free

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// `[one, two, four, trailing]` under native, then the same four under `$oracle`.
fn counts() -> Vec<i64> {
    let out = call_beside_value(file!(), ":user::native-and-oracle")
        .expect("fixture should fire cleanly on both engines");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    assert_eq!(items.len(), 8, "witness shape changed: {items:?}");
    items
        .into_iter()
        .map(|v| match v {
            Value::i64(x) => *x,
            other => panic!("expected i64; got {other:?}"),
        })
        .collect()
}

#[test]
fn a_guard_followed_by_two_fact_conditions_still_matches() {
    let c = counts();
    assert_eq!(
        c[1], 1,
        "the reported bug: a `:where` followed by TWO fact conditions matched nothing. \
         Got {} where 1 is correct. This is a SILENT wrong answer — it compiles, runs \
         and exits 0, so nothing but a positive control distinguishes it from a real \
         empty result.",
        c[1]
    );
}

#[test]
fn the_frontier_loops_rather_than_taking_one_extra_step() {
    let c = counts();
    assert_eq!(
        c[2], 1,
        "a `:where` followed by FOUR fact conditions must also match. If this is 0 while \
         the two-condition case passes, the fix walked one extra join instead of looping \
         the frontier — the bug is still there, just deeper."
    );
}

#[test]
fn the_positions_that_already_worked_still_work() {
    let c = counts();
    assert_eq!((c[0], c[3]), (1, 1), "one-after and trailing are the control rows");
}

#[test]
fn native_agrees_with_the_oracle_at_every_guard_position() {
    let c = counts();
    assert_eq!(
        &c[0..4],
        &c[4..8],
        "native and the $oracle must agree wherever the guard sits. They did not before \
         2026-08-24: native returned 0 for the two-and-more case while the oracle \
         returned 1, and no fixture in the corpus placed a guard mid-chain."
    );
}
