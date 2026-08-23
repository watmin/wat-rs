use crate::ast::WatAST;
use crate::span::Span;
use crate::scope::{fresh_scope, ScopeId};
use crate::runtime::{Environment, SymbolTable, Value};
use crate::value::TrackedValue;
use std::collections::HashMap;
use std::sync::Arc;

use super::error::{MacroError, MacroErrorKind};
use super::registry::{MacroDef, MacroRegistry};
use super::parse::{is_defmacro_form, parse_defmacro_form};

/// Maximum nesting depth for macro expansion. Enforced in `expand_form`
/// to guard against infinite-recursive macros. Defined here (next to
/// its enforcement) and re-exported via `mod.rs`.
pub const EXPANSION_DEPTH_LIMIT: usize = 512;

/// Expand every macro call in `forms` to fixpoint. Returns the expanded
/// AST list.
/// Expand USER forms — the common case (macroexpand primitives, resolution, tests).
/// Privileged stdlib expansion calls [`expand_all_with`] with `Privilege::Stdlib` directly
/// (only `freeze/env.rs`'s stdlib pass). The privilege is explicit, never ambient.
pub fn expand_all(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
    env: &Environment,
    sym: &SymbolTable,
) -> super::ExpandBatch {
    expand_all_with(forms, registry, env, sym, crate::resolve::Privilege::User)
}

pub fn expand_all_with(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
    env: &Environment,
    sym: &SymbolTable,
    privilege: crate::resolve::Privilege,
) -> super::ExpandBatch {
    // Arc 029 slice 1: handle macro-generating-macros. A macro call
    // may expand to a `(:wat::core::defmacro ...)` registration for
    // a new macro — e.g., `:wat::test::make-deftest` expanding to a
    // fully-configured deftest variant. Register each such form as
    // it appears so subsequent forms in the stream can invoke the
    // new macro.
    //
    // Arc 030 slice 2: registry is now `&mut` — new defmacros
    // generated at expansion time persist to the caller's registry,
    // so the frozen world sees them (required for runtime
    // macroexpand / macroexpand-1 primitives to find them).
    let mut out = Vec::with_capacity(forms.len());
    for form in forms {
        let expanded = expand_form(form, registry, 0, env, sym, privilege)?;
        if is_defsurface_form(&expanded) {
            // Arc 294 item 9a — a defsurface's `:messages` children are `defrecord`/
            // `defenum` calls; `expand_form`'s child-recursion (above) already expanded
            // each `defrecord` into its `(do (recordtype …)(defmacro …))` kwargs companion,
            // but left it NESTED inside the `:messages` vector, where this loop's own
            // `is_do_or_let_containing_defmacro` check (below) never looks — the surface
            // form itself is not a `do`/`let`. Route the SAME hoist-and-splice treatment the ordinary
            // top-level `do` case gets (`hoist_top_level_form`) over each `:messages`
            // child, so their `recordtype`/`defenum` decl splices to top level — BEFORE the
            // surface form, so the surface's `:features` method sigs can resolve them. The
            // `:messages` vector inside the surface form is left untouched: `defservice`'s
            // shipped `surface-forms` carrier re-hoists it identically in the forked child
            // (wat/service.wat:1027-1130).
            //
            // The SPLICE is what is load-bearing here, and it is why this arm SURVIVES now
            // that `expand_form` registers sequentially (arc 294 item 9a — see its doc).
            // Sequential registration subsumed this arm's *macro-registration* half: a
            // `:messages` child's companion `defmacro` now registers during expansion, when
            // the nested `(do (recordtype …)(defmacro …))` container's own body-walk reaches
            // it. But expansion CANNOT do the other half: `expand_form` returns ONE form per
            // form, so it can never lift a nested `recordtype`/`defenum` DECL out to the
            // top-level stream — and only a top-level decl is walked by the downstream
            // `register_types`/`register_defines` passes that mint the type and its
            // accessors. Deleting this arm was measured (the whole floor): 49 regressions,
            // and their errors name that exact half — `#wat.resolve/UnresolvedReference
            // {:path ":S::Msg/field" :context "call head — not a builtin, not a registered
            // function"}` and `kwargs-construct: type :S::Msg is not a registered aggregate`.
            // Registration was never the complaint. Hoisting is a STAGE, not an ordering.
            let (hoisted, surface_form) = hoist_surface_messages(expanded, registry, privilege)?;
            out.extend(hoisted);
            out.push(surface_form);
        } else {
            out.extend(hoist_top_level_form(expanded, registry, privilege)?);
        }
    }
    Ok(out)
}

/// Returns `true` if `form` is a `(:wat::core::defsurface ...)` form.
fn is_defsurface_form(form: &WatAST) -> bool {
    if let WatAST::List(items, _) = form {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            return head == ":wat::core::defsurface";
        }
    }
    false
}

/// Process one already-expanded top-level form for hoisting: register a bare
/// top-level `defmacro`, hoist-and-splice a `do`-wrapped macro-companion (the
/// defstruct/defrecord kwargs-companion shape), hoist-in-place a `let`-wrapped
/// one, or pass the form through unchanged. Returns the 0-or-more forms that
/// belong at the top level, in order. Factored out of `expand_all`'s per-form
/// loop (arc 294 item 9a) so `hoist_surface_messages` can apply the IDENTICAL
/// treatment to a `defsurface`'s `:messages` children, which arrive in the
/// same shapes.
fn hoist_top_level_form(
    expanded: WatAST,
    registry: &mut MacroRegistry,
    privilege: crate::resolve::Privilege,
) -> Result<Vec<WatAST>, MacroError> {
    if is_defmacro_form(&expanded) {
        let def = parse_defmacro_form(expanded)?;
        registry.register(def, privilege)?;
        Ok(vec![])
    } else if is_do_or_let_containing_defmacro(&expanded) {
        // Arc 260.1b — a macro-generating-macro (e.g. defn's kwargs branch) may
        // emit a `do` form whose children include a `defmacro` registration. Walk
        // the `do`'s children: register any defmacro children immediately (so
        // subsequent forms in the stream can invoke the new macro) and strip them
        // from the `do` body, keeping the remaining non-defmacro children.
        //
        // Arc 294 item 9a follow-on (Gap: "companion trapped in a nested `let`
        // body") — a `defstruct`/`defrecord` call sitting in a top-level `let`
        // body (instead of `do`) expands to the SAME `(do (structtype …)
        // (defmacro …))` companion shape, just one level deeper. The container
        // itself is `let`-headed, and `let` (unlike `do`) introduces bindings —
        // its body cannot be freely spliced to the top level (that would drop
        // any body form's dependency on the let's bound variables out of
        // scope). `hoist_defmacros_from_container` handles both container kinds
        // uniformly, but only a `do` container's surviving children are eligible
        // for the top-level splice below; a `let` keeps its wrapper.
        let rebuilt = hoist_defmacros_from_container(expanded, registry, privilege)?;
        match rebuilt {
            WatAST::List(items, span) => {
                let is_do = matches!(
                    items.first(),
                    Some(WatAST::Keyword(h, _)) if h == ":wat::core::do"
                );
                if is_do {
                    // A macro-emission `do` is a registration WRAPPER, never a
                    // value-position do (see `hoist_defmacros_from_container`'s
                    // contract): its surviving children — after the defmacro
                    // siblings were hoisted out — are all top-level
                    // declaration/registration forms (structtype/def/extend-type/
                    // Record::def). SPLICE them up into the top-level form stream
                    // so each registers as its own top-level declaration, exactly
                    // as if emitted at top level. Leaving them wrapped in a `(do
                    // …)` makes it a value-position do whose declaration children
                    // a later registration pass strips, emptying the do and
                    // tripping the checker's "do requires ≥1 form" wall (arc 294
                    // item 9a: the defstruct/defrecord kwargs companion emits
                    // precisely `(do (structtype …) (defmacro …))`). skip(1) drops
                    // the `do` head; the all-defmacro case (items == [do])
                    // splices nothing — the same elision the empty-do check gave
                    // before.
                    //
                    // Arc 278 — a macro that GENERATES a service emits `(do
                    // (defsurface ... :messages [...]) (defservice :satisfies
                    // ...))`. A surviving child here may itself be a
                    // defsurface; splicing it up RAW (as any other child)
                    // leaves its `:messages` recordtype/defenum decls nested,
                    // never hoisted to top level -- the exact gap
                    // `hoist_surface_messages` exists to close for a DIRECT
                    // top-level defsurface (see `expand_all_with` above).
                    // Route each surviving child through the SAME dispatch
                    // `expand_all_with` uses: a defsurface child goes through
                    // `hoist_surface_messages` (its hoisted decls precede it,
                    // the surface form itself unchanged); every other child
                    // splices up raw, identical to before.
                    let mut spliced = Vec::with_capacity(items.len());
                    for child in items.into_iter().skip(1) {
                        if is_defsurface_form(&child) {
                            let (hoisted, surface_form) =
                                hoist_surface_messages(child, registry, privilege)?;
                            spliced.extend(hoisted);
                            spliced.push(surface_form);
                        } else {
                            spliced.push(child);
                        }
                    }
                    Ok(spliced)
                } else {
                    // `let` — keep the form wrapped. Its now-unwrapped, defmacro-
                    // free body (e.g. a bare `structtype` sibling flattened up
                    // from what used to be a nested `(do (structtype …)
                    // (defmacro …))`) is exactly what `register_types`'
                    // `splice_type_decls` / `register_defines`' do/let-body
                    // recursion (arc 170 slice 3 Gap J) already know how to walk
                    // and register downstream — no second registration path
                    // needed.
                    Ok(vec![WatAST::List(items, span)])
                }
            }
            other => Ok(vec![other]),
        }
    } else {
        Ok(vec![expanded])
    }
}

