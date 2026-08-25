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
//! ## Special-form boundary discipline
//!
//! A namespaced symbol sitting in a **data** position (a quoted form, a `match`
//! arm's pattern) must NOT be rewritten. The boundary-head classification lives
//! once in [`super::boundary::quote_boundary`] and is shared with
//! [`super::walk::check_form`] — both passes match it exhaustively, so they
//! cannot drift on which heads capture arguments as data. This pass applies the
//! one invariant — *never rewrite a symbol in a data position* — to every such
//! position (`quote`/`forms`/`define`, `quasiquote` templates, and the patterns
//! of `matches?`/`cond`/`match`).
//!
//! ## Dual-read (arc 251.1b)
//!
//! Keyword-FQDN heads (`:wat::core::+`) pass through untouched — the normalize
//! pass only rewrites `WatAST::Symbol` nodes. Dual-read holds until the hard-cut
//! at arc 251.5.

use crate::ast::WatAST;
use crate::edn::render::ns_to_wat_path;
use crate::macros::MacroRegistry;
use crate::runtime::SymbolTable;
use super::boundary::{is_unquote_escape, is_where_form, quote_boundary, Boundary};
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
        WatAST::Symbol(ref ident, ref span) if ident.is_reference() => {
            match resolve_namespaced_symbol(ident.as_str(), span, sym, macros) {
                Ok(kw) => kw,
                Err(e) => {
                    errors.push(e);
                    form // leave the symbol in place so the walk continues
                }
            }
        }

        // List: a special-form boundary may capture some arguments as data.
        // The head's `Boundary` (classified once in `super::boundary`, shared
        // with `walk`) decides which child regions are live code to rewrite and
        // which are data to leave untouched — the SAME invariant normalize
        // already applies to quoted forms, extended to every data position.
        // Exhaustive match: a new boundary variant is a compile error here until
        // handled, so walk and normalize cannot drift on the boundary-head set.
        WatAST::List(items, span) => {
            let boundary = match items.first() {
                Some(WatAST::Keyword(k, _)) => quote_boundary(k),
                _ => Boundary::Ordinary,
            };
            let new_items = match boundary {
                // Ordinary call: every child is live code — rewrite throughout.
                Boundary::Ordinary => items
                    .into_iter()
                    .map(|c| normalize_form(c, sym, macros, errors))
                    .collect(),
                // quote / forms / define: every argument is data. A quoted
                // `(wat.core/+ ...)` must keep its symbol, not be rewritten.
                Boundary::AllData => items,
                // quasiquote: template data except unquote/unquote-splicing escapes.
                Boundary::Quasiquote => normalize_quasiquote_form(items, sym, macros, errors),
                // matches?: only the subject (items[1]) is code; pattern is data.
                Boundary::MatchesSubject => normalize_matches(items, sym, macros, errors),
                // match: scrutinee + arm bodies are code; arm patterns are data (arc 258.5, no `-> :T`).
                Boundary::Match => normalize_match(items, sym, macros, errors),
                // make-rule (arc 278 task #78): rule name is code; the quoted
                // :when vector is data except each where-form's body (code);
                // the quoted :then vector is untouched data. Mirrors `walk`'s
                // `check_make_rule_when` exactly — see its doc for why the
                // already-expanded where-body call heads still need this pass.
                Boundary::MakeRule => normalize_make_rule(items, sym, macros, errors),
            };
            WatAST::List(new_items, span)
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

        // Leaf nodes (and a bare Symbol without `/`) carry no namespaced symbol
        // ref to rewrite — pass through unchanged.
        other => other,
    }
}

/// Normalize a `:wat::core::quasiquote` list. The template (items[1]) is data
/// EXCEPT inside `unquote` / `unquote-splicing` escapes (live code).
fn normalize_quasiquote_form(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut normalized_items = Vec::with_capacity(items.len());
    let mut iter = items.into_iter();
    // Keep the head keyword as-is.
    normalized_items.extend(iter.next());
    // items[1] = template (if present) — descend quasiquote-aware.
    if let Some(template) = iter.next() {
        normalized_items.push(normalize_quasiquote_template(template, sym, macros, errors));
    }
    // Any remaining items pass through unchanged (shouldn't appear in a
    // well-formed quasiquote, but be conservative).
    normalized_items.extend(iter);
    normalized_items
}

/// Normalize a `:wat::form::matches?` list. Only the subject (items[1]) is code;
/// the pattern (items[2..]) is DSL data — left untouched, mirroring `walk`.
fn normalize_matches(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut out = Vec::with_capacity(items.len());
    let mut iter = items.into_iter();
    out.extend(iter.next()); // matches? head, as-is
    if let Some(subject) = iter.next() {
        out.push(normalize_form(subject, sym, macros, errors)); // subject: code
    }
    out.extend(iter); // pattern + any extra args: data, as-is
    out
}

/// Normalize a `:wat::core::match` list. Arc 258.5 — bare match: the scrutinee
/// (items[1]) and each arm body are code; the arms (items[2..]) each pattern is
/// data — left untouched, mirroring `walk`. The `-> :T` ascription is retired.
fn normalize_match(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut out = Vec::with_capacity(items.len());
    let mut iter = items.into_iter();
    out.extend(iter.next()); // match head, as-is
    if let Some(scrutinee) = iter.next() {
        out.push(normalize_form(scrutinee, sym, macros, errors)); // scrutinee: code
    }
    for arm in iter {
        match arm {
            WatAST::List(arm_items, arm_span) => {
                let mut new_arm = Vec::with_capacity(arm_items.len());
                let mut ai = arm_items.into_iter();
                new_arm.extend(ai.next()); // pattern (arm_items[0]): data, as-is
                if let Some(body) = ai.next() {
                    new_arm.push(normalize_form(body, sym, macros, errors)); // body: code
                }
                new_arm.extend(ai); // any trailing arm items: data, as-is
                out.push(WatAST::List(new_arm, arm_span));
            }
            // Non-list arm (e.g. a bare-symbol wildcard): live code.
            other => out.push(normalize_form(other, sym, macros, errors)),
        }
    }
    out
}

