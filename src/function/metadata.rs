//! # fn-form binding-metadata peel
//!
//! Domain-scoped utility used by sibling sub-modules within `src/function/`.
//! Kept separate from parse.rs (parser logic) and eval.rs/infer.rs (tier logic)
//! so each sub-file stays focused on its single concern.

use crate::ast::WatAST;

/// Peels the binding-level metadata preamble from fn-form args if present.
///
/// Stone 241.6 — the `defn` macro expands
/// `(defn :name {meta} [args] -> :ret body)` to
/// `(def :name (fn {meta} [args] -> :ret body))`. The `{meta}` at args[0]
/// is binding-level metadata (a `(:wat::core::HashMap ...)` list); it has
/// already been stored in `binding_metadata` by `try_parse_fn_shape_def` at
/// register-defines time. Strip it off so eval/infer see the real signature
/// at `args[0]`.
///
/// Returns the original slice unchanged if no metadata preamble is detected.
pub(super) fn peel_metadata_preamble(args: &[WatAST]) -> &[WatAST] {
    if args.is_empty() {
        return args;
    }
    match &args[0] {
        WatAST::List(meta_items, _) => {
            let is_metadata_map = meta_items
                .first()
                .map(|h| matches!(h, WatAST::Keyword(k, _) if k == ":wat::core::HashMap"))
                .unwrap_or(false);
            if is_metadata_map { &args[1..] } else { args }
        }
        _ => args,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    /// Line 22: `peel_metadata_preamble` returns an empty slice unchanged when
    /// called with an empty slice. The early-return guard is the production safety
    /// net for zero-arg fn-form evaluation.
    #[test]
    fn peel_empty_slice_returns_empty() {
        let args: &[WatAST] = &[];
        let result = peel_metadata_preamble(args);
        assert_eq!(result.len(), 0, "empty input must return empty slice; got len {}", result.len());
    }
}
