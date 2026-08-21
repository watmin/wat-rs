//! Arc 278 — "alpha is fire-scoped": a natively-fired `Session` must not return an
//! alpha-memory the oracle does not. Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! v1 of this gate assumed the oracle (`fire-rules$oracle`) returns a populated alpha and re-pointed
//! everything at it; that assumption was FALSE and a rider's STOP-4 caught it before anything shipped
//! (`fire-stratified`, `rete.wat:1817-1820`, explicitly zeroes alpha-memory/beta-memory when it packs
//! its final `Session`). Native `fire-rules` and `fire-rules$oracle` both return empty alpha.
//! `9d9a4e77` measured serializing a populated alpha at 31.3% of fire on its own. Clearing it in
//! `fire_fixpoint_delta` (the function unprimed `fire-rules` runs) closes the divergence and removes
//! the cost as a side effect.
//!
//! `fire_once_session` is deliberately UNTOUCHED: native `fire-once` mirrors the oracle's `fire-once$oracle`,
//! which genuinely fills alpha (`rete.wat:1462`) — narrowing the cut to the fixpoint verb only is what
//! keeps that single-pass pair aligned, per DESIGN-STONE-alpha-is-fire-scoped.md v2.
//!
//! Workload: `probe_arc278_2b_insert_alpha.wat`'s smallest alpha-populating shape (`:afs::Temp` +
//! `(> ?t 20)`, staging a matching fact (25) and a non-matching one (15)), extended with a non-empty
//! RHS (`:afs::Hot`) so a derived-fact differential exists too — 2b's RHS was empty.
//!
//! What would turn this red once it is green — the R59 question, answered before the assertions were
//! written (v1's lesson, encoded: every claim about what the ORACLE returns is asserted here, never
//! assumed):
//!   (1) native-alpha-key-count != 0        — the clear landed somewhere off the measured fire path,
//!       or a later edit reverted / bypassed it.
//!   (2) oracle-alpha-key-count != 0        — `fire-stratified` started carrying alpha through after
//!       all, which would mean the divergence this stone closes was never real, or is real no longer.
//!   (3) (1) != (2)                         — the divergence is not actually closed: native and the
//!       oracle disagree on alpha.
//!   (4) single-pass-alpha-key-count == 0   — THE ANCHOR. The workload stopped matching (a broken
//!       condition, a wrong staged value), which would make (1)/(2)/(3) all vacuously true over a
//!       no-match run rather than a proof of anything.
//!   (5) native-derived-count != oracle-derived-count, or either == 0 — the alpha-clear moved the
//!       RESULT, which the contract forbids (the clear sits at the fixpoint fire path's own freeze
//!       boundary, after production is already computed; it must never touch production), or the
//!       differential itself is vacuous (no rule ever fired).
//!
//! Run: cargo nextest run --release -E 'test(/alpha_is_fire_scoped/)'

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn count(entry: &str) -> i64 {
    match call_beside_value(file!(), entry).unwrap_or_else(|e| panic!("eval {entry}: {e:?}")) {
        Value::i64(n) => n,
        other => panic!("{entry}: expected i64; got {other:?}"),
    }
}

/// 1 — the clear happened: a natively-fired (fixpoint) Session carries zero alpha keys.
#[test]
fn native_alpha_is_cleared() {
    let native = count(":user::native-alpha-key-count");
    assert_eq!(native, 0, "native fixpoint fire must clear alpha before freeze; got {native} keys");
}

/// 2 + 3 — the oracle's own state, asserted not assumed: `fire-rules$oracle` returns alpha empty, and
/// that matches native's now-cleared alpha — the divergence this stone exists to close.
#[test]
fn oracle_alpha_matches_native_at_zero() {
    let oracle = count(":user::oracle-alpha-key-count");
    assert_eq!(oracle, 0, "oracle fire-rules$oracle must return alpha empty (fire-stratified); got {oracle} keys");
    let native = count(":user::native-alpha-key-count");
    assert_eq!(native, oracle, "the divergence must be closed: native==oracle (alpha); native={native} oracle={oracle}");
}

/// 4 — THE ANCHOR: guards 1-3 against vacuous-green. The single-pass verb (deliberately untouched by
/// this stone, and aligned with the oracle's own `fire-once`) proves this workload really populates
/// alpha — without this, a workload matching nothing would pass 1-3 just as green.
#[test]
fn single_pass_alpha_is_the_anchor() {
    let single_pass = count(":user::single-pass-alpha-key-count");
    assert!(single_pass > 0, "fire-once must still populate alpha (untouched by this stone); got {single_pass} keys");
}

/// 5 — the RESULT is untouched: native and oracle derive the same non-zero count.
#[test]
fn derived_result_is_unmoved() {
    let native = count(":user::native-derived-count");
    let oracle = count(":user::oracle-derived-count");
    assert_eq!(native, oracle, "native==oracle (derived); native={native} oracle={oracle}");
    assert!(native > 0, "the differential must not be vacuous; got {native} derived facts");
}
