//! `:wat::rete::{pure?,deterministic?,total?,primitive?,vocabulary-admitted?,
//! cond-has-deferred-constraint?,alpha-match,alpha-match-local,alpha-match-under}` — arc 255
//! Stone P6-c-W5a, the P6-c campaign's fifth wave (5a): the READ-ONLY half of `:wat::rete::`'s
//! 28-verb giant-match surface — the six `?` predicates plus the three alpha-matchers.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-P6-c-W5a-rete-predicates.md`.
//!
//! Nine verbs, moved verbatim out of `runtime.rs`'s giant match with their real arities
//! declared (shim-owned; every hand-rolled `args.len() != N` guard this wave retires is named
//! in each fn's own doc below). The OTHER 19 `:wat::rete::` verbs (the session-mutating half —
//! `fire-*`, `insert-*`, `arm-session`, `release-session`, `import`, `export`, the `$native`
//! twins, plus `lower`/`collect-rules`/`step-payload`/`axis-violation`/`eval-test`/`eval-insert`)
//! are NOT this wave and stay in the giant match, per the brief's affirmative cut.
//!
//! ★ `:wat::rete::` is deliberately ABSENT from `effectful_by_prefix` (`src/runtime.rs`) — this
//! wave's whole premise is that these nine are read-only, so nothing here widens that list.
//! `declared_purity_vs_effectful_by_prefix_census` (`src/intrinsic/mod.rs`) would go RED the
//! moment any of the nine were wrongly declared `@Purity Effectful`; none is.
//!
//! ## The six `?` predicates
//!
//! `pure?`/`deterministic?`/`total?`/`primitive?` share one body shape (`eval_axis_predicate_impl`
//! below — the direct successor of `rete/purity.rs`'s now-deleted `eval_axis_predicate`): eval the
//! one call argument to a quoted `WatAST`, then run a read-only structural walk
//! (`is_pure_expr`/`is_deterministic_expr`/`is_total_expr`/`is_rete_primitive_expr`, still in
//! `rete/purity.rs`) over it plus a `&SymbolTable` reference — a FRESH cycle-guard `HashSet` per
//! call, no IO, no mutation, terminates even over a mutually-recursive user fn body.
//! `vocabulary-admitted?` (`rete_vocabulary_admitted`, `rete/vocabulary.rs`) is a fixed
//! prefix-table lookup on a string; `cond-has-deferred-constraint?`
//! (`cond_has_deferred_constraint`, `rete/matcher.rs`) is a finite structural walk over the
//! already-evaluated condition's clauses. All six: Pure, Deterministic, Total.
//!
//! ## The three alpha-matchers
//!
//! `alpha-match`/`alpha-match-local`/`alpha-match-under` eval their `cond`/`fact`(/`bindings`)
//! call arguments (ordinary call-by-value, not itself an effect) and then hand the
//! ALREADY-EVALUATED values to `alpha_match_inner`/`alpha_match_inner_local`/
//! `alpha_match_inner_seeded` (`rete/matcher.rs`) — each documented on itself as "the pure core:
//! no `Environment`, no `eval_inner`" — which reads the condition's clauses and the fact's field
//! slice structurally and either returns a binding array or `None` (Clara no-error: a
//! non-matching/malformed condition is a miss, never a raise). The one call inside that core
//! worth naming, `crate::rete::kernel::census_count`, is a `#[cfg(test)]`-only thread-local
//! counter gated further by an explicit `with_count_census` opt-in; under `#[cfg(not(test))]`
//! (every release build) it is a literal empty-body no-op — it changes no observable behavior or
//! return value on any input, in test or release. All three: Pure, Deterministic, Total. See
//! `src/rete/purity.rs`'s `intrinsic_meta` for where these nine rulings actually live (the
//! `@Purity`/`@Determinism` tags below are this REGISTRY's copy of the same fact, independently
//! grounded on the same bodies — not derived from one another).

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::rete::matcher::{
    alpha_match_inner, alpha_match_inner_local, alpha_match_inner_seeded, attach_fact_bind,
    class_field_names, cond_has_deferred_constraint, fact_from_value, pack_alpha_match_option,
};
use crate::rete::purity::{
    is_deterministic_expr, is_pure_expr, is_rete_primitive_expr, is_total_expr,
};
use crate::rete::vocabulary::rete_vocabulary_admitted;
use crate::runtime::eval_inner;
use crate::value::{Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};