/// Arc 294 item 9a — walk a `(:wat::core::defsurface … :messages [ <children> ] …)`
/// form's `:messages` vector (if present) and hoist each child through
/// [`hoist_top_level_form`] (registering companion `defmacro`s, collecting the
/// surviving `recordtype`/`defenum` decls to splice to top level). Returns the
/// hoisted top-level forms and the surface form itself UNCHANGED (its `:messages`
/// vector still carries the original post-expansion children — the carrier
/// re-hoists them the same way in the forked child). A no-`:messages` surface
/// (non-peer natures) yields no hoisted forms, a no-op.
fn hoist_surface_messages(
    surface_form: WatAST,
    registry: &mut MacroRegistry,
    privilege: crate::resolve::Privilege,
) -> Result<(Vec<WatAST>, WatAST), MacroError> {
    let mut hoisted = Vec::new();
    if let WatAST::List(items, _) = &surface_form {
        let mut it = items.iter();
        while let Some(node) = it.next() {
            if let WatAST::Keyword(k, _) = node {
                if k == ":messages" {
                    if let Some(WatAST::Vector(msgs, _)) = it.next() {
                        for child in msgs.iter().cloned() {
                            hoisted.extend(hoist_top_level_form(child, registry, privilege)?);
                        }
                    }
                    break;
                }
            }
        }
    }
    Ok((hoisted, surface_form))
}

/// The body-start index for a do/let container: a `do`'s body begins right
/// after its head keyword (index 1); a `let`'s body begins after its head AND
/// its bindings vector (index 2). `None` for anything else. Shared by
/// [`is_do_or_let_containing_defmacro`] and [`hoist_defmacros_from_container`]
/// so the two can never drift on which items are "head/bindings" vs "body".
fn container_body_start(head: &str) -> Option<usize> {
    match head {
        ":wat::core::do" => Some(1),
        ":wat::core::let" => Some(2),
        _ => None,
    }
}

/// Returns `true` if `form` is a `(:wat::core::do ...)` or `(:wat::core::let
/// [...] ...)` form that contains a `(:wat::core::defmacro ...)` at ANY
/// do/let-nesting depth (in the BODY only — a `let`'s bindings vector is never
/// walked). Used by `expand_all` to detect macro-generating-macros (e.g.
/// `defn`'s kwargs branch, or a `defstruct`/`defrecord` invocation) that emit
/// their `defmacro` registration inside a do/let wrapper.
///
/// Recurses through nested do/lets: a macro that emits a macro that emits a
/// macro (e.g. `defservice` → a kwargs `defn` → its companion `defmacro`, or a
/// `defstruct` invocation sitting in a top-level `let` body → its own `(do
/// (structtype …) (defmacro …))` companion) nests the `defmacro` further down.
/// The check is depth-unbounded — it does not count levels (one rung → a
/// fixpoint), so self-emission composes at any nesting, in any do/let mix.
fn is_do_or_let_containing_defmacro(form: &WatAST) -> bool {
    if let WatAST::List(items, _) = form {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if let Some(body_start) = container_body_start(head) {
                return items
                    .iter()
                    .skip(body_start)
                    .any(|child| is_defmacro_form(child) || is_do_or_let_containing_defmacro(child));
            }
        }
    }
    false
}

/// Walk a `(:wat::core::do ...)` / `(:wat::core::let [...] ...)` form,
/// registering any `defmacro` at ANY do/let-nesting depth and stripping it
/// from the body; non-defmacro children are kept in order. Returns the
/// rebuilt form (same head — `do` stays `do`, `let` stays `let` with its
/// bindings vector untouched). Called only when
/// `is_do_or_let_containing_defmacro` returns true.
///
/// Recurses into nested do/let children that themselves contain a `defmacro`
/// (the macros-emitting-macros-emitting-macros case, or a `defstruct` call
/// buried in a nested `let`): a child `(do … (defmacro …))` / `(let […] …
/// (defmacro …))` is rebuilt by hoisting from it too, so a `defmacro` born any
/// number of macro-emission hops deep still registers. Depth-unbounded by
/// construction.
///
/// Flatten policy differs by the CHILD's own kind, not the parent's: a
/// surviving `do` child is flattened — spliced directly into the parent's
/// body — because a `do` introduces no bindings, so its children are
/// semantically identical whether nested or hoisted flat. A surviving `let`
/// child is kept WRAPPED — its body may read the let's bound variables, and
/// splicing would drop them out of scope — so it's pushed back as a single
/// (now defmacro-free) `let` form. This mirrors `register_types`'
/// `splice_type_decls` (arc 170 slice 3 Gap J), which applies the identical
/// do-flattens/let-wraps distinction one stage later, for type decls instead
/// of macro registrations.
fn hoist_defmacros_from_container(
    form: WatAST,
    registry: &mut MacroRegistry,
    privilege: crate::resolve::Privilege,
) -> Result<WatAST, MacroError> {
    let (items, span) = match form {
        WatAST::List(items, span) => (items, span),
        other => return Ok(other), // guard: caller guarantees it's a List
    };
    let body_start = match items.first() {
        Some(WatAST::Keyword(head, _)) => match container_body_start(head) {
            Some(n) => n,
            None => return Ok(WatAST::List(items, span)), // guard: not a do/let
        },
        _ => return Ok(WatAST::List(items, span)),
    };
    let mut new_items = Vec::with_capacity(items.len());
    let mut iter = items.into_iter();
    // Keep the head keyword (and, for `let`, the bindings vector) untouched.
    for _ in 0..body_start {
        if let Some(head_or_bindings) = iter.next() {
            new_items.push(head_or_bindings);
        }
    }
    for child in iter {
        if is_defmacro_form(&child) {
            let def = parse_defmacro_form(child)?;
            registry.register(def, privilege)?;
        } else if is_do_or_let_containing_defmacro(&child) {
            // A defmacro nested in a child do/let (a macro emitted by a macro
            // emitted by a macro, or a defstruct/defrecord invocation sitting
            // in a nested let body). Recurse to register the nested defmacro,
            // then apply the do-flattens/let-wraps policy documented above.
            let rebuilt = hoist_defmacros_from_container(child, registry, privilege)?;
            let is_do = matches!(
                &rebuilt,
                WatAST::List(inner, _) if matches!(inner.first(), Some(WatAST::Keyword(h, _)) if h == ":wat::core::do")
            );
            match rebuilt {
                WatAST::List(inner, _) if is_do => new_items.extend(inner.into_iter().skip(1)),
                other => new_items.push(other),
            }
        } else {
            new_items.push(child);
        }
    }
    Ok(WatAST::List(new_items, span))
}

/// One macro-expansion step. Arc 030 — the core of
/// `:wat::core::macroexpand-1`. If `form` is a macro call (list
/// whose head is a registered macro keyword), apply the macro's
/// template with the call-site args and return the result. If
/// `form` is not a macro call, return it unchanged.
///
/// Unlike [`expand::expand_form`], does NOT recurse into children and does
/// NOT fixpoint. Matches Common Lisp / Clojure `macroexpand-1`.
pub fn expand_once(
    form: WatAST,
    registry: &MacroRegistry,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    if let WatAST::List(items, span) = &form {
        if let Some(WatAST::Keyword(head, head_span)) = items.first() {
            if let Some(def) = registry.get(head) {
                let head_span = head_span.clone();
                let args = items[1..].to_vec();
                return expand_macro_call(def, args, span.clone(), head_span, env, sym);
            }
        }
    }
    Ok(form)
}

/// Fully expand a single form to fixpoint with an IMMUTABLE registry — the eval-time
/// READ→EXPAND→EVAL step (`eval_in_frozen` and the boot/machinery source-eval sites).
///
/// Unlike [`expand_all`] (the startup pass, which registers defmacros through a `&mut`
/// registry), this expands an *expression for evaluation*: the CALLER's registry must not
/// change — `eval_in_frozen` holds a `&FrozenWorld`, and an eval-time form that mints a
/// macro is refused by `refuse_mutation_forms` on the expansion anyway. So the caller's
/// registry is borrowed shared and a THROWAWAY clone absorbs any expansion-time
/// registration (arc 294 item 9a: `expand_form` now registers a `do`/`let` body's
/// `defmacro` children as it walks, so a sibling can call them — see its doc). The clone
/// dies with the call; the frozen world is untouched.
///
/// Recurses into children and fixpoints via [`expand_form`] (full-Lisp raw-args
/// semantics). This is what lets a source-written kwargs construction — even one inside
/// a Rust string literal handed to `eval_in_frozen` — expand and evaluate; the prime
/// `:T'` is then needed only in *generated* code (macro output), never in written source.
pub fn expand_fully(
    form: WatAST,
    registry: &MacroRegistry,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    let mut scratch = registry.clone();
    expand_form(form, &mut scratch, 0, env, sym, crate::resolve::Privilege::User)
}

