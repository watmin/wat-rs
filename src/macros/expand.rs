use crate::ast::WatAST;
use crate::span::Span;
use crate::identifier::{fresh_scope, ScopeId};
use crate::runtime::{Environment, SymbolTable, TrackedValue, Value};
use std::collections::HashMap;
use std::sync::Arc;

use super::error::{MacroError, MacroErrorKind};
use super::registry::{MacroDef, MacroRegistry};
use super::parse::{is_defmacro_form, parse_defmacro_form, EXPANSION_DEPTH_LIMIT};

/// Expand every macro call in `forms` to fixpoint. Returns the expanded
/// AST list.
pub fn expand_all(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Vec<WatAST>, MacroError> {
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
        let expanded = expand_form(form, registry, 0, env, sym)?;
        if is_defmacro_form(&expanded) {
            let def = parse_defmacro_form(expanded)?;
            registry.register(def)?;
        } else {
            out.push(expanded);
        }
    }
    Ok(out)
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
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if let Some(def) = registry.get(head) {
                let args = items[1..].to_vec();
                return expand_macro_call(def, args, span.clone(), env, sym);
            }
        }
    }
    Ok(form)
}

/// Expand a single form. Recursively expands children, then checks
/// whether the resulting node is itself a macro call; if so, expand it,
/// and continue to fixpoint.
pub(super) fn expand_form(
    form: WatAST,
    registry: &MacroRegistry,
    depth: usize,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    if depth > EXPANSION_DEPTH_LIMIT {
        return Err(MacroError {
            span: form.span().clone(), // Pattern B: the form being expanded
            kind: MacroErrorKind::ExpansionDepthExceeded { limit: EXPANSION_DEPTH_LIMIT },
        });
    }

    match form {
        WatAST::List(items, list_span) => {
            // Arc 029 / 030: do NOT recurse into bodies of forms that
            // carry DATA rather than evaluable code:
            // - `(:wat::core::quasiquote X)` — macro template; inner
            //   "macro calls" are deferred until the enclosing macro
            //   fires. Pre-emptive expansion would corrupt the template.
            // - `(:wat::core::quote X)` — literal AST value; X is data,
            //   not code. Recursing would eagerly expand macro calls
            //   that the user wanted to observe, not execute. Arc 030
            //   macroexpand primitives rely on quote preserving the
            //   raw form.
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                if head == ":wat::core::quasiquote" || head == ":wat::core::quote" {
                    return Ok(WatAST::List(items, list_span));
                }
            }

            // Recurse into children first. This gives us the shape
            // (expanded-head expanded-args...) — any inner macro calls
            // resolved before we check the outer for a macro call.
            let expanded_children: Result<Vec<_>, _> = items
                .into_iter()
                .map(|c| expand_form(c, registry, depth + 1, env, sym))
                .collect();
            let expanded_children = expanded_children?;

            // rune:solvere(historical-shape) — keyword/of + threading are transitional in-pass Rust desugars; HARD-CUT when reborn as wat code in arc 249.3/249.4. Splitting to a module now would mint a home destined for deletion.
            //
            // Arc 170 slice 3 Gap A — `:wat::core::keyword/of` macro special-form.
            // Recognized AFTER child recursion (so unquote sites `~name` have
            // already been substituted to their argument keywords) and BEFORE
            // the generic macro dispatch so it fires for both bare keyword/of
            // calls AND keyword/of calls produced inside quasiquote templates.
            //
            // Expansion order:
            //   1. quasiquote walk_template fires, substituting ~input-type etc.
            //   2. The quasiquote result (now containing a concrete keyword/of
            //      list) is returned as WatAST.
            //   3. expand_form is called on that result (fixpoint loop in
            //      expand_macro_call). Children are recursed (this
            //      pass); keyword/of is recognised here.
            //
            // This means: at the point this arm fires, all unquote sites have
            // already resolved to concrete WatAST::Keyword nodes. The children
            // of the keyword/of form ARE the final keyword ASTs.
            if let Some(WatAST::Keyword(head, _)) = expanded_children.first() {
                if head == ":wat::core::keyword/of" {
                    return construct_keyword_of(&expanded_children, list_span);
                }
            }

            // Arc 249 — threading macros `->` (thread-first) and `->>`
            // (thread-last).  Recognized AFTER child recursion and BEFORE the
            // registered-macro dispatch, mirroring the keyword/of built-in
            // above.  The head is a bare WatAST::Symbol whose name is "->" or
            // "->>"; in all other positions (e.g. the infix return-arrow in a
            // defn signature) "->" is never a list head, so there is no clash.
            if let Some(WatAST::Symbol(head, _)) = expanded_children.first() {
                match head.as_str() {
                    "->" => {
                        return thread_desugar(&expanded_children[1..], false, list_span);
                    }
                    "->>" => {
                        return thread_desugar(&expanded_children[1..], true, list_span);
                    }
                    _ => {}
                }
            }

            // Is the (now-expanded) head a registered macro?
            if let Some(WatAST::Keyword(head, _)) = expanded_children.first() {
                if let Some(def) = registry.get(head) {
                    // Macro call — expand this call site. Pass the
                    // outer list's span so the expansion can inherit
                    // it (call-site span, per arc 016 slice 1
                    // DESIGN: generated forms inherit the caller's
                    // span).
                    let args = expanded_children[1..].to_vec();
                    let expanded = expand_macro_call(def, args, list_span.clone(), env, sym)?;
                    // Re-expand the result to fixpoint.
                    return expand_form(expanded, registry, depth + 1, env, sym);
                }
            }

            // Not a macro call — preserve the outer list's span.
            Ok(WatAST::List(expanded_children, list_span))
        }
        // Arc 167 slice 1 — recurse into vector children so a
        // macro call buried inside a fn-sig vector (slice 2
        // territory) still expands. Vectors carry no head-keyword
        // dispatch, so the macro-call detection arm at the head
        // doesn't apply.
        WatAST::Vector(items, vec_span) => {
            let expanded_children: Result<Vec<_>, _> = items
                .into_iter()
                .map(|c| expand_form(c, registry, depth + 1, env, sym))
                .collect();
            Ok(WatAST::Vector(expanded_children?, vec_span))
        }
        other => Ok(other),
    }
}

