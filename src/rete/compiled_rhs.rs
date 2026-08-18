//! Arc 278 — DESIGN-STONE-compiled-rhs.md: the RHS compiles once; `build_insert_fact` stops
//! re-deriving a static program.
//!
//! ## ⚠ Not a perf stone (mirrors `compiled_cond.rs`'s amendment)
//!
//! `build_insert_fact` (`matcher.rs`) re-derives its program from a `WatAST` on **every derived
//! fact**: it re-validates the `(:Type arg…)` fact-form shape, re-detects
//! kwargs-vs-positional, re-allocates the class `String`, and — via `resolve_operand` — rebuilds
//! each `?var` lookup key with a fresh `Value::String(Arc::new(name.to_string()))`, a `String`
//! allocation plus an `Arc` allocation, for a key fixed at rule-compile time. Measured on the
//! fanout cell: 120,000 key allocations, exactly 3.00 per derived fact (240,000 heap allocations
//! rebuilding three constants in one fire). The point, as with `compiled_cond.rs`, is correctness
//! — a static program should not be re-derived dynamically — not a timing win.
//!
//! ## ★ THE ONE CONTRACT DECISION (DESIGN-STONE-compiled-rhs.md)
//!
//! **Compile per RULE at setup, exactly where `compiled_conds` is already built; the produced
//! `Value` is byte-identical** to what `build_insert_fact` would produce for the same
//! `(insert_form, bindings)`. Per derived fact the whole function becomes: walk `ops`,
//! `bindings.get(k).cloned()` or `v.clone()`, build the record. Nothing else.
//!
//! This eliminates, per fact: the form validation, the kwargs detection, the class allocation
//! (the class `String` itself is still allocated once per fact for `AggregateValue::record`,
//! which takes an owned `String` — interning it is a different, out-of-scope stone), and both key
//! allocations per field. It keeps, because they are irreducible: N trie lookups, N `Arc` bumps,
//! the fields `Vec`, and the `AggregateValue`.
//!
//! `build_insert_fact` is **NOT deleted**: it stays as the reference implementation and the other
//! half of the differential, exactly as `alpha_match_inner` did for the LHS (`compiled_cond.rs`).
//!
//! ## Arc 278 Stone B (DESIGN-STONE-then-is-a-vector-of-singular-facts.md § "Stone B") — the
//! third `RhsOp`, and what it does NOT claim
//!
//! Flip 4 (CURRENT-STATE): widening (b) is a compiled `Program` on the one
//! `Expr` core — `lower` once at setup, `exec_value` per derived fact. The
//! interpreter (`build_insert_fact` / `eval_rhs_expr`) remains the other half
//! of the differential. A `LowerError` falls the whole form back to
//! `build_insert_fact` (same door as a fn-headed item). Widening (a) (an
//! item's head may be a fn) is still not represented — `compile_rhs` returns
//! `None` for a fn-headed item (`BRIEF-then-user-forms.md` STOP-3).

use std::sync::Arc;

use crate::ast::WatAST;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, ValueSnapshot};
use crate::value::value::AggregateValue;
use crate::runtime::Value;

