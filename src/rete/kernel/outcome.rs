//! `(:wat::rete::FireOutcome)` — a fire's bounded result as a matchable VALUE.
//!
//! ── WHY THIS MODULE EXISTS ───────────────────────────────────────────────────────────────────
//!
//! A session carries two ceilings — `max-fire-rounds` and `max-session-bytes` — and both are
//! enforced at RUNTIME because neither can be proven at load. eBPF refuses an unbounded program
//! statically; this arc MEASURED that we cannot follow it: a guarded counter's bound is its SEED,
//! which is input data (`12cdf4081` — *provably TERMINATING, not provably BOUNDED*). So the
//! failure is irreducibly dynamic, and a dynamic failure in this substrate is a value a caller
//! must match, never a raise that unwinds past them. Builder: *"let's impose session's strict
//! limits via totality."*
//!
//! The wall this joins is not new — `RecvOutcome`, `SendOutcome`, `TrySendOutcome`, `CloseOutcome`
//! put it on comms one layer over (`DESIGN-recv-outcome-wall.md`). The engine's own verbs were the
//! odd ones out, and they are the two with ceilings.
//!
//! ── WHERE THE WALL STANDS, AND WHY THE RAISE BELOW IT IS NOT A HOLE ──────────────────────────
//!
//! The fixpoint (`fire/delta.rs`) still RAISES a located `RuntimeError` on a breach, and this
//! module converts it at the verb boundary. That raise is **rust-to-rust**: it never reaches wat,
//! because every wat-facing fire verb passes through here. The wall is at the LANGUAGE boundary,
//! which is where totality is owed. Pushing the enum down into `fire_fixpoint_delta_armed` itself
//! is the tidier shape and is deliberately NOT done yet — it has three callers (`fire-once`,
//! `fire-rules`, and the query path at `fire/rules.rs:425`), so it becomes worth doing when the
//! second door arrives, not on the strength of one.
//!
//! ⛔ **DO NOT ADD A SECOND CONVERSION SITE.** If a new fire verb needs an outcome, call
//! [`fire_result_to_outcome`]. Two places deciding which breach maps to which arm is the drift
//! this arc pulls out most often.

use std::sync::Arc;

use crate::runtime::{EvalBreak, EnumValue, RuntimeErrorKind, Value};

/// `(:wat::rete::FireOutcome)`, declared in `wat/rete.wat`.
const FIRE_OUTCOME_TYPE: &str = ":wat::rete::FireOutcome";

/// `FireOutcome::Fired [session]` — the happy path.
fn fired(session: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: FIRE_OUTCOME_TYPE.into(),
        variant_name: "Fired".into(),
        names: crate::runtime::builtin_enum_variant_names(FIRE_OUTCOME_TYPE, "Fired"),
        fields: vec![session],
    }))
}

/// `FireOutcome::MemoryCeilingExceeded [limit used rounds]`.
fn memory_ceiling_exceeded(limit: usize, used: usize, rounds: usize) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: FIRE_OUTCOME_TYPE.into(),
        variant_name: "MemoryCeilingExceeded".into(),
        names: crate::runtime::builtin_enum_variant_names(
            FIRE_OUTCOME_TYPE,
            "MemoryCeilingExceeded",
        ),
        fields: vec![
            Value::i64(limit as i64),
            Value::i64(used as i64),
            Value::i64(rounds as i64),
        ],
    }))
}

/// `FireOutcome::RoundCapExceeded [cap still-deriving]`.
fn round_cap_exceeded(cap: usize, still_deriving: usize) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: FIRE_OUTCOME_TYPE.into(),
        variant_name: "RoundCapExceeded".into(),
        names: crate::runtime::builtin_enum_variant_names(FIRE_OUTCOME_TYPE, "RoundCapExceeded"),
        fields: vec![
            Value::i64(cap as i64),
            Value::i64(still_deriving as i64),
        ],
    }))
}

/// Turn a fire's `Result` into a `FireOutcome` VALUE.
///
/// The two ceiling breaches become matchable arms; **everything else propagates unchanged.** That
/// asymmetry is the design, not an oversight: a ceiling is a bound the substrate CHOSE and the
/// caller can act on (fire in batches, raise the ceiling, bound the derivation). A malformed
/// session or a type error is a bug in the program, and turning those into arms would hand every
/// caller a match over failures they cannot do anything about — the "make every error a value"
/// overreach that a closed outcome enum exists to avoid.
pub(crate) fn fire_result_to_outcome(r: Result<Value, EvalBreak>) -> Result<Value, EvalBreak> {
    match r {
        Ok(session) => Ok(fired(session)),
        Err(EvalBreak::Diagnostic(e)) => match e.kind() {
            RuntimeErrorKind::SessionMemoryCeilingExceeded {
                limit,
                used,
                rounds,
            } => Ok(memory_ceiling_exceeded(*limit, *used, *rounds)),
            RuntimeErrorKind::FixpointRoundCapExceeded {
                cap,
                still_deriving,
            } => Ok(round_cap_exceeded(*cap, *still_deriving)),
            _ => Err(EvalBreak::Diagnostic(e)),
        },
        Err(other) => Err(other),
    }
}