/// Normalize a `:wat::rete::make-rule` call. items[0]=head (as-is),
/// items[1]=rule name (code), items[2]=quoted `:when` vector (data except
/// each where-form's body — see [`normalize_make_rule_when`]), items[3..]=
/// quoted `:then` vector and any trailing args (untouched data — task #61
/// already ruled derived fact fields are copies only; STOP — do not touch).
fn normalize_make_rule(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut out = Vec::with_capacity(items.len());
    let mut iter = items.into_iter();
    out.extend(iter.next()); // make-rule head, as-is
    if let Some(name) = iter.next() {
        out.push(normalize_form(name, sym, macros, errors)); // rule name: code
    }
    if let Some(when_arg) = iter.next() {
        out.push(normalize_make_rule_when(when_arg, sym, macros, errors));
    }
    out.extend(iter); // :then vector + any trailing args: data, as-is
    out
}

/// Normalize a `make-rule` call's `:when` argument. Expected shape
/// `(:wat::core::quote [<condition>...])` — mirrors `walk`'s
/// `check_make_rule_when` (see its doc for the shape assumption and the
/// STOP-2 hazard this avoids: a condition pattern's aggregate-shaped head
/// must never be rewritten as if it were a call). Anything not shaped like a
/// literal quoted vector is left untouched — conservative by construction.
fn normalize_make_rule_when(
    when_arg: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    let WatAST::List(qitems, qspan) = when_arg else { return when_arg };
    let is_quote = matches!(qitems.first(), Some(WatAST::Keyword(h, _)) if h == ":wat::core::quote");
    if !is_quote {
        return WatAST::List(qitems, qspan);
    }
    let mut qiter = qitems.into_iter();
    let mut new_q = Vec::with_capacity(2);
    new_q.extend(qiter.next()); // quote head, as-is
    if let Some(vec_node) = qiter.next() {
        new_q.push(normalize_make_rule_conditions(vec_node, sym, macros, errors));
    }
    new_q.extend(qiter); // shouldn't appear in a well-formed quote; conservative
    WatAST::List(new_q, qspan)
}

/// Normalize the condition vector inside a `make-rule`'s quoted `:when` arg —
/// per-element dispatch to [`normalize_make_rule_condition`].
fn normalize_make_rule_conditions(
    vec_node: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    let WatAST::Vector(conds, vspan) = vec_node else { return vec_node };
    let new_conds = conds
        .into_iter()
        .map(|cond| normalize_make_rule_condition(cond, sym, macros, errors))
        .collect();
    WatAST::Vector(new_conds, vspan)
}

/// Normalize one `:when` condition. A `(:wat::rete::where <body>...)` form's
/// body is code — normalized like any other. Every other condition (a fact
/// pattern) is byte-identical, untouched.
fn normalize_make_rule_condition(
    cond: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    let WatAST::List(citems, cspan) = cond else { return cond };
    let is_where = matches!(citems.first(), Some(WatAST::Keyword(h, _)) if is_where_form(h));
    if !is_where {
        return WatAST::List(citems, cspan);
    }
    let mut citer = citems.into_iter();
    let mut new_c = Vec::with_capacity(citer.len().max(1));
    new_c.extend(citer.next()); // where head, as-is
    for body in citer {
        new_c.push(normalize_form(body, sym, macros, errors)); // body: code
    }
    WatAST::List(new_c, cspan)
}

/// Walk a quasiquote template, normalizing only inside unquote/unquote-splicing
/// escapes (live code). The rest of the template is data — recurse structurally
/// only to find nested escape forms, but do NOT rewrite symbols in data positions.
///
/// Parallel to [`super::quote::check_quasiquote_template`]: same template walk,
/// opposite ownership — this consumes the node and rebuilds it (rewriting escape
/// symbols); that borrows the node and pushes errors (resolving escape heads).
/// Both gate the escape boundary on [`is_unquote_escape`], so they cannot drift.
fn normalize_quasiquote_template(
    node: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    if let WatAST::List(items, span) = node {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if is_unquote_escape(head) {
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
    assert!(symbol_text.contains('/'), "caller guarantees '/' present");
    let namespace = wat_reader::identifier::receiver(symbol_text);
    let local_name = wat_reader::identifier::method(symbol_text);

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
    // program uses symbol-head type-members — the corpus is keyword-spelled.
    // rune:exigere(attested-arc) — correct `Type/member` symbol normalization lands when
    // symbol-head type-members first appear, at arc 251 stone 251.5 (HARD-CUT the
    // keyword-as-type/operator surface); DESIGN at
    // docs/arc/2026/06/251-types-as-forms/DESIGN.md.

    // Primary did not resolve → located error naming the unknown entity.
    Err(UnresolvedReference {
        path: primary.clone(),
        context: "namespaced symbol ref — not a builtin, not a registered function (arc 251)",
        span: span.clone(),
    })
}
