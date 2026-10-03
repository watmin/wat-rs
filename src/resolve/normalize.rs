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
//! Given `a.b/c` — split on the FIRST `/` → ns=`a.b`, name=`c` — the **call-head**
//! keyword is `reconstruct_call_path`: the namespace names a known type and the
//! local name itself contains no `/` → join with `/`
//! (`wat.core.Option/expect` → `:wat::core::Option/expect`); otherwise
//! `ns_to_wat_path` (`wat.core/+` → `:wat::core::+`, `u/a/b` → `:u::a/b`).
//! A `/` after the first is a name character. The 255.86 member join still
//! decides `::` versus `/` only for a local name that does not itself contain
//! `/`. Identity of a type *name* stays `::` always — that door cannot see
//! call vs name position. See the NOTE in `resolve_namespaced_symbol`.
//!
//! A symbol in a binder position (`let`, `fn` / `lambda` parameters, a
//! two-element `match` binder, a hash-destructure binder) is
//! `{$bound, <whole spelling>}` via `Identifier::into_bound`. A body
//! reference whose `env_key` is in that scope is bound the same way and is
//! not rewritten to a keyword. An unbound reference still is. A variant
//! head is not a binder.
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

use std::cell::RefCell;
use std::collections::HashSet;

use crate::ast::WatAST;
use crate::edn::render::ns_to_wat_path;
use crate::macros::MacroRegistry;
use crate::runtime::SymbolTable;
use crate::scope::Identifier;
use crate::value::FunctionBody;
use super::boundary::{is_unquote_escape, is_where_form, quote_boundary, Boundary};
use super::error::{ResolveError, UnresolvedReference};
use super::walk::is_resolvable_call_head;

#[derive(Clone, Copy)]
enum ScopedHead {
    Let,
    Fn,
}

thread_local! {
    /// Frames of `env_key`s. The outermost entry ([`ScopeEnter`]) starts
    /// empty; each `let` / `fn` / `match` arm pushes one frame. A contains
    /// walks every frame, so an inner binder shadows by being found first
    /// without cloning the outer set.
    static SCOPE: RefCell<Vec<HashSet<String>>> = const { RefCell::new(Vec::new()) };
}

/// Saves the scope stack and starts empty. Drop restores it, so a nested
/// normalize on the same thread cannot see the caller's binders.
struct ScopeEnter {
    prev: Vec<HashSet<String>>,
}

impl ScopeEnter {
    fn fresh() -> Self {
        let prev = SCOPE.with(|c| std::mem::take(&mut *c.borrow_mut()));
        ScopeEnter { prev }
    }
}

impl Drop for ScopeEnter {
    fn drop(&mut self) {
        let prev = std::mem::take(&mut self.prev);
        SCOPE.with(|c| *c.borrow_mut() = prev);
    }
}

/// One frame. Drop pops it. Declare this inside [`ScopeEnter`], never the
/// other way around: dropping the enter first would pop a frame off the
/// restored stack.
struct FrameGuard;

impl FrameGuard {
    fn push() -> Self {
        Self::push_with(std::iter::empty())
    }

    fn push_with(keys: impl IntoIterator<Item = String>) -> Self {
        let mut frame = HashSet::new();
        for k in keys {
            if k != "_" {
                frame.insert(k);
            }
        }
        SCOPE.with(|c| c.borrow_mut().push(frame));
        FrameGuard
    }
}

impl Drop for FrameGuard {
    fn drop(&mut self) {
        SCOPE.with(|c| {
            c.borrow_mut().pop();
        });
    }
}

fn scope_contains(key: &str) -> bool {
    SCOPE.with(|c| c.borrow().iter().rev().any(|f| f.contains(key)))
}

fn scope_insert_ident(id: &Identifier) {
    if id.as_str() == "_" {
        return;
    }
    let key = crate::scope::env_key(id).into_owned();
    SCOPE.with(|c| {
        if let Some(top) = c.borrow_mut().last_mut() {
            top.insert(key);
        }
    });
}

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
    let _enter = ScopeEnter::fresh();
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

