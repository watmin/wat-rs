//! Arc 278 — Stone 7-a: negation (`:not` / NegationNode) in the ORACLE (`rete.wat` compile + fire).
//! RED at HEAD (no NegationNode: compile-condition treats `(:wat::rete::not …)` as an unsatisfiable
//! alpha → the rule never fires). GREEN when 7-a lands. Contract: DESIGN-STONE-7-negation.md.
//!
//! Probed through the ORACLE (`fire-rules-spec`) — 7-a builds the oracle NegationNode; the native port +
//! differential are 7-b. A `:not` passes a token iff NO fact matches the negated condition for that
//! token's bindings (the shared `?loc` must agree — the join-filter half).
//!
//! Run: cargo test --release -p wat --test probe_arc278_7a_negation_oracle

use wat::freeze::call_beside;
use wat::runtime::Value;

fn count(entry: &str) -> Result<i64, String> {
    match call_beside(file!(), entry).map_err(|e| format!("eval: {e:?}"))? {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64; got {other:?}")),
    }
}

/// 1 — `:not` PASSES when the negated fact is ABSENT: Temp(Oslo), no Maintenance → 1 Unattended.
#[test]
fn negation_passes_when_absent() {
    let r = count(":user::unattended-count-absent");
    assert!(matches!(r, Ok(1)), "no Maintenance at Oslo → 1 Unattended; got {r:?}");
}

/// 2 — `:not` BLOCKS when the negated fact is PRESENT and MATCHES: Temp(Oslo) + Maintenance(Oslo) → 0.
#[test]
fn negation_blocks_when_present_matching() {
    let r = count(":user::unattended-count-present-matching");
    assert!(matches!(r, Ok(0)), "Maintenance at Oslo → 0 Unattended; got {r:?}");
}

/// 3 — `:not` PASSES when a negated fact exists but at a DIFFERENT binding (the shared-var join-filter):
/// Temp(Oslo) + Maintenance(Bergen) → the Bergen maintenance does NOT match ?loc=Oslo → 1 Unattended.
#[test]
fn negation_passes_when_present_different_binding() {
    let r = count(":user::unattended-count-present-different");
    assert!(matches!(r, Ok(1)), "Maintenance at Bergen ≠ Oslo → 1 Unattended; got {r:?}");
}
