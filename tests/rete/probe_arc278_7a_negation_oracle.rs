//! Arc 278 stone 7-a — negation (`:not` / NegationNode) in the oracle (`fire-rules$oracle`).
//! Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! Probed through `fire-rules$oracle` — 7-a builds the oracle NegationNode; the native port +
//! differential are 7-b. A `:not` passes a token iff NO fact matches the negated condition for that
//! token's bindings (the shared `?loc` must agree — the join-filter half). Absent → 1 Unattended;
//! present-matching → 0; present-different-binding → 1.
//!
//! Run: cargo test --release -p wat --test probe_arc278_7a_negation_oracle

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

fn count(entry: &str) -> Result<i64, RuntimeError> {
    match call_beside_value(file!(), entry)? {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: format!("count({entry})"),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
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
