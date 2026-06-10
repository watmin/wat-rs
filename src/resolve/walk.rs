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
use super::reserved::is_reserved_prefix;
use super::rust_use::collect_use_declarations;
use super::quote::check_quasiquote_template;

/// Check that every call-position keyword-path reference in `forms`
/// resolves somewhere. Returns Ok iff all references are known;
/// otherwise reports every failure at once.
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

// rune:intueri(length) — a multi-way call-head-boundary dispatch (the quote-family
// boundaries + match/cond pattern-arm boundaries + generic children() recursion). Each
// arm is short and carries its own arc-attribution comment; the length is an
// orchestration sequence over the language's special-form boundaries, not braided
// concerns. The match-arm boundary is the natural extraction candidate if it grows.
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

            // Arc 170 slice 3 Gap F-2 — quote-family boundary.
            //
            // Quote-family forms capture their arguments as AST data;
            // the arguments are NOT live code and must not be walked for
            // call-head resolution.
            //
            // :wat::core::forms — variadic data-capture; ALL arguments are
            //   data. Do not recurse into any child.
            //
            // :wat::core::quote — single argument is data. Do not recurse.
            //
            // :wat::core::quasiquote — the template argument is data EXCEPT
            //   inside :wat::core::unquote / :wat::core::unquote-splicing
            //   escape forms. Use quasiquote-aware descent: recurse only
            //   through the unquote/unquote-splicing children, treating the
            //   rest of the template as opaque data.
            //
            // Nested quasiquote (depth > 1) is out of scope for F-2:
            // a (:wat::core::quasiquote ...) encountered INSIDE a quasiquote
            // template is treated as data (not descended into). This is
            // conservative and correct for all current callers; if nested-
            // quasiquote resolver semantics are needed, a dedicated arc
            // should address them.
            if head == ":wat::core::forms" || head == ":wat::core::quote" {
                // Arguments are data — do not recurse into any child.
                return;
            }
            // Stone 241.11 — HARD CUT: :wat::core::define is retired.
            // The checker (step 8) rejects it with a structured MalformedForm
            // retirement remedy. The resolver must NOT recurse into the body
            // (step 7 runs before the checker); the body's call references are
            // irrelevant since the form itself will be rejected.
            if head == ":wat::core::define" {
                return;
            }
            if head == ":wat::core::quasiquote" {
                // Template is data except inside unquote/unquote-splicing.
                // items[1] is the template argument (if present).
                if let Some(template) = items.get(1) {
                    check_quasiquote_template(template, sym, macros, use_decls, unresolved);
                }
                return;
            }

            // Arc 098 — :wat::form::matches? boundary (mirrors quote-family above).
            //
            // (:wat::form::matches? SUBJECT PATTERN)
            //
            // SUBJECT (items[1]) is ordinary code — let-bound locals, constructor
            // calls — and MUST be walked for call-head resolution.
            //
            // PATTERN (items[2]) is DSL data owned by the matches? grammar walker
            // (src/check.rs `infer_form_matches`).  Its head is a struct-type name
            // in pattern position (e.g. `:test::PaperResolved`), which is NOT a
            // call-head — it is a struct name the checker validates against the
            // struct registry.  The clause sub-forms inside the pattern use DSL
            // clause heads (`=`, `<`, `>`, `:not=`, `:and`, `:or`, `:not`,
            // `:where`) that are likewise not ordinary call heads.  Resolving them
            // as call heads would always produce false `UnresolvedReference` errors.
            //
            // Do NOT recurse into items[2..] (the pattern and any extra args).
            // Recurse ONLY into items[1] (the subject).
            if head == ":wat::form::matches?" {
                if let Some(subject) = items.get(1) {
                    check_form(subject, sym, macros, use_decls, unresolved);
                }
                return;
            }

            // Arc 245 room 4 — :wat::core::cond `:else` marker boundary
            // (the finer sibling of the matches? boundary above).
            //
            // (:wat::core::cond -> :T (test1 r1) ... (:else default))
            //
            // Unlike matches?'s pattern, cond's arms ARE ordinary code — every
            // test and every result must be walked. The ONE exception is the
            // `:else` marker heading the default arm: it is the cond grammar's
            // DSL keyword (check.rs `infer_cond` owns it), not a call head.
            // Walking it as a call head produces a false UnresolvedReference.
            //
            // Walk every child as usual, but for an arm headed by the `:else`
            // keyword, skip the marker and walk only the arm's body.
            if head == ":wat::core::cond" {
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

            // Arc 245 long-tail — :wat::core::match arm boundary.
            //
            // (:wat::core::match scrutinee -> :T arm1 arm2 ...)
            //
            // Each arm is `(pattern body)`. The PATTERN is DSL data owned by
            // check.rs `infer_match` — its head keyword is a variant name or
            // Option/Result constructor in PATTERN position, not a call head.
            // Examples: `(:None false)`, `(:wat::core::Some x)`, `((:Ns::E::V) body)`.
            //
            // Walking arm patterns as call heads produces false
            // UnresolvedReference errors (e.g. `:None` in `(:None false)`).
            //
            // Structure: items[1]=scrutinee (walk), items[2..3]=`-> :T` (skip),
            // items[4..]=arms; each arm is `(pattern body)` — walk only the body.
            if head == ":wat::core::match" {
                // Walk the scrutinee (items[1]) as live code.
                if let Some(scrutinee) = items.get(1) {
                    check_form(scrutinee, sym, macros, use_decls, unresolved);
                }
                // Skip items[2] (`->` symbol) and items[3] (return type keyword).
                // Walk each arm's body (items[1] of the arm); skip the pattern
                // (items[0] of the arm) — it is DSL data, not a call head.
                for arm in items.iter().skip(4) {
                    if let WatAST::List(arm_items, _) = arm {
                        // arm_items[0] = pattern (DSL data — skip call-head check)
                        // arm_items[1] = body (live code — walk)
                        if let Some(body) = arm_items.get(1) {
                            check_form(body, sym, macros, use_decls, unresolved);
                        }
                        // Also walk nested call forms inside the PATTERN only for
                        // composite patterns like `((:wat::core::Some x) body)`:
                        // the inner `(:wat::core::Some x)` is itself a list whose
                        // head IS a reserved prefix and is fine to walk — but the
                        // outermost pattern head (e.g. `:None`) is the DSL marker.
                        // We skip the outermost pattern head entirely to avoid the
                        // false-positive; nested resolution inside composite patterns
                        // is handled by the recursive walk of the arm body above.
                        // (The pattern sub-forms like `(:wat::core::Some x)` are data
                        // nodes that the checker owns; resolving them is unnecessary
                        // and can only produce false positives.)
                    } else {
                        // Non-list arm (e.g. a bare symbol wildcard) — walk it.
                        check_form(arm, sym, macros, use_decls, unresolved);
                    }
                }
                return;
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
