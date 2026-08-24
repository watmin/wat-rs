//! RHS fact construction — `build_insert_fact` / `eval_insert`.
//!
//! The `:then` dual of `matcher.rs`'s alpha-match. Native fire runs
//! `exec_compiled_rhs`; this file is the interpreter / differential.

use crate::ast::WatAST;
use crate::rete::eval_test::eval_rhs_expr;
use crate::rete::matcher::resolve_operand;
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::value::value::AggregateValue;
use crate::span::Span;
use std::sync::Arc;

/// Kwargs `(:Type :field v …)` vs positional `(:Type v …)`. Shared by
/// `build_insert_fact`, `compile_rhs`, `lower_construct`, and the freeze wall.
pub(crate) fn rete_kwargs_value_asts(args: &[WatAST]) -> Vec<&WatAST> {
    if rete_is_kwargs(args) {
        args.iter().skip(1).step_by(2).collect()
    } else {
        args.iter().collect()
    }
}

pub(crate) fn rete_is_kwargs(args: &[WatAST]) -> bool {
    args.len() >= 2
        && args.len().is_multiple_of(2)
        && args.iter().step_by(2).all(|a| matches!(a, WatAST::Keyword(_, _)))
}

// ─── Public entry point: RHS insert-form evaluator ───────────────────────────