// ─── Arc 249 — threading macros `->` / `->>` ────────────────────────────────
//
// `thread_desugar` is called from `expand_form` when the list head is the bare
// symbol `->` (thread-first, `last=false`) or `->>` (thread-last, `last=true`).
//
// Contract:
//   `acc_and_steps` = expanded_children[1..] from the threading form
//                     (the head symbol itself has been consumed by the caller).
//   `last`          = false → thread-first (inject acc as first arg of each step)
//                     true  → thread-last  (inject acc as last  arg of each step)
//   `list_span`     = the outer `->` / `->>` form's span; carried onto every
//                     constructed WatAST::List node (call-site span convention,
//                     matching how keyword/of / the macro-call path inherit it).
//
// Left-fold semantics:
//   acc ← acc_and_steps[0]
//   for each step in acc_and_steps[1..]:
//     non-list step `f`        → new acc = (f acc)      [both forms identical]
//     list step `(f a b …)`   → thread-first: (f acc a b …)
//                              → thread-last:  (f a b … acc)
//
// Special cases:
//   empty acc_and_steps (e.g. `(->)`)   → ArityMismatch error (≥1 form required)
//   single-element (e.g. `(-> x)`)      → identity; return x unchanged.
//
fn thread_desugar(
    acc_and_steps: &[WatAST],
    last: bool,
    list_span: Span,
) -> Result<WatAST, MacroError> {
    if acc_and_steps.is_empty() {
        return Err(MacroError {
            span: list_span,
            kind: MacroErrorKind::ArityMismatch {
                name: if last { "->>" } else { "->" }.to_string(),
                expected: 1,
                got: 0,
            },
        });
    }

    // acc starts as the first element (the initial value / accumulator).
    let mut acc = acc_and_steps[0].clone();

    // Fold each step left.
    for step in &acc_and_steps[1..] {
        acc = match step {
            // Non-list step (bare Symbol or Keyword `f`) → `(f acc)` for both forms.
            WatAST::Symbol(_, _) | WatAST::Keyword(_, _) => {
                WatAST::List(vec![step.clone(), acc], list_span.clone())
            }
            // List step `(f a b …)`:
            //   thread-first → `(f acc a b …)`
            //   thread-last  → `(f a b … acc)`
            WatAST::List(step_items, _) => {
                let mut new_items = step_items.clone();
                if last {
                    // thread-last: inject acc at the end
                    new_items.push(acc);
                } else {
                    // thread-first: inject acc after the head (position 1)
                    if new_items.is_empty() {
                        // Degenerate empty-list step `()` — treat as bare
                        // application with no head; wrap as `(acc)`.
                        new_items.push(acc);
                    } else {
                        new_items.insert(1, acc);
                    }
                }
                WatAST::List(new_items, list_span.clone())
            }
            // Any other shape: wrap as a 1-arg call, same as the non-list branch.
            other => WatAST::List(vec![other.clone(), acc], list_span.clone()),
        };
    }

    Ok(acc)
}

