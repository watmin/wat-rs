use crate::ast::WatAST;
use crate::span::Span;

use super::error::{MacroError, MacroErrorKind};
use super::registry::{MacroDef, MacroRegistry};

pub const EXPANSION_DEPTH_LIMIT: usize = 512;

/// Walk `forms`, register every `(:wat::core::defmacro ...)` into
/// `registry`, and return the remaining forms in order.
pub fn register_defmacros(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
) -> Result<Vec<WatAST>, MacroError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_defmacro_form(&form) {
            let def = parse_defmacro_form(form)?;
            registry.register(def)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

/// Stdlib-registration variant of [`register_defmacros`] that
/// bypasses the `:wat::std::*` reserved-prefix gate. Called by the
/// startup pipeline on the baked stdlib sources; user source still
/// goes through [`register_defmacros`] so mis-namespaced user
/// defmacros halt at startup.
pub fn register_stdlib_defmacros(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
) -> Result<Vec<WatAST>, MacroError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_defmacro_form(&form) {
            let def = parse_defmacro_form(form)?;
            registry.register_stdlib(def)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

pub(super) fn is_defmacro_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items, _)
            if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::defmacro")
    )
}

/// Parse `(:wat::core::defmacro :name::path [p <- :T ...] -> :Ret body)`.
///
/// Stone 241.17 — canonical Vector-of-triples shape mirroring defn (arc 166).
///
/// New shape (6 items):
///   items[0] = `:wat::core::defmacro` keyword (head)
///   items[1] = macro name keyword
///   items[2] = argspec Vector (`[name <- :T ...]`)
///   items[3] = `->` symbol
///   items[4] = return-type keyword
///   items[5] = body
///
/// Optional metadata-map shape (7 items):
///   items[0] = `:wat::core::defmacro` keyword (head)
///   items[1] = macro name keyword
///   items[2] = metadata map (`{...}`) — stored per Stone 241.6 binding_metadata discipline
///   items[3] = argspec Vector
///   items[4] = `->` symbol
///   items[5] = return-type keyword
///   items[6] = body
///
/// HARD-CUT rejection (Stone 241.17): old 3-item paren-pair-with-type form emits
/// `MalformedDefmacro` with structured reason pointing at the canonical shape.
/// Per `feedback_hard_cut_admits_no_bypasses` — no compatibility shim.
///
/// `parse_defmacro_signature` DELETED (Stone 241.17). The canonical argspec parser
/// (`parse_argspec_triples`) is the sole argspec parser across fn/defn/defclause/defmacro.
pub(super) fn parse_defmacro_form(form: WatAST) -> Result<MacroDef, MacroError> {
    let (items, list_span) = match form {
        WatAST::List(items, span) => (items, span),
        _ => {
            return Err(MacroError {
                // arc 138: no span — form was not a List, no span to extract.
                span: Span::unknown(),
                kind: MacroErrorKind::MalformedDefmacro {
                    reason: "expected list form".into(),
                },
            })
        }
    };

    // HARD-CUT: 3-item old paren-pair form is REJECTED (Stone 241.17).
    // Old form: (:wat::core::defmacro (:name (param :T) ... -> :Ret) body)
    // Per `feedback_hard_cut_admits_no_bypasses` — no shim; no backward compat path.
    if items.len() == 3 && matches!(items.get(1), Some(WatAST::List(_, _))) {
        return Err(MacroError {
            span: list_span,
            kind: MacroErrorKind::MalformedDefmacro {
                reason: "old defmacro signature shape (paren-pair-with-type) is retired (Stone 241.17); use canonical Vector-of-triples form: (:wat::core::defmacro :name [param <- :Type ...] -> :Ret body)".into(),
            },
        });
    }

    // Determine if metadata-map is present: 7 items vs 6 items.
    // 6-item canonical: head name argvec -> rettype body
    // 7-item with-metadata: head name meta argvec -> rettype body
    let (name_item, argvec_item, arrow_item, rettype_item, body_item) =
        match items.as_slice() {
            [_, name, argvec, arrow, rettype, body] => {
                // 6-item canonical shape: arity enforced by the pattern.
                (name.clone(), argvec.clone(), arrow.clone(), rettype.clone(), body.clone())
            }
            [_, name, _meta, argvec, arrow, rettype, body] => {
                // 7-item with-metadata: metadata-map stored by binding_metadata discipline; ignored in macro parse.
                (name.clone(), argvec.clone(), arrow.clone(), rettype.clone(), body.clone())
            }
            _ => {
                return Err(MacroError {
                    span: list_span,
                    kind: MacroErrorKind::MalformedDefmacro { reason: format!(
                        "expected (:wat::core::defmacro :name [arg <- :T ...] -> :Ret body) — 6 items (or 7 with metadata-map); got {} elements",
                        items.len()
                    ) },
                });
            }
        };

    // items[1] must be the macro name keyword.
    let name = match name_item {
        WatAST::Keyword(k, _) => k,
        other => {
            return Err(MacroError { span: other.span().clone(), kind: MacroErrorKind::MalformedDefmacro { reason: "macro name (item 1) must be a keyword-path (e.g. `:my::macro`)".into() } });
        }
    };

    // items[2] (or items[3] with metadata) must be the argspec Vector.
    let (argvec_items, argvec_span) = match argvec_item {
        WatAST::Vector(items, span) => (items, span),
        other => {
            return Err(MacroError { span: other.span().clone(), kind: MacroErrorKind::MalformedDefmacro { reason: "argspec must be a Vector `[name <- :T ...]`".into() } });
        }
    };

    // Arrow symbol `->` must follow argspec.
    if !arrow_item.is_bare_symbol("->") {
        return Err(MacroError { span: arrow_item.span().clone(), kind: MacroErrorKind::MalformedDefmacro { reason: "expected `->` symbol after argspec Vector".into() } });
    }

    // Return-type keyword.
    match &rettype_item {
        WatAST::Keyword(_, _) => {}
        other => {
            return Err(MacroError { span: other.span().clone(), kind: MacroErrorKind::MalformedDefmacro { reason: "expected return-type keyword after `->`".into() } });
        }
    }

    // Route argspec through canonical parser — third major consumer after fn + defclause.
    // `allow_rest_binder: true` mirrors defclause (arc 174 / Stone 241.3/241.4).
    let spec = crate::argspec::parse_argspec_triples(
        &argvec_items,
        ":wat::core::defmacro",
        &argvec_span,
        crate::argspec::ParseOptions { allow_rest_binder: true },
    ).map_err(MacroError::from)?;

    // Extract param names only — MacroDef carries names, not types.
    let params: Vec<String> = spec.fixed_params.into_iter().map(|(name, _ty)| name).collect();
    let rest_param: Option<String> = spec.rest_param.map(|(name, _ty)| name);

    Ok(MacroDef {
        name,
        params,
        rest_param,
        body: body_item,
        span: list_span,
    })
}

// Stone 241.17 — parse_defmacro_signature DELETED (~80 lines of arc 010/150 paren-pair parser).
// `:wat::core::defmacro` signature shape migrated from paren-pair-with-type form to canonical
// Vector-of-triples form mirroring arc 166 defn shape.
// The HARD-CUT-rejection arm in parse_defmacro_form fires for any old 3-item paren-pair form.
// `parse_argspec_triples` (Stone 241.1's canonical parser) is now the third major consumer
// after fn (Stones 241.2) and defclause (Stone 241.3/241.4).
// Per `feedback_hard_cut_admits_no_bypasses` — no compatibility shim.
