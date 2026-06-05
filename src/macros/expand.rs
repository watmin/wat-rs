use crate::ast::WatAST;
use crate::span::Span;
use crate::scope::{fresh_scope, ScopeId};
use crate::runtime::{Environment, SymbolTable, TrackedValue, Value};
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
pub fn expand_all(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
    env: &Environment,
    sym: &SymbolTable,
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
    expansion_depth: usize,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, MacroError> {
    if expansion_depth > EXPANSION_DEPTH_LIMIT {
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
            let expanded_children: super::ExpandBatch = items
                .into_iter()
                .map(|c| expand_form(c, registry, expansion_depth + 1, env, sym))
                .collect();
            let expanded_children = expanded_children?;

            // Arc 249 Stone 249.4a — keyword/of is now a registered wat macro in
            // core.wat; Rust built-in DELETED. Dispatch falls through to the
            // registered-macro path below.

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
                    return expand_form(expanded, registry, expansion_depth + 1, env, sym);
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
            let expanded_children: super::ExpandBatch = items
                .into_iter()
                .map(|c| expand_form(c, registry, expansion_depth + 1, env, sym))
                .collect();
            Ok(WatAST::Vector(expanded_children?, vec_span))
        }
        other => Ok(other),
    }
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
    expand_template(&def.body, &bindings, macro_scope, &def.name, &call_site_span, def.rest_param.as_deref(), env, sym)
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
    let result_tv = crate::macros::eval::macro_eval_pre_validated(template, &body_env, sym)
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