// ─── Arc 170 slice 3 Gap A — keyword/of construction ────────────────────────
//
// `construct_keyword_of` is called from `expand_form` when the list head is
// `:wat::core::keyword/of`. At this point child recursion has already fired,
// so all `~unquote` sites in the enclosing quasiquote template have been
// substituted with their argument keyword ASTs.
//
// Contract:
//   children[0]  = keyword/of head (`:wat::core::keyword/of`) — consumed/skipped
//   children[1]  = parametric head keyword (e.g. `:wat::kernel::Receiver`)
//   children[2+] = argument keywords (e.g. `:wat::core::i64`)
//
// Produces a single `WatAST::Keyword` whose text is:
//   `{head_text}<{arg1_text},{arg2_text},...>`
// where head_text = head keyword text WITHOUT leading ':',
// and   argN_text = argument keyword text WITHOUT leading ':'.
//
// Errors:
//   - Arity < 2 (just `(keyword/of)` or `(keyword/of :Head)` with no args)
//   - Any child is not a `WatAST::Keyword`
fn construct_keyword_of(
    children: &[WatAST],
    span: Span,
) -> Result<WatAST, MacroError> {
    // children[0] is the ":wat::core::keyword/of" head itself.
    // We need children[1] (parametric head) + children[2+] (args).
    if children.len() < 3 {
        // Need at least: keyword/of-head + parametric-head + 1 arg.
        return Err(MacroError {
            span,
            kind: MacroErrorKind::MalformedTemplate {
                reason: format!(
                    "keyword/of requires at least 2 arguments (head keyword + ≥1 arg keyword); \
                     got {} argument(s)",
                    children.len().saturating_sub(1)
                ),
            },
        });
    }

    // Helper to name the AST variant for error messages.
    fn ast_kind(node: &WatAST) -> &'static str {
        match node {
            WatAST::Keyword(_, _)     => "keyword",
            WatAST::IntLit(_, _)      => "int-literal",
            WatAST::FloatLit(_, _)    => "float-literal",
            WatAST::BoolLit(_, _)     => "bool-literal",
            WatAST::StringLit(_, _)   => "string-literal",
            // Arc 244 — NilLit joins the literal group.
            WatAST::NilLit(_)         => "nil-literal",
            WatAST::Symbol(_, _)      => "symbol",
            WatAST::List(_, _)        => "list",
            WatAST::Vector(_, _)      => "vector",
            WatAST::StructPattern(_, _) => "struct-pattern",
        }
    }

    // Extract the parametric head keyword text (children[1]).
    let head_text = match &children[1] {
        WatAST::Keyword(k, _) => {
            // Strip leading ':'.
            k.strip_prefix(':').unwrap_or(k.as_str()).to_string()
        }
        other => {
            return Err(MacroError {
                span,
                kind: MacroErrorKind::MalformedTemplate {
                    reason: format!(
                        "keyword/of: head (arg 1) must be a keyword AST; got {}",
                        ast_kind(other)
                    ),
                },
            });
        }
    };

    // Extract each argument keyword text (children[2+]).
    let mut arg_texts = Vec::with_capacity(children.len() - 2);
    for (i, child) in children[2..].iter().enumerate() {
        match child {
            WatAST::Keyword(k, _) => {
                // Strip leading ':'.
                let text = k.strip_prefix(':').unwrap_or(k.as_str()).to_string();
                arg_texts.push(text);
            }
            other => {
                return Err(MacroError {
                    span,
                    kind: MacroErrorKind::MalformedTemplate {
                        reason: format!(
                            "keyword/of: argument {} must be a keyword AST; got {}",
                            i + 1,
                            ast_kind(other)
                        ),
                    },
                });
            }
        }
    }

    // Construct the parametric keyword text: `head<arg1,arg2,...>`.
    let constructed = format!(":{}<{}>", head_text, arg_texts.join(","));
    Ok(WatAST::Keyword(constructed, span))
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
                    kind: MacroErrorKind::ArityMismatch {
                        name: def.name.clone(),
                        expected: fixed_arity,
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

    let macro_scope = fresh_scope();
    expand_template(&def.body, &bindings, macro_scope, &def.name, &call_site_span, def.rest_param.as_deref(), env, sym)
}

