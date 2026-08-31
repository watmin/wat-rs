//! `:wat::core::Option/expect` — arc 255 Stone A-2-ii-b-0, the first `:wat::core::Option::*`
//! verb to get a registry home. `:wat::core::Some` joined it arc 255 Stone A-2-ii-b-1 — the
//! tagged `Option` constructor `meter-2` made visible and parked; its sibling `Ok`/`Err`
//! constructors live in `src/intrinsic/result.rs`. `:wat::core::Option/try` joined arc 255
//! Stone the-option-result-siblings, homing the last `:wat::core::Option::*` verb — its
//! `Result`-side twin `Result/try` and `Option/expect`'s sibling `Result/expect` both live in
//! `src/intrinsic/result.rs`.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-2-ii-b-0-the-accessor-path-verbs-get-homes.md`
//! (`Option/expect`), `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-2-ii-b-1-the-option-result-constructors-get-homes.md`
//! (`Some`), `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-the-option-result-siblings.md`
//! (`Option/try`).
//!
//! Thin `#[wat_intrinsic]` delegates over pre-existing named fns (`src/runtime.rs`) — bodies do
//! not move, per the brief's two-layer architecture (`src/intrinsic/<ns>.rs` registers and
//! delegates, the implementation stays where it lived). `Option/expect` is homed here (not left
//! as a literal match arm) so a generated record accessor that raises through it en route to
//! `Record/field-at` stops classifying `impure`/`Unreviewed` when reached through an environment
//! binding — see `DESIGN-STONE-A-2-ii-b-0-the-accessor-path-verbs-get-homes.md`. `Some` is homed
//! for the same reason, one hop earlier: `(Some self)` is the accessor body's OUTER form, and it
//! denied the same way until this stone — see
//! `DESIGN-STONE-A-2-ii-b-1-the-option-result-constructors-get-homes.md`.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::{Environment, EvalBreak, SymbolTable, Value};
use crate::span::Span;

/// `(:wat::core::Option/expect opt msg) -> :T` — arc 108, canonical post-Stone-241.15 form.
///
/// On `Some(v)` returns `v`. On `None`, evaluates `msg` and raises (`panic_any`, caught by the
/// substrate's `catch_unwind`) — see `eval_option_expect`'s own doc for the full raise mechanics.
/// Homed here arc 255 Stone A-2-ii-b-0 with its real (2) arity declared; the hand-rolled
/// `args.len() != 2` guard in `eval_option_expect` retires. The body is unchanged, still in
/// `src/runtime.rs`.
///
/// **Purity ground:** both args are evaluated by ordinary call-by-value (not itself an effect).
/// Past that, the body only matches the already-evaluated `Option` and either returns the
/// wrapped value or evaluates `msg` to build a panic payload — no `eval_inner`/`apply_function`
/// on caller-supplied code beyond the two argument evaluations themselves. Pure ∧ Deterministic.
///
/// **Totality ground — pinned in the DESIGN, ruled by
/// `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`:** on `None`, this verb
/// `panic_any`s — a raise, not a matchable outcome. A raise can be deterministic and located and
/// still not be a value the caller can `match`. `Partial`.
///
/// **Expand-time ground —** Pure ∧ Deterministic and safe to evaluate during expansion; a
/// `Partial` verb can still be expand-time-legal, exactly as `macros/eval.rs` says for
/// `:wat::i64::/`'s division-by-zero: "a compile-time failure instead of a runtime one, which is
/// strictly better. Totality and expand-time legality are different axes." Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Projection
/// @arg     opt (:wat::core::Option :- [T]) the option unwrapped
/// @arg     msg :wat::core::String the message evaluated and raised if `opt` is `None`
/// @ret     :T the wrapped value, if `opt` is `Some`
/// @example (:wat::core::Option/expect (:wat::core::Some 3) "unreachable") #=> 3
/// @see     :wat::core::Record/field-at
#[wat_intrinsic(":wat::core::Option/expect")]
pub(crate) fn eval_option_expect(
    opt: &WatAST,
    msg: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_option_expect(
        ":wat::core::Option/expect",
        &[opt.clone(), msg.clone()],
        list_span,
        env,
        sym,
    )
}

