//! `:wat::stream::{empty,cons,next}` — arc 255 Stone P6-c-W2, the P6-c campaign's
//! second wave.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-P6-c-W2-stream-program-stdlib.md`.
//!
//! Three of the arc 118 lazy-Stream foundation primitives, moved verbatim out of
//! `runtime.rs`'s giant match. `:wat::stream::lazy` is a SPECIAL FORM (capture-don't-eval,
//! mirrors `quote`) and stays there — `DESTINATION_LEDGER`
//! (`wat-scripts/hunt/p6c-disposition-census.py`) rules it separately.
//!
//! ★ Same H-1a arity fix as W1's `:wat::config::*`: all three declared a variadic
//! `&[WatAST]` they used only to reject via a hand-rolled length check — publishing a
//! fictional `Arity::Variadic` for verbs whose real arity is 0/2/1. Real arity now,
//! shim-owned: `empty` drops `args` entirely (0 leading params); `cons`/`next` swap their
//! `&[WatAST]` + length check for individually typed `&WatAST` leading params (2 and 1).
//!
//! ★★ `next`'s purity is NOT a copy of `cons`/`empty`'s, and is not inferred from the
//! namespace. `cons`/`empty` are constructors: no side effect is possible regardless of
//! what ends up inside the cell — the tail is stored, never entered. `next` FORCES a
//! thunk (`crate::stream::realize` calls `apply_function` on a captured wat closure for a
//! `Thunk`, or runs a Rust closure for a `NativeThunk`) — running arbitrary user code this
//! verb has no way to bound. That is exactly the shape `:wat::core::apply`/`:wat::eval`
//! are left deliberately unclassified for in `rete/purity.rs` ("purity is the form's, like
//! apply"). Independent corroboration, not copied: `src/macros/eval.rs`'s `is_pure_total`
//! expand-time allowlist already listed `cons`/`empty`/`lazy` as safe to evaluate at macro
//! expansion time, and already did NOT list `next` — before this stone ever touched either
//! file. So `next` is homed `@Purity Effectful @Determinism Nondeterministic`: a grounded
//! ruling, not a refusal, and not a guess.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::{builtin_enum_variant_names, eval_inner, no_field_names};
use crate::span::Span;
use crate::value::{
    EnumValue, EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};

/// Arc 118.11a — the type path of `next`'s matchable outcome enum
/// (`(:wat::stream::NextOutcome :- [T])`, registered in `types.rs`). Moved here with
/// `eval_stream_next_intrinsic` — `next_outcome_item`/`next_outcome_exhausted` have no
/// other caller.
const NEXT_OUTCOME_TYPE: &str = ":wat::stream::NextOutcome";

/// `NextOutcome::Item [value <- T, rest <- (Stream :- [T])]` — the forced head plus the
/// undrained tail, both from the SAME single force. Mirrors `recv_outcome_message`.
fn next_outcome_item(value: Value, rest: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: NEXT_OUTCOME_TYPE.into(),
        variant_name: "Item".into(),
        names: builtin_enum_variant_names(NEXT_OUTCOME_TYPE, "Item"),
        fields: vec![value, rest],
    }))
}

/// `NextOutcome::Exhausted []` — the named end; no more elements. Mirrors
/// `recv_outcome_closed`.
fn next_outcome_exhausted() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: NEXT_OUTCOME_TYPE.into(),
        variant_name: "Exhausted".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `(:wat::stream::empty) -> (Stream :- [T])`. The Empty terminator.
///
/// Zero-arg constructor producing `Value::wat__stream__Stream(Arc::new(Stream::Empty))`.
/// No side effect is possible: it reads nothing, evaluates nothing, and its output never
/// varies — the same shape `:wat::uuid::nil`/`:wat::time::now`'s SIBLING zero-param
/// constructors take (unlike `time::now`, this one is also deterministic: there is no
/// external source to sample).
///
/// **Expand-time ground —** pure/total (no IO, no randomness, no channels): safe at
/// macro-expansion time. Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc
/// 255 expand-T4a; arc 118.2a), from its `:wat::stream::*` primitives group; the verdict is
/// that list's.
///
/// Arc 255 Stone the-registry-answers-first-wave-2 — re-derived from `eval_stream_empty_intrinsic`
/// immediately below: a zero-arg constructor with a single unconditional `Ok(...)` return, no
/// argument to be malformed, no raise path at all. Total.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @ret     (:wat::stream::Stream :- [T]) the Empty terminator
/// @example (:wat::core::stream->vec [] (:wat::stream::empty)) #=> []
#[wat_intrinsic(":wat::stream::empty")]
pub(crate) fn eval_stream_empty_intrinsic() -> Result<Value, EvalBreak> {
    Ok(Value::wat__stream__Stream(Arc::new(
        crate::stream::Stream::Empty,
    )))
}

