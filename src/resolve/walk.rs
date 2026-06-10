//! Call-head resolution walk — the core of the name-resolution pass.
//!
//! [`resolve_references`] — the public entry point (two-pass: collect use!
//! declarations, then walk all call heads).
//! [`check_form`] — the recursive AST walker that resolves call heads.
//! [`is_resolvable_call_head`] — the predicate that decides whether a keyword
//! head is a valid call target.

use crate::ast::WatAST;
use crate::macros::MacroRegistry;
use crate::runtime::SymbolTable;
use super::error::{ResolveError, UnresolvedReference};
use super::boundary::{quote_boundary, Boundary};
use super::reserved::is_reserved_prefix;
use super::rust_use::collect_use_declarations;
use super::quote::check_quasiquote_template;

/// Check that every call-position keyword-path reference in `forms`
/// resolves somewhere. Returns Ok iff all references are known;
/// otherwise reports every failure at once.
///
/// `forms` is the program's **top-level** form sequence (the freeze residue).
/// Pass 1's `use!` scan is program-global precisely because it reads the
/// top-level forms; a nested slice would silently lose `use!` declarations
/// hoisted above it. The sole caller (`freeze.rs` step 7) always passes the
/// top-level residue. (The `&[WatAST]` type cannot express "top-level only";
/// a `TopLevelForms` newtype would make it structural — tracked, not urgent.)
pub fn resolve_references(
    forms: &[WatAST],
    sym: &SymbolTable,
    macros: &MacroRegistry,
) -> Result<(), ResolveError> {
    let mut unresolved = Vec::new();

    // Pass 1: collect `(:wat::core::use! :rust::...)` top-level
    // declarations. Validates against the rust-deps registry. Program-
    // global scope (one use! anywhere enables the symbol everywhere).
    // rune:sequi(ambient-context) — rust-deps registry is a write-once dispatch
    // table installed at startup; threading it through every resolver/eval
    // signature would bloat every call site for a read-only config surface,
    // not domain state.
    let registry = crate::rust_deps::registry();
    let mut use_decls = crate::rust_deps::UseDeclarations::new();
    for form in forms {
        collect_use_declarations(form, registry, &mut use_decls, &mut unresolved);
    }

    // Pass 2: walk all call heads, including nested. Every :rust::* call
    // head must be covered by one of the use! declarations from pass 1.
    for form in forms {
        check_form(form, sym, macros, &use_decls, &mut unresolved);
    }
    if unresolved.is_empty() {
        Ok(())
    } else {
        Err(ResolveError::UnresolvedReferences(unresolved))
    }
}

// rune:intueri(length) — a call-head check followed by a `match` on the head's
// [`Boundary`] (classified once in `super::boundary`, shared with `normalize`).
// Each arm is short, carries its own arc-attribution, and applies this pass's
// action (call-head resolution) to that boundary's code regions. The length is an
// orchestration sequence over the language's special-form boundaries, not braided
// concerns — and the boundary-head SET no longer lives here, so it cannot drift.
pub(super) fn check_form(
    form: &WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    use_decls: &crate::rust_deps::UseDeclarations,
    unresolved: &mut Vec<UnresolvedReference>,
) {
    // Walker-specific List-head logic: call-head resolution and quote-family
    // boundary guards apply only to List forms with Keyword heads.
    if let WatAST::List(items, _) = form {
        if let Some(WatAST::Keyword(head, head_span)) = items.first() {
            if !is_resolvable_call_head(head, sym, macros) {
                unresolved.push(UnresolvedReference {
                    path: head.clone(),
                    context: if macros.contains(head) {
                        "macro call survived expansion (expansion pass ran before this check?)"
                    } else {
                        "call head — not a builtin, not a registered function"
                    },
                    span: head_span.clone(),
                });
            }

            // Additional :rust::* enforcement: the call head must be
            // covered by a `(:wat::core::use! :rust::Type)` declaration
            // SOMEWHERE in the program. The declared type path prefixes
            // the method path — `:rust::lru::LruCache::new` is covered
            // by a use! of `:rust::lru::LruCache`.
            if head.starts_with(":rust::") {
                let covered = use_decls
                    .list()
                    .any(|decl| head == decl || head.starts_with(&format!("{}::", decl)));
                if !covered {
                    unresolved.push(UnresolvedReference {
                        path: head.clone(),
                        context:
                            ":rust::* reference not covered by any (:wat::core::use! ...) declaration",
                        span: head_span.clone(),
                    });
                }
            }

            // Special-form argument boundary. The head's [`Boundary`] (classified
            // once in `super::boundary`) decides which child regions are live code
            // to resolve and which are data to leave alone. This `match` is
            // exhaustive: a new boundary variant is a compile error here until it
            // is handled — the structural guarantee that walk and normalize cannot
            // drift on the boundary-head set.
            match quote_boundary(head) {
                // quote / forms / define — every argument is data. (define is
                // retired at the checker (Stone 241.11); the resolver still must
                // not walk its body, since step 7 runs before the rejection.)
                Boundary::AllData => return,

                // quasiquote (arc 170 F-2) — the template (items[1]) is data
                // except inside unquote / unquote-splicing escapes. Nested
                // quasiquote is treated as opaque data (out of scope for F-2).
                Boundary::Quasiquote => {
                    if let Some(template) = items.get(1) {
                        check_quasiquote_template(template, sym, macros, use_decls, unresolved);
                    }
                    return;
                }

                // matches? (arc 098) — only the subject (items[1]) is code; the
                // pattern (items[2..]) is DSL data owned by check.rs
                // `infer_form_matches` (struct-name head + clause keywords, none
                // of which are call heads).
                Boundary::MatchesSubject => {
                    if let Some(subject) = items.get(1) {
                        check_form(subject, sym, macros, use_decls, unresolved);
                    }
                    return;
                }

                // cond (arc 245 room 4) — every arm is ordinary code, EXCEPT a
                // leading `:else` marker (cond's own DSL keyword, owned by
                // check.rs `infer_cond`): skip the marker, walk the arm body.
                Boundary::Cond => {
                    for item in items.iter().skip(1) {
                        if let WatAST::List(arm_items, _) = item {
                            if let Some(WatAST::Keyword(arm_head, _)) = arm_items.first() {
                                if arm_head == ":else" {
                                    for body in arm_items.iter().skip(1) {
                                        check_form(body, sym, macros, use_decls, unresolved);
                                    }
                                    continue;
                                }
                            }
                        }
                        check_form(item, sym, macros, use_decls, unresolved);
                    }
                    return;
                }

                // match (arc 245 long-tail) — items[1]=scrutinee (walk),
                // items[2..=3]=`-> :T` (skip), items[4..]=arms. Each arm is
                // `(pattern body)`: the pattern is DSL data owned by check.rs
                // `infer_match` (variant/constructor head, not a call head), so
                // walk only the body (arm_items[1]). A bare-symbol wildcard arm is
                // walked directly.
                Boundary::Match => {
                    if let Some(scrutinee) = items.get(1) {
                        check_form(scrutinee, sym, macros, use_decls, unresolved);
                    }
                    for arm in items.iter().skip(4) {
                        if let WatAST::List(arm_items, _) = arm {
                            if let Some(body) = arm_items.get(1) {
                                check_form(body, sym, macros, use_decls, unresolved);
                            }
                        } else {
                            check_form(arm, sym, macros, use_decls, unresolved);
                        }
                    }
                    return;
                }

                // Not a boundary — fall through to the generic children() walk.
                Boundary::Ordinary => {}
            }
        }
    }
    // Arc 212 — generic recursion via children() covers List, Vector, Map,
    // and Set uniformly. Call-head resolution fires on List forms;
    // the generic recursion ensures call forms nested inside bracketed
    // shapes (e.g., let-binding vector RHSes) are still resolved.
    // children() returns &[] for leaf nodes (no-op).
    for child in form.children().iter() {
        check_form(child, sym, macros, use_decls, unresolved);
    }
}