/// Expand a single form. Recursively expands children, then checks
/// whether the resulting node is itself a macro call; if so, expand it,
/// and continue to fixpoint.
///
/// Arc 294 item 9a — **registration is SEQUENTIAL during expansion**. The engine's
/// top-level promise (`expand_all`'s header: "Register each such form as it appears so
/// subsequent forms in the stream can invoke the new macro") holds inside a `do`/`let`
/// BODY too: a body child is expanded, any `defmacro` it *is* registers immediately, and
/// only then is the next sibling expanded. Without this, a macro emitting ONE container
/// that both mints a companion `defmacro` and USES it (a `defservice`'s `do`: its
/// `::Record`/`::State` companions plus the `serve` defn whose handlers construct those
/// types) left the construction RAW — the companion had not registered when the sibling
/// was walked — and it died at eval with `UnknownFunction`. Sequential registration is
/// depth-unbounded by recursion: a `defmacro` nested in a child `do`/`let` registers when
/// THAT container's own body-walk reaches it.
///
/// The registration is register-ONLY: the `defmacro` form stays in the tree. Stripping and
/// splicing remain [`hoist_top_level_form`]'s job, which runs after the whole form is
/// expanded and needs to still SEE the `defmacro` to know the container is a registration
/// wrapper. Its re-`register` of the same def is a no-op (arc 054 structural equivalence).
pub(super) fn expand_form(
    form: WatAST,
    registry: &mut MacroRegistry,
    expansion_depth: usize,
    env: &Environment,
    sym: &SymbolTable,
    privilege: crate::resolve::Privilege,
) -> Result<WatAST, MacroError> {
    if expansion_depth >= EXPANSION_DEPTH_LIMIT {
        return Err(MacroError {
            span: form.span().clone(), // Pattern B: the form being expanded
            kind: MacroErrorKind::ExpansionDepthExceeded { limit: EXPANSION_DEPTH_LIMIT },
        });
    }

    match form {
        WatAST::List(items, list_span) => {
            // Data forms — NOT expanded. quote/forms/literal (`Boundary::AllData`) and
            // quasiquote (`Boundary::Quasiquote`) carry DATA, not code; recursing would
            // eagerly expand macro calls the caller means to observe or template, not
            // execute (arc 029/030; the macroexpand primitives rely on it). Reuses
            // `resolve::boundary`'s ALREADY-established classification — so this doesn't
            // drift into a second, hand-rolled copy of the same language fact (arc 278: this
            // exact set had drifted, omitting `:wat::core::forms`, so a `forms` block's
            // arguments — data for another world — were macro-expanded in the parent's).
            // Both variants are named on purpose: `quasiquote` classifies as
            // `Boundary::Quasiquote`, not `AllData`, and its behaviour must not change.
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                if matches!(crate::resolve::boundary::quote_boundary(head), crate::resolve::boundary::Boundary::AllData | crate::resolve::boundary::Boundary::Quasiquote) {
                    return Ok(WatAST::List(items, list_span));
                }
            }

            // `:wat::form::matches?` — substrate special form, never a registered macro
            // itself. Only the subject (items[1]) is code; the pattern (items[2..]) is
            // DSL data owned by check.rs's `infer_form_matches` grammar walker (arc 098).
            // Reuses `resolve::boundary`'s ALREADY-established classification
            // (`Boundary::MatchesSubject`, consulted by the resolve-time `walk`/`normalize`
            // passes) so this doesn't drift into a second, hand-rolled copy of the same
            // language fact. Without this, full-Lisp child-recursion (below) walks into
            // the pattern and — post arc 294 item 9a's construction flip — finds an
            // aggregate-shaped pattern head (e.g. `:test::PaperResolved`) that is now a
            // registered kwargs companion macro, firing `kwargs-lower` on raw DSL clauses
            // as if they were kv-pairs.
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                if matches!(crate::resolve::boundary::quote_boundary(head), crate::resolve::boundary::Boundary::MatchesSubject) {
                    let mut iter = items.into_iter();
                    let mut new_items = Vec::with_capacity(2);
                    new_items.push(iter.next().expect("head keyword just matched"));
                    if let Some(subject) = iter.next() {
                        new_items.push(expand_form(subject, registry, expansion_depth + 1, env, sym, privilege)?);
                    }
                    new_items.extend(iter); // pattern (items[2..]) — DSL data, untouched
                    return Ok(WatAST::List(new_items, list_span));
                }
            }

            // `:wat::rete::make-rule` (arc 278 task #78 —
            // DESIGN-STONE-where-bodies-expand-at-compile-time.md). A `where` body was
            // previously NEVER macro-expanded: `defrule` quotes its `:when` vector verbatim,
            // and the quote-family check above returns a quoted form untouched — so a macro
            // (e.g. rete-spelled `cond`) used inside a `(:wat::rete::where …)` clause reached
            // `eval_test_core`'s raw `eval_inner` call unexpanded and died with
            // `UnknownFunction`. `make-rule` is the ONE door every rule producer funnels
            // through (`defrule`'s template, `sift-rules-defsvc`'s generator, hand-built rule
            // literals, direct calls) — hooking `defrule` alone would silently miss the other
            // three. Reuses `resolve::boundary`'s classification (`Boundary::MakeRule`,
            // shared with the resolve-time `walk`/`normalize` passes) so this doesn't drift
            // into a second, hand-rolled copy — mirrors the `MatchesSubject` handling
            // immediately above for the identical hazard: expand a condition PATTERN as code
            // (STOP-2) and its aggregate-shaped head — a registered kwargs companion macro
            // post arc-294 item 9a — fires `kwargs-lower` on raw DSL clauses.
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                if matches!(crate::resolve::boundary::quote_boundary(head), crate::resolve::boundary::Boundary::MakeRule) {
                    return expand_make_rule(items, list_span, registry, expansion_depth, env, sym, privilege);
                }
            }

            // ── Full-Lisp macro dispatch (arc 294 item 9a): a macro receives its args RAW.
            // Standard homoiconic semantics: if the head names a registered macro, expand
            // THIS call with the caller's *unexpanded* arg forms, then re-expand the macro
            // OUTPUT to fixpoint (the `return expand_form(...)`). Arg forms that flow into
            // the output as code get expanded there, in the caller's context — identical to
            // the old children-first result for ordinary code. What differs: args a macro
            // QUOTES or treats as DATA (rete `defrule` patterns; any user DSL) reach it
            // untouched, because we no longer eagerly expand them before the macro fires.
            // The output fixpoint IS the macro engine; dropping the extra input-expansion
            // pass makes wat a full Lisp — and needs NO "these args are data" allowlist:
            // user macros get the same semantics for free (no built-in form is blessed).
            //
            // Pre-flip this deviation was invisible — type-keywords were FUNCTIONS, which
            // the eager pass leaves alone; the construction flip turned every aggregate
            // type-keyword into a MACRO, exposing (and here removing) it.
            //
            // Arc 109 stone "a type reference is not an expression": that exposure has a
            // second face. `(:user::R :- [T])` is a TYPE REFERENCE — `R` a `defrecord`-minted
            // name — but this dispatch only checked the HEAD, so it fired R's registered
            // kwargs companion (arc 294 item 9a, below) and expanded the whole form into
            // `(:wat::core::kwargs-construct :user::R :- [T])`, a CONSTRUCTOR CALL that then
            // lands in a type slot and fails with a diagnostic blaming the binder vector
            // rather than naming the real defect. A form whose element 1 is the `:-` binder
            // marker (`types::is_binder_marker` — KEYWORD, never Symbol; matching Symbol here
            // silently never fires) can never be a value expression: `:-`'s param-spec sits
            // in a RESERVED position, so nothing needs to sniff it, by exactly the same
            // reasoning the binder-marker doctrine already relies on. This is a SHAPE test,
            // not a per-head slot list — `(Head :- [args])` carries the marker at index 1,
            // while a DECLARATION (`(defn :name :- [T] …)`) carries it at index 2, so the two
            // are distinguishable without knowing any head's grammar. It fixes every
            // macro-minted type reference at once (`defrecord`, `defstruct`,
            // `holon::defrecord`, and any future companion) rather than the ones known today.
            // The companion macro itself is UNTOUCHED and still fires for the kwargs
            // constructor call `(:user::R :field v)` — that shape has NO `:-` at index 1, so
            // it never reaches this guard and is not this stone's concern.
            //
            // STONE-finish-the-param-spec (arc 109) — refines the shape test above.
            // `items.get(1).is_some_and(is_binder_marker)` alone cannot tell a bare TYPE
            // REFERENCE (`(:user::R :- [T])`, nothing after the bracket) from a
            // PARAMETERIZED CONSTRUCTOR CALL (`(:user::R :- [T] :field v)`, real args
            // after it) — both carry the marker at index 1. Only the FORMER is a type
            // reference; the LATTER is exactly the value-application position 3 this
            // stone teaches. `peel_param_spec` (the one door, `types.rs`) tells them
            // apart: `is_type_reference` now requires the peeled REST to be empty too.
            // When the rest is non-empty, the marker+bracket are dropped from the args
            // handed to the companion macro — `(:user::R :- [T] :field v)` reaches the
            // SAME kwargs-construct path `(:user::R :field v)` already does, and T is
            // bound from the field VALUE exactly as the unmarked call already binds it
            // (mirrors A's exemplar route: peel through the door, then let the existing,
            // unmarked-form machinery do the rest — not a second binding mechanism).
            //
            // Arc 294 item 9a (sequential registration): the `contains` probe + scoped
            // `get` keep the `&MacroDef` borrow alive only until `expand_macro_call`
            // returns the OWNED expansion; the registry is then free to be re-borrowed
            // `&mut` for the output fixpoint (which may register the expansion's own
            // `defmacro` children). Cheaper than cloning the MacroDef on every call.
            if let Some(WatAST::Keyword(head, head_span)) = items.first() {
                let (type_args, rest_after_marker) = crate::types::peel_param_spec(&items[1..]);
                let is_type_reference = type_args.is_some() && rest_after_marker.is_empty();
                if registry.contains(head) && !is_type_reference {
                    let head_span = head_span.clone();
                    let args = rest_after_marker.to_vec();
                    let expanded = {
                        let def = registry.get(head).expect("contains checked immediately above");
                        expand_macro_call(def, args, list_span.clone(), head_span, env, sym)?
                    };
                    return expand_form(expanded, registry, expansion_depth + 1, env, sym, privilege);
                }
            }

            // Arc 300.1 — faithful-Clojure dual surface: a namespaced Symbol head
            // (`wat.core/defn`) dispatches exactly as its keyword FQDN (`:wat::core::defn`)
            // would, with the same RAW-args semantics. A `/`-bearing symbol that is NOT a
            // macro falls through to the child-recursion (its call-position ref rewriting
            // happens downstream). The macro's own body ignores the head token, so no head
            // rewrite is needed here — only the raw tail args flow in.
            if let Some(WatAST::Symbol(ident, ident_span)) = items.first() {
                if ident.is_reference() {
                    let head_span = ident_span.clone();
                    let primary = crate::edn_shim::ns_to_wat_path(ident.receiver(), ident.method());
                    if registry.contains(&primary) {
                        let args = items[1..].to_vec();
                        let expanded = {
                            let def = registry.get(&primary).expect("contains checked immediately above");
                            expand_macro_call(def, args, list_span.clone(), head_span, env, sym)?
                        };
                        return expand_form(expanded, registry, expansion_depth + 1, env, sym, privilege);
                    }
                }
            }

            // Arc 294 item 9a — a `do`/`let` BODY is a SEQUENCE, not a set: its children
            // are walked one at a time, each child's own `defmacro` registering before the
            // NEXT sibling is expanded (see this fn's doc for why). `container_body_start`
            // is the shared head/bindings-vs-body fact (`do` → 1, `let` → 2), so this walk
            // can never drift from `is_do_or_let_containing_defmacro` /
            // `hoist_defmacros_from_container` on which items are body. The head keyword
            // and (for `let`) the bindings vector are expanded exactly as the plain
            // child-walk below would — only the BODY tail is order-sensitive.
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                if let Some(body_start) = container_body_start(head) {
                    let mut out = Vec::with_capacity(items.len());
                    let mut iter = items.into_iter();
                    for _ in 0..body_start {
                        match iter.next() {
                            Some(head_or_bindings) => out.push(expand_form(
                                head_or_bindings, registry, expansion_depth + 1, env, sym, privilege,
                            )?),
                            // A `let` shorter than its own head+bindings is malformed; leave
                            // it to the checker's diagnostic rather than inventing one here.
                            None => break,
                        }
                    }
                    for child in iter {
                        let expanded = expand_form(child, registry, expansion_depth + 1, env, sym, privilege)?;
                        if is_defmacro_form(&expanded) {
                            // The ONE registration path (`parse_defmacro_form` → `register`),
                            // the same pair `hoist_top_level_form` uses. Register-only: the
                            // form stays in `out` for the hoist pass to strip/splice.
                            registry.register(parse_defmacro_form(expanded.clone())?, privilege)?;
                        }
                        out.push(expanded);
                    }
                    return Ok(WatAST::List(out, list_span));
                }
            }

            // NOT a macro call — recurse into children so nested macros in ordinary code
            // (function-call args, let bindings, vector/map elements) still expand. A
            // non-macro head is a function or special form; its sub-forms are code.
            let expanded_children: super::ExpandBatch = items
                .into_iter()
                .map(|c| expand_form(c, registry, expansion_depth + 1, env, sym, privilege))
                .collect();
            let expanded_children = expanded_children?;
            Ok(WatAST::List(expanded_children, list_span))
        }
        // Arc 167 slice 1 — recurse into vector children so a
        // macro call buried inside a fn-sig vector (slice 2
        // territory) still expands. Vectors carry no head-keyword
        // dispatch, so the macro-call detection arm at the head
        // doesn't apply.
        WatAST::Vector(items, vec_span) => {
            let expanded_children: super::ExpandBatch = items
                .into_iter()
                .map(|c| expand_form(c, registry, expansion_depth + 1, env, sym, privilege))
                .collect();
            Ok(WatAST::Vector(expanded_children?, vec_span))
        }
        other => Ok(other),
    }
}