/// `(:wat::core::Some v) -> (:wat::core::Option :- [T])` — the tagged constructor of the
/// built-in `Option` enum (058-030). `v`'s dual is the nullary keyword literal `:None`
/// (handled directly in `eval`, not a dispatch arm — out of scope here, see the DESIGN's
/// `Out of scope = REJECTED`); together they are the only way to produce `Value::Option`.
///
/// Homed here arc 255 Stone A-2-ii-b-1 with its real (1) arity declared; the hand-rolled
/// `args.len() != 1` guard in `eval_some_ctor` retires (unreachable once the shim itself
/// enforces arity 1 before calling in). The body is unchanged, still in `src/runtime.rs`. Its
/// `WatAST::Keyword(k, _) if k == ":wat::core::Some"` guard arm in `eval_list` is retired too —
/// `dispatch_keyword_head` now reaches this same body through the registry. The pre-existing
/// bare-Symbol form (`(Some v)`, no `:wat::core::` prefix) is untouched: a different dispatch
/// path this registration does not reach.
///
/// **Purity ground:** the one arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only wraps the already-evaluated value in `Value::Option(Arc::new(Some(_)))`
/// — no `eval_inner`/`apply_function` on caller-supplied code beyond that one argument
/// evaluation. Pure ∧ Deterministic.
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
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     v :T the value to wrap
/// @ret     (:wat::core::Option :- [T]) `v` wrapped as `Some`
/// @example (:wat::core::Some 3) #=> (:wat::core::Some 3)
/// @see     :wat::core::Option/expect
/// @see     :wat::core::Ok
/// @see     :wat::core::Err
#[wat_intrinsic(":wat::core::Some")]
pub(crate) fn eval_some_ctor(
    v: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_some_ctor(std::slice::from_ref(v), list_span, env, sym)
}

/// `(:wat::core::Option/try <option-expr>) -> :T` — the Option-side mirror of
/// `:wat::core::Result/try` (`src/intrinsic/result.rs`'s `eval_result_try`). Unwraps a
/// `(:wat::core::Option :- [T])` to its inner `T`, or short-circuits the enclosing
/// Option-returning function with `:None`.
///
/// Homed here arc 255 Stone the-option-result-siblings with its real (1) arity declared; the
/// hand-rolled `args.len() != 1` guard in `eval_option_try` retires. The body is unchanged,
/// still in `src/runtime.rs`.
///
/// **Purity ground:** the one arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only matches the already-evaluated `Option` and either returns
/// the wrapped value or raises a propagate signal — no `eval_inner`/`apply_function` on
/// caller-supplied code beyond that one argument evaluation. Pure ∧ Deterministic.
///
/// **Totality ground — pinned in the DESIGN, ruled by
/// `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`:** on `:None`, this verb
/// returns `Err(EvalBreak::Signal(EvalSignal::OptionPropagate))` — a **signal**, not a raise.
/// `runtime.rs:19493-19495`'s `apply_function` catches it and packages it as the enclosing
/// function's own `Value::Option(Arc::new(None))` return, mirroring `TryPropagate`'s handling
/// just above it (`:19458`, quoted on `Result/try`'s doc) — the checker
/// (`crate::check::infer_option_try`) guarantees the enclosing function returns
/// `(:wat::core::Option :- [_])` whenever its body contains an `Option/try`, so the wrap is
/// always a matchable `Some`/`None` arm. `Total` — the opposite verdict from its `expect`
/// sibling above, because the body does the opposite thing: propagate, not panic.
///
/// **Expand-time ground —** Pure ∧ Deterministic ∧ Total; safe to evaluate during expansion.
/// Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Projection
/// @arg     opt (:wat::core::Option :- [T]) the option unwrapped
/// @ret     :T the wrapped value, if `opt` is `Some`; otherwise short-circuits the enclosing
///   function with `:None`
/// @example (:wat::core::Option/try (:wat::core::Some 3)) #=> 3
/// @see     :wat::core::Option/expect
/// @see     :wat::core::Result/try
#[wat_intrinsic(":wat::core::Option/try")]
pub(crate) fn eval_option_try(
    opt: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_option_try(
        ":wat::core::Option/try",
        std::slice::from_ref(opt),
        list_span,
        env,
        sym,
    )
}