/// One resolved RHS field — a `?var` lookup (key pre-built once, at compile time), a bare
/// literal (built once), or (arc 278 Stone B, widening (b)) a fenced call-form EXPRESSION
/// captured once as `WatAST`, un-shape-detected again. Any value-position AST node that is none
/// of these (a bare non-`?` symbol, a `:field` keyword reference — RHS has no current fact — or a
/// Vector/Map/Set literal) makes [`compile_rhs`] return `None` for the WHOLE form, falling back
/// to `build_insert_fact` for that form rather than inventing a shape this stone does not claim.
#[derive(Clone, Debug)]
pub(crate) enum RhsOp {
    /// `?var` — the PRE-BUILT `Value::String` key, never rebuilt per fact. Execution does
    /// `bindings.get(&key).cloned()`.
    ///
    /// The second field is the operand AST's debug rendering, built ONCE here so the unbound-var
    /// error this op can raise is **byte-identical** to `build_insert_fact`'s. That matters
    /// because the arm is REACHABLE: `--check` on a rule whose `:then` names a `?var` its `:when`
    /// never binds exits 0 (`validate_and_reorder_then` validates the SHAPE — insert head, fact type, field names, positional arity — but never inspects the value-position OPERANDS, so an unbound `?var` and a nested form both pass), so the failure surfaces at fire time. A diagnostic that changes text
    /// depending on which internal path happened to run is a difference the caller can see and
    /// cannot explain; the design's "same SHAPE of error" was too loose a contract, and this is
    /// the correction.
    Bind(Value, String),
    /// A literal value, built once at compile time.
    Lit(Value),
    /// Flip 4 — a value-position call form, lowered once onto the one `Expr` core.
    /// The freeze-time wat fence (`then-item-fence`) has already proven it pure ∧
    /// deterministic and rete-namespaced-or-composed. `lower` failing returns
    /// `None` for the whole form (same door as a fn-headed item): the fire path
    /// falls back to `build_insert_fact`. Executed via `exec_value` — prologue
    /// is token bindings → slots; the `Value` becomes the field.
    Expr(Arc<crate::rete::expr_ir::Program>),
}

/// An insert-form compiled once, at setup, from the immutable rule set — the pre-resolved dual of
/// `build_insert_fact`. Built by [`compile_rhs`]; run by [`exec_compiled_rhs`].
pub(crate) enum CompiledRhs {
    /// `(:Type field…)` constructor.
    Record {
        class: String,
        names: Arc<Vec<String>>,
        ops: Vec<RhsOp>,
    },
    /// Fn-headed `:then` item — the whole form is one `Program`.
    Call(Arc<crate::rete::expr_ir::Program>),
}