/// Shared body for the four single-arg WatAST axis predicates: eval `expr` to a quoted
/// `WatAST`, apply `classify`. Direct successor of `rete/purity.rs`'s deleted
/// `eval_axis_predicate` — same shape, arity now shim-owned rather than hand-checked here.
fn eval_axis_predicate_impl(
    op: &'static str,
    classify: fn(&WatAST, &SymbolTable) -> bool,
    expr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let val = eval_inner(expr, env, sym)?.value_owned();
    let ast = match val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(
                expr.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::bool(classify(&ast, sym)))
}

/// `(:wat::rete::pure? expr) -> :wat::core::bool` — is `expr` effect-free (no IO/mutation/spawn)?
///
/// Arc 278 Stone 6a — one of the rete condition fence's four conjuncts (pure ∧ deterministic ∧
/// total ∧ primitive?). `:wat::uuid::v4` is pure (it does no IO) despite being random — see
/// `deterministic?` for the axis that catches that. Default-deny: proven by intrinsic metadata
/// or a transitive user fn walk; everything else is refused.
///
/// Arity was previously a hand-rolled `args.len() != 1` check inside `rete/purity.rs`'s shared
/// `eval_axis_predicate` (arc 255 Stone P6-c-W5a retired it); now the shim's real declared arity.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     expr :wat::WatAST the quoted expression form (from `:wat::core::quote`), walked structurally, never evaluated
/// @ret     :wat::core::bool whether every head in `expr`'s transitive walk is effect-free
/// @example (:wat::rete::pure? (:wat::core::quote (:wat::core::+ 1 2))) #=> true
#[wat_intrinsic(":wat::rete::pure?")]
pub(crate) fn eval_rete_pure_intrinsic(
    expr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_axis_predicate_impl(":wat::rete::pure?", is_pure_expr, expr, env, sym)
}

/// `(:wat::rete::deterministic? expr) -> :wat::core::bool` — is `expr` referentially transparent
/// (same inputs → same output)?
///
/// Arc 278 Stone 6a — the second fence conjunct. `:wat::uuid::v4` is NOT deterministic (random)
/// even though it is pure — the two axes are genuinely orthogonal. Same walk, same
/// `OpMeta`/default-deny discipline as `pure?`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     expr :wat::WatAST the quoted expression form, walked structurally, never evaluated
/// @ret     :wat::core::bool whether every head in `expr`'s transitive walk is referentially transparent
/// @example (:wat::rete::deterministic? (:wat::core::quote (:wat::uuid::v4))) #=> false
#[wat_intrinsic(":wat::rete::deterministic?")]
pub(crate) fn eval_rete_deterministic_intrinsic(
    expr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_axis_predicate_impl(":wat::rete::deterministic?", is_deterministic_expr, expr, env, sym)
}

/// `(:wat::rete::total? expr) -> :wat::core::bool` — is `expr` domain-total (defined on ALL its
/// inputs, not merely effect-free and referentially transparent)?
///
/// Arc 278 Stone 6a / BRIEF-total-t1-the-axis-unarmed.md — the third fence conjunct, ARMED:
/// `compile-condition` (`wat/rete/compile.wat`) consults this directly as the third conjunct.
/// `first`/`i64::/`/`i64::mod` are all pure AND deterministic yet **partial** (undefined on an
/// empty vector / a zero divisor) — `total?` is the axis that catches what the other two cannot.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     expr :wat::WatAST the quoted expression form, walked structurally, never evaluated
/// @ret     :wat::core::bool whether every head in `expr`'s transitive walk is defined on all its inputs
/// @example (:wat::rete::total? (:wat::core::quote (:wat::core::= 1 1))) #=> true
#[wat_intrinsic(":wat::rete::total?")]
pub(crate) fn eval_rete_total_intrinsic(
    expr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_axis_predicate_impl(":wat::rete::total?", is_total_expr, expr, env, sym)
}