/// Expand a `:wat::rete::make-rule` call (arc 278 task #78). `items[1]` (rule
/// name) is ordinary code, expanded normally. `items[2]` (the quoted `:when`
/// vector) is data EXCEPT the body of each `(:wat::rete::where …)` form inside
/// it — see [`expand_make_rule_when`]. `items[3..]` (the quoted `:then` vector
/// and any trailing args) pass through byte-identical: the RHS is a separate
/// question (task #61 already ruled derived fact fields are copies only).
fn expand_make_rule(
    items: Vec<WatAST>,
    list_span: crate::span::Span,
    registry: &mut MacroRegistry,
    expansion_depth: usize,
    env: &Environment,
    sym: &SymbolTable,
    privilege: crate::resolve::Privilege,
) -> Result<WatAST, MacroError> {
    let mut iter = items.into_iter();
    let mut out = Vec::with_capacity(4);
    out.extend(iter.next()); // make-rule head, as-is
    // items[1]: rule name — ordinary code.
    if let Some(name) = iter.next() {
        out.push(expand_form(name, registry, expansion_depth + 1, env, sym, privilege)?);
    }
    // items[2]: quoted :when vector — expand only each where-form's body.
    if let Some(when_arg) = iter.next() {
        out.push(expand_make_rule_when(when_arg, registry, expansion_depth + 1, env, sym, privilege)?);
    }
    // items[3..]: quoted :then vector + any trailing args — untouched data.
    out.extend(iter);
    Ok(WatAST::List(out, list_span))
}

/// Expand a `make-rule` call's `:when` argument. Expected shape
/// `(:wat::core::quote [<condition>...])` — every measured producer quotes
/// the `:when` vector this way (see `expand_make_rule`'s doc and
/// `resolve::boundary::Boundary::MakeRule`'s doc for the census). A
/// `when_arg` NOT shaped like a literal quote — a computed `:wat::WatAST`
/// expression with no syntactic vector to search for `where` forms in — is
/// returned untouched: conservative by construction, same discipline
/// `expand_form`'s `MatchesSubject` arm above uses for a shape it doesn't
/// recognize.
fn expand_make_rule_when(
    when_arg: WatAST,
    registry: &mut MacroRegistry,
    expansion_depth: usize,
    env: &Environment,
    sym: &SymbolTable,
    privilege: crate::resolve::Privilege,
) -> Result<WatAST, MacroError> {
    let WatAST::List(qitems, qspan) = when_arg else { return Ok(when_arg) };
    let is_quote = matches!(qitems.first(), Some(WatAST::Keyword(h, _)) if h == ":wat::core::quote");
    if !is_quote {
        return Ok(WatAST::List(qitems, qspan));
    }
    let mut qiter = qitems.into_iter();
    let mut new_q = Vec::with_capacity(2);
    new_q.extend(qiter.next()); // quote head, as-is
    if let Some(vec_node) = qiter.next() {
        new_q.push(expand_make_rule_conditions(vec_node, registry, expansion_depth + 1, env, sym, privilege)?);
    }
    new_q.extend(qiter); // shouldn't appear in a well-formed quote; conservative
    Ok(WatAST::List(new_q, qspan))
}

/// Expand the condition vector inside a `make-rule`'s quoted `:when` arg —
/// per-element dispatch to [`expand_make_rule_condition`].
fn expand_make_rule_conditions(
    vec_node: WatAST,
    registry: &mut MacroRegistry,
    expansion_depth: usize,
    env: &Environment,
    sym: &SymbolTable,
    privilege: crate::resolve::Privilege,
) -> Result<WatAST, MacroError> {
    let WatAST::Vector(conds, vspan) = vec_node else { return Ok(vec_node) };
    let mut new_conds = Vec::with_capacity(conds.len());
    for cond in conds {
        new_conds.push(expand_make_rule_condition(cond, registry, expansion_depth + 1, env, sym, privilege)?);
    }
    Ok(WatAST::Vector(new_conds, vspan))
}

/// Expand one `:when` condition. A `(:wat::rete::where <body>...)` form's
/// body is code — expanded to fixpoint like any other code region (this is
/// the whole point: a macro like `cond` used inside a `where` is now visible
/// to the expander). Every other condition (a fact pattern — STOP-2:
/// aggregate-shaped head, a registered kwargs companion macro post arc-294
/// item 9a) is byte-identical, untouched.
fn expand_make_rule_condition(
    cond: WatAST,
    registry: &mut MacroRegistry,
    expansion_depth: usize,
    env: &Environment,
    sym: &SymbolTable,
    privilege: crate::resolve::Privilege,
) -> Result<WatAST, MacroError> {
    let WatAST::List(citems, cspan) = cond else { return Ok(cond) };
    let is_where = matches!(citems.first(), Some(WatAST::Keyword(h, _)) if crate::resolve::boundary::is_where_form(h));
    if !is_where {
        return Ok(WatAST::List(citems, cspan));
    }
    let mut citer = citems.into_iter();
    let mut new_c = Vec::with_capacity(citer.len().max(1));
    new_c.extend(citer.next()); // where head, as-is
    for body in citer {
        new_c.push(expand_form(body, registry, expansion_depth + 1, env, sym, privilege)?);
    }
    Ok(WatAST::List(new_c, cspan))
}

