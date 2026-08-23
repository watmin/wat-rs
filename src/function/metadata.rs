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
    // Arc 257 slice 1 — use the authoritative is_metadata_map() predicate
    // (accepts both WatAST::Map and the legacy List-with-HashMap-head form).
    if args[0].is_metadata_map() { &args[1..] } else { args }
}

/// Arc 109 gamma-i — peels an optional `:- [T U ...]` type-param binder from
/// fn-form args, immediately after the (already-peeled) metadata preamble
/// and immediately before the args-vector. Mirrors `types.rs`'s
/// `is_binder_marker` + `take_declared_binder` pairing: `:-` lexes as a
/// KEYWORD (not a Symbol) — measured this session; matching the wrong node
/// kind here means the peel silently never fires and the caller sees the
/// ORIGINAL "expected a vector ... got keyword" error, which reads like
/// "not implemented" rather than "matched the wrong node kind."
///
/// Returns `(None, args)` unchanged when no binder is present — every
/// existing `fn`/`defn` form (bare or `<T,U>`-spelled) is untouched. When
/// present, consumes the `:-` keyword and the `Vector` that must follow it;
/// returns the binder's bare names (in source order) and the remaining args
/// slice (the args-vector / `->` / :ret-type / body...).
///
/// A malformed binder shape (`:-` not followed by a `[...]` Vector) is left
/// UNPEELED — `(None, args)` — so the existing `parse_fn_signature_prefix`
/// diagnostic ("expected a vector ... got keyword") fires naturally on the
/// stray `:-` keyword, rather than this peel inventing a second error path.
pub(crate) fn peel_type_binder(args: &[WatAST]) -> (Option<Vec<String>>, &[WatAST]) {
    let (peeled, rest) = crate::types::peel_param_spec(args);
    match peeled {
        Some(items) => {
            let names: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    WatAST::Symbol(id, _) if !id.is_reference() => Some(id.as_str().to_string()),
                    _ => None,
                })
                .collect();
            (Some(names), rest)
        }
        None => (None, args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