/// True if `head` resolves as a call target.
///
/// `pub(super)` — used by [`super::normalize`] to validate candidate FQDN keywords
/// before rewriting a namespaced symbol ref (arc 251 stone 251.1b).
pub(super) fn is_resolvable_call_head(head: &str, sym: &SymbolTable, macros: &MacroRegistry) -> bool {
    // Kernel, algebra, std, config, and core prefixes are reserved for
    // the language; accept them as-is. A wrong name under those
    // prefixes (e.g. :wat::holon::Bogus) fails DOWNSTREAM at
    // runtime or lowering, but the name-resolution pass is scoped
    // to catch "no such namespace" mistakes, not "wrong name inside
    // a known namespace" mistakes. The spec's name-resolution layer
    // wants the path-prefix shape validated; leaf-level validation
    // is the type checker's concern.
    if is_reserved_prefix(head) {
        return true;
    }
    // Arc 139 — strip turbofish `<T,...>` before sym.get. The
    // substrate registers user defines under the canonical name
    // (sans turbofish); call sites that use turbofish resolve to
    // the same function. See `canonical_callable_name` in runtime.rs
    // for the full rationale (symmetric registration vs lookup).
    let canonical = crate::runtime::canonical_callable_name(head);
    // A user-registered function.
    if sym.get(canonical).is_some() {
        return true;
    }
    // Stone 241.9 — unit enum variants are stored in `sym.unit_variants`
    // (not `sym.functions`) after `register_enum_methods` (step 6.5).
    // In defenum's positional grammar, unit variant arms in `match` appear
    // as `(:Ns::E::V body)` — a list whose head is the variant keyword.
    // The resolver must accept these as valid call heads; without this check,
    // `defn` bodies that use unit variant match arms fail resolve (the `defn`
    // form stays in `residue` and is walked by step 7, unlike `define` bodies
    // which are consumed by `register_defines`).
    if sym.unit_variants.contains_key(canonical) {
        return true;
    }
    // A macro call — shouldn't survive expansion, but accept for
    // completeness. The checker notes it as suspicious in the
    // context string when a macro is the reason.
    if macros.contains(head) {
        return true;
    }
    // Arc 245 long-tail — single-segment keyword call heads (field accessors).
    //
    // A keyword without `::` in its body (e.g. `:magnitude`, `:x`, `:port`,
    // `:None`) in call position is a user field/HashMap accessor candidate,
    // or an Option/Result constructor short-form in pattern position (handled
    // via the :wat::core::match boundary in check_form).
    //
    // check.rs `infer_list` (Stone 234.3c) accepts these when the receiver
    // type is a Record, Struct, or HashMap; the runtime dispatches them via
    // `keyword_accessor_record` / `keyword_accessor_struct` / HashMap key
    // lookup. The resolver is the wrong layer to reject them — only the
    // checker knows the receiver's type, and the match boundary in check_form
    // prevents arm-pattern heads from reaching this path anyway.
    //
    // Discriminant: starts with `:` and contains no `::`.
    // Multi-segment user paths like `:my::app::missing` still fail here.
    if head.starts_with(':') && !head.contains("::") {
        return true;
    }
    false
}
