//! Arc 251 Stone 251.1b — namespaced symbol-ref normalization.
//!
//! A `WatAST::Symbol` whose name contains `/` is a **namespaced ref**
//! (`wat.core/+`, `wat.type/i64`, `wat.core/foldl`) — distinguished from a
//! bare local binder (`x`, `acc`) by the presence of `/`. This pass rewrites
//! every such symbol to the `WatAST::Keyword(fqdn, span)` it names, so the
//! UNTOUCHED downstream dispatch (`eval_list` / `dispatch_keyword_head`) resolves
//! it. Bare symbols (no `/`) are left untouched — they are local binders.
//!
//! ## Mapping
//!
//! Given `a.b/c` — split on the LAST `/` → ns=`a.b`, name=`c` — the keyword FQDN is
//! `ns_to_wat_path(ns, name)` = `:` + ns(`.`→`::`) + `::` + name
//! (`wat.core/+` → `:wat::core::+`). If it passes the resolution predicate the symbol
//! rewrites to that keyword; otherwise a located error names the unknown entity. There
//! is NO `Type/member` fallback — see the NOTE in `resolve_namespaced_symbol` for why a
//! `/`-preserving candidate is structurally unreachable, and the named latent gap for
//! type-member symbol heads.
//!
//! ## Quote-family boundary discipline
//!
//! Mirrors the boundary discipline in [`super::walk::check_form`]:
//! - `:wat::core::forms` / `:wat::core::quote` / `:wat::core::define` — all
//!   arguments are data; return the form unchanged.
//! - `:wat::core::quasiquote` — the template argument is data EXCEPT inside
//!   `:wat::core::unquote` / `:wat::core::unquote-splicing` escapes (live code).
//!
//! ## Dual-read (arc 251.1b)
//!
//! Keyword-FQDN heads (`:wat::core::+`) pass through untouched — the normalize
//! pass only rewrites `WatAST::Symbol` nodes. Dual-read holds until the hard-cut
//! at arc 251.5.

use crate::ast::WatAST;
use crate::edn_shim::ns_to_wat_path;
use crate::macros::MacroRegistry;
use crate::runtime::SymbolTable;
use super::error::{ResolveError, UnresolvedReference};
use super::walk::is_resolvable_call_head;

/// Normalize all namespaced symbol refs in `forms`.
///
/// Returns the rewritten AST. Collects ALL located errors before returning so
/// the user can fix them in a single pass (matches `resolve_references`
/// semantics). A namespaced symbol that resolves to NEITHER primary nor fallback
/// candidate emits an [`UnresolvedReference`] with the original span — never a
/// bare `UnboundSymbol`.
///
/// Called from `freeze.rs` BEFORE [`super::walk::resolve_references`] so the
/// rewritten AST flows through the rest of the pipeline.
pub fn normalize_symbol_refs(
    forms: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
) -> Result<Vec<WatAST>, ResolveError> {
    let mut errors: Vec<UnresolvedReference> = Vec::new();
    let out = forms
        .into_iter()
        .map(|form| normalize_form(form, sym, macros, &mut errors))
        .collect();
    if errors.is_empty() {
        Ok(out)
    } else {
        Err(ResolveError::UnresolvedReferences(errors))
    }
}

/// Recursively normalize one form. Quote-family boundaries halt descent.
fn normalize_form(
    form: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    match form {
        // Namespaced symbol: the only node type this pass rewrites.
        WatAST::Symbol(ref ident, ref span) if ident.as_str().contains('/') => {
            match resolve_namespaced_symbol(ident.as_str(), span, sym, macros) {
                Ok(kw) => kw,
                Err(e) => {
                    errors.push(e);
                    form // leave the symbol in place so the walk continues
                }
            }
        }

        // List: check for quote-family boundary before recursing.
        WatAST::List(items, span) => {
            WatAST::List(normalize_list(items, sym, macros, errors), span)
        }

        // Vector: recurse uniformly (no boundary guards needed).
        WatAST::Vector(items, span) => {
            let new_items = items
                .into_iter()
                .map(|c| normalize_form(c, sym, macros, errors))
                .collect();
            WatAST::Vector(new_items, span)
        }

        // Map: recurse over keys and values.
        WatAST::Map(pairs, span) => {
            let new_pairs = pairs
                .into_iter()
                .map(|(k, v)| {
                    (
                        normalize_form(k, sym, macros, errors),
                        normalize_form(v, sym, macros, errors),
                    )
                })
                .collect();
            WatAST::Map(new_pairs, span)
        }

        // Set: recurse uniformly.
        WatAST::Set(items, span) => {
            let new_items = items
                .into_iter()
                .map(|c| normalize_form(c, sym, macros, errors))
                .collect();
            WatAST::Set(new_items, span)
        }

        // All other leaf nodes (IntLit, FloatLit, BoolLit, StringLit, NilLit,
        // Keyword, bare Symbol without `/`) — pass through untouched.
        other => other,
    }
}