/// Expand a single macro call. Allocates a fresh [`ScopeId`], walks the
/// template substituting parameters with argument ASTs, adds the macro
/// scope to every template-origin symbol, returns the expansion.
///
/// Variadic macros (MacroDef with `rest_param: Some(_)`) accept
/// `args.len() >= params.len()`. The first N args bind positionally to
/// the fixed params; the rest are wrapped in a `WatAST::List` and
/// bound to the rest-name. The template's `,@rest-name` splice drops
/// those elements into the surrounding list context at expansion.
pub(super) fn expand_macro_call(
    def: &MacroDef,
    args: Vec<WatAST>,
    call_site_span: Span,
    head_span: Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    let fixed_arity = def.params.len();
    match &def.rest_param {
        None => {
            if args.len() != fixed_arity {
                return Err(MacroError {
                    span: call_site_span.clone(), // Pattern B: macro call-site span
                    kind: MacroErrorKind::ArityMismatch {
                        name: def.name.clone(),
                        expected: fixed_arity,
                        got: args.len(),
                    },
                });
            }
        }
        Some(_) => {
            if args.len() < fixed_arity {
                return Err(MacroError {
                    span: call_site_span.clone(), // Pattern B: macro call-site span
                    kind: MacroErrorKind::ArityTooFew {
                        name: def.name.clone(),
                        minimum: fixed_arity,
                        got: args.len(),
                    },
                });
            }
        }
    }

    let mut bindings: HashMap<String, WatAST> = HashMap::new();
    let mut iter = args.into_iter();
    for param in &def.params {
        bindings.insert(
            param.clone(),
            iter.next().expect("arity checked above"),
        );
    }
    if let Some(rest_name) = &def.rest_param {
        let rest: Vec<WatAST> = iter.collect();
        // Rest-list wrapper inherits the call-site span — the
        // `,@rest` splice drops these into the template's
        // surrounding context.
        bindings.insert(rest_name.clone(), WatAST::List(rest, call_site_span.clone()));
    }

    // rune:sequi(host-idiom) — fresh_scope() draws a process-global AtomicU64
    // monotonic counter (identifier.rs); the ScopeId it returns is threaded
    // explicitly downstream (expand_template → walk_template). The counter
    // carries no domain state, only global hygiene-scope uniqueness; threading
    // a mutable counter through the whole expansion tree would expose it
    // through every caller's signature.
    let macro_scope = fresh_scope();
    // Arc 278 §4 — push this invocation's call-site span for the duration of
    // its expansion so `:wat::kernel::macro-call-site` (runtime.rs, gated
    // pure-total via macros/eval.rs) can read it regardless of which
    // downstream path evaluates the macro body: the bare-quasiquote path
    // (`expand_quasiquote_body` → `walk_template` → `unquote_argument` →
    // `macro_eval`) or the program-body path (`expand_program_body` →
    // `macro_eval_pre_validated`). Both are reached only through
    // `expand_template` below, so pushing once here (RAII pop on return)
    // covers both uniformly and matches "one push per macro invocation."
    let _mcs = crate::value::MacroCallSiteGuard::push(call_site_span.clone(), def.name.clone());
    let expanded = expand_template(&def.body, &bindings, macro_scope, &def.name, &call_site_span, def.rest_param.as_deref(), env, sym)?;
    // Arc 170: a macro that rewrites a user's call (e.g. kwargs-lower rewriting
    // `(svc/start …)` into `(svc/start$impl …)`) must not leave the template's OWN
    // file/line as the only frame a user-facing failure can report. `restamp_unknown_spans`
    // rewalks the expansion and repoints any node whose span still names a *different* file
    // than the call site (i.e. it came from the macro's own definition, not from the user's
    // spliced arguments) at the call site.
    //
    // Arc 233 / 170 / 167 (198 span-fix rider): a diagnostic span anchors to the NARROWEST
    // user-source node that IS the offence, never to its enclosing form and never to
    // synthetic expansion output. Most restamped nodes have no narrower user-source
    // counterpart than "the whole call", so `call_site_span` (the invocation's full list
    // span) remains their fallback. The one exception this function knows about: a
    // synthesized Keyword node whose VALUE equals the macro's own dispatch name (`def.name`)
    // — e.g. a companion ctor macro rebuilding its type keyword via a runtime
    // `keyword-node` call (`aggregate_kwargs_companion_source`, src/macros/parse.rs) — is,
    // by construction, always a re-mint of the exact token the user wrote as the call's
    // head. For that one case `head_span` (the real span of `items.first()` at the call
    // site, captured before dispatch) is the narrower, still-genuinely-user-source anchor.
    // This is an identity check against the exact string used to look up THIS expansion,
    // not a spelling-based inference of provenance (contrast the recurring
    // `ends_with("'")`-style class this repo already knows to be wrong).
    Ok(restamp_unknown_spans(expanded, &call_site_span, &head_span, &def.name))
}

/// Arc 170: repoint every node in `form` whose span's file differs from
/// `call_site`'s file at `call_site` itself.
///
/// A macro expansion is a mix of two kinds of nodes: template forms parsed from the
/// macro's OWN definition file (e.g. `wat/core.wat`), and nodes spliced in from the
/// user's arguments (`~`/`~@`) which already carry the user's real, precise spans. The
/// former misattribute any failure inside them to the macro's definition site instead of
/// the call the user actually wrote; the latter are already correct and more precise than
/// the call site (e.g. a specific arg's line) and must be left alone. Comparing `file`
/// (not the full span) is what distinguishes the two: a spliced argument lives in the
/// caller's file already, a template form lives in the macro's file.
///
/// LIMITATION (write this down, do not let it be rediscovered): a macro DEFINED and
/// USED in the same file is invisible to this check — `form`'s template-node spans and
/// `call_site`'s span share a `file`, so no node is restamped. That is acceptable: the
/// file is already correct in that case, and only the line/col may still point at the
/// macro's definition rather than the specific call, which is a strictly smaller defect
/// than the cross-file case this fixes (the file being entirely absent from the stack).
///
/// SECOND LIMITATION, discovered empirically (arc 170) and load-bearing for why this
/// function does NOT recurse into a nested `(:wat::core::defmacro ...)` node's own
/// children: kwargs-style `defn` expands into a `do` block that, among other things,
/// *emits a companion `defmacro`* whose body is itself an unexpanded template (a thin
/// forwarder to `:wat::core::kwargs-lower`). That nested body is DATA, not code executed
/// as part of `defn`'s own expansion — it becomes the companion macro's `MacroDef.body`,
/// to be walked and restamped again, fresh, against the companion macro's OWN call site
/// the next time a user invokes it. If this function recursed into it here, the
/// template's `wat/core.wat` spans would get restamped to `defn`'s call site (the
/// `defn` invocation's line) — landing in the right FILE (defn and its companion macro
/// share a file, the user's) but the WRONG LINE (the `defn` line, not the actual call
/// site of the kwargs fn). Confirmed empirically: without this exclusion,
/// `probe-call-site-kwargs.wat`'s kwargs frame reported the probe file at the `defn`
/// line; with it, it reports the probe file at the actual call line. So: restamp a
/// nested `defmacro` form's own span (it's still a real node in this expansion), but
/// leave its subtree untouched — it belongs to a future, separate restamp pass.
///
/// Exhaustive over every `WatAST` variant on purpose — no `_ =>` catch-all — so a new
/// variant added later fails to compile here instead of silently passing through
/// unrestamped.
fn restamp_unknown_spans(form: WatAST, call_site: &Span, head_span: &Span, head_name: &str) -> WatAST {
    fn restamp_span(span: &Span, call_site: &Span) -> Span {
        if span.file != call_site.file {
            call_site.clone()
        } else {
            span.clone()
        }
    }

    match form {
        WatAST::IntLit(v, s) => WatAST::IntLit(v, restamp_span(&s, call_site)),
        WatAST::FloatLit(v, s) => WatAST::FloatLit(v, restamp_span(&s, call_site)),
        WatAST::RationalLit(v, s) => WatAST::RationalLit(v, restamp_span(&s, call_site)),
        WatAST::BigIntLit(v, s) => WatAST::BigIntLit(v, restamp_span(&s, call_site)),
        WatAST::BoolLit(v, s) => WatAST::BoolLit(v, restamp_span(&s, call_site)),
        WatAST::StringLit(v, s) => WatAST::StringLit(v, restamp_span(&s, call_site)),
        WatAST::NilLit(s) => WatAST::NilLit(restamp_span(&s, call_site)),
        // Arc 233 / 170 / 167 (198 span-fix rider): a Keyword node whose value IS the
        // macro's own dispatch name (`head_name`) is, by construction, a re-mint of the
        // exact token the user wrote as the call's head — see the doc comment at the call
        // site in `expand_macro_call`. It gets the narrower `head_span` instead of the
        // whole-call `call_site`; every other synthesized Keyword keeps the existing
        // whole-call fallback.
        WatAST::Keyword(v, s) => {
            if s.file != call_site.file && v == head_name {
                WatAST::Keyword(v, head_span.clone())
            } else {
                WatAST::Keyword(v, restamp_span(&s, call_site))
            }
        }
        WatAST::Symbol(v, s) => WatAST::Symbol(v, restamp_span(&s, call_site)),
        WatAST::List(items, s) => {
            let is_nested_defmacro = matches!(
                items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::defmacro"
            );
            if is_nested_defmacro {
                WatAST::List(items, restamp_span(&s, call_site))
            } else {
                WatAST::List(
                    items.into_iter().map(|c| restamp_unknown_spans(c, call_site, head_span, head_name)).collect(),
                    restamp_span(&s, call_site),
                )
            }
        }
        WatAST::Vector(items, s) => WatAST::Vector(
            items.into_iter().map(|c| restamp_unknown_spans(c, call_site, head_span, head_name)).collect(),
            restamp_span(&s, call_site),
        ),
        WatAST::Map(pairs, s) => WatAST::Map(
            pairs
                .into_iter()
                .map(|(k, v)| (
                    restamp_unknown_spans(k, call_site, head_span, head_name),
                    restamp_unknown_spans(v, call_site, head_span, head_name),
                ))
                .collect(),
            restamp_span(&s, call_site),
        ),
        WatAST::Set(items, s) => WatAST::Set(
            items.into_iter().map(|c| restamp_unknown_spans(c, call_site, head_span, head_name)).collect(),
            restamp_span(&s, call_site),
        ),
    }
}

/// Dispatcher: inspect the template's top-level shape once and route to the
/// appropriate expansion path.
///
/// The template's top-level form is either:
/// - a **bare quasiquote** `(:wat::core::quasiquote X)` — the existing hygienic
///   path via `expand_quasiquote_body` → `walk_template` (sets-of-scopes hygiene,
///   UNCHANGED).
/// - a **program body** (any other form) — the `macro_eval` path introduced in
///   arc 249 stone 249.2b-ii, handled by `expand_program_body`.
///
/// `rest_param` names the variadic rest parameter (if any); passed to
/// `expand_program_body` for correct value-binding semantics.
// All eight are the genuine macro-invocation context (template, bindings,
// scope, name, call-site span, rest-param, env, sym); none is removable
// (arc 249 struere). clippy's 7-arg threshold is a heuristic this
// hygiene-critical dispatcher doesn't fit.
// rune:struere(host-constraint) — all eight params are the genuine macro-invocation context; a context struct would cost more than it saves across the three walkers.
// NOTE: each branch uses a different subset: macro_scope → quasiquote path only; rest_param → program-body path only.
#[allow(clippy::too_many_arguments)]
fn expand_template(
    template: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_scope: ScopeId,
    macro_name: &str,
    call_site_span: &Span,
    rest_param: Option<&str>,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    // Head-only check: is this a quasiquote-headed form?
    // Using `is_quasiquote_form` (head-only) ensures a malformed
    // `(:wat::core::quasiquote a b)` (wrong arity) is caught here
    // rather than silently misrouting to the program-body path.
    if is_quasiquote_form(template) {
        // Quasiquote path: consumes `macro_scope` for sets-of-scopes hygiene tagging;
        // `rest_param` is unused here (rest-param semantics are baked into bindings by expand_macro_call).
        // Arity check: a quasiquote template must have exactly one body form.
        match template {
            WatAST::List(items, _) => match quasiquote_inner(items) {
                Some(body) => {
                    // Well-formed: `(:wat::core::quasiquote X)`.
                    expand_quasiquote_body(body, bindings, macro_scope, macro_name, call_site_span, env, sym)
                }
                None => {
                    // Quasiquote-headed but wrong arity (0 or ≥2 body forms).
                    Err(MacroError {
                        span: call_site_span.clone(),
                        kind: MacroErrorKind::MalformedTemplate {
                            reason: format!(
                                "macro {} — malformed quasiquote template: \
                                 (:wat::core::quasiquote ...) requires exactly one body form, \
                                 got {} element(s)",
                                macro_name,
                                items.len().saturating_sub(1),
                            ),
                        },
                    })
                }
            },
            _ => unreachable!("is_quasiquote_form guarantees List"),
        }
    } else {
        // Program-body path: consumes `rest_param` to bind the variadic rest as Value::Vec;
        // `macro_scope` is unused here (no sets-of-scopes tagging; hygiene enforced by Gate E).
        expand_program_body(template, bindings, macro_name, call_site_span, rest_param, env, sym)
    }
}