/// `(:wat::stream::cons head tail) -> (Stream :- [T])`. Strict-head Cons cell.
///
/// `head` is evaluated (strict); `tail` is evaluated and must be a `Stream` (it may itself
/// be a Thunk — O(1), no forcing here). A pure reshape: it stores exactly what it is
/// handed and never enters `tail` to look inside — forcing (and whatever `tail` might do
/// when forced) is `:wat::stream::next`'s job, not this one's.
///
/// **Expand-time ground —** pure/total (no IO, no randomness, no channels): safe at
/// macro-expansion time. Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc
/// 255 expand-T4a; arc 118.2a), from its `:wat::stream::*` primitives group; the verdict is
/// that list's.
///
/// Arc 255 Stone the-registry-answers-first-wave-2 — re-derived from `eval_stream_cons_intrinsic`
/// immediately below: `tail`'s declared type is `(:wat::stream::Stream :- [T])`, this verb is
/// checked normally (not on `intrinsic/mod.rs`'s `FROZEN_CHECKER_DEBT_LEDGER`), so a well-typed
/// call's `tail_val` is always `Value::wat__stream__Stream`; the `other =>` `TypeMismatch` arm
/// is checker-impossible, same shape as `:wat::hashmap::keys`. Total.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     head :T the value prepended, evaluated strictly
/// @arg     tail (:wat::stream::Stream :- [T]) the stream tail (may itself be an unforced Thunk)
/// @ret     (:wat::stream::Stream :- [T]) the new Cons cell
/// @example (:wat::core::stream->vec [] (:wat::stream::cons 1 (:wat::stream::empty))) #=> [1]
#[wat_intrinsic(":wat::stream::cons")]
pub(crate) fn eval_stream_cons_intrinsic(
    head: &WatAST,
    tail: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let head_val = eval_inner(head, env, sym)?.value_owned();
    let tail_val = eval_inner(tail, env, sym)?.value_owned();
    let tail_stream = match tail_val {
        Value::wat__stream__Stream(st) => st,
        other => {
            return Err(RuntimeError::new(
                tail.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::stream::cons".into(),
                    expected: "wat::stream::Stream",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };
    Ok(Value::wat__stream__Stream(Arc::new(
        crate::stream::Stream::Cons { head: head_val, tail: tail_stream },
    )))
}

/// `(:wat::stream::next s) -> (NextOutcome :- [T])`. The pull primitive.
///
/// Forces `s` to WHNF via `crate::stream::realize` (exactly one force per call —
/// `realize`'s own loop already stops at the first `Empty`/`Cons`; this fn does not add a
/// second forcing loop or a cache of its own) and destructures the result: `Empty` →
/// `Exhausted`; `Cons{head, tail}` → `Item(head, rest)`.
///
/// ★ `realize` forcing a `Thunk` calls `apply_function` on a captured wat closure (the
/// body of `(:wat::stream::lazy <body>)`), and forcing a `NativeThunk` runs a Rust closure
/// backing the lazy `map`/`filter`/`take`/`drop` family — either can run ARBITRARY code:
/// I/O, a clock read, randomness, a `raise`, another `next` on an unrelated stream. `next`
/// itself has no way to bound what it is about to run, so its purity cannot honestly be
/// `Pure`/`Deterministic` — that would be exactly the lie `:wat::core::apply`/`:wat::eval`
/// are deliberately left unclassified in `rete/purity.rs` to avoid ("purity is the form's,
/// like apply"). Corroboration, not derivation: `src/macros/eval.rs`'s `is_pure_total`
/// expand-time-safe allowlist already listed `cons`/`empty`/`lazy` and already did NOT
/// list `next`, before this stone ever touched either file — the same conclusion, reached
/// independently, by a completely different mechanism (what is safe to run early) built
/// for a completely different reason.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      ControlFlow
/// @arg     s (:wat::stream::Stream :- [T]) the stream forced to WHNF
/// @ret     (:wat::stream::NextOutcome :- [T]) `Item(value, rest)`, or `Exhausted`
/// @example-norun (:wat::stream::next (:wat::stream::cons 1 (:wat::stream::empty))) #=> (:wat::stream::NextOutcome::Item 1 (:wat::stream::empty))
#[wat_intrinsic(":wat::stream::next")]
pub(crate) fn eval_stream_next_intrinsic(
    s: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let seq_val = eval_inner(s, env, sym)?.value_owned();
    let seq = match seq_val {
        Value::wat__stream__Stream(st) => st,
        other => {
            return Err(RuntimeError::new(
                s.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::stream::next".into(),
                    expected: "wat::stream::Stream",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };
    let whnf = crate::stream::realize(&seq, sym, list_span)?;
    match whnf.as_ref() {
        crate::stream::Stream::Empty => Ok(next_outcome_exhausted()),
        crate::stream::Stream::Cons { head, tail } => Ok(next_outcome_item(
            head.clone(),
            Value::wat__stream__Stream(Arc::clone(tail)),
        )),
        // INVARIANT (src/stream/mod.rs `realize` doc): realize always terminates
        // with Empty or Cons; Thunk/NativeThunk is the only transitional state.
        crate::stream::Stream::Thunk(_) | crate::stream::Stream::NativeThunk(_) => {
            unreachable!("realize returns only Empty|Cons (WHNF invariant, src/stream/mod.rs)")
        }
    }
}
