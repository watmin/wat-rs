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

/// `FireOutcome::Fired [value]` — the happy path.
///
/// The payload is whatever the fire produced: a `Session` for `fire-rules`/`fire-once`, an
/// `Explained` for `fire-rules-explain`. The enum is parametric precisely so both fit one type
/// (`wat/rete.wat`); Rust builds the same variant either way, since a `Value` carries its own tag.
fn fired(value: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: FIRE_OUTCOME_TYPE.into(),
        variant_name: "Fired".into(),
        names: crate::runtime::builtin_enum_variant_names(FIRE_OUTCOME_TYPE, "Fired"),
        fields: vec![value],
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

// ── `(:wat::rete::InsertOutcome)` — the staging door's twin ───────────────────

/// `(:wat::rete::InsertOutcome)`, declared in `wat/rete.wat`.
const INSERT_OUTCOME_TYPE: &str = ":wat::rete::InsertOutcome";

/// `InsertOutcome::Inserted [session]` — the happy path.
fn inserted(session: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: INSERT_OUTCOME_TYPE.into(),
        variant_name: "Inserted".into(),
        names: crate::runtime::builtin_enum_variant_names(INSERT_OUTCOME_TYPE, "Inserted"),
        fields: vec![session],
    }))
}

/// `InsertOutcome::MemoryCeilingExceeded [limit used staged]`.
fn insert_memory_ceiling_exceeded(limit: usize, used: usize, staged: usize) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: INSERT_OUTCOME_TYPE.into(),
        variant_name: "MemoryCeilingExceeded".into(),
        names: crate::runtime::builtin_enum_variant_names(
            INSERT_OUTCOME_TYPE,
            "MemoryCeilingExceeded",
        ),
        fields: vec![
            Value::i64(limit as i64),
            Value::i64(used as i64),
            Value::i64(staged as i64),
        ],
    }))
}

/// Turn a staging `Result` into an `InsertOutcome` VALUE.
///
/// ⛔ **ONE ARM, NOT TWO** — the mirror of [`fire_result_to_outcome`], and deliberately narrower.
/// `insert` runs no rounds, so `FixpointRoundCapExceeded` is not merely unhandled here, it is
/// **unreachable**: nothing on the staging path can construct it. Adding a third arm "for
/// symmetry" would put a branch in the reader's way that no input can take.
///
/// Everything that is not the staging ceiling propagates unchanged, for the reason stated on
/// [`fire_result_to_outcome`]: a ceiling is a bound the substrate chose and the caller can act on;
/// a malformed session or a non-Record fact is a bug in the program, and turning those into arms
/// would hand every caller a match over failures they cannot do anything about.
pub(crate) fn insert_result_to_outcome(r: Result<Value, EvalBreak>) -> Result<Value, EvalBreak> {
    match r {
        Ok(session) => Ok(inserted(session)),
        Err(EvalBreak::Diagnostic(e)) => match e.kind() {
            RuntimeErrorKind::SessionMemoryCeilingExceededOnInsert {
                limit,
                used,
                staged,
            } => Ok(insert_memory_ceiling_exceeded(*limit, *used, *staged)),
            _ => Err(EvalBreak::Diagnostic(e)),
        },
        Err(other) => Err(other),
    }
}

// ── `(:wat::rete::CompileOutcome)` — the termination verdict ──────────────────

/// `(:wat::rete::CompileOutcome)`, declared in `wat/rete.wat`.
const COMPILE_OUTCOME_TYPE: &str = ":wat::rete::CompileOutcome";

/// `CompileOutcome::Compiled [session]` — the armed session.
fn compiled(session: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: COMPILE_OUTCOME_TYPE.into(),
        variant_name: "Compiled".into(),
        names: crate::runtime::builtin_enum_variant_names(COMPILE_OUTCOME_TYPE, "Compiled"),
        fields: vec![session],
    }))
}

/// `CompileOutcome::MayNotTerminate [rule fact-type]`.
fn may_not_terminate(rule: &str, fact_type: &str) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: COMPILE_OUTCOME_TYPE.into(),
        variant_name: "MayNotTerminate".into(),
        names: crate::runtime::builtin_enum_variant_names(COMPILE_OUTCOME_TYPE, "MayNotTerminate"),
        fields: vec![
            Value::String(Arc::new(rule.to_string())),
            Value::String(Arc::new(fact_type.to_string())),
        ],
    }))
}

/// Turn `arm-session`'s `Result` into a `CompileOutcome` VALUE.
///
/// ⛔ **ONE ARM CONVERTS; THE REST STILL RAISE, AND THAT IS THE DESIGN.** `arm-session` can also
/// refuse an `ArityMismatch` or a `Session` argument that is not a Session. Those are BUGS IN THE
/// PROGRAM — statically preventable, and nothing a caller could branch on — so they propagate
/// unchanged. `RuleSetMayNotTerminate` is different in kind: it is a judgement about the caller's
/// **data**, reachable only for rule sets built at runtime (declared rules are refused at freeze,
/// before the program runs), and its diagnostic names an action the author can take.
///
/// The line is the same one [`fire_result_to_outcome`] draws for the ceilings: a bound or verdict
/// the caller can act on becomes a value; a malformed call stays a raise.
pub(crate) fn compile_result_to_outcome(r: Result<Value, EvalBreak>) -> Result<Value, EvalBreak> {
    match r {
        Ok(session) => Ok(compiled(session)),
        Err(EvalBreak::Diagnostic(e)) => match e.kind() {
            RuntimeErrorKind::RuleSetMayNotTerminate { rule, fact_type } => {
                Ok(may_not_terminate(rule, fact_type))
            }
            _ => Err(EvalBreak::Diagnostic(e)),
        },
        Err(other) => Err(other),
    }
}