/// Existing hygienic path (bare quasiquote body).
///
/// `walk_template` adds sets-of-scopes hygiene to template-origin symbols.
/// Every existing stdlib macro goes through here.
fn expand_quasiquote_body(
    qb: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_scope: ScopeId,
    macro_name: &str,
    call_site_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    walk_template(qb, bindings, macro_scope, macro_name, call_site_span, 1, env, sym)
}

/// New program-body path (arc 249 stone 249.2b-ii).
///
/// Gate E — hygiene bound: refuse a program body whose quasiquote templates
/// introduce a literal name in a binder position (`:wat::core::let` or
/// `:wat::core::fn`). eval_quasiquote adds no hygiene scopes, so such a body
/// could silently capture caller-site names. Default-deny; emit
/// ProgramBodyIntroducesName so a future "allow" can't silently admit the
/// capturing case.
///
/// Params are bound as quoted form-values in a body env, `macro_eval`
/// evaluates the body, and the result is converted back to a `WatAST`.
///
/// `rest_param` names the variadic rest parameter (if any) so this path
/// can bind it as `Value::Vec([watast...])` rather than a `WatAST::List`
/// (which is the encoding used by the quasiquote-template path).
fn expand_program_body(
    template: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_name: &str,
    call_site_span: &Span,
    rest_param: Option<&str>,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    // check_program_body_hygiene was hoisted to parse_defmacro_form (definition time);
    // validate_pure_total on the template was also hoisted. Only per-call validations
    // of substituted/computed forms (in unquote_argument/splice_argument) remain here.

    // Build a body env with params bound as quoted form-values.
    //   - fixed param   → Value::wat__WatAST(Arc::new(arg_form))
    //   - rest param    → Value::Vec(Arc::new([wat__WatAST(arg0), ...]))
    // This is what lets `(foldl … nums)` fold over the arg-forms: each `n` is a
    // `wat__WatAST` value, and `` `(:wat::core::i64::+ ~acc ~n) `` → eval_quasiquote
    // evaluates `~n` → value_to_watast → the arg-form spliced in.
    let mut builder = env.child();
    for (name, ast_form) in bindings {
        if rest_param == Some(name.as_str()) {
            // The rest-param binding from expand_macro_call is WatAST::List(elems, _).
            // Convert each element to Value::wat__WatAST and collect into Value::Vec.
            let elems: &[WatAST] = match ast_form {
                WatAST::List(items, _) => items.as_slice(),
                // expand_macro_call always wraps rest args in WatAST::List; any other
                // shape means the caller violated the invariant.
                _ => unreachable!("rest-param binding is always WatAST::List per expand_macro_call"),
            };
            let vals: Vec<Value> = elems
                .iter()
                .map(|a| Value::wat__WatAST(Arc::new(a.clone())))
                .collect();
            builder = builder.bind_unknown_span(
                name.clone(),
                TrackedValue::from(Value::Vec(Arc::new(vals))),
            );
        } else {
            // Fixed param: bind as a quoted form-value (Value::wat__WatAST).
            builder = builder.bind_unknown_span(
                name.clone(),
                TrackedValue::from(Value::wat__WatAST(Arc::new(ast_form.clone()))),
            );
        }
    }
    let body_env = builder.build();

    // Evaluate the program body using the pre-validated path. The template is the
    // immutable definition body, already validated ONCE at definition time by
    // validate_macro_definition (the hoist — arc 249 stone O). Skipping re-validation
    // here is intentional; see macro_eval_pre_validated in eval.rs for the invariant.
    // Arc 296: surface structured cause chain instead of collapsing to prose.
    let result_tv = crate::macros::eval::macro_eval_pre_validated(template, &body_env, sym)
        .map_err(|e| MacroError {
            span: call_site_span.clone(),
            kind: MacroErrorKind::ProgramBodyEvalFailed {
                macro_name: macro_name.to_string(),
                cause: Box::new(e),
            },
        })?;

    // Convert the result Value to a WatAST expansion form.
    // value_to_watast handles: i64/f64/bool/String/keyword/nil literals, wat__WatAST (direct),
    // holon__HolonAST (via holon_to_watast). Other shapes (Struct/Enum/Vec/HashMap) error.
    crate::runtime::value_to_watast(
        &format!("macro {} body result", macro_name),
        result_tv.value_owned(),
        call_site_span.clone(),
    )
    .map_err(|e| MacroError {
        span: call_site_span.clone(),
        kind: MacroErrorKind::MalformedTemplate {
            reason: format!(
                "macro {} — program body result could not be converted to AST: {}",
                macro_name, e
            ),
        },
    })
}

// ─── Arc 249 Stone 249.2b-ii — Gate E: hygiene-bound check ──────────────────
//
// Refuse a program body whose quasiquote templates introduce a literal name
// in a binder position. eval_quasiquote adds no hygiene scopes, so a literal
// binder in a program-body quasiquote could capture caller-site names.
//
// Detection:
//   Walk the program body; when a `(:wat::core::quasiquote inner)` form is
//   found, walk `inner` to check for name-introducing binders:
//     - `(:wat::core::let [binder val ...] ...)` — binders at even indices
//       (0, 2, 4, …) of the binding vector; a literal `WatAST::Symbol` is refused.
//     - `(:wat::core::fn [name <- :T ...] ...)` — param names at positions
//       0, 3, 6, … (argspec triples) of the params vector; a literal
//       `WatAST::Symbol` is refused.
//
// A `~-unquote` in a template appears as `(:wat::core::unquote X)` (a List),
// NOT as a bare Symbol — so bare Symbol == literal name introduction.

/// Walk the program body looking for quasiquote sub-forms, then check each
/// quasiquote template for name-introducing binders.
pub(super) fn check_program_body_hygiene(
    body: &WatAST,
    call_site_span: &Span,
    macro_name: &str,
) -> Result<(), MacroError> {
    match body {
        WatAST::List(items, _) => {
            if let Some(inner) = quasiquote_inner(items) {
                // Entered a quasiquote template: check it for literal-name binders.
                check_quasiquote_for_literal_binders(inner, call_site_span, macro_name)?;
            } else {
                // Walk sub-forms recursively — quasiquote can appear at any depth in
                // the program body (e.g., inside an `if` branch or a `fn` body).
                for item in items {
                    check_program_body_hygiene(item, call_site_span, macro_name)?;
                }
            }
        }
        WatAST::Vector(items, _) => {
            for item in items {
                check_program_body_hygiene(item, call_site_span, macro_name)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Single delegating entry point for definition-time validation of a defmacro body.
///
/// Hoist: validation runs ONCE at definition time (called by `parse_defmacro_form`)
/// rather than per invocation, so a bad program-body macro fails at definition,
/// not silently at first use — arc 249 stone O.
///
/// Only applies to program-body templates (non-quasiquote bodies);
/// quasiquote bodies use the `walk_template` path which does not call
/// `validate_pure_total`. Callers must check `is_quasiquote_form` before
/// invoking this function (and skip it for quasiquote bodies).
///
/// Composes `check_program_body_hygiene` (Gate E: hygiene check, expand.rs) and
/// `validate_pure_total` (default-deny purity gate, eval.rs). Both are pure
/// predicates of the immutable body form; neither has side effects.
pub(super) fn validate_macro_definition(
    body: &WatAST,
    defmacro_span: &Span,
    macro_name: &str,
) -> Result<(), MacroError> {
    check_program_body_hygiene(body, defmacro_span, macro_name)?;
    super::eval::validate_pure_total(body).map_err(|e| MacroError {
        span: defmacro_span.clone(),
        kind: MacroErrorKind::MalformedDefmacro {
            reason: format!("program-body macro purity check failed at definition: {}", e.kind),
        },
    })
}

/// Returns `true` if `form` is a quasiquote-headed form (head-only check;
/// arity is NOT required to be 2). Used by both the parse-time discriminant
/// (is this a quasiquote template or a program body?) and the expand-time
/// path-router (`expand_template`). Keeping the head-only test shared ensures
/// that a malformed `(:wat::core::quasiquote a b)` (wrong arity) is treated
/// consistently at both sites instead of silently misrouting at expand time.
pub(super) fn is_quasiquote_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items, _)
            if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::quasiquote")
    )
}

/// If `items` is `(:wat::core::quasiquote X)` (exactly 2-element List with
/// quasiquote head), return `Some(&X)`. Otherwise `None`.
/// A 3-or-more element quasiquote-headed list returns `None` — the caller
/// (`expand_template`) then routes to `expand_program_body`, which will fail
/// with a meaningful `MalformedTemplate` error rather than silently
/// misrouting. Use `is_quasiquote_form` to test the head alone.
fn quasiquote_inner(items: &[WatAST]) -> Option<&WatAST> {
    if items.len() == 2 {
        if let Some(WatAST::Keyword(k, _)) = items.first() {
            if k == ":wat::core::quasiquote" {
                return items.get(1);
            }
        }
    }
    None
}

/// Walk a quasiquote template body; refuse if any `let`/`fn` form introduces
/// a literal name in binder position. Recurse into nested lists (but NOT into
/// nested quasiquotes — those would be inside a deeper quasiquote context and
/// their binders don't introduce names at this expansion level).
fn check_quasiquote_for_literal_binders(
    template: &WatAST,
    call_site_span: &Span,
    macro_name: &str,
) -> Result<(), MacroError> {
    if let WatAST::List(items, _) = template {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            // Stop recursion at nested quasiquotes — their content is data at this level.
            if head == ":wat::core::quasiquote" {
                return Ok(());
            }
            if head == ":wat::core::let" {
                // args[0] is the binding vector: [binder val binder val ...]
                // Binders are at even indices (0, 2, 4, …).
                // Non-Vector binder arm (items.get(1) is not a Vector): CORRECT pass-through —
                // a non-Vector binder introduces no names at this level; malformed let forms
                // get eval's own diagnostics when the expansion is evaluated.
                if let Some(WatAST::Vector(binder_items, _)) = items.get(1) {
                    let mut i = 0;
                    while i < binder_items.len() {
                        if let WatAST::Symbol(ident, _) = &binder_items[i] {
                            return Err(MacroError {
                                span: call_site_span.clone(),
                                kind: MacroErrorKind::ProgramBodyIntroducesName {
                                    macro_name: macro_name.to_string(),
                                    binder: ident.as_str().to_owned(),
                                },
                            });
                        }
                        i += 2;
                    }
                }
            } else if head == ":wat::core::fn" {
                // args[0] is the params vector: [name <- :T name <- :T ...]
                // Scan every position; a Symbol that is not a `->`/`<-`/`&`
                // marker is a param name being introduced. (Stepping by 1 +
                // marker-exclusion handles the `&`-rest case, which breaks the
                // positional triple-cadence.)
                // Non-Vector params arm (items.get(1) is not a Vector): CORRECT pass-through —
                // a non-Vector params form introduces no names at this level; malformed fn forms
                // get eval's own diagnostics when the expansion is evaluated.
                if let Some(WatAST::Vector(param_items, _)) = items.get(1) {
                    let mut i = 0;
                    while i < param_items.len() {
                        if let WatAST::Symbol(ident, _) = &param_items[i] {
                            let s = ident.as_str();
                            // Exclude `->` and `<-` and `&` markers.
                            if s != "->" && s != "<-" && s != "&" {
                                return Err(MacroError {
                                    span: call_site_span.clone(),
                                    kind: MacroErrorKind::ProgramBodyIntroducesName {
                                        macro_name: macro_name.to_string(),
                                        binder: s.to_owned(),
                                    },
                                });
                            }
                        }
                        i += 1;
                    }
                }
            }
        }
        // Recurse into sub-forms.
        for item in items {
            check_quasiquote_for_literal_binders(item, call_site_span, macro_name)?;
        }
    }
    Ok(())
}

