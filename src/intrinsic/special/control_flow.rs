//! Special-form doc entry for `:wat::core::if` — arc 255.SF.

use wat_macros::wat_special_form;

/// Evaluate `cond`; when `:true`, evaluate and return `then`, else evaluate
/// and return `else`. The untaken branch is never evaluated — that is why `if`
/// is a special form, not an ordinary function. Purity and determinism are
/// preserved: `if` itself adds no effects; the branches carry the decision.
/// Each branch is the conditional control flow for the expression.
///
/// **Totality ground —** `if` has no totality of its own. Like `Purity`/`Determinism`
/// immediately above, it PRESERVES: it is total exactly when its taken branch is, the same
/// sentence `Totality::Preserving` was minted with (arc 255 total-T1). T4a transcribed this
/// as `@Total Total`, inconsistent with its own two sibling axes; total-T4b corrects it to
/// `Preserving` here. The fence's derived verdict is unchanged either way
/// (`Preserving` satisfies the axis exactly as `Total` does — see `intrinsic/mod.rs:1038`'s
/// `matches!(purity, Pure | Preserving)` convention), which is what makes the correction
/// safe to land alongside the derivation rather than needing its own stone.
/// @added 1.0.0
/// @Category ControlFlow
/// @Purity Preserving
/// @Determinism Preserving
/// @Total       Preserving
/// @arg cond :wat::core::Bool the condition to branch on
/// @arg then :T returned when cond is :true (the taken branch)
/// @arg else :T returned when cond is :false (the taken branch)
/// @ret :T the taken branch value; both branches unify to T
/// @example (:wat::core::if true 1 2) #=> 1
/// @example (:wat::core::if false 1 2) #=> 2
#[wat_special_form(":wat::core::if")]
pub(crate) struct If;