/// Handle list normalization with quote-family boundary discipline.
///
/// Mirrors `check_form`'s boundary logic exactly:
/// - `quote` / `forms` / `define` → return all items unchanged (data).
/// - `quasiquote` → recurse only through `unquote`/`unquote-splicing` escapes.
/// - Everything else → normalize all children.
fn normalize_list(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    // Peek at the head to detect quote-family boundaries.
    let head_kw: Option<String> = match items.first() {
        Some(WatAST::Keyword(k, _)) => Some(k.clone()),
        _ => None,
    };

    if let Some(ref head) = head_kw {
        // Quote / forms / define: entire argument list is data. Return as-is.
        if head == ":wat::core::quote"
            || head == ":wat::core::forms"
            || head == ":wat::core::define"
        {
            return items;
        }

        // Quasiquote: template is data except inside unquote/unquote-splicing.
        if head == ":wat::core::quasiquote" {
            let mut out = Vec::with_capacity(items.len());
            let mut iter = items.into_iter();
            // Keep the head keyword as-is.
            out.extend(iter.next());
            // items[1] = template (if present) — descend quasiquote-aware.
            if let Some(template) = iter.next() {
                out.push(normalize_quasiquote_template(template, sym, macros, errors));
            }
            // Any remaining items passed through unchanged (shouldn't appear
            // in well-formed quasiquote, but be conservative).
            out.extend(iter);
            return out;
        }
    }

    // Default: normalize all items recursively.
    items
        .into_iter()
        .map(|item| normalize_form(item, sym, macros, errors))
        .collect()
}

/// Walk a quasiquote template, normalizing only inside unquote/unquote-splicing
/// escapes (live code). The rest of the template is data — recurse structurally
/// only to find nested escape forms, but do NOT rewrite symbols in data positions.
fn normalize_quasiquote_template(
    node: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    if let WatAST::List(items, span) = node {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if head == ":wat::core::unquote" || head == ":wat::core::unquote-splicing" {
                // Escape: argument is live code — full normalization.
                let new_items = items
                    .into_iter()
                    .map(|c| normalize_form(c, sym, macros, errors))
                    .collect();
                return WatAST::List(new_items, span);
            }
        }
        // Non-escape list inside the template: recurse structurally (to find
        // nested escapes) but do NOT rewrite the head or any data symbols.
        let new_items = items
            .into_iter()
            .map(|c| normalize_quasiquote_template(c, sym, macros, errors))
            .collect();
        WatAST::List(new_items, span)
    } else {
        // Atoms (Symbol, Keyword, literals) in template data position: pass through.
        // Structural recursion: non-list containers inside templates.
        match node {
            WatAST::Vector(items, span) => WatAST::Vector(
                items
                    .into_iter()
                    .map(|c| normalize_quasiquote_template(c, sym, macros, errors))
                    .collect(),
                span,
            ),
            WatAST::Map(pairs, span) => WatAST::Map(
                pairs
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            normalize_quasiquote_template(k, sym, macros, errors),
                            normalize_quasiquote_template(v, sym, macros, errors),
                        )
                    })
                    .collect(),
                span,
            ),
            WatAST::Set(items, span) => WatAST::Set(
                items
                    .into_iter()
                    .map(|c| normalize_quasiquote_template(c, sym, macros, errors))
                    .collect(),
                span,
            ),
            other => other,
        }
    }
}

/// Map a namespaced symbol name (`wat.core/+`) to its keyword FQDN candidate
/// (`:wat::core::+`) and validate it resolves. Returns the rewritten
/// `WatAST::Keyword` on success, or a located `UnresolvedReference` error.
fn resolve_namespaced_symbol(
    symbol_text: &str,
    span: &crate::span::Span,
    sym: &SymbolTable,
    macros: &MacroRegistry,
) -> Result<WatAST, UnresolvedReference> {
    // Split on the LAST `/` → (namespace, local_name).
    let slash_pos = symbol_text.rfind('/').expect("caller guarantees '/' present");
    let namespace = &symbol_text[..slash_pos];
    let local_name = &symbol_text[slash_pos + 1..];

    // `ns_to_wat_path` replaces `.` with `::` and joins with `::`:
    // `wat.core/+` → `:wat::core::+`.
    let primary = ns_to_wat_path(namespace, local_name);

    if is_resolvable_call_head(&primary, sym, macros) {
        return Ok(WatAST::Keyword(primary, span.clone()));
    }

    // NOTE — there is intentionally NO `Type/member` fallback (purgare, 251.1b ward).
    // A `/`-preserving candidate (`:wat::core::HashMap/length`) is structurally
    // unreachable: for any `:wat::`/`:rust::` head the PRIMARY already passes
    // `is_resolvable_call_head` via the reserved-prefix shortcut (it accepts the
    // namespace without leaf validation), so primary-fail never happens for the
    // reserved namespaces; and non-reserved entities register under `:ns::name`
    // keys (never `:ns/name`), so a `/`-shaped candidate matches nothing there either.
    // LATENT GAP, named not buried: a type-member SYMBOL head (`wat.core.HashMap/length`)
    // normalizes to `:wat::core::HashMap::length`, which passes resolve but is NOT the
    // runtime op (`:wat::core::HashMap/length`), so it would not dispatch. No current
    // program uses symbol-head type-members (the corpus is keyword-spelled); correct
    // `Type/member` symbol normalization is deferred to the 251.5 corpus cut, where
    // symbol-head type-members first appear.

    // Primary did not resolve → located error naming the unknown entity.
    Err(UnresolvedReference {
        path: primary.clone(),
        context: "namespaced symbol ref — not a builtin, not a registered function (arc 251)",
        span: span.clone(),
    })
}