/// Flatten a slice of template children (items of a List or Vector),
/// handling unquote-splicing inline. Returns the flat `Vec<WatAST>`
/// for the parent to wrap in its own container
/// (`WatAST::List` or `WatAST::Vector`).
///
/// Extracted from `walk_template`'s List and Vector arms, which are
/// identical except for the final constructor.
// All eight are the genuine macro-invocation context (items, bindings,
// scope, name, call-site span, depth, env, sym); none is removable
// (arc 249 struere). clippy's 7-arg threshold is a heuristic this
// hygiene-critical flatten helper doesn't fit.
// rune:struere(host-constraint) — all eight params are the genuine macro-invocation context; a context struct would cost more than it saves across the three walkers.
#[allow(clippy::too_many_arguments)]
fn flatten_template_children(
    items: &[WatAST],
    bindings: &HashMap<String, WatAST>,
    macro_scope: ScopeId,
    macro_name: &str,
    call_site_span: &Span,
    depth: u32,
    env: &Environment,
    sym: &SymbolTable,
) -> super::ExpandBatch {
    let mut out = Vec::with_capacity(items.len());
    for child in items {
        if let WatAST::List(child_items, _) = child {
            if let Some(splice_arg) =
                match_unquote(child_items, ":wat::core::unquote-splicing")
            {
                if depth == 1 {
                    let spliced =
                        splice_argument(splice_arg, bindings, macro_name, env, sym)?;
                    out.extend(spliced);
                    continue;
                } else {
                    // Preserve + peel: walk arg at depth-1,
                    // rebuild `(:wat::core::unquote-splicing ...)`.
                    let inner = walk_template(
                        splice_arg,
                        bindings,
                        macro_scope,
                        macro_name,
                        call_site_span,
                        depth - 1,
                        env,
                        sym,
                    )?;
                    out.push(WatAST::List(
                        vec![
                            WatAST::Keyword(
                                ":wat::core::unquote-splicing".into(),
                                call_site_span.clone(),
                            ),
                            inner,
                        ],
                        call_site_span.clone(),
                    ));
                    continue;
                }
            }
        }
        out.push(walk_template(
            child,
            bindings,
            macro_scope,
            macro_name,
            call_site_span,
            depth,
            env,
            sym,
        )?);
    }
    Ok(out)
}

/// Walk a quasiquoted form, expanding `,x` unquotes to their argument
/// ASTs, `,@x` unquote-splicing to their list elements, and tagging
/// every template-origin symbol with the macro scope.
///
/// rune:solvere(load-bearing-coupling) — qq depth-walk is mirrored in 3 sites
/// (walk_template / validate_quasiquote_template / walk_quasiquote); the depth
/// rule (nested +1, fire-at-depth-1, peel-deeper) is one contract that must
/// change in all three in sync; a unifying visitor would obscure three readable
/// single-purpose walkers.
///
/// Arc 016 slice 1: template-origin nodes (those built from the
/// defmacro's template, not from unquoted user args) inherit the
/// `call_site_span` — the span of the macro INVOCATION in user
/// source, not the template's span in the defmacro file. Matches
/// Racket's sets-of-scopes approach: when a user reads a failure
/// message, they want a pointer to their own code, not the
/// library's template.
///
/// Arc 029 slice 1: `depth` tracks how many layers of quasiquote
/// we're inside. Entry from `expand_template` is at depth 1 (the
/// outer `(:wat::core::quasiquote ...)` has just been stripped).
/// Encountering another `(:wat::core::quasiquote X)` in the template
/// bumps depth and preserves the wrapper. `(:wat::core::unquote X)`
/// at depth 1 substitutes; at depth > 1 it preserves the wrapper
/// and walks X at depth-1. Same discipline for
/// `(:wat::core::unquote-splicing X)`. This enables macro-
/// generating-macro patterns like `:wat::test::make-deftest` where
/// some unquotes fire at the outer expansion and others survive
/// for the inner macro's eventual expansion.
// All eight are the genuine macro-invocation context (template, bindings,
// scope, name, call-site span, depth, env, sym); none is removable
// (arc 249 struere). clippy's 7-arg threshold is a heuristic this
// hygiene-critical walker doesn't fit.
// rune:struere(host-constraint) — all eight params are the genuine macro-invocation context; a context struct would cost more than it saves across the three walkers.
#[allow(clippy::too_many_arguments)]
fn walk_template(
    form: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_scope: ScopeId,
    macro_name: &str,
    call_site_span: &Span,
    depth: u32,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    match form {
        WatAST::List(items, _) => {
            // Nested quasiquote — bump depth, preserve the wrapper.
            // Arc 029 slice 1.
            if let Some(arg) = match_unquote(items, ":wat::core::quasiquote") {
                let inner = walk_template(
                    arg,
                    bindings,
                    macro_scope,
                    macro_name,
                    call_site_span,
                    depth + 1,
                    env,
                    sym,
                )?;
                return Ok(WatAST::List(
                    vec![
                        WatAST::Keyword(
                            ":wat::core::quasiquote".into(),
                            call_site_span.clone(),
                        ),
                        inner,
                    ],
                    call_site_span.clone(),
                ));
            }

            // Unquote — fires at depth 1, preserves + peels at depth > 1.
            if let Some(arg) = match_unquote(items, ":wat::core::unquote") {
                if depth == 1 {
                    return unquote_argument(arg, bindings, env, sym);
                } else {
                    let inner = walk_template(
                        arg,
                        bindings,
                        macro_scope,
                        macro_name,
                        call_site_span,
                        depth - 1,
                        env,
                        sym,
                    )?;
                    return Ok(WatAST::List(
                        vec![
                            WatAST::Keyword(
                                ":wat::core::unquote".into(),
                                call_site_span.clone(),
                            ),
                            inner,
                        ],
                        call_site_span.clone(),
                    ));
                }
            }

            // Walk each child, handling unquote-splicing inline.
            let out = flatten_template_children(
                items,
                bindings,
                macro_scope,
                macro_name,
                call_site_span,
                depth,
                env,
                sym,
            )?;
            Ok(WatAST::List(out, call_site_span.clone()))
        }
        WatAST::Symbol(ident, _) => {
            // Template-origin symbol — add the macro scope to its scope set.
            Ok(WatAST::Symbol(
                ident.add_scope(macro_scope),
                call_site_span.clone(),
            ))
        }
        // Arc 167 slice 1 — recurse into vector children so a
        // macro template that contains a fn-sig vector (slice 2
        // territory) walks through its parameters with the same
        // hygiene scoping as a list.
        //
        // Arc 200 Gap 2 — Vector templates also dispatch
        // unquote-splicing on List children, mirroring the List
        // branch above. Lispers expect `[~@xs]` to expand to a
        // Vector with `xs`'s elements spliced in, identically to
        // how `(~@xs)` works inside a List template.
        WatAST::Vector(items, _) => {
            // Arc 167 slice 1 / Arc 200 Gap 2 — Vector arm mirrors List arm;
            // flatten_template_children handles both identically.
            let out = flatten_template_children(
                items,
                bindings,
                macro_scope,
                macro_name,
                call_site_span,
                depth,
                env,
                sym,
            )?;
            Ok(WatAST::Vector(out, call_site_span.clone()))
        }
        // Arc 257 slice 1 — Map/Set literals: walk k/v and elements so that
        // unquotes/splices inside them are expanded correctly.
        WatAST::Map(pairs, _) => {
            let mut out_pairs: Vec<(WatAST, WatAST)> = Vec::with_capacity(pairs.len());
            for (k, v) in pairs {
                let wk = walk_template(k, bindings, macro_scope, macro_name, call_site_span, depth, env, sym)?;
                let wv = walk_template(v, bindings, macro_scope, macro_name, call_site_span, depth, env, sym)?;
                out_pairs.push((wk, wv));
            }
            Ok(WatAST::Map(out_pairs, call_site_span.clone()))
        }
        WatAST::Set(items, _) => {
            let mut out: Vec<WatAST> = Vec::with_capacity(items.len());
            for child in items {
                out.push(walk_template(child, bindings, macro_scope, macro_name, call_site_span, depth, env, sym)?);
            }
            Ok(WatAST::Set(out, call_site_span.clone()))
        }
        // Literals and keywords pass through unchanged; keywords carry
        // no scope tracking.
        other => Ok(other.clone()),
    }
}

