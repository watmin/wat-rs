//! THREE native-vs-`$oracle` DIVERGENCES, ALL FOUND BY THE RETE FUZZER — ALL NOW FIXED.
//!
//! Found 2026-08-25 by `wat-scripts/fuzz/rete-differential.wat` as its grammar widened — 76
//! mismatches of 828 generated shapes, decomposing into exactly three families. Every one
//! reproduced minimally and standalone, and every one was SILENT: a caller got a wrong row count
//! with no error. The families are described below in the present tense of the DEFECT, because
//! the reproduction is the thing worth keeping; see *Status* at the bottom for what closed them.
//!
//! The breakdown is clean enough to be a diagnosis rather than a symptom list: 22 at the
//! accumulate shape (families A and B), 54 at `:not`-over-a-derived-class (family C), the latter
//! all at depth >= 1 and never at depth 0 — exactly the dependence stratified negation should
//! have.
//!
//! ## Family A — a LEADING accumulate emits one row per FIXPOINT ROUND
//!
//! `rows == rounds`, measured: a 1-rule inert chain gives 2, a 2-rule chain gives 3, the oracle
//! holds at 1. The chain derives facts the query never reads; its only role is to make the
//! fixpoint iterate.
//!
//! **This is the same class as the leading `:not` / `:exists` defect fixed on 2026-08-24
//! (`71d0e700e`) — and that fix did not reach accumulate.** The correctness mechanism there is
//! `leading_emitted` persisting ACROSS rounds (`fire/delta.rs`, declared outside the round loop);
//! whatever the accumulate path does, it is not that.
//!
//! ## Family B — a SECOND `where` after an accumulate matches NOTHING
//!
//! `qB1` and `qB2` differ by exactly one trailing, trivially-true `(where (> 1 0))`. `qB1` agrees
//! with the oracle at 1; `qB2` drops native to **0** while the oracle holds at 1. Independent of
//! chain depth — it reproduces at depth 0, so it is not a fixpoint issue at all.
//!
//! ## Why the existing corpus could not see either
//!
//! The accumulate axes (`accum`, `min-finding`) compare DERIVED FACTS, and `production_delta`
//! dedups those by value — so a rule deriving one distinct fact reads identically whether the
//! token passed once or four times. Every query here carries the rule's own LHS, so `query` reads
//! beta, below the dedup. That is the whole reason the fuzzer was built this way.
//!
//! ## Status: ALL THREE CLOSED — 2026-08-26. This file is now a REGRESSION gate.
//!
//! Every test here is live; nothing is `#[ignore]`d. The three families are closed and the
//! fuzzer's divergence count is **0**, so any red here is a regression with a named shape rather
//! than a known defect.
//!
//! - **B** closed 2026-08-26: a `:where` binding nothing sorts ABOVE the accumulate, and the
//!   accumulate pass (3.25) ran before the filter pass (3.5), so that leading Test had never
//!   fired when the accumulate read its parent delta.
//! - **A and C** closed 2026-08-26, and they turned out to be **ONE root, not two**: a query's
//!   NON-MONOTONIC condition was evaluated inside the fixpoint instead of once against the closed
//!   world. A constrained query is harvested from `wm.beta`, which accumulates across rounds and
//!   is never cleared, so it holds tokens that were only true of the round that produced them —
//!   one per round for a leading accumulate (A), and a round-0 negation token that nothing
//!   retracts when a later round derives the fact (C). Fixed at `fire_unstratified`
//!   (`src/rete/kernel/fire/rules.rs`), which gives the unstratified path the ending the
//!   stratified path always had.
//!
//! The agreeing controls stay non-ignored and are the reason a wrong fix could not have passed
//! here: they guard the shapes that already worked, so agreement bought by breaking the absent
//! case or by failing to derive the fact at all fails in this same file.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// `[B1-native B1-oracle  B2-native B2-oracle  A2-native A2-oracle  A3-native A3-oracle]`
fn rows() -> Vec<i64> {
    let out = call_beside_value(file!(), ":user::rows").expect("eval :user::rows");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    let got: Vec<i64> = items
        .iter()
        .map(|v| match v {
            Value::i64(n) => *n,
            other => panic!("expected i64; got {other:?}"),
        })
        .collect();
    assert_eq!(got.len(), 8, "witness shape changed: {got:?}");
    got
}