/// `(:wat::rete::primitive? expr) -> :wat::core::bool` — LAW A: is `expr` composed ONLY of rete
/// primitives, at any depth?
///
/// Arc 278 #57 — the fourth fence conjunct, the builder's law: "the entire rete query language
/// may only be composed from rete primitives." Named `primitive?` rather than `rete-primitive?`
/// because the namespace already says rete, exactly as `pure?` is not `rete-pure?`. A
/// core-spelled structural-guard form (`:wat::core::cond`/`match`/`fn`) is REFUSED here even
/// though `pure?`/`deterministic?`/`total?` accept it over ordinary code — admission is a
/// separate, stricter question than purity, and only the rete-namespaced twin
/// (`:wat::rete::core::cond`) is admitted.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     expr :wat::WatAST the quoted expression form, walked structurally, never evaluated
/// @ret     :wat::core::bool whether every head in `expr`'s transitive walk is a rete primitive
/// @example (:wat::rete::primitive? (:wat::core::quote (:wat::rete::core::cond (true 1) (:else 2)))) #=> true
#[wat_intrinsic(":wat::rete::primitive?")]
pub(crate) fn eval_rete_primitive_intrinsic(
    expr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_axis_predicate_impl(":wat::rete::primitive?", is_rete_primitive_expr, expr, env, sym)
}