/// Compile one `(:Type arg…)` fact-form (arc 278 Stone A: no more `:wat::rete::insert`
/// wrapper). All the validation `build_insert_fact` does per fact — fact-form shape,
/// kwargs-vs-positional detection, and classifying each value-position AST node — happens
/// HERE, once.
///
/// Arc 278 Stone B, widening (a) — takes `sym` now, SOLELY to defend against miscompiling a
/// fn-call item: `head` must name a known aggregate type (`sym.types()`, the SAME registry
/// `build_insert_fact` reads at fire time) or this returns `None` immediately, deferring the
/// WHOLE item to `build_insert_fact`'s fn-call branch. Without this check, a fn-call item whose
/// args happen to all be `?var`/literal (e.g. `(:usr::make-rate ?c ?w)`) would silently fall
/// through the OLD per-arg loop below and be miscompiled as a 2-field record of class
/// `"usr::make-rate"` — a wrong answer, not a raise. This stone's compiled fast path does not
/// implement the fn-call shape (`BRIEF-then-user-forms.md` STOP-3: "report it rather than
/// falling the whole form back to the interpreter silently" — reported here, in this doc comment
/// and the rider's own report, not silently): every fn-headed `:then` item runs interpreted.
///
/// Returns `Ok(None)` whenever `build_insert_fact` would either raise an error for this form (a
/// static, compile-time-provable property — the fallback then raises the identical error at
/// fire time) or contains a value-position node this stone's op model does not represent: a bare
/// non-`?` symbol, a `:field` keyword reference, or a Vector/Map/Set literal. A fenced `List`
/// that `lower` refuses is `Err` — the old form is gone; fire does not walk the WatAST.
pub(crate) fn compile_rhs(
    fact_form: &WatAST,
    sym: &SymbolTable,
) -> Result<Option<CompiledRhs>, EvalBreak> {
    let fact_items = match fact_form {
        WatAST::List(items, _) if !items.is_empty() => items.as_slice(),
        _ => return Ok(None),
    };
    let type_keyword = match &fact_items[0] {
        WatAST::Keyword(k, _) => k.as_str(),
        _ => return Ok(None),
    };
    let names = match sym.types().and_then(|t| t.get(type_keyword)) {
        Some(crate::types::TypeDef::Aggregate(a)) => a.names_arc(),
        _ => {
            // Widening (a) — fn-headed item. Lower the whole call. A LowerError
            // refuses (STOP-3): do not walk build_insert_fact_call.
            let program = crate::rete::expr_ir::lower(fact_form, sym)
                .map_err(crate::rete::expr_ir::LowerError::into_eval)?;
            return Ok(Some(CompiledRhs::Call(Arc::new(program))));
        }
    };
    let class = type_keyword.strip_prefix(':').unwrap_or(type_keyword).to_string();

    // Arc 294 item 9a — kwargs `(:Type :field1 v1 :field2 v2)` vs legacy positional
    // `(:Type v1 v2)`, exactly `build_insert_fact`'s detection.
    let args = &fact_items[1..];
    let is_kwargs = args.len() >= 2
        && args.len() % 2 == 0
        && args.iter().step_by(2).all(|a| matches!(a, WatAST::Keyword(_, _)));
    let value_asts: Vec<&WatAST> = if is_kwargs {
        args.iter().skip(1).step_by(2).collect()
    } else {
        args.iter().collect()
    };

    let mut ops: Vec<RhsOp> = Vec::with_capacity(value_asts.len());
    for arg in value_asts {
        let op = match arg {
            WatAST::Symbol(ident, _) if ident.as_str().starts_with('?') => RhsOp::Bind(
                Value::String(Arc::new(ident.as_str().to_string())),
                // Built once, at compile time — see the `Bind` doc. `build_insert_fact` renders
                // `format!("{arg:?}")` into its error's `got`; matching it exactly is what keeps
                // the two paths indistinguishable to a caller who hits the unbound-var case.
                format!("{arg:?}"),
            ),
            WatAST::IntLit(n, _) => RhsOp::Lit(Value::i64(*n)),
            WatAST::FloatLit(x, _) => RhsOp::Lit(Value::f64(*x)),
            WatAST::BoolLit(b, _) => RhsOp::Lit(Value::bool(*b)),
            WatAST::StringLit(s, _) => RhsOp::Lit(Value::String(Arc::new(s.clone()))),
            // Flip 4 — a fenced call form MUST lower. A LowerError is a fire
            // refuse, not a walk of the WatAST. `None` below is only the
            // unrepresentable shapes (fn-headed item, `:field`, Vector/Map/Set).
            WatAST::List(..) => {
                let program = crate::rete::expr_ir::lower(arg, sym)
                    .map_err(crate::rete::expr_ir::LowerError::into_eval)?;
                RhsOp::Expr(Arc::new(program))
            }
            // A bare non-`?` symbol, a `:field` keyword (RHS has no current fact), or a
            // Vector/Map/Set literal — `resolve_operand` would return `None` for these too, and
            // this stone's op model does not represent them either: fall back to
            // `build_insert_fact` for the WHOLE form, which raises the identical error at fire
            // time rather than miscompiling it.
            _ => return Ok(None),
        };
        ops.push(op);
    }

    Ok(Some(CompiledRhs::Record { class, names, ops }))
}