/// The CONTROL, and it must keep passing: fact condition + accumulate + ONE where agrees.
/// Without this, a fix for family B could "succeed" by breaking the case that already worked.
#[test]
fn accumulate_after_a_fact_condition_agrees_with_the_oracle() {
    let r = rows();
    assert_eq!(
        (r[0], r[1]),
        (1, 1),
        "the one-`where` accumulate shape stopped agreeing — native {} oracle {}. Full witness {r:?}",
        r[0],
        r[1]
    );
}

#[test]
fn a_second_where_after_an_accumulate_must_not_kill_the_match() {
    let r = rows();
    assert_eq!(
        r[1], 1,
        "oracle sanity: the two-`where` shape should match 1. Witness {r:?}"
    );
    assert_eq!(
        r[2], r[3],
        "native and oracle disagree on a shape differing from the agreeing control by ONE \
         trivially-true trailing `where`: native {} oracle {}. Witness {r:?}",
        r[2], r[3]
    );
}

#[test]
fn a_leading_accumulate_passes_once_per_fire_not_once_per_round() {
    let r = rows();
    // Slots 4 and 6 are the SAME query over the SAME facts; only the inert chain's length —
    // and so the round count — differs. Both must be 1, and they must equal each other: a fix
    // that special-cases the first round would satisfy one and not the other.
    assert_eq!(
        (r[4], r[6]),
        (1, 1),
        "leading accumulate emitted once per ROUND, not once per fire — 2-round chain gave {} \
         rows, 3-round chain gave {} (expected 1 and 1). Witness {r:?}",
        r[4],
        r[6]
    );
}

/// `[C-noChain-native C-noChain-oracle  C-chain-native C-chain-oracle  S2-native S2-oracle]`
fn rows_c() -> Vec<i64> {
    let out = call_beside_value(file!(), ":user::rows-c").expect("eval :user::rows-c");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    let got: Vec<i64> = items
        .iter()
        .map(|v| match v {
            Value::i64(n) => *n,
            other => panic!("expected i64; got {other:?}"),
        })
        .collect();
    assert_eq!(got.len(), 6, "witness shape changed: {got:?}");
    got
}

/// CONTROL, not ignored: with no rule deriving `S2`, the negation passes in both engines.
/// A fix for family C must not achieve agreement by breaking the absent case.
#[test]
fn negation_over_an_underived_class_agrees() {
    let r = rows_c();
    assert_eq!(
        (r[0], r[1]),
        (1, 1),
        "with S2 never derived, `not S2` must pass in both engines — native {} oracle {}. \
         Witness {r:?}",
        r[0],
        r[1]
    );
}

/// CONTROL, not ignored: both engines really do derive `S2`. Without this, family C could be
/// misread as native simply failing to derive the fact.
#[test]
fn both_engines_derive_the_fact_the_negation_should_see() {
    let r = rows_c();
    assert_eq!(
        (r[4], r[5]),
        (1, 1),
        "S2 must be derived by BOTH engines for family C to mean what it claims — native {} \
         oracle {}. Witness {r:?}",
        r[4],
        r[5]
    );
}

#[test]
fn negation_over_a_derived_class_must_see_the_derivation() {
    let r = rows_c();
    assert_eq!(
        r[3], 0,
        "oracle sanity: with S2 derived, `not S2` must block. Witness {r:?}"
    );
    assert_eq!(
        r[2], r[3],
        "native's `not` over a DERIVED class ignored the derivation: it passes ({}) where the \
         oracle blocks ({}), while native's own query confirms S2 is present. Stratified \
         negation requires the negated class's stratum to be complete first. Witness {r:?}",
        r[2], r[3]
    );
}
