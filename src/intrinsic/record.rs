//! `:wat::core::Record/field-at` — arc 255 Stone A-2-ii-b-0, the first `:wat::core::Record::*`
//! accessor verb to get a registry home.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-2-ii-b-0-the-accessor-path-verbs-get-homes.md`.
//!
//! Thin `#[wat_intrinsic]` delegate over the pre-existing `eval_record_field_at`
//! (`src/runtime.rs`) — the body does not move, per the brief's two-layer architecture
//! (`src/intrinsic/<ns>.rs` registers and delegates, the implementation stays where it lived).
//! Homed here so a generated record accessor's `(Record/field-at ... 0)` tail stops
//! classifying `impure`/`Unreviewed` when reached through an environment binding — see
//! `DESIGN-STONE-A-2-ii-b-0-the-accessor-path-verbs-get-homes.md`.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::{Environment, EvalBreak, SymbolTable, Value};
use crate::span::Span;

/// `(:wat::core::Record/field-at record index) -> :T` — arc 234 Stone 234.2a.
///
/// Positional accessor for a Record/HolonRecord Aggregate: returns `fields[index]`. Consumed by
/// the Stone 234.2b `defrecord` macro's per-field accessor codegen. Homed here arc 255 Stone
/// A-2-ii-b-0 with its real (2) arity declared; the hand-rolled `args.len() != 2` guard in
/// `eval_record_field_at` retires. The body is unchanged, still in `src/runtime.rs`.
///
/// **Purity ground:** both args are evaluated by ordinary call-by-value (not itself an effect).
/// Past that, the body only reads the already-evaluated receiver's `fields` vec (rejecting
/// anything that is not a non-`Struct` `Aggregate`) and indexes it — no
/// `eval_inner`/`apply_function` on caller-supplied code beyond the two argument evaluations.
/// Pure ∧ Deterministic.
///
/// **Totality ground — pinned in the DESIGN, measured at the site:**
/// `eval_record_field_at`'s bounds check, `if index < 0 || (index as usize) >= fields.len()`,
/// returns `Err(RuntimeErrorKind::TypeMismatch)` on an out-of-range index — an
/// `EvalBreak::Diagnostic`, which "surfaces to user code as an error"
/// (`src/value/signal.rs`'s own doc on the variant), i.e. a raise, not a wat-level
/// `Option`/`Result` the caller can `match`. Per
/// `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`, a raise is not a
/// matchable outcome regardless of how deterministic or well-located it is. `Partial`.
///
/// **Expand-time ground —** Pure ∧ Deterministic and safe to evaluate during expansion; a
/// `Partial` verb can still be expand-time-legal, exactly as `macros/eval.rs` says for
/// `:wat::i64::/`'s division-by-zero. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Partial
/// @ExpandTime    Legal
/// @Category      Projection
/// @arg     record :wat::core::Record the receiver — a Record/HolonRecord Aggregate (not Struct)
/// @arg     index :wat::core::i64 the zero-based positional field index; raises a TypeMismatch if negative or out of bounds
/// @ret     :T the field value at `fields[index]`
/// @example (:wat::core::do (:wat::core::defrecord :probe::FieldAtExample [sk <- :wat::core::i64]) (:wat::core::Record/field-at (:probe::FieldAtExample :sk 7) 0)) #=> 7
/// @see     :wat::core::Option/expect
#[wat_intrinsic(":wat::core::Record/field-at")]
pub(crate) fn eval_record_field_at(
    record: &WatAST,
    index: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_record_field_at(&[record.clone(), index.clone()], list_span, env, sym)
}