/// Execute a compiled RHS form against one token's bindings. Returns exactly what
/// `build_insert_fact(insert_form, bindings)` would for the SAME `(insert_form, bindings)`: same
/// class, same field values, same order (STOP-2's differential).
///
/// Raises a BYTE-IDENTICAL error to `build_insert_fact`'s when a `?var` this form references is
/// not bound in `bindings` — the only failure mode a validly-compiled program can hit at fire
/// time, and a REACHABLE one: `--check` on a rule whose `:then` names a `?var` its `:when` never
/// binds exits 0 (`validate_and_reorder_then` validates the SHAPE — insert head, fact type, field names, positional arity — but never inspects the value-position OPERANDS, so an unbound `?var` and a nested form both pass), so this
/// surfaces at fire time rather than at compile time. Never panics.
///
/// Flip 4 — `RhsOp::Expr` is a compiled `Program`. `sym` is still required
/// (`CallUser` / accessors). The interpreter (`build_insert_fact`) remains
/// the other half of the differential.
pub(crate) fn exec_compiled_rhs(
    c: &CompiledRhs,
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-insert";
    match c {
        CompiledRhs::Call(program) => {
            match crate::rete::expr_ir::exec_value(program, bindings, sym, &program.span)? {
                v @ Value::Aggregate(_) => Ok(v),
                other => Err(RuntimeError::new(
                    program.span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "the fn to return a fact (a Record/Struct) — the rule-compile fence should \
                                   have refused a non-fact return type",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into()),
            }
        }
        CompiledRhs::Record { class, names, ops } => {
            let mut fields: Vec<Value> = Vec::with_capacity(ops.len());
            for op in ops {
                let v = match op {
                    RhsOp::Bind(key, ast_debug) => match bindings.get(key) {
                        Some(v) => v.clone(),
                        None => {
                            return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                                op: OP.into(),
                                expected: "resolvable operand (?var, literal, or a fenced expression) in RHS fact-form",
                                got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(ast_debug.clone())))),
                            }).into());
                        }
                    },
                    RhsOp::Lit(v) => v.clone(),
                    RhsOp::Expr(program) => {
                        crate::rete::expr_ir::exec_value(program, bindings, sym, &program.span)?
                    }
                };
                fields.push(v);
            }
            Ok(Value::Aggregate(Arc::new(AggregateValue::record(
                class.clone(),
                names.clone(),
                Arc::new(fields),
            ))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::EvalBreak;
    // arc 298.3 — `RuntimeErrorKind` derives `wat_edn::ToEdn`; the trait must be in scope to call
    // `.to_edn()`. This is the span-free half of `RuntimeError::to_edn()`
    // (`splice_span(kind.to_edn(), span)`), which is exactly why the comparison below uses it.
    use wat_edn::ToEdn;

    /// ★ THE DIFFERENTIAL ON THE **`Result`** — the test this arc did not have.
    ///
    /// Every differential in arc 278 compares a SUCCESS: `native == oracle` compares derived fact
    /// SETS, `compiled_cond`'s compares BINDINGS ON MATCH, and `alpha_match_inner` returns
    /// `Option`, so the LHS pair never had an error to compare and the habit was never formed.
    /// Consequence, found by hand on 2026-08-01 rather than by any gate: the compiled RHS shipped
    /// an unbound-`?var` error whose `got` differed from `build_insert_fact`'s, and the floor was
    /// 4241/4241 green through it — because **nothing anywhere fires a rule with an unbound `?var`
    /// in `:then`**, and nothing compares two impls' failures. That arm is REACHABLE: `--check` on
    /// such a rule exits 0 (`validate_and_reorder_then` validates the SHAPE — insert head, fact type, field names, positional arity — but never inspects the value-position OPERANDS, so an unbound `?var` and a nested form both pass), so it surfaces at fire time.
    ///
    /// **Spans are excluded on purpose, and that is not a loosening.** `RuntimeError::new` stamps
    /// `rust_caller_span!()`, so one error is raised in `matcher.rs` and the other in this file;
    /// they can never be byte-equal and should not be. What must match is the KIND — `op`,
    /// `expected`, `got` — compared through `RuntimeErrorKind`'s `wat_edn::ToEdn` derive (arc
    /// 298.3), which carries no span. Structural, not a `contains`.
    ///
    /// The third arm is the one that matters most: **one path succeeding while the other fails**
    /// is a worse defect than differing text, and nothing today would have noticed that either.
    #[test]
    fn compiled_rhs_result_identical_to_interpreter() {
        fn bindings_of(pairs: &[(&str, i64)]) -> crate::value::pmap::PMap {
            crate::value::pmap::PMap::from_pairs(
                pairs.iter().map(|(k, v)| (Value::String(Arc::new((*k).to_string())), Value::i64(*v))),
            )
        }
        fn kind_edn(e: &EvalBreak) -> String {
            match e {
                EvalBreak::Diagnostic(re) => wat_edn::write(&re.kind().to_edn()),
                other => panic!("expected a Diagnostic, got {other:?}"),
            }
        }

        // Arc 278 Stone B — `compile_rhs` now consults `sym.types()` to tell a fact-type
        // constructor head from a fn head (see `compile_rhs`'s doc). Build a REAL SymbolTable
        // through the freeze pipeline (`build_env`) rather than a bare/empty one, so `:fan::Pair`
        // resolves as a registered Aggregate and the compiled fast path is actually exercised —
        // an empty `SymbolTable::default()` would make every case here defer to the interpreter
        // and silently stop testing the compiled path at all.
        let type_src = r#"
(:wat::core::defrecord :fan::Pair
  [key <- :wat::core::i64
   lid <- :wat::core::i64
   rid <- :wat::core::i64])
"#;
        let forms = crate::parse_all!(type_src).expect("parse the fixture record decl");
        let sym = crate::freeze::env::build_env(forms).expect("build_env").symbols;

        let bound = bindings_of(&[("?k", 1), ("?l", 2), ("?r", 3)]);
        let cases: &[(&str, &crate::value::pmap::PMap)] = &[
            // 1. positional, every ?var bound
            ("(:fan::Pair ?k ?l ?r)", &bound),
            // 2. kwargs — the arc-294 9a form
            ("(:fan::Pair :key ?k :lid ?l)", &bound),
            // 3. every literal kind, mixed with a bind
            ("(:fan::Pair ?k 7 true)", &bound),
            ("(:fan::Pair 1.5 \"s\" ?r)", &bound),
            // 4. ★ AN UNBOUND ?var — RED before the byte-identical error fix
            ("(:fan::Pair ?k ?nope ?r)", &bound),
            // 5. Arc 278 Stone B, widening (b) — a fenced EXPRESSION operand (`RhsOp::Expr`),
            //    composed of an admitted rete op. This USED to be shape-5 below (the two-op
            //    model's rejection case); a `List` is no longer rejected.
            ("(:fan::Pair (:wat::rete::core::i64::+ ?k 1 :undefined 0) ?l ?r)", &bound),
            // 6. a fenced expression that is itself unbound underneath — proves the Expr op's
            //    error path (not just its success path) is exercised, and stays byte-identical.
            ("(:fan::Pair (:wat::rete::core::i64::+ ?nope 1 :undefined 0) ?l ?r)", &bound),
        ];

        for (src, binds) in cases {
            let ast = crate::parse_one!(*src).unwrap_or_else(|e| panic!("parse {src}: {e:?}"));
            let compiled = match compile_rhs(&ast, &sym) {
                Ok(Some(c)) => c,
                Ok(None) => panic!("{src} unexpectedly failed to compile — a case here regressed"),
                Err(e) => panic!("{src} lower refused a fenced then-expr: {e:?}"),
            };
            match (exec_compiled_rhs(&compiled, binds, &sym), crate::rete::matcher::build_insert_fact(&ast, binds, &sym)) {
                (Ok(a), Ok(b)) => assert_eq!(
                    a, b,
                    "compiled and interpreted RHS produced DIFFERENT facts for {src}"
                ),
                (Err(a), Err(b)) => assert_eq!(
                    kind_edn(&a),
                    kind_edn(&b),
                    "compiled and interpreted RHS produced different DIAGNOSTICS for {src} — \
                     the caller can see which internal path ran, which it must not be able to"
                ),
                (a, b) => panic!(
                    "one path succeeded and the other failed for {src} — the worst of the three \
                     outcomes, and the one no existing differential would catch.\n\
                     compiled: {:?}\ninterpreted: {:?}",
                    a.is_ok(),
                    b.is_ok()
                ),
            }
        }
    }
}
