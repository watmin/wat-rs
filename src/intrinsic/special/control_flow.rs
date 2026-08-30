//! Special-form doc entry for `:wat::core::if` — arc 255.SF.

use wat_macros::wat_special_form;

/// Evaluate `cond`; when `:true`, evaluate and return `then`, else evaluate
/// and return `else`. The untaken branch is never evaluated — that is why `if`
/// is a special form, not an ordinary function. Purity and determinism are
/// preserved: `if` itself adds no effects; the branches carry the decision.
/// Each branch is the conditional control flow for the expression.
/// @added 1.0.0
/// @Category ControlFlow
/// @Purity Preserving
/// @Determinism Preserving
/// @Total       Unreviewed
/// @arg cond :wat::core::Bool the condition to branch on
/// @arg then :T returned when cond is :true (the taken branch)
/// @arg else :T returned when cond is :false (the taken branch)
/// @ret :T the taken branch value; both branches unify to T
/// @example (:wat::core::if true 1 2) #=> 1
/// @example (:wat::core::if false 1 2) #=> 2
#[wat_special_form(":wat::core::if")]
pub(crate) struct If;
