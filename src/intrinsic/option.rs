//! `:wat::core::Option/expect` — arc 255 Stone A-2-ii-b-0, the first `:wat::core::Option::*`
//! verb to get a registry home.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-2-ii-b-0-the-accessor-path-verbs-get-homes.md`.
//!
//! Thin `#[wat_intrinsic]` delegate over the pre-existing `eval_option_expect` (`src/runtime.rs`)
//! — the body does not move, per the brief's two-layer architecture (`src/intrinsic/<ns>.rs`
//! registers and delegates, the implementation stays where it lived). Homed here (not left as a
//! literal match arm) so a generated record accessor that raises through `Option/expect` en
//! route to `Record/field-at` stops classifying `impure`/`Unreviewed` when reached through an
//! environment binding — see `DESIGN-STONE-A-2-ii-b-0-the-accessor-path-verbs-get-homes.md`.

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
/// @Total         Partial
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
