//! Special-form doc entry for `:wat::core::let` — arc 255.SF.

use wat_macros::wat_special_form;

/// Bind each `<expr>` to its `<binder>` in order (sequential; later binders
/// see earlier ones), then evaluate the body forms in the enriched scope,
/// returning the value of the last form. The scope is closed after `let` returns.
/// @added 1.0.0
/// @Category Binding
/// @Purity Preserving
/// @Determinism Preserving
/// @Total       Unreviewed
/// @syntax (let [<binder> <expr> ...] <body>+)
/// @ret :T the value of the final body form
/// @example (:wat::core::let [x 1 y 2] (:wat::i64::+ x y)) #=> 3
#[wat_special_form(":wat::core::let")]
pub(crate) struct Let;
