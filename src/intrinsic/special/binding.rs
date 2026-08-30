//! Special-form doc entry for `:wat::core::let` — arc 255.SF.

use wat_macros::wat_special_form;

/// Bind each `<expr>` to its `<binder>` in order (sequential; later binders
/// see earlier ones), then evaluate the body forms in the enriched scope,
/// returning the value of the last form. The scope is closed after `let` returns.
///
/// **Totality ground —** `let` has no totality of its own. Like `Purity`/`Determinism`
/// immediately above, it PRESERVES: it is total exactly when its bindings and body are, the
/// same sentence `Totality::Preserving` was minted with (arc 255 total-T1). T4a transcribed
/// this as `@Total Total`, inconsistent with its own two sibling axes; total-T4b corrects it
/// to `Preserving` here. The fence's derived verdict is unchanged either way (`Preserving`
/// satisfies the axis exactly as `Total` does — see `intrinsic/mod.rs:1038`'s
/// `matches!(purity, Pure | Preserving)` convention), which is what makes the correction
/// safe to land alongside the derivation rather than needing its own stone.
/// @added 1.0.0
/// @Category Binding
/// @Purity Preserving
/// @Determinism Preserving
/// @Total       Preserving
/// @syntax (let [<binder> <expr> ...] <body>+)
/// @ret :T the value of the final body form
/// @example (:wat::core::let [x 1 y 2] (:wat::i64::+ x y)) #=> 3
#[wat_special_form(":wat::core::let")]
pub(crate) struct Let;