/// Stone 251.8c — run the same Symbol→Keyword rewrite over function bodies
/// stored on the symbol table.
///
/// `normalize_symbol_refs` rewrites the program residue. `check_program` type-checks
/// `FunctionBody::Wat` snapshots taken at `register_defines` (step 6), which is
/// BEFORE that rewrite (step 7). A namespaced Symbol call head in a function body
/// therefore never reaches `infer_list`'s Keyword path — args/arity are skipped.
/// Applying the existing no-form pass to the AST check actually walks makes one
/// path; it does not teach `infer_list` to accept Symbols.
pub fn normalize_stored_function_bodies(
    symbols: &mut SymbolTable,
    macros: &MacroRegistry,
) -> Result<(), ResolveError> {
    let _enter = ScopeEnter::fresh();
    let mut errors: Vec<UnresolvedReference> = Vec::new();
    let paths: Vec<String> = symbols
        .functions_iter()
        .filter(|(_, f)| matches!(f.body, FunctionBody::Wat(_)))
        .map(|(p, _)| p.clone())
        .collect();
    for path in paths {
        let body = match symbols.get(&path) {
            Some(f) => match &f.body {
                FunctionBody::Wat(b) => (**b).clone(),
                FunctionBody::Native => continue,
            },
            None => continue,
        };
        let mut keys = Vec::new();
        if let Some(f) = symbols.get(&path) {
            for p in &f.params {
                let k = crate::scope::env_key(p);
                if k.as_ref() != "_" {
                    keys.push(k.into_owned());
                }
            }
            if let Some(r) = &f.rest_param {
                if r != "_" {
                    keys.push(r.clone());
                }
            }
        }
        let _frame = FrameGuard::push_with(keys);
        let new = normalize_form(body, symbols, macros, &mut errors);
        drop(_frame);
        symbols.replace_wat_body(&path, new);
    }
    if errors.is_empty() {
        Ok(())
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
        // A body reference that is in scope is a binder, not a keyword.
        // This runs before the reference rewrite so a slashed local used
        // as a call head stays a symbol (`is_reference` false) and eval
        // looks it up locally.
        WatAST::Symbol(ident, span) if scope_contains(crate::scope::env_key(&ident).as_ref()) => {
            WatAST::Symbol(ident.into_bound(), span)
        }

        // Namespaced symbol: the only node type this pass rewrites.
        WatAST::Symbol(ref ident, ref span) if ident.is_reference() => {
            match resolve_namespaced_symbol(ident.as_str(), span, sym, macros, false) {
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
            // Symbol `(wat.core/match …)` is the same boundary as the keyword.
            // `head_fqdn` borrows the head node. Resolve it before the arms move `items`.
            let (boundary, scoped) = {
                let head = items.first().and_then(crate::declare::parse::head_fqdn);
                let boundary = match head.as_deref() {
                    Some(h) => quote_boundary(h),
                    None => Boundary::Ordinary,
                };
                let scoped = match head.as_deref() {
                    Some(":wat::core::let") => Some(ScopedHead::Let),
                    Some(":wat::core::fn") | Some(":wat::core::lambda") => Some(ScopedHead::Fn),
                    _ => None,
                };
                (boundary, scoped)
            };
            // `head_fqdn` classifies a symbol head (`wat.core/match`) as the
            // boundary, but the checker matches the keyword node. Leaving the
            // symbol in place made every match arm infer as a value vector.
            let items = if matches!(boundary, Boundary::Ordinary) {
                items
            } else {
                rewrite_boundary_head(items, sym, macros, errors)
            };
            // Arc 255 Stone ② — `(Head :- [T …] rest…)` is a TYPE BINDER. `peel_param_spec`
            // is arc 109's one door for this triple; `items.len() >= 3` guards it against
            // the empty-list panic (`&items[1..]` on a 0-len slice — the first strike's
            // STOP-4 regression). It applies whether the head is a `Symbol` or an
            // already-normalized `Keyword`.
            //
            // ⛔ CLASSIFIED AFTER the `Boundary`, and gated on `Ordinary`. A boundary head
            // CAPTURES ITS ARGUMENTS AS DATA, and this pass's one invariant — *never
            // rewrite a symbol in a data position* — outranks the binder shape. Measured:
            // testing the binder first made `(:wat::core::quote :- [T] (my.app/undefined 1))`
            // resolve a symbol inside QUOTED data, where clean main never looks. No corpus
            // form pairs a boundary head with a binder today, so this costs nothing and
            // keeps the invariant structural rather than incidental.
            // Stone 255.88 — binder positions, before the generic rewrite
            // turns a slashed binder into a keyword. Quote and quasiquote
            // are not Ordinary, so a template `let` stays data.
            if matches!(boundary, Boundary::Ordinary) {
                match scoped {
                    Some(ScopedHead::Let) => {
                        return WatAST::List(
                            normalize_scoped_let(items, sym, macros, errors),
                            span,
                        );
                    }
                    Some(ScopedHead::Fn) => {
                        return WatAST::List(
                            normalize_scoped_fn(items, sym, macros, errors),
                            span,
                        );
                    }
                    None => {}
                }
            }
            if matches!(boundary, Boundary::Ordinary)
                && items.len() >= 3
                && crate::types::peel_param_spec(&items[1..]).0.is_some()
            {
                let new_items = normalize_type_binder_form(items, sym, macros, errors);
                return WatAST::List(new_items, span);
            }
            let new_items = match boundary {
                // Ordinary call: 255.5 — a child after a return arrow is a
                // type slot (`also_accept_type`); items[1] of a declare form
                // is a name, not a reference.
                Boundary::Ordinary => normalize_ordinary_list(items, sym, macros, errors),
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

        // Vector: a child after a param-annotation arrow (`<-` / `:-`) is a
        // type slot. Other children are ordinary (match-arm bodies, value vecs).
        WatAST::Vector(items, span) => {
            let new_items = normalize_vector_items(items, sym, macros, errors);
            WatAST::Vector(new_items, span)
        }

        // Map: recurse over keys and values.
        WatAST::Map(pairs, span) => {
            let new_pairs = pairs
                .into_iter()
                .map(|(k, v)| {
                    (
                        normalize_value_position(k, sym, macros, errors),
                        normalize_value_position(v, sym, macros, errors),
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

/// Rewrite a boundary form's symbol head to the keyword the checker matches.
/// Arguments stay on the boundary (the caller already classified them).
fn rewrite_boundary_head(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut iter = items.into_iter();
    let Some(head) = iter.next() else {
        return Vec::new();
    };
    let head = if matches!(&head, WatAST::Symbol(id, _) if id.is_reference()) {
        normalize_form(head, sym, macros, errors)
    } else {
        head
    };
    let mut out = Vec::with_capacity(1 + iter.size_hint().0);
    out.push(head);
    out.extend(iter);
    out
}

/// Ordinary list children: type slots after a return arrow; name slot at
/// items[1] of a declare-role form. Derived from `is_return_arrow` and
/// `is_declare_role_head` — not a form list.
fn normalize_ordinary_list(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut out = Vec::with_capacity(items.len());
    let mut iter = items.into_iter();
    let Some(head) = iter.next() else {
        return out;
    };
    let new_head = normalize_form(head, sym, macros, errors);
    let is_declare = match &new_head {
        WatAST::Keyword(k, _) => crate::intrinsic::is_declare_role_head(k),
        _ => false,
    };
    out.push(new_head);
    let mut prev_return_arrow = false;
    for (idx, c) in (1usize..).zip(iter) {
        let this_return_arrow = crate::types::is_return_arrow(&c);
        let new = if is_declare && idx == 1 {
            normalize_name_slot(c, sym, macros, errors)
        } else if prev_return_arrow {
            normalize_type_slot(c, sym, macros, errors)
        } else {
            normalize_value_position(c, sym, macros, errors)
        };
        out.push(new);
        prev_return_arrow = this_return_arrow;
    }
    out
}

/// Vector children: the sibling after `<-` / `:-` is a type annotation.
fn normalize_vector_items(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut out = Vec::with_capacity(items.len());
    let mut prev_ann = false;
    for c in items {
        let this_ann = crate::types::is_param_annotation_arrow(&c);
        let new = if prev_ann {
            normalize_type_slot(c, sym, macros, errors)
        } else {
            normalize_value_position(c, sym, macros, errors)
        };
        out.push(new);
        prev_ann = this_ann;
    }
    out
}

/// A name in value position, not a call head. A scalar `def` such as
/// `wat.kernel/STDIO-WRITE-CHUNK-CHARS` is a keyword literal until runtime
/// registration; requiring it to already be a function is what made the
/// converted constant unresolved while the keyword spelling loaded.
/// Nested lists are still calls — their heads stay on [`normalize_form`].
fn normalize_value_position(
    node: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    match node {
        WatAST::Symbol(ident, span) if scope_contains(crate::scope::env_key(&ident).as_ref()) => {
            WatAST::Symbol(ident.into_bound(), span)
        }
        WatAST::Symbol(ref ident, ref span) if ident.is_reference() => {
            match resolve_namespaced_symbol(ident.as_str(), span, sym, macros, false) {
                Ok(kw) => kw,
                Err(_) => {
                    let namespace = wat_reader::identifier::receiver(ident.as_str());
                    let local_name = wat_reader::identifier::method(ident.as_str());
                    WatAST::Keyword(ns_to_wat_path(namespace, local_name), span.clone())
                }
            }
        }
        other => normalize_form(other, sym, macros, errors),
    }
}

/// A type slot — annotation, return, nested parametric arg, tuple element.
///
/// Not a call head. `resolve_namespaced_symbol` runs `reconstruct_call_path`,
/// which joins `my.Journal/Req` as a member (`:my::Journal/Req`) because
/// `Journal` is a type. The declaration registered the identity
/// `:my::Journal::Req` via `ns_to_wat_path`. Same door as
/// [`normalize_name_slot`] and [`normalize_type_vector`]. The annotation
/// wall, not this pass, says whether the name is a real type.
fn normalize_type_slot(
    node: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    match node {
        WatAST::Symbol(ref ident, ref span) if ident.is_reference() => {
            let namespace = wat_reader::identifier::receiver(ident.as_str());
            let local_name = wat_reader::identifier::method(ident.as_str());
            WatAST::Keyword(ns_to_wat_path(namespace, local_name), span.clone())
        }
        WatAST::List(items, span)
            if items.len() >= 3 && crate::types::peel_param_spec(&items[1..]).0.is_some() =>
        {
            WatAST::List(
                normalize_type_binder_form(items, sym, macros, errors),
                span,
            )
        }
        WatAST::List(items, span) => {
            let new_items = items
                .into_iter()
                .map(|c| normalize_type_slot(c, sym, macros, errors))
                .collect();
            WatAST::List(new_items, span)
        }
        WatAST::Vector(items, span) => {
            let new_items = items
                .into_iter()
                .map(|c| normalize_type_slot(c, sym, macros, errors))
                .collect();
            WatAST::Vector(new_items, span)
        }
        other => normalize_form(other, sym, macros, errors),
    }
}

/// Declaration name: rewrite to the identity keyword. Not a reference,
/// not a type slot — do not ask `is_resolvable_call_head`.
fn normalize_name_slot(
    node: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    match node {
        WatAST::Symbol(ref ident, ref span) if ident.is_reference() => {
            let namespace = wat_reader::identifier::receiver(ident.as_str());
            let local_name = wat_reader::identifier::method(ident.as_str());
            WatAST::Keyword(ns_to_wat_path(namespace, local_name), span.clone())
        }
        other => normalize_form(other, sym, macros, errors),
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
            WatAST::Vector(arm_items, arm_span) => {
                let mut new_arm = Vec::with_capacity(arm_items.len());
                match arm_items.len() {
                    2 => {
                        // `[_ body]` / `[binder body]` / hash-destructure.
                        // The pattern stays data (no keyword rewrite). A
                        // binder symbol becomes `{$bound, spelling}` so the
                        // body resolves locally. A variant head is the
                        // 3-element arm, not this one.
                        let mut ai = arm_items.into_iter();
                        let pat = ai.next();
                        let body = ai.next();
                        let _frame = FrameGuard::push();
                        if let Some(pat) = pat {
                            new_arm.push(rebind_two_element_pattern(pat));
                        }
                        if let Some(body) = body {
                            new_arm.push(normalize_form(body, sym, macros, errors));
                        }
                    }
                    3 => {
                        // `[Variant map body]`. The head stays a reference
                        // so `parse_match_arm` still sees a variant. Field
                        // patterns bind; a reference-symbol vector head
                        // inside a field stays a constructor.
                        let mut ai = arm_items.into_iter();
                        let head = ai.next();
                        let fields = ai.next();
                        let body = ai.next();
                        let _frame = FrameGuard::push();
                        if let Some(head) = head {
                            new_arm.push(head);
                        }
                        if let Some(fields) = fields {
                            new_arm.push(rebind_variant_fields(fields));
                        }
                        if let Some(body) = body {
                            new_arm.push(normalize_form(body, sym, macros, errors));
                        }
                    }
                    _ => new_arm.extend(arm_items),
                }
                out.push(WatAST::Vector(new_arm, arm_span));
            }
            WatAST::List(arm_items, arm_span) => {
                // Retired `(pattern body)` — still skip the pattern so a dual-read
                // never rewrites a variant head as a call; the checker/runtime refuse it.
                let mut new_arm = Vec::with_capacity(arm_items.len());
                let mut ai = arm_items.into_iter();
                new_arm.extend(ai.next());
                if let Some(body) = ai.next() {
                    new_arm.push(normalize_form(body, sym, macros, errors));
                }
                new_arm.extend(ai);
                out.push(WatAST::List(new_arm, arm_span));
            }
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
    // :then vector + any trailing args: DATA — contents untouched. 251.8d-ii SIXTH:
    // the quote HEAD is not data, it is the boundary marker, and `check.rs::infer_list`
    // only recognizes the keyword node. A symbol-spelled `(wat.core/quote …)` left here
    // is type-checked as an ordinary call and its fact vector walked as code.
    for rest in iter {
        out.push(normalize_quote_head_only(rest, sym, macros, errors));
    }
    out
}

/// Rewrite a `(wat.core/quote …)` boundary head to `:wat::core::quote` and leave every
/// argument byte-identical. Used for `make-rule`'s `:then` argument, which is pure data
/// under the [`Boundary::MakeRule`] contract — only the marker is re-spelled.
fn normalize_quote_head_only(
    node: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    let WatAST::List(qitems, qspan) = node else { return node };
    let is_quote = matches!(
        qitems
            .first()
            .and_then(crate::declare::parse::head_fqdn)
            .as_deref(),
        Some(":wat::core::quote")
    );
    if !is_quote {
        return WatAST::List(qitems, qspan);
    }
    WatAST::List(rewrite_boundary_head(qitems, sym, macros, errors), qspan)
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
    // 251.8d-ii SIXTH — the quote head is read through `head_fqdn`, the same door
    // `normalize_form` uses one frame up, so `(wat.core/quote …)` — what a
    // faithful-Clojure `defrule` template emits — is the same boundary as
    // `(:wat::core::quote …)`. The Keyword arm of that door is byte-identical.
    let is_quote = matches!(
        qitems
            .first()
            .and_then(crate::declare::parse::head_fqdn)
            .as_deref(),
        Some(":wat::core::quote")
    );
    if !is_quote {
        return WatAST::List(qitems, qspan);
    }
    // ⛔ And the head must be REWRITTEN, not merely recognized: `make-rule`'s
    // `:when` argument never reaches `normalize_form`'s generic boundary path
    // (this fn intercepts it), and `check.rs::infer_list` matches the KEYWORD
    // node — a surviving `wat.core/quote` symbol head is type-checked as an
    // ordinary call, which walks the quoted condition vector as CODE.
    let qitems = rewrite_boundary_head(qitems, sym, macros, errors);
    let mut qiter = qitems.into_iter();
    let mut new_q = Vec::with_capacity(2);
    new_q.extend(qiter.next()); // quote head, already normalized above
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
    let is_where = citems.first().is_some_and(is_where_form);
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
        if let Some(head) = items.first() {
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

fn name_is_registered(head: &str, sym: &SymbolTable, macros: &MacroRegistry) -> bool {
    crate::intrinsic::registry().contains(head)
        || sym.get(head).is_some()
        || sym.has_def_value(head)
        || macros.contains(head)
}

/// A binding the call will actually reach: a function, a defclause value, or
/// a macro. Intrinsic membership includes retired spellings, which are not
/// this.
fn name_has_binding(head: &str, sym: &SymbolTable, macros: &MacroRegistry) -> bool {
    sym.get(head).is_some()
        || sym.has_def_value(head)
        || macros.contains(head)
        || crate::intrinsic::registry().lookup_entry(head).is_some()
        || crate::rust_deps::registry().get_symbol(head).is_some()
}

/// Map a namespaced symbol name (`wat.core/+`) to its keyword FQDN candidate
/// (`:wat::core::+`) and validate it resolves. Returns the rewritten
/// `WatAST::Keyword` on success, or a located `UnresolvedReference` error.
///
/// `also_accept_type` widens acceptance to the UNION of "resolvable call head"
/// OR "known type" — arc 255 Stone ②'s contract for the HEAD of a `(Head :- […]
/// rest…)` binder, where the position's grammar admits either a genuine call
/// head (`(:wat::core::HashSet :- [T] "a" "b")`, a constructor call) or a type
/// reference (`(wat.type/Tuple :- [wat.type/i64])`). `false` for every other
/// caller — a type is legal in the binder head position because that
/// position's grammar admits one, not in an arbitrary call position.
fn resolve_namespaced_symbol(
    symbol_text: &str,
    span: &crate::span::Span,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    also_accept_type: bool,
) -> Result<WatAST, UnresolvedReference> {
    // Split on the FIRST `/` → (namespace, local_name) — the builder's
    // ruling, 2026-09-19. `receiver`/`method` ARE the namespace splitter on
    // this path, which is why the ruling governs them too and not only
    // `Identifier::bare`.
    assert!(symbol_text.contains('/'), "caller guarantees '/' present");
    let namespace = wat_reader::identifier::receiver(symbol_text);
    let local_name = wat_reader::identifier::method(symbol_text);

    // 255.88 — a `/` in the local name is a name character. `u/a/b` resolves
    // as `:u::a/b` (`ns_to_wat_path`), and `other_join` must not read that
    // `/` as a type-member join (`:u::a::b`). The 255.86 wall still runs
    // when the local name has no `/`: `wat.core.Option/expect` reconstructs
    // to `:wat::core::Option/expect` because `Option` is a type. Keyword
    // spellings never enter this function.
    let name_holds_a_slash = local_name.contains('/');
    let primary = if name_holds_a_slash {
        ns_to_wat_path(namespace, local_name)
    } else {
        match sym.types() {
            Some(env) => crate::types::reconstruct_call_path(namespace, local_name, env),
            None => ns_to_wat_path(namespace, local_name),
        }
    };

    // The other join allocates. A held, resolvable primary is the call, so
    // the alt string is not built. A retired primary is resolvable and has
    // no binding; an alt that DOES have a binding still wins, which is why
    // that comparison stays in front of `is_resolvable_call_head`.
    let primary_bound = name_has_binding(&primary, sym, macros);
    if !primary_bound && !name_holds_a_slash {
        if let Some(alt) = crate::types::other_join_spelling(&primary) {
            if name_has_binding(&alt, sym, macros) {
                return Ok(WatAST::Keyword(alt, span.clone()));
            }
            if !is_resolvable_call_head(&primary, sym, macros)
                && name_is_registered(&alt, sym, macros)
            {
                return Ok(WatAST::Keyword(alt, span.clone()));
            }
        }
    }
    if is_resolvable_call_head(&primary, sym, macros) {
        return Ok(WatAST::Keyword(primary, span.clone()));
    }
    if primary_bound && !name_holds_a_slash {
        if let Some(alt) = crate::types::other_join_spelling(&primary) {
            if name_is_registered(&alt, sym, macros) {
                return Ok(WatAST::Keyword(alt, span.clone()));
            }
        }
    }

    // Arc 255 Stone ② — the binder-head union's second acceptance. Routed
    // through `TypeEnv::is_known_type`, the ONE DOOR also used by
    // `:wat::runtime::is-type?` (`src/reflect/verbs.rs`), so the
    // `:wat::type::` canonicalization and the three-store union are never
    // written a second time here.
    if also_accept_type && sym.types().is_some_and(|types| types.is_known_type(&primary)) {
        return Ok(WatAST::Keyword(primary, span.clone()));
    }

    // Arc 255 Stone ⑤-E — `:wat::type::` is a TYPE-ONLY namespace: a name under
    // it is never a call head, in ANY position, so it is asked the TYPE
    // question regardless of `also_accept_type`. The namespace IS the
    // position for wat.type. Other type namespaces (`wat`, `wat.core`,
    // `wat.time`, `wat.rete`) also hold functions, so they need the flag
    // (255.5). Routed through the same one door (`TypeEnv::is_known_type`) so
    // the `:wat::type::` → `:wat::core::` canonicalization is never
    // reimplemented.
    if primary.starts_with(":wat::type::")
        && sym.types().is_some_and(|types| types.is_known_type(&primary))
    {
        return Ok(WatAST::Keyword(primary, span.clone()));
    }

    // 255.4 — the member join is `/`, always. Name/annotation position is a
    // different class (position grammar), not a second join.

    // Primary did not resolve → located error naming the unknown entity.
    Err(UnresolvedReference {
        path: primary.clone(),
        context: "namespaced symbol ref — not a builtin, not a registered function (arc 251)",
        span: span.clone(),
        remedy: None,
    })
}

/// Normalize a `(Head :- [T …] rest…)` type-binder form (arc 255 Stone ②). The
/// caller (`normalize_form`'s `List` arm) has already established `items.len()
/// >= 3` and that `items[1..]` peels as `(marker, [types], rest…)`.
///
/// - `items[0]` (head): the UNION — resolvable call head OR known type (see
///   [`normalize_type_binder_head`]).
/// - `items[1]` (`:-`): unchanged.
/// - `items[2]` (type vector): rewritten to keyword FQDNs with NO validation —
///   arc 296 P-1's annotation wall independently owns whether the names are
///   real types (DESIGN "shapes ruled out (b)": validating here against
///   `TypeEnv::contains` refuses `Tuple`, which is structural type syntax, and
///   `:wat::type::Infer`, a marker, not a registered type).
/// - `items[3..]` (value args): ordinary code — `(:wat::core::HashSet :- [T]
///   v1 v2)` carries live values after the type vector, and they normalize
///   exactly as a value position (STOP-2's guard: this must NOT be treated as
///   opaque data). A bare symbol here is a value, not a call head: the
///   protocol slot of `(extend-type :- [P …] Child Surface …)` is a surface
///   name (`wat.capability/Capability`), which is registered and is not a
///   function. `normalize_form` asks every reference symbol to be a call head
///   and refuses that surface.
fn normalize_type_binder_form(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut iter = items.into_iter();
    let head = iter.next().expect("caller checked items.len() >= 3");
    let marker = iter.next().expect("caller checked items.len() >= 3");
    let type_vec = iter.next().expect("caller checked items.len() >= 3");

    let new_head = normalize_type_binder_head(head, sym, macros, errors);
    let new_type_vec = normalize_type_vector(type_vec);

    let mut out = Vec::with_capacity(3);
    out.push(new_head);
    out.push(marker);
    out.push(new_type_vec);
    out.extend(iter.map(|c| normalize_value_position(c, sym, macros, errors)));
    out
}

/// The head of a `:-` type binder is asked the UNION: is it a resolvable call
/// head (a genuine constructor call, `(:wat::core::HashSet :- [T] "a" "b")`) OR
/// a known type (a type reference, `(wat.type/Tuple :- [wat.type/i64])`)? Both
/// are legal in this position's grammar, so both are asked — nothing is
/// exempted (the first strike's defect: treating the head as type syntax and
/// skipping call-head validation let `(my.app/totally-bogus :- [i64] 1)`
/// through silently).
///
/// Widened via `resolve_namespaced_symbol`'s `also_accept_type` flag —
/// binder heads (Stone ②) and 255.5 type slots (annotation, return,
/// nested parametric args, tuple elements). Call position stays `false`.
fn normalize_type_binder_head(
    head: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> WatAST {
    match head {
        WatAST::Symbol(ref ident, ref span) if ident.is_reference() => {
            match resolve_namespaced_symbol(ident.as_str(), span, sym, macros, true) {
                Ok(kw) => kw,
                Err(e) => {
                    errors.push(e);
                    head // leave the symbol in place so the walk continues
                }
            }
        }
        other => normalize_form(other, sym, macros, errors),
    }
}

/// Rewrite a `:-` binder's type-argument vector to keyword FQDNs with NO
/// validation — arc 296 P-1's annotation wall independently owns whether the
/// names are real types (see [`normalize_type_binder_form`]'s doc). A
/// referencing `Symbol` (`wat.type/i64`) becomes the `Keyword` it names
/// (`ns_to_wat_path`, the same mapping `resolve_namespaced_symbol` uses); a
/// bare symbol (no `/`) is a type VARIABLE and is left untouched. Recurses
/// through `List`/`Vector` so a nested parametric (`(V :- [T])`) is reached —
/// including a nested binder's own head, which is type syntax here too, not a
/// call head to validate.
fn normalize_type_vector(node: WatAST) -> WatAST {
    match node {
        WatAST::Symbol(ref ident, ref span) if ident.is_reference() => {
            let namespace = wat_reader::identifier::receiver(ident.as_str());
            let local_name = wat_reader::identifier::method(ident.as_str());
            WatAST::Keyword(ns_to_wat_path(namespace, local_name), span.clone())
        }
        WatAST::List(items, span) => WatAST::List(
            items.into_iter().map(normalize_type_vector).collect(),
            span,
        ),
        WatAST::Vector(items, span) => WatAST::Vector(
            items.into_iter().map(normalize_type_vector).collect(),
            span,
        ),
        other => other,
    }
}

/// `let` binders. Each RHS is normalized with the binders so far, then the
/// pattern is bound. An odd-length or non-vector binding vector falls
/// through to the ordinary rewrite so the existing diagnostic still fires.
fn normalize_scoped_let(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut iter = items.into_iter();
    let Some(head) = iter.next() else {
        return Vec::new();
    };
    let head = normalize_form(head, sym, macros, errors);
    let Some(bindings) = iter.next() else {
        return vec![head];
    };
    let WatAST::Vector(pairs, bspan) = bindings else {
        let mut out = vec![head, normalize_form(bindings, sym, macros, errors)];
        out.extend(iter.map(|c| normalize_form(c, sym, macros, errors)));
        return out;
    };
    if pairs.len() % 2 != 0 {
        let pairs = pairs
            .into_iter()
            .map(|c| normalize_form(c, sym, macros, errors))
            .collect();
        let mut out = vec![head, WatAST::Vector(pairs, bspan)];
        out.extend(iter.map(|c| normalize_form(c, sym, macros, errors)));
        return out;
    }
    let _frame = FrameGuard::push();
    let mut new_pairs = Vec::with_capacity(pairs.len());
    let mut pit = pairs.into_iter();
    while let (Some(pat), Some(rhs)) = (pit.next(), pit.next()) {
        let rhs = normalize_form(rhs, sym, macros, errors);
        new_pairs.push(rebind_let_pattern(pat));
        new_pairs.push(rhs);
    }
    let mut out = vec![head, WatAST::Vector(new_pairs, bspan)];
    out.extend(iter.map(|c| normalize_form(c, sym, macros, errors)));
    out
}

/// `fn` / `lambda`. Parameter names are binders. Types and the return
/// slot are normalized on the outer scope, then the body sees the params.
fn normalize_scoped_fn(
    items: Vec<WatAST>,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> Vec<WatAST> {
    let mut iter = items.into_iter();
    let Some(head) = iter.next() else {
        return Vec::new();
    };
    let head = normalize_form(head, sym, macros, errors);
    let mut args: Vec<WatAST> = iter.collect();
    let meta = if args.first().is_some_and(|n| n.is_metadata_map()) {
        Some(normalize_form(args.remove(0), sym, macros, errors))
    } else {
        None
    };
    let binder = if args.len() >= 2
        && crate::types::is_param_annotation_arrow(&args[0])
        && matches!(args.get(1), Some(WatAST::Vector(_, _)))
    {
        let marker = args.remove(0);
        let tv = normalize_type_slot(args.remove(0), sym, macros, errors);
        Some((marker, tv))
    } else {
        None
    };
    if args.is_empty() {
        let mut out = vec![head];
        if let Some(m) = meta {
            out.push(m);
        }
        if let Some((marker, tv)) = binder {
            out.push(marker);
            out.push(tv);
        }
        return out;
    }
    let params_node = args.remove(0);
    let (params_node, keys) = rewrite_param_vector(params_node, sym, macros, errors);
    let _frame = FrameGuard::push_with(keys);
    let mut out = Vec::with_capacity(4 + args.len());
    out.push(head);
    if let Some(m) = meta {
        out.push(m);
    }
    if let Some((marker, tv)) = binder {
        out.push(marker);
        out.push(tv);
    }
    out.push(params_node);
    let mut prev_return = false;
    for c in args {
        let this_return = crate::types::is_return_arrow(&c);
        let new = if prev_return {
            normalize_type_slot(c, sym, macros, errors)
        } else {
            normalize_form(c, sym, macros, errors)
        };
        out.push(new);
        prev_return = this_return;
    }
    out
}

fn rewrite_param_vector(
    node: WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    errors: &mut Vec<UnresolvedReference>,
) -> (WatAST, Vec<String>) {
    let WatAST::Vector(items, span) = node else {
        return (normalize_form(node, sym, macros, errors), Vec::new());
    };
    let mut keys = Vec::new();
    let mut out = Vec::with_capacity(items.len());
    let mut items = items.into_iter().peekable();
    while let Some(item) = items.next() {
        let WatAST::Symbol(id, span) = item else {
            out.push(normalize_form(item, sym, macros, errors));
            continue;
        };
        if id.as_str() == "&" {
            out.push(WatAST::Symbol(id, span));
            let Some(name_node) = items.next() else {
                break;
            };
            if let WatAST::Symbol(nid, nspan) = name_node {
                if nid.as_str() != "_" {
                    keys.push(crate::scope::env_key(&nid).into_owned());
                }
                out.push(WatAST::Symbol(nid.into_bound(), nspan));
            } else {
                out.push(normalize_form(name_node, sym, macros, errors));
            }
            if items.peek().is_some_and(crate::types::is_param_annotation_arrow) {
                out.push(items.next().expect("peeked"));
                if let Some(ty) = items.next() {
                    out.push(normalize_type_slot(ty, sym, macros, errors));
                }
            }
            continue;
        }
        if matches!(id.as_str(), ":-" | "<-" | "->") {
            out.push(WatAST::Symbol(id, span));
            continue;
        }
        if id.as_str() != "_" {
            keys.push(crate::scope::env_key(&id).into_owned());
        }
        out.push(WatAST::Symbol(id.into_bound(), span));
        if items.peek().is_some_and(crate::types::is_param_annotation_arrow) {
            out.push(items.next().expect("peeked"));
            if let Some(ty) = items.next() {
                out.push(normalize_type_slot(ty, sym, macros, errors));
            }
        }
    }
    (WatAST::Vector(out, span), keys)
}

fn rebind_let_pattern(node: WatAST) -> WatAST {
    match node {
        WatAST::Symbol(id, span) => {
            scope_insert_ident(&id);
            WatAST::Symbol(id.into_bound(), span)
        }
        WatAST::Vector(items, span) => {
            WatAST::Vector(items.into_iter().map(rebind_let_pattern).collect(), span)
        }
        WatAST::Map(pairs, span) => rebind_let_map(pairs, span),
        other => other,
    }
}

fn rebind_let_map(pairs: Vec<(WatAST, WatAST)>, span: crate::span::Span) -> WatAST {
    let is_keys = pairs.len() == 1
        && matches!(
            &pairs[0],
            (WatAST::Keyword(k, _), WatAST::Vector(_, _)) if k == ":keys"
        );
    if is_keys {
        let (k, v) = pairs.into_iter().next().expect("len 1");
        return WatAST::Map(vec![(k, rebind_let_pattern(v))], span);
    }
    let is_hash = !pairs.is_empty()
        && pairs.iter().all(|(k, v)| {
            matches!(k, WatAST::Symbol(_, _)) && matches!(v, WatAST::Keyword(_, _))
        });
    if is_hash {
        let pairs = pairs
            .into_iter()
            .map(|(k, v)| (rebind_let_pattern(k), v))
            .collect();
        return WatAST::Map(pairs, span);
    }
    WatAST::Map(pairs, span)
}

fn rebind_two_element_pattern(node: WatAST) -> WatAST {
    match node {
        WatAST::Symbol(id, span) => {
            scope_insert_ident(&id);
            WatAST::Symbol(id.into_bound(), span)
        }
        WatAST::Map(pairs, span) => rebind_let_map(pairs, span),
        other => other,
    }
}

/// Variant field patterns. Keys stay. A reference-symbol vector or list
/// head is a constructor and is not rebound.
fn rebind_variant_fields(node: WatAST) -> WatAST {
    match node {
        WatAST::Map(pairs, span) => {
            let pairs = pairs
                .into_iter()
                .map(|(k, v)| (k, rebind_match_subpattern(v)))
                .collect();
            WatAST::Map(pairs, span)
        }
        other => other,
    }
}

fn rebind_match_subpattern(node: WatAST) -> WatAST {
    match node {
        WatAST::Symbol(id, span) => {
            scope_insert_ident(&id);
            WatAST::Symbol(id.into_bound(), span)
        }
        WatAST::Vector(items, span) => {
            let skip = match items.first() {
                Some(WatAST::Keyword(_, _)) => 1,
                Some(WatAST::Symbol(id, _)) if id.is_reference() => 1,
                _ => 0,
            };
            let items = items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    if i < skip {
                        item
                    } else {
                        rebind_match_subpattern(item)
                    }
                })
                .collect();
            WatAST::Vector(items, span)
        }
        WatAST::List(items, span) => {
            let items = items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    if i == 0 {
                        item
                    } else {
                        rebind_match_subpattern(item)
                    }
                })
                .collect();
            WatAST::List(items, span)
        }
        WatAST::Map(pairs, span) => {
            let pairs = pairs
                .into_iter()
                .map(|(k, v)| (k, rebind_match_subpattern(v)))
                .collect();
            WatAST::Map(pairs, span)
        }
        other => other,
    }
}