/// If `items` is `(head arg)` for the given head keyword, return `arg`.
fn match_unquote<'a>(items: &'a [WatAST], head_kw: &str) -> Option<&'a WatAST> {
    if items.len() != 2 {
        return None;
    }
    match items.first() {
        Some(WatAST::Keyword(k, _)) if k == head_kw => items.get(1),
        _ => None,
    }
}

/// Returns `true` if `form` is a `WatAST::List` whose first element is a
/// `WatAST::Keyword` — the arc-143 discriminant for "evaluate at expand-time"
/// vs "treat as already-substituted literal data".
///
/// WatAST::List is the only carrier; a Keyword head is the eval-vs-data
/// discriminant by arc-143 design (no separate computed-unquote AST variant).
/// Used by both `unquote_argument` and `splice_argument` to share the heuristic.
fn is_callable_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items, _) if items.first().map(|h| matches!(h, WatAST::Keyword(_, _))).unwrap_or(false)
    )
}

/// Walk `form`, replacing every `WatAST::Symbol` whose name is a key
/// in `bindings` with the bound AST. Recursive on `WatAST::List`.
/// Other variants (keywords, literals) pass through unchanged.
///
/// Used by `unquote_argument` and `splice_argument` to substitute macro
/// parameters into a List expression BEFORE evaluating it at expand-time.
/// Arc 143 slice 2.
pub(super) fn substitute_bindings(form: &WatAST, bindings: &HashMap<String, WatAST>) -> WatAST {
    match form {
        WatAST::Symbol(ident, _) => {
            if let Some(bound) = bindings.get(ident.as_str()) {
                bound.clone()
            } else {
                form.clone()
            }
        }
        WatAST::List(items, span) => {
            let new_items: Vec<WatAST> = items
                .iter()
                .map(|item| substitute_bindings(item, bindings))
                .collect();
            WatAST::List(new_items, span.clone())
        }
        // Arc 167 slice 1 — recurse into vector children so a
        // macro-parameter symbol buried inside a fn-sig vector
        // (slice 2 territory) is substituted just like inside a
        // list.
        WatAST::Vector(items, span) => {
            let new_items: Vec<WatAST> = items
                .iter()
                .map(|item| substitute_bindings(item, bindings))
                .collect();
            WatAST::Vector(new_items, span.clone())
        }
        other => other.clone(),
    }
}

/// `,X` — the argument is either a macro parameter (substitute its
/// bound AST), a List expression to evaluate at expand-time (arc 143
/// slice 2), or an already-substituted literal value from a prior
/// expansion pass (arc 029 slice 1: the tail-end of the `,,X`
/// resolution path).
///
/// **Backward-compat heuristic (arc 143 slice 2):** a `WatAST::List`
/// whose first element is a `WatAST::Keyword` is treated as a
/// callable expression (e.g., `(:wat::core::i64::+ a 1)`) and
/// evaluated at expand-time with macro params substituted. A List
/// whose head is NOT a Keyword (e.g., a data list from a `,,X`
/// outer-pass substitution) returns as-is, preserving pre-slice-2
/// behavior.
pub(super) fn unquote_argument(
    arg: &WatAST,
    bindings: &HashMap<String, WatAST>,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    match arg {
        WatAST::Symbol(ident, sym_span) => match bindings.get(ident.as_str()) {
            Some(bound) => Ok(bound.clone()),
            None => Err(MacroError {
                span: sym_span.clone(), // Pattern A: symbol span
                kind: MacroErrorKind::UnboundMacroParam { name: ident.as_str().to_owned() },
            }),
        },
        // Arc 143 slice 2: a List whose head is a Keyword is a callable
        // expression (see `is_callable_form`). Substitute macro params, evaluate,
        // convert result to WatAST.
        _ if is_callable_form(arg) => {
            let span = arg.span().clone();
            let substituted = substitute_bindings(arg, bindings);
            // F5 CLOSED (arc 249 stone 249.2b-i): routed through `macro_eval`,
            // the DEFAULT-DENY fenced evaluator. An impure `,(expr)` (any head
            // not on the blessed pure-combinator allow-list) now errors here
            // instead of running. Hash-IS-identity determinism is enforced by
            // construction. See docs/arc/2026/06/249-total-pure-macros/DESIGN-STONE-249.2b.md.
            let val = crate::macros::eval::macro_eval(&substituted, env, sym)?.value_owned();
            crate::runtime::value_to_watast(",(expr)", val, span.clone()).map_err(|e| {
                MacroError {
                    span: span.clone(),
                    kind: MacroErrorKind::MalformedTemplate {
                        reason: format!("computed unquote value_to_watast failed: {}", e),
                    },
                }
            })
        }
        // Already-substituted literal (from a `,,X` outer pass or any
        // other macro that built `(:wat::core::unquote <value>)`
        // directly, or a non-callable List). Return as-is.
        _ => Ok(arg.clone()),
    }
}

/// `,@X` — argument must be a parameter bound to a List AST, a
/// callable List expression to evaluate at expand-time (arc 143
/// slice 2), or an already-substituted List value (arc 029 slice 1:
/// the `,,@X` resolution tail); splice its elements into the
/// surrounding list context.
///
/// **Backward-compat heuristic (arc 143 slice 2):** same as
/// `unquote_argument` — a List whose head is a Keyword is evaluated;
/// otherwise it's treated as an already-substituted list and its
/// elements are spliced directly.
fn splice_argument(
    arg: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_name: &str,
    env: &Environment,
    sym: &SymbolTable,
) -> super::ExpandBatch {
    match arg {
        WatAST::Symbol(ident, sym_span) => {
            let bound = bindings
                .get(ident.as_str())
                .ok_or_else(|| MacroError {
                    span: sym_span.clone(), // Pattern A: symbol span
                    kind: MacroErrorKind::UnboundMacroParam { name: ident.as_str().to_owned() },
                })?;
            match bound {
                WatAST::List(items, _) => Ok(items.clone()),
                // Arc 200 Gap 1 — Vector-bound symbols splice identically
                // to List-bound symbols. Lispers expect `~@xs` to splice
                // whether `xs` was captured from a `(...)` or a `[...]`
                // sub-form at the call site.
                WatAST::Vector(items, _) => Ok(items.clone()),
                other => Err(MacroError {
                    span: other.span().clone(), // Pattern A: bound value's span
                    kind: MacroErrorKind::SpliceNotSequence {
                        name: ident.as_str().to_owned(),
                        got: other.variant_name(),
                    },
                }),
            }
        }
        // Arc 143 slice 2: a List whose head is a Keyword is callable
        // (see `is_callable_form`) — evaluate at expand-time, then splice.
        _ if is_callable_form(arg) => {
            let span = arg.span().clone();
            let substituted = substitute_bindings(arg, bindings);
            // F5 CLOSED (arc 249 stone 249.2b-i): routed through `macro_eval`,
            // the DEFAULT-DENY fenced evaluator. An impure splice-expr is now
            // refused. See `unquote_argument` above + DESIGN-STONE-249.2b.md.
            let val = crate::macros::eval::macro_eval(&substituted, env, sym)
                .map_err(|e| MacroError {
                    span: span.clone(),
                    kind: MacroErrorKind::MalformedTemplate {
                        reason: format!(
                            "macro {} — computed unquote-splicing eval failed: {}",
                            macro_name, e
                        ),
                    },
                })?
                .value_owned();
            // Result must be a Vec; extract elements, convert each to WatAST.
            match val {
                crate::runtime::Value::Vec(elems) => {
                    let ast_elems: super::ExpandBatch = elems
                        .iter()
                        .map(|v| {
                            crate::runtime::value_to_watast(
                                ",@(expr)",
                                v.clone(),
                                span.clone(),
                            )
                            .map_err(|e| MacroError {
                                span: span.clone(),
                                kind: MacroErrorKind::MalformedTemplate {
                                    reason: format!(
                                        "macro {} — computed unquote-splicing element conversion failed: {}",
                                        macro_name, e
                                    ),
                                },
                            })
                        })
                        .collect();
                    ast_elems
                }
                other => Err(MacroError {
                    span: span.clone(),
                    kind: MacroErrorKind::MalformedTemplate {
                        reason: format!(
                            "macro {} — computed unquote-splicing ',@(expr)' evaluated to {}; \
                             expected a Vec",
                            macro_name,
                            other.type_name()
                        ),
                    },
                }),
            }
        }
        // Already-substituted list value.
        WatAST::List(items, _) => Ok(items.clone()),
        other => Err(MacroError {
            span: other.span().clone(), // Pattern A: offending node's span
            kind: MacroErrorKind::MalformedTemplate {
                reason: format!(
                    "macro {} — unquote-splicing ',@X' requires a list (parameter \
                     or already-substituted value); got {}",
                    macro_name,
                    other.variant_name()
                ),
            },
        }),
    }
}