/// `build_insert_fact` — the pure inner of `eval_insert`.
///
/// Arc 278 Stone A (DESIGN-STONE-then-is-a-vector-of-singular-facts.md): `:then` is now a
/// vector of BARE fact-forms — the `(:wat::rete::insert …)` RHS marker wrapper is gone (the
/// engine is inserts-only by doctrine, so naming it per entry said nothing). `fact_form` IS
/// the fact-form directly: `(:RecordType arg…)`.
///
/// Given `fact_form` and the token `bindings`, validates the form, resolves each fact-arg via
/// `resolve_operand` (empty fact-fields/names: RHS has no current fact), and builds the
/// `Value::Aggregate` record.
///
/// Called from `eval_insert` (after arg evaluation) and from the compiled-rhs
/// differential (`compiled_rhs` tests). Native production runs `exec_compiled_rhs`
/// (`rhs_must_compile`); it does not walk this function.
///
/// Arc 278 Stone B (DESIGN-STONE-then-is-a-vector-of-singular-facts.md § "Stone B") — takes
/// `sym` now: widening (a) means `fact_items[0]` may name a plain fn instead of a fact-type
/// constructor, and only `sym.types()`/`sym.functions` can tell the two apart at fire time (the
/// freeze-time wat fence, `then-item-fence`, already proved whichever this is is legal — this is
/// the SAME registry read, once more, to pick the execution shape). See
/// [`build_insert_fact_call`] for the fn-call branch.
///
/// Raises `RuntimeError` on malformed form or unresolved operand. Never panics.
pub(crate) fn build_insert_fact(
    fact_form: &WatAST,
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-insert";
    // Arc 278 — splitting the production pass (34.9ms, 34% of a fact-heavy fire) into its parts
    // BEFORE drawing a stone against it. Coarse marks only (~52ns/pair x 4 x 40,000 derived facts):
    // read as PROPORTIONS, and read the enclosing `production` total against its un-instrumented
    // 34.963ms to see the tax. Allocation COUNTS use counters (~1-2ns) — the house method for a
    // level where a timer would tax the thing it measures.
    let __pv = crate::rete::kernel::phase_start();

    // Validate the fact form: must be a List `(:RecordType arg…)` with a keyword head.
    // Borrow (do NOT clone) — this runs once per derived fact; cloning the form AST per fact was
    // pure waste (the fan-out residual). We only read items[0]/len here.
    let fact_items = match fact_form {
        WatAST::List(items, _) if !items.is_empty() => items.as_slice(),
        _ => {
            return Err(RuntimeError::new(fact_form.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "fact-form List (:RecordType arg…)",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(fact_form.clone())))),
            }).into());
        }
    };
    // Head of fact-form must be a keyword naming the record type.
    let type_keyword = match &fact_items[0] {
        WatAST::Keyword(k, _) => k.as_str(),
        other => {
            return Err(RuntimeError::new(other.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "keyword (record type) as fact-form head",
                got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{other:?}"))))),
            }).into());
        }
    };
    // Arc 278 Stone B, widening (a) — the item head may now be EITHER a fact-type constructor
    // (the fast path below, UNCHANGED) OR a fn whose declared return type is a fact type ("has
    // its own argument convention" — plain positional call args, not field values;
    // `BRIEF-then-user-forms.md` § "(a) THE ITEM HEAD"). `sym.types()` is the SAME registry
    // `validate_then_form`'s `lookup_fields` reads at freeze time (Rust-side); here it
    // disambiguates at FIRE time. The freeze-time wat fence (`then-item-fence`, wired into
    // `compile-rule`) already proved this item legal before this ever runs — this check only
    // picks which of the two (already-proven-safe) execution shapes to take.
    let names = match sym.types().and_then(|t| t.get(type_keyword)) {
        Some(crate::types::TypeDef::Aggregate(a)) => a.names_arc(),
        _ => return build_insert_fact_call(fact_form, type_keyword, &fact_items[1..], bindings, sym),
    };

    // class = keyword stripped of leading ':' (Arc 293.R2.1: colon-free).
    // A String allocated per derived fact for a class name fixed at compile time — NOT counted by
    // `match:key-alloc`, which arms only the two resolve_operand sites.
    crate::rete::kernel::census_count("prod:class-alloc");
    let class = type_keyword.strip_prefix(':').unwrap_or(type_keyword).to_string();
    crate::rete::kernel::phase_end("  ├ prod:validate", __pv);
    let __ps = crate::rete::kernel::phase_start();

    // Arc 294 item 9a — a defrule :then RHS fact-form may be written in KWARGS form
    // `(:Type :field1 v1 :field2 v2)` (the flip's encouraged form, symmetric with the
    // field-named :when patterns) or the legacy positional `(:Type v1 v2)`. After the
    // type-vs-fn head split (`sym.types()` above), kwargs skip field names and take VALUES
    // in written order — fields are authored in the type's declaration order (both the
    // kwargs migration and the macro companion emit declaration order).
    // Out-of-declaration-order kwargs map positionally.
    let args = &fact_items[1..];
    let value_asts = rete_kwargs_value_asts(args);
    // Resolve each value via `resolve_rhs_value` (fenced Lists + `?var` + literal).
    // RHS has no current fact. None → malformed rule.
    crate::rete::kernel::census_count_n("prod:vec-alloc", 2); // value_asts + fields
    let mut fields: Vec<Value> = Vec::with_capacity(value_asts.len());
    crate::rete::kernel::phase_end("  ├ prod:shape", __ps);
    let __pr = crate::rete::kernel::phase_start();
    for arg in value_asts {
        match resolve_rhs_value(arg, bindings, sym)? {
            Some(v) => fields.push(v),
            None => {
                return Err(RuntimeError::new(arg.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "resolvable operand (?var, literal, or a fenced expression) in RHS fact-form",
                    got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{arg:?}"))))),
                }).into());
            }
        }
    }

    crate::rete::kernel::phase_end("  ├ prod:resolve", __pr);
    let __pc = crate::rete::kernel::phase_start();
    crate::rete::kernel::census_count_n("prod:record-alloc", 2); // AggregateValue + the fields Arc
    let out = Value::Aggregate(Arc::new(AggregateValue::record(class, names, Arc::new(fields))));
    crate::rete::kernel::phase_end("  ├ prod:construct", __pc);
    Ok(out)
}

