//! `:wat::core::Ok` / `:wat::core::Err` — arc 255 Stone A-2-ii-b-1, the tagged `Result`
//! constructors `meter-2` made visible and parked. Siblings of `:wat::core::Some`
//! (`src/intrinsic/option.rs`) — same shape, same `eval_list` keyword-guard dispatch, same
//! parked `KNOWN_UNREVIEWED` row, homed the same stone. No pre-existing `src/result/` or
//! `Result`-namespaced module to extend, so this is a new "own home, same shape" file — the
//! same call `src/intrinsic/list.rs`/`bytes.rs`/`char.rs`/`regex.rs` made for a self-contained,
//! few-verb family with no `Result`-shaped verbs already homed elsewhere.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-2-ii-b-1-the-option-result-constructors-get-homes.md`.
//!
//! Thin `#[wat_intrinsic]` delegates over pre-existing named fns (`src/runtime.rs`) — bodies do
//! not move, per the brief's two-layer architecture (`src/intrinsic/<ns>.rs` registers and
//! delegates, the implementation stays where it lived).

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::{Environment, EvalBreak, SymbolTable, Value};
use crate::span::Span;

/// `(:wat::core::Ok v) -> (:wat::core::Result :- [T E])` — the `Ok`-arm tagged constructor of
/// the built-in `Result` enum. `v`'s dual is `:wat::core::Err`, just below.
///
/// Homed here arc 255 Stone A-2-ii-b-1 with its real (1) arity declared; the hand-rolled
/// `args.len() != 1` guard in `eval_ok_ctor` retires (unreachable once the shim itself enforces
/// arity 1 before calling in). The body is unchanged, still in `src/runtime.rs`. Its
/// `WatAST::Keyword(k, _) if k == ":wat::core::Ok"` guard arm in `eval_list` is retired too —
/// `dispatch_keyword_head` now reaches this same body through the registry. The pre-existing
/// bare-Symbol form (`(Ok v)`, no `:wat::core::` prefix) is untouched: a different dispatch
/// path this registration does not reach.
///
/// **Purity ground:** the one arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only wraps the already-evaluated value in
/// `Value::Result(Arc::new(Ok(_)))` — no `eval_inner`/`apply_function` on caller-supplied code
/// beyond that one argument evaluation. Pure ∧ Deterministic.
///
/// **Totality ground — pinned in the DESIGN:** the only `return Err` in the body is the arity
/// check, which retires on homing (the shim enforces arity 1 before this fn is ever called).
/// Past that the wrap cannot fail — there is no raise, no bounds check, nothing to deny.
/// `Total`.
///
/// **Expand-time ground —** Pure ∧ Deterministic ∧ Total; safe to evaluate during expansion.
/// Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     v :T the value to wrap
/// @ret     (:wat::core::Result :- [T E]) `v` wrapped as `Ok`
/// @example (:wat::core::Ok 3) #=> (:wat::core::Ok 3)
/// @see     :wat::core::Err
/// @see     :wat::core::Some
#[wat_intrinsic(":wat::core::Ok")]
pub(crate) fn eval_ok_ctor(
    v: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_ok_ctor(std::slice::from_ref(v), list_span, env, sym)
}

/// `(:wat::core::Err v) -> (:wat::core::Result :- [T E])` — the `Err`-arm tagged constructor of
/// the built-in `Result` enum. `v`'s dual is `:wat::core::Ok`, just above.
///
/// ⚠ **`Err` is a constructor, not a failure.** `(Err v)` *builds* a `Result` value; it does not
/// raise. Under `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`, a matchable
/// error-bearing arm is exactly the shape that ruling calls **total** — `Err` is that shape, not
/// the raising shape the ruling calls `Partial`. Nothing in this body ever `panic_any`s or
/// returns an `EvalBreak::Diagnostic`; it constructs a value the caller `match`es, same as `Ok`.
///
/// Homed here arc 255 Stone A-2-ii-b-1 with its real (1) arity declared; the hand-rolled
/// `args.len() != 1` guard in `eval_err_ctor` retires (unreachable once the shim itself enforces
/// arity 1 before calling in). The body is unchanged, still in `src/runtime.rs`. Its
/// `WatAST::Keyword(k, _) if k == ":wat::core::Err"` guard arm in `eval_list` is retired too —
/// `dispatch_keyword_head` now reaches this same body through the registry. The pre-existing
/// bare-Symbol form (`(Err v)`, no `:wat::core::` prefix) is untouched: a different dispatch
/// path this registration does not reach.
///
/// **Purity ground:** the one arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only wraps the already-evaluated value in
/// `Value::Result(Arc::new(Err(_)))` — no `eval_inner`/`apply_function` on caller-supplied code
/// beyond that one argument evaluation. Pure ∧ Deterministic.
///
/// **Totality ground — pinned in the DESIGN, see the warning above:** the only `return Err`
/// (Rust's `Result::Err`, the shim's own arity failure — not this verb's own `:wat::core::Err`)
/// in the body is the arity check, which retires on homing (the shim enforces arity 1 before
/// this fn is ever called). Past that the wrap cannot fail. `Total`.
///
/// **Expand-time ground —** Pure ∧ Deterministic ∧ Total; safe to evaluate during expansion.
/// Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     v :E the value to wrap
/// @ret     (:wat::core::Result :- [T E]) `v` wrapped as `Err`
/// @example (:wat::core::Err "boom") #=> (:wat::core::Err "boom")
/// @see     :wat::core::Ok
/// @see     :wat::core::Some
#[wat_intrinsic(":wat::core::Err")]
pub(crate) fn eval_err_ctor(
    v: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_err_ctor(std::slice::from_ref(v), list_span, env, sym)
}