/// `(:wat::rete::vocabulary-admitted? head) -> :wat::core::bool` — THE ADMISSION TEST surfaced
/// for wat callers: does `head` fall inside a declared rete-vocabulary sub-namespace
/// (`RETE_MODULES`, `rete/vocabulary.rs`)?
///
/// Arc 278 #55 slice one. Decoupled from `pure?`/`deterministic?`/`total?`/`primitive?`, which
/// classify an EXPRESSION; this classifies a HEAD NAME against the module-set boundary alone,
/// independent of whether that head is pure. Not consulted by `compile-condition` — the fence's
/// Law A check is `primitive?`. Takes a QUOTED keyword, mirroring the four predicates' own
/// `:wat::WatAST` argument shape — NOT a bare `:wat::core::keyword` value: a bare keyword literal
/// naming a REGISTERED function resolves at check time to that function's `Fn` type, not a
/// `:wat::core::keyword` value, so an unquoted head name cannot reach this predicate as data.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     head :wat::WatAST a `:wat::WatAST` holding a quoted Keyword (a head name), from `:wat::core::quote`
/// @ret     :wat::core::bool whether `head` falls inside a declared rete-vocabulary sub-namespace
/// @example (:wat::rete::vocabulary-admitted? (:wat::core::quote :wat::rete::core::cond)) #=> true
#[wat_intrinsic(":wat::rete::vocabulary-admitted?")]
pub(crate) fn eval_rete_vocabulary_admitted_intrinsic(
    head: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::vocabulary-admitted?";
    let val = eval_inner(head, env, sym)?.value_owned();
    let name = match val {
        Value::wat__WatAST(ref a) => match a.as_ref() {
            WatAST::Keyword(k, _) => k.clone(),
            other => {
                return Err(RuntimeError::new(
                    head.span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: ":wat::WatAST holding a Keyword (a quoted head name)",
                        got: Box::new(ValueSnapshot::of(&Value::String(std::sync::Arc::new(
                            format!("{other:?}"),
                        )))),
                    },
                )
                .into());
            }
        },
        other => {
            return Err(RuntimeError::new(
                head.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (a quoted keyword from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::bool(rete_vocabulary_admitted(&name)))
}

/// `(:wat::rete::cond-has-deferred-constraint? cond) -> :wat::core::bool` — does an inline
/// constraint in `cond` name a `?var` this condition does not bind?
///
/// Arc 278 Stone 3-ish (beta prep). Those constraints are cross-condition join keys
/// (`?v < ?m` after an accum, say) — the empty-seed alpha matcher would wrongly treat them as an
/// unbound-var mismatch; `:not`/`:exists` instead re-check the full condition at beta time via
/// `alpha-match-under`, seeded with the token's left-accumulated bindings. Purely structural: it
/// never resolves `cond`'s type head against the type registry, so a fictional/undeclared type
/// name in `cond` does not stop it from answering.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     cond :wat::WatAST the quoted condition form (from `:wat::core::quote`), walked structurally, never evaluated
/// @ret     :wat::core::bool whether an inline constraint in `cond` references a `?var` not bound by `cond` itself
/// @example (:wat::rete::cond-has-deferred-constraint? (:wat::core::quote (:some::Type (?t <- :value) (:wat::rete::i64::> ?t ?m)))) #=> true
#[wat_intrinsic(":wat::rete::cond-has-deferred-constraint?")]
pub(crate) fn eval_rete_cond_has_deferred_constraint_intrinsic(
    cond: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::cond-has-deferred-constraint?";
    let cond_val = eval_inner(cond, env, sym)?.value_owned();
    let cond_ast = match cond_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(
                cond.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (condition form)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::bool(cond_has_deferred_constraint(&cond_ast)))
}

/// Shared body for `alpha-match`/`alpha-match-local` — the only difference between the two wat
/// verbs is whether the empty-seed matcher DEFERS (`local = true`) or REJECTS (`local = false`) a
/// constraint naming a `?var` this condition does not bind. Direct successor of
/// `rete/matcher.rs`'s deleted `eval_alpha_match_kind` — same shape, arity now shim-owned.
fn eval_alpha_match_kind_impl(
    op: &'static str,
    cond: &WatAST,
    fact: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    local: bool,
) -> Result<Value, EvalBreak> {
    let cond_val = eval_inner(cond, env, sym)?.value_owned();
    let cond_ast = match cond_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(
                cond.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: ":wat::WatAST (condition form from quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    let fact_val = eval_inner(fact, env, sym)?.value_owned();
    let f = match fact_from_value(&fact_val) {
        Some(f) => f,
        None => {
            return Err(RuntimeError::new(
                fact.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: ":wat::core::Record (a record fact)",
                    got: Box::new(ValueSnapshot::of(&fact_val)),
                },
            )
            .into());
        }
    };

    let field_names = class_field_names(sym, f.class_fqdn);

    let matched = if local {
        alpha_match_inner_local(&cond_ast, f.class_fqdn, f.fields, &field_names)
    } else {
        alpha_match_inner(&cond_ast, f.class_fqdn, f.fields, &field_names)
    };
    let result = matched.map(|b| attach_fact_bind(&cond_ast, &fact_val, b));
    pack_alpha_match_option(result)
}

/// `(:wat::rete::alpha-match cond fact) -> (:wat::core::Option :- [(:wat::core::PersistentMap :- [:wat::core::String V])])`
///
/// Arc 278 Stone 2a — the rete single-fact matcher. `cond` arrives as DATA (a quoted AST, never
/// `eval_inner`'d past this point); `fact` is a `:wat::core::Record` (a `Value::Aggregate` whose
/// nature is Record or HolonRecord — a Struct is refused, Clara semantics: wrong fact shape is a
/// type error, not a silent miss). `Some(bindings)` iff the fact's class matches the condition
/// head AND every clause holds; `None` otherwise (Clara no-error — a failed CONSTRAINT is never a
/// raise). Bindings key logic-var name strings (`"?t"`) to their field-typed values.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     cond :wat::WatAST the quoted condition form `(:Type clause…)` (from `:wat::core::quote`)
/// @arg     fact :wat::core::Record the fact to test the condition against
/// @ret     (:wat::core::Option :- [(:wat::core::PersistentMap :- [:wat::core::String V])]) `Some(bindings)` on a match, `None` on any mismatch
/// @example (:wat::core::do (:wat::core::defrecord :probe::AlphaMatchTemp [value <- :wat::core::i64]) (:wat::rete::alpha-match (:wat::core::quote (:probe::AlphaMatchTemp (?t <- :value) (:wat::rete::i64::> ?t 20))) (:probe::AlphaMatchTemp :value 25))) #=> (:wat::core::Some (:wat::core::PersistentMap "?t" 25))
#[wat_intrinsic(":wat::rete::alpha-match")]
pub(crate) fn eval_rete_alpha_match_intrinsic(
    cond: &WatAST,
    fact: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_alpha_match_kind_impl(":wat::rete::alpha-match", cond, fact, env, sym, false)
}

/// `(:wat::rete::alpha-match-local cond fact) -> (:wat::core::Option :- [(:wat::core::PersistentMap :- [:wat::core::String V])])`
///
/// Same matcher as `alpha-match`, empty-seeded, but a `?var` in an inline constraint that this
/// condition does NOT bind is DEFERRED (treated as a pass, not a mismatch) rather than rejected —
/// `:wat::rete::cond-has-deferred-constraint?` names exactly which conditions this changes
/// anything for. Those facts still enter alpha; `:not`/`:exists` re-check the full condition
/// against the token's left-accumulated bindings at beta time via `alpha-match-under`. Join
/// alphas must not use this variant — a deferred join constraint would be lost.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     cond :wat::WatAST the quoted condition form `(:Type clause…)` (from `:wat::core::quote`)
/// @arg     fact :wat::core::Record the fact to test the condition against
/// @ret     (:wat::core::Option :- [(:wat::core::PersistentMap :- [:wat::core::String V])]) `Some(bindings)` on a match, `None` on any mismatch
/// @example (:wat::core::do (:wat::core::defrecord :probe::AlphaMatchLocalTemp [value <- :wat::core::i64]) (:wat::rete::alpha-match-local (:wat::core::quote (:probe::AlphaMatchLocalTemp (?t <- :value) (:wat::rete::i64::> ?t 20))) (:probe::AlphaMatchLocalTemp :value 25))) #=> (:wat::core::Some (:wat::core::PersistentMap "?t" 25))
#[wat_intrinsic(":wat::rete::alpha-match-local")]
pub(crate) fn eval_rete_alpha_match_local_intrinsic(
    cond: &WatAST,
    fact: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_alpha_match_kind_impl(":wat::rete::alpha-match-local", cond, fact, env, sym, true)
}

/// `(:wat::rete::alpha-match-under cond fact bindings) -> (:wat::core::Option :- [(:wat::core::PersistentMap :- [:wat::core::String V])])`
///
/// Same matcher as `alpha-match`, but `bindings` (a token's left-accumulated `?var`s) SEED the
/// clause fold instead of starting empty. Used by the oracle `:not`/`:exists` filter so a
/// constraint naming a left-bound var (`?v < ?m` after an accum) is checked as a real beta
/// constraint, not silently lost as an alpha miss.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     cond :wat::WatAST the quoted condition form `(:Type clause…)` (from `:wat::core::quote`)
/// @arg     fact :wat::core::Record the fact to test the condition against
/// @arg     bindings (:wat::core::PersistentMap :- [:wat::core::String V]) the token's already-bound `?var`s, seeding the clause fold
/// @ret     (:wat::core::Option :- [(:wat::core::PersistentMap :- [:wat::core::String V])]) `Some(bindings)` (seed plus any new binds) on a match, `None` on any mismatch
/// @example (:wat::core::do (:wat::core::defrecord :probe::AlphaMatchUnderTemp [value <- :wat::core::i64]) (:wat::rete::alpha-match-under (:wat::core::quote (:probe::AlphaMatchUnderTemp (?p <- :value) (:wat::rete::i64::> ?p ?m))) (:probe::AlphaMatchUnderTemp :value 25) (:wat::core::PersistentMap "?m" 20))) #=> (:wat::core::Some (:wat::core::PersistentMap "?m" 20 "?p" 25))
#[wat_intrinsic(":wat::rete::alpha-match-under")]
pub(crate) fn eval_rete_alpha_match_under_intrinsic(
    cond: &WatAST,
    fact: &WatAST,
    bindings: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::alpha-match-under";

    let cond_val = eval_inner(cond, env, sym)?.value_owned();
    let cond_ast = match cond_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(
                cond.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (condition form from quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    let fact_val = eval_inner(fact, env, sym)?.value_owned();
    let f = match fact_from_value(&fact_val) {
        Some(f) => f,
        None => {
            return Err(RuntimeError::new(
                fact.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::core::Record (a record fact)",
                    got: Box::new(ValueSnapshot::of(&fact_val)),
                },
            )
            .into());
        }
    };

    let binds_val = eval_inner(bindings, env, sym)?.value_owned();
    let seed: Vec<(Value, Value)> = match &binds_val {
        Value::wat__core__PersistentMap(pm) => {
            pm.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        }
        other => {
            return Err(RuntimeError::new(
                bindings.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::core::PersistentMap (token bindings)",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };

    let field_names = class_field_names(sym, f.class_fqdn);

    let result = alpha_match_inner_seeded(&cond_ast, f.class_fqdn, f.fields, &field_names, &seed)
        .map(|b| attach_fact_bind(&cond_ast, &fact_val, b));
    pack_alpha_match_option(result)
}