/// Arc 278 Stone B, widening (a) — the FN-CALL branch of [`build_insert_fact`]: `head` does not
/// name a known aggregate type, so (by the freeze-time `then-item-fence`'s own proof) it names a
/// user fn whose declared return type is a fact type. Its "arguments" are the fn's OWN positional
/// parameters — a DIFFERENT convention from a constructor's field values, so no kwargs detection
/// applies here (`BRIEF-then-user-forms.md` § "(a) THE ITEM HEAD": *"the kwargs
/// reorder-to-declaration-order logic … applies to a constructor, not to a fn call, which has its
/// own argument convention"*).
///
/// Resolves each arg via [`resolve_rhs_value`] (widening (b) applies to a fn call's args too),
/// applies the fn, and checks the result is a fact (an `Aggregate`) — defensively: the
/// freeze-time fence already proved the fn's DECLARED return type is a fact type, so reaching a
/// non-`Aggregate` result here would mean the fence was bypassed or a checker gap let a
/// mistyped fn through, never an expected path. Never panics, never silently drops.
fn build_insert_fact_call(
    fact_form: &WatAST,
    head: &str,
    args: &[WatAST],
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-insert";
    let func = match sym.get(head) {
        Some(f) => f.clone(),
        None => {
            return Err(RuntimeError::new(fact_form.span().clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "':then' item head '{head}' names neither a known fact-type constructor nor \
                     a registered fn — the rule-compile fence should have refused this"
                ),
            }).into());
        }
    };
    let mut vals: Vec<Value> = Vec::with_capacity(args.len());
    for arg in args {
        match resolve_rhs_value(arg, bindings, sym)? {
            Some(v) => vals.push(v),
            None => {
                return Err(RuntimeError::new(arg.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "resolvable operand (?var, literal, or a fenced expression) in a RHS fn-call arg",
                    got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{arg:?}"))))),
                }).into());
            }
        }
    }
    let result = crate::runtime::apply_function(func, vals, sym, fact_form.span().clone())
        .map_err(EvalBreak::from)?;
    if crate::rete::matcher::is_record_fact(&result) {
        Ok(result)
    } else {
        Err(RuntimeError::new(fact_form.span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "the fn to return a fact (a Record) — the rule-compile fence should \
                       have refused a non-fact return type",
            got: Box::new(ValueSnapshot::of(&result)),
        }).into())
    }
}

/// Arc 278 Stone B — the RHS-only operand resolver: tries the plain [`resolve_operand`] first
/// (unchanged fast path — `?var` / `:field` / literal), and if that returns `None` AND `arg` is a
/// call form (a `List`), falls through to a FENCED evaluation (widening (b)). Lives here, NOT
/// inside `resolve_operand` itself, so `:when`'s LHS matching (which shares that fn) is untouched
/// (`BRIEF-then-user-forms.md` STOP-5: "Do NOT touch `:when`").
///
/// The freeze-time wat fence (`then-item-fence`) has already proven any `List` reaching here is
/// pure ∧ deterministic ∧ total ∧ rete-primitive (declaration-derived constructors still admitted
/// via `head_ok`'s first door) — the SAME four-axis warrant `eval_test_core` already relies on
/// for a `where` predicate. [`eval_rhs_expr`] is the interpreter half of that evaluation.
pub(crate) fn resolve_rhs_value(
    arg: &WatAST,
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<Option<Value>, EvalBreak> {
    if let Some(v) = resolve_operand(arg, &[], &[], bindings) {
        return Ok(Some(v));
    }
    match arg {
        WatAST::List(..) => Ok(Some(eval_rhs_expr(arg, bindings, sym)?)),
        _ => Ok(None),
    }
}

/// `(:wat::rete::eval-insert <fact-form: :wat::WatAST> <bindings: :wat::core::PersistentMap>)
/// -> :wat::core::Record`
///
/// The RHS dual of `eval_alpha_match`: where alpha-match is `(cond, fact) → Option<bindings>`,
/// eval-insert is `(fact-form, bindings) → fact`. Both sides reuse `resolve_operand`. Arc 278
/// Stone A: `fact-form` is a bare `(:Type arg…)` — the `insert` RHS-marker wrapper is gone.
///
/// Entry point dispatched by `dispatch_keyword_head_value` in `runtime.rs`.
/// Evaluates both arguments, then delegates to `build_insert_fact` for the pure inner.
///
/// Raises `RuntimeError` on arity mismatch, type mismatch, malformed form, or
/// unresolved operand. Never panics, never silently drops.
pub(crate) fn eval_insert(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-insert";
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        }).into());
    }

    // Evaluate arg[0]: must be Value::wat__WatAST wrapping a List.
    let form_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let form_ast = match form_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST (fact form from quote)",
                got: Box::new(ValueSnapshot::of(&other)),
            }).into());
        }
    };

    // Evaluate arg[1]: must be Value::wat__core__PersistentMap (token bindings). `build_insert_fact`
    // is now typed to `PMap` directly (DESIGN-STONE-token-bindings-promoting) — no trie
    // materialisation at this boundary; the value IS the field.
    let bindings_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    let bindings: crate::value::pmap::PMap = match bindings_val {
        Value::wat__core__PersistentMap(ref m) => m.clone(),
        other => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::PersistentMap (token bindings)",
                got: Box::new(ValueSnapshot::of(&other)),
            }).into());
        }
    };

    // Interpreter / differential door. Native production runs `exec_compiled_rhs`.
    build_insert_fact(&form_ast, &bindings, sym)
}
