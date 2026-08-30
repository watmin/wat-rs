//! Special-form doc entry for `:wat::core::let` — arc 255.SF.

use wat_macros::wat_special_form;

/// Bind each `<expr>` to its `<binder>` in order (sequential; later binders
/// see earlier ones), then evaluate the body forms in the enriched scope,
/// returning the value of the last form. The scope is closed after `let` returns.
///
/// **Totality ground —** a value/control-flow op with no domain restriction: a well-typed
/// call always returns; type mismatches are the type checker's concern, not this axis's,
/// the same convention `pure`/`deterministic` already use. Grouped with `if` (and the
/// generic `=`/`not=`/`<`/`and`/`or`/`not` outside this stone's 27) in `rete/purity.rs`'s
/// `total` sub-list (arc 255 total-T4a); the verdict is that list's, made by reading the
/// implementation.
/// @added 1.0.0
/// @Category Binding
/// @Purity Preserving
/// @Determinism Preserving
/// @Total       Total
/// @syntax (let [<binder> <expr> ...] <body>+)
/// @ret :T the value of the final body form
/// @example (:wat::core::let [x 1 y 2] (:wat::i64::+ x y)) #=> 3
#[wat_special_form(":wat::core::let")]
pub(crate) struct Let;