/// Walk a macro template, substituting `,param` and `,@param` at
/// unquote sites and adding the macro scope to template-origin symbols.
///
/// The template's top-level form is either:
/// - a **bare quasiquote** `(:wat::core::quasiquote X)` — the existing hygienic path
///   via `walk_template` (sets-of-scopes hygiene, UNCHANGED).
/// - a **program body** (any other form) — the new `macro_eval` path introduced in
///   arc 249 stone 249.2b-ii: params are bound as quoted form-values in a body env,
///   `macro_eval` evaluates the body, and the result is converted back to a `WatAST`.
///
/// `rest_param` names the variadic rest parameter (if any) so the program path can
/// bind it as `Value::Vec([watast...])` rather than a `WatAST::List` (which is the
/// encoding used for the quasiquote-template walk-template path).
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
    // ── Dispatch: bare quasiquote (existing hygienic path) vs program body (new path) ──
    let quasi_body = match template {
        WatAST::List(items, _) if items.len() == 2 => match items.first() {
            Some(WatAST::Keyword(k, _)) if k == ":wat::core::quasiquote" => Some(&items[1]),
            _ => None,
        },
        _ => None,
    };

    if let Some(qb) = quasi_body {
        // ── Existing hygienic path (bare quasiquote body) — UNCHANGED ──
        // `walk_template` adds sets-of-scopes hygiene to template-origin symbols.
        // This path is untouched: every existing stdlib macro goes through here.
        return walk_template(qb, bindings, macro_scope, macro_name, call_site_span, 1, env, sym);
    }

    // ── New program-body path (arc 249 stone 249.2b-ii) ──
    //
    // Gate E — hygiene bound: refuse a program body whose quasiquote templates
    // introduce a literal name in a binder position (`:wat::core::let` or
    // `:wat::core::fn`). eval_quasiquote adds no hygiene scopes, so such a body
    // could silently capture caller-site names. Default-deny; emit
    // ProgramBodyIntroducesName so a future "allow" can't silently admit the
    // capturing case.
    check_program_body_hygiene(template, call_site_span, macro_name)?;

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
                _ => &[],
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

    // Evaluate the program body under the fenced evaluator.
    // macro_eval runs validate_pure_total (DEFAULT-DENY purity gate) then runtime::eval.
    let result_tv = crate::macros::eval::macro_eval(template, &body_env, sym)
        .map_err(|e| MacroError {
            span: call_site_span.clone(),
            kind: MacroErrorKind::MalformedTemplate {
                reason: format!("macro {} — program body eval failed: {}", macro_name, e),
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
fn check_program_body_hygiene(
    body: &WatAST,
    call_site_span: &Span,
    macro_name: &str,
) -> Result<(), MacroError> {
    match body {
        WatAST::List(items, _) => {
            if let Some(inner) = items_is_quasiquote(items) {
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

/// If `items` is `(:wat::core::quasiquote X)` (2-element List with quasiquote head),
/// return `Some(&X)`. Otherwise `None`.
fn items_is_quasiquote(items: &[WatAST]) -> Option<&WatAST> {
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
                if let Some(binding_vec) = items.get(1) {
                    if let WatAST::Vector(binder_items, _) = binding_vec {
                        let mut i = 0;
                        while i < binder_items.len() {
                            if let WatAST::Symbol(ident, _) = &binder_items[i] {
                                return Err(MacroError {
                                    span: call_site_span.clone(),
                                    kind: MacroErrorKind::ProgramBodyIntroducesName {
                                        macro_name: macro_name.to_string(),
                                        binder: ident.name.clone(),
                                    },
                                });
                            }
                            i += 2;
                        }
                    }
                }
            } else if head == ":wat::core::fn" {
                // args[0] is the params vector: [name <- :T name <- :T ...]
                // Param names are at positions 0, 3, 6, … (argspec triples).
                if let Some(params_vec) = items.get(1) {
                    if let WatAST::Vector(param_items, _) = params_vec {
                        let mut i = 0;
                        while i < param_items.len() {
                            if let WatAST::Symbol(ident, _) = &param_items[i] {
                                // Exclude `->` and `<-` and `&` markers.
                                if ident.name != "->" && ident.name != "<-" && ident.name != "&" {
                                    return Err(MacroError {
                                        span: call_site_span.clone(),
                                        kind: MacroErrorKind::ProgramBodyIntroducesName {
                                            macro_name: macro_name.to_string(),
                                            binder: ident.name.clone(),
                                        },
                                    });
                                }
                            }
                            i += 1;
                        }
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

/// Walk a slice of template children (items of a List or Vector),
/// handling unquote-splicing and `for`-comprehension inline. Returns
/// the flat `Vec<WatAST>` for the parent to wrap in its own container
/// (`WatAST::List` or `WatAST::Vector`).
///
/// Extracted from `walk_template`'s List and Vector arms, which are
/// identical except for the final constructor.
#[allow(clippy::too_many_arguments)]
fn splice_children(
    items: &[WatAST],
    bindings: &HashMap<String, WatAST>,
    macro_scope: ScopeId,
    macro_name: &str,
    call_site_span: &Span,
    depth: u32,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Vec<WatAST>, MacroError> {
    let mut out = Vec::with_capacity(items.len());
    for child in items {
        if let WatAST::List(child_items, _) = child {
            if let Some(splice_arg) =
                match_unquote(child_items, ":wat::core::unquote-splicing")
            {
                if depth == 1 {
                    // Arc 248 slice 1 — bounded `for` comprehension in
                    // splice position: `,@(:wat::core::for [x xs] tmpl)`.
                    // Recognize BEFORE falling through to splice_argument.
                    if let WatAST::List(for_items, for_span) = splice_arg {
                        if let Some((binder_name, list_name, element_tmpl)) =
                            match_for_comprehension(for_items)
                        {
                            // Resolve the list from the current bindings.
                            let elements: Vec<WatAST> = match bindings.get(list_name) {
                                Some(WatAST::List(elems, _)) => elems.clone(),
                                Some(WatAST::Vector(elems, _)) => elems.clone(),
                                Some(other) => {
                                    return Err(MacroError {
                                        span: for_span.clone(),
                                        kind: MacroErrorKind::MalformedTemplate {
                                            reason: format!(
                                                "macro {} — `for` list binding `{}` \
                                                 must be a List or Vector; got {}",
                                                macro_name,
                                                list_name,
                                                other.variant_name()
                                            ),
                                        },
                                    });
                                }
                                None => {
                                    return Err(MacroError {
                                        span: for_span.clone(),
                                        kind: MacroErrorKind::UnboundMacroParam {
                                            name: list_name.to_string(),
                                        },
                                    });
                                }
                            };
                            // Iterate: bind binder per element, walk template,
                            // collect results. Each iteration is independent —
                            // the binder binding is local and does not persist.
                            for element in &elements {
                                let mut iter_bindings = bindings.clone();
                                iter_bindings
                                    .insert(binder_name.to_string(), element.clone());
                                let expanded = walk_template(
                                    element_tmpl,
                                    &iter_bindings,
                                    macro_scope,
                                    macro_name,
                                    call_site_span,
                                    depth,
                                    env,
                                    sym,
                                )?;
                                out.push(expanded);
                            }
                            continue;
                        }
                    }
                    // Not a `for` form — fire the regular splice.
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
            let out = splice_children(
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
            // splice_children handles both identically.
            let out = splice_children(
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
        // Literals and keywords pass through unchanged; keywords carry
        // no scope tracking.
        other => Ok(other.clone()),
    }
}

// rune:solvere(historical-shape) — template-local recognizers kept beside their sole caller walk_template; match_for_comprehension rehomes with the for-comprehension in arc 249.4.

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

/// If `items` is `(:wat::core::for [binder list-name] template)`, return
/// `(binder_name, list_name, template)`. Returns `None` otherwise.
///
/// Arc 248 slice 1 — bounded template comprehension. The form is
/// recognized in splice position inside `walk_template`; it iterates a
/// finite list and instantiates a template per element (map, not eval).
fn match_for_comprehension<'a>(
    items: &'a [WatAST],
) -> Option<(&'a str, &'a str, &'a WatAST)> {
    if items.len() != 3 {
        return None;
    }
    // Head must be `:wat::core::for`.
    match items.first() {
        Some(WatAST::Keyword(k, _)) if k == ":wat::core::for" => {}
        _ => return None,
    }
    // Second element: `[binder list-name]` — a Vector with exactly 2 Symbols.
    let (binder_name, list_name) = match &items[1] {
        WatAST::Vector(binder_vec, _) if binder_vec.len() == 2 => {
            match (&binder_vec[0], &binder_vec[1]) {
                (WatAST::Symbol(b, _), WatAST::Symbol(l, _)) => {
                    (b.name.as_str(), l.name.as_str())
                }
                _ => return None,
            }
        }
        _ => return None,
    };
    // Third element: the element template.
    Some((binder_name, list_name, &items[2]))
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
            if let Some(bound) = bindings.get(&ident.name) {
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
        WatAST::Symbol(ident, sym_span) => match bindings.get(&ident.name) {
            Some(bound) => Ok(bound.clone()),
            None => Err(MacroError {
                span: sym_span.clone(), // Pattern A: symbol span
                kind: MacroErrorKind::UnboundMacroParam { name: ident.name.clone() },
            }),
        },
        // Arc 143 slice 2: a List whose head is a Keyword is a
        // callable expression. Substitute macro params, evaluate,
        // convert result to WatAST.
        WatAST::List(items, span)
            if items
                .first()
                .map(|h| matches!(h, WatAST::Keyword(_, _)))
                .unwrap_or(false) =>
        {
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
) -> Result<Vec<WatAST>, MacroError> {
    match arg {
        WatAST::Symbol(ident, sym_span) => {
            let bound = bindings
                .get(&ident.name)
                .ok_or_else(|| MacroError {
                    span: sym_span.clone(), // Pattern A: symbol span
                    kind: MacroErrorKind::UnboundMacroParam { name: ident.name.clone() },
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
                        name: ident.name.clone(),
                        got: other.variant_name(),
                    },
                }),
            }
        }
        // Arc 143 slice 2: a List whose head is a Keyword — evaluate
        // at expand-time, then splice the resulting Vec elements.
        WatAST::List(items, span)
            if items
                .first()
                .map(|h| matches!(h, WatAST::Keyword(_, _)))
                .unwrap_or(false) =>
        {
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
                    let ast_elems: Result<Vec<WatAST>, _> = elems
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
