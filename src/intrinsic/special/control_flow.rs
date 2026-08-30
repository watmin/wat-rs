//! Special-form doc entry for `:wat::core::if` — arc 255.SF.

use wat_macros::wat_special_form;

/// Evaluate `cond`; when `:true`, evaluate and return `then`, else evaluate
/// and return `else`. The untaken branch is never evaluated — that is why `if`
/// is a special form, not an ordinary function. Purity and determinism are
/// preserved: `if` itself adds no effects; the branches carry the decision.
/// Each branch is the conditional control flow for the expression.
///
/// **Totality ground —** a value/control-flow op with no domain restriction: a well-typed
/// call always returns; type mismatches are the type checker's concern, not this axis's,
/// the same convention `pure`/`deterministic` already use. Grouped with `let` (and the
/// generic `=`/`not=`/`<`/`and`/`or`/`not` outside this stone's 27) in `rete/purity.rs`'s
/// `total` sub-list (arc 255 total-T4a); the verdict is that list's, made by reading the
/// implementation.
/// @added 1.0.0
/// @Category ControlFlow
/// @Purity Preserving
/// @Determinism Preserving
/// @Total       Total
/// @arg cond :wat::core::Bool the condition to branch on
/// @arg then :T returned when cond is :true (the taken branch)
/// @arg else :T returned when cond is :false (the taken branch)
/// @ret :T the taken branch value; both branches unify to T
/// @example (:wat::core::if true 1 2) #=> 1
/// @example (:wat::core::if false 1 2) #=> 2
#[wat_special_form(":wat::core::if")]
pub(crate) struct If;
