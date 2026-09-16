//! Arc 278 — an accumulate result is SUPERSEDED, not extended, when its source grows.
//!
//! THE ORACLE WAS THE ONE THAT WAS WRONG. `fire-fixpoint`'s `merge-facts` is add-only, so a
//! `Tally` derived from `(acc::count :from Out)` while Out held 0 elements stayed in the fact
//! set after Out grew to 1 and then 2 — one stale fact per intermediate state. A fact asserting
//! the count is zero, standing while the count is two. Every differential taken against the
//! oracle inherits a defect like that, which is why this one is worth a probe of its own.
//!
//! CLARA 0.24.0 IS THE AUTHORITY HERE, not native. Re-derived for this strike, not taken from
//! the design note on trust (`docs/arc/2026/06/278-rules-engine/strike-oracle-acc-refire/`,
//! `clara-two-changes.clj`):
//!
//! ```text
//! $ clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}} :paths ["."]}' -M -m t2
//! clara Tally count = 1  values = [2]
//! ```
//!
//! Clara keeps exactly ONE Tally, holding the FINAL count. Native agrees on all three shapes;
//! that agreement is corroboration, not the definition.
//!
//! MEASURED, this fixture's witness `[rows, sum-of-n]` per shape:
//!
//! ```text
//!   shape   count goes   Clara   native   oracle BEFORE   oracle AFTER
//!   empty   0            [1 0]   [1 0]    [1 0]           [1 0]
//!   one     0→1          [1 1]   [1 1]    [2 1]  ✗        [1 1]
//!   two     0→1→2        [1 2]   [1 2]    [3 3]  ✗        [1 2]
//! ```
//!
//! `sum-of-n` is carried alongside the row count because neither number alone can see the whole
//! defect: on `one` the pre-fix sum was already 1 (n=0 plus n=1), so a sum-only test was green
//! on a shape that was wrong; and on `empty` an over-correction into "never emit" would answer
//! `[0 0]` while looking perfect on `one` and `two`.
//!
//! THE FIX, and why it is the ORACLE'S route and not native's. Native retracts through
//! incremental delta propagation (`fire_fixpoint_delta`, token-level). Porting that would make
//! every future oracle differential vacuous — agreement by construction. The oracle instead says
//! it model-theoretically, in its own vocabulary of whole-set replay: `fire-fixpoint` now GROWS
//! to the closure exactly as before, then SHRINKS by `F := F ∩ (base ∪ D(F))` until stable —
//! keep only what one honest replay still stands behind. For a monotone rule set the shrink
//! retains everything on its first step and stops, so nothing else in the corpus moves.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_oracle_accumulate_supersedes

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// `[empty-rows empty-sum one-rows one-sum two-rows two-sum]` under native,
/// then the same six under `$oracle`.
fn witness() -> Vec<i64> {
    let out = call_beside_value(file!(), ":user::native-and-oracle")
        .expect("fixture should fire cleanly on both engines");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    assert_eq!(items.len(), 12, "witness shape changed: {items:?}");
    items
        .into_iter()
        .map(|v| match v {
            Value::i64(x) => *x,
            other => panic!("expected i64; got {other:?}"),
        })
        .collect()
}

#[test]
fn a_superseded_accumulate_result_does_not_stand() {
    let w = witness();
    let (nat, orc) = (&w[0..6], &w[6..12]);

    assert_eq!(
        &orc[2..6],
        &[1, 1, 1, 2],
        "Clara 0.24.0 keeps ONE Tally holding the FINAL count: [rows sum] = [1 1] when the \
         count goes 0→1, and [1 2] when it goes 0→1→2. The oracle answered [2 1] and [3 3] \
         before this fix — it kept the Tally derived from every intermediate count, so a fact \
         asserting n=0 stood while the count was 2. Got oracle {orc:?}"
    );
    assert_eq!(
        nat, orc,
        "native and the oracle must agree on all three shapes. This is CORROBORATION, not the \
         definition — Clara is the authority. If this row is the only one that reddens, suspect \
         the oracle was made to mirror native rather than fixed on its own terms."
    );
}

#[test]
fn an_always_empty_accumulate_still_emits_n_zero() {
    let w = witness();
    let (nat_empty, orc_empty) = (&w[0..2], &w[6..8]);

    assert_eq!(
        orc_empty,
        &[1, 0],
        "THE REGRESSION GUARD. All three engines agree an accumulate whose source is empty \
         FOREVER still emits one fact carrying n=0 — emitting is correct; the defect was \
         failing to SUPERSEDE. An over-correction into \"never emit when an accumulate is \
         involved\" passes both changing shapes and lands here as [0 0]. Got {orc_empty:?}"
    );
    assert_eq!(
        nat_empty, orc_empty,
        "native agrees the always-empty count emits n=0 too"
    );
}
