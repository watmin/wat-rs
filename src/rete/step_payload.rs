//! Explain `DerivationStep` payload — `eval_step_payload`.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::rete::kernel::{alpha_cond_of, session_network};
use crate::rete::clause::{classify_constraint_head, classify_rete_clause, ReteClauseShape};
use crate::rete::matcher::{
    alpha_pattern, class_field_names, fact_from_value, resolve_operand, value_to_ast_literal,
    FieldNames,
};
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::span::Span;
use crate::value::value::AggregateValue;
use std::sync::Arc;

// ─── P12c: step-payload ───────────────────────────────────────────────────────

/// `(:wat::rete::step-payload session alpha-id bindings sfact supporting) -> :wat::rete::DerivationStep`
///
/// Arc 278 Stone P12c — the explain payload builder. Given one (sfact, alpha-id) match edge
/// from a Token's matches chain, builds the full `DerivationStep` payload:
///
/// - **pattern**: the matched condition's fact-type FQDN (AlphaNode tests[0] head keyword).
/// - **bindings** (per-step): the binder-clause vars that THIS condition bound, projected
///   from the token's accumulated bindings.
/// - **constraints**: the rule's satisfied predicates with bound values substituted:
///   `(:wat::rete::i64::< -5 0)` from `(:wat::rete::i64::< ?c 0)` with `?c=-5`.
///
/// **Faithfulness by construction**: `classify_rete_clause` + `resolve_operand` reconstruct
/// the matched clause for the payload. Native fire matches via `exec_compiled_with_key_ids`
/// (STOP-1), not `alpha_match_inner` (the oracle). Substituted values still cannot drift
/// from the classifier's spelling of what matched.
///
/// Arc 255 Stone P6-c-W5c — moved verbatim into `#[wat_intrinsic]` with its real (5) arity
/// declared; the hand-rolled `args.len() != 5` guard this wave retires lived right here.
///
/// **Purity ground:** all five args are evaluated by ordinary call-by-value (not itself an
/// effect — the same shape `alpha-match`'s wrapper is Pure for). Past that, the body only reads
/// already-evaluated values: `session_network`/`alpha_cond_of` read the session's compiled
/// network, `classify_rete_clause`/`resolve_operand` are pure structural walks (the same ones
/// `step-payload`'s own doc calls "faithfulness by construction"), and the two `OnceLock`s
/// (`STEP_CLASS_FQDN`, `derivation_step_names`) cache a fixed, compile-time-constant class name
/// and field-name table process-wide — the same boilerplate pattern already used by `export.rs`
/// (Effectful) and `purity.rs`'s pure axis predicates alike, so it is infrastructure, not a
/// per-call effect. No `eval_inner`/`apply_function` on caller-supplied code anywhere in this
/// body. The built `DerivationStep` record is freshly allocated and returned to the caller;
/// nothing outlives the call beyond the two process-wide constant caches.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     session :wat::rete::Session the compiled session (network read via `session_network`)
/// @arg     alpha_id :wat::core::i64 the AlphaNode id for this condition
/// @arg     bindings :wat::core::PersistentMap the token's accumulated bindings
/// @arg     sfact :wat::core::Record the supporting fact for this edge
/// @arg     supporting :wat::rete::DerivationNode the pre-computed recursive node for `sfact`
/// @ret     :wat::rete::DerivationStep the per-edge explain payload (pattern, per-step bindings, substituted constraints, supporting)
/// @example (:wat::core::do (:wat::core::defrecord :probe::StepPayloadExampleTemp [celsius <- :wat::core::i64]) (:wat::core::defrecord :probe::StepPayloadExampleResult [celsius <- :wat::core::i64]) (:wat::rete::defrule :probe::step-payload-example-rule :when [(:probe::StepPayloadExampleTemp (?c <- :celsius) (:wat::rete::i64::< ?c 20))] :then [(:probe::StepPayloadExampleResult ?c)]) (:wat::core::let [rules (:wat::rete::collect-rules :probe) session (:wat::rete::compile rules) session (:wat::rete::insert session (:probe::StepPayloadExampleTemp :celsius 10)) ex (:wat::rete::fire-rules-explain session) support (:wat::rete::Explained/support ex) result (:probe::StepPayloadExampleResult :celsius 10) sv (:wat::core::Option/expect (:wat::map::get support result) "sv") tok (:wat::rete::Support/token sv) matches (:wat::rete::Token/matches tok) bindings (:wat::rete::Token/bindings tok) m0 (:wat::core::Option/expect (:wat::core::get matches 0) "m0") sfact (:wat::core::first m0) alpha-id (:wat::core::second m0) sess (:wat::rete::Explained/session ex) supporting (:wat::rete::explain ex sfact)] (:wat::rete::step-payload sess alpha-id bindings sfact supporting))) #=> (:wat::rete::DerivationStep :supporting (:wat::rete::DerivationNode :fact (:probe::StepPayloadExampleTemp :celsius 10) :rule :wat::core::None :via (:wat::core::PersistentVector)) :pattern "probe::StepPayloadExampleTemp" :bindings (:wat::core::PersistentMap "?c" 10) :constraints (:wat::core::PersistentVector (:wat::core::quote (:wat::rete::i64::< 10 20))))
#[wat_intrinsic(":wat::rete::step-payload")]
// arc 255 Stone P6-c-W5c — the SECOND 5-arg verb the registry has carved, and the same
// arithmetic as the first: 5 wat args + the `env`/`sym`/`list_span` context tail = 8, over
// clippy's 7. `#[expect]`, not `#[allow]`, so it goes RED the moment it stops being needed —
// see `src/intrinsic/kernel/resource.rs:411` (`spawn-process`), whose comment carries the full
// derivation and calls itself "the first"; it is now one of two. EARNED, not unfinished: the
// count is imposed by the `#[wat_intrinsic]` ABI, and the alternatives are changing that ABI or
// declining to register verbs above arity 4 — which would mean keeping the fictional variadic
// arity this whole campaign exists to retire.
#[expect(clippy::too_many_arguments)]
pub(crate) fn eval_step_payload(
    session: &WatAST,
    alpha_id: &WatAST,
    bindings: &WatAST,
    sfact: &WatAST,
    supporting: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::step-payload";

    // ── Evaluate all 5 arguments ──────────────────────────────────────────────
    let session_span  = session.span().clone();
    let alpha_id_span = alpha_id.span().clone();
    let bindings_span = bindings.span().clone();
    let sfact_span    = sfact.span().clone();

    let session_val  = crate::runtime::eval_inner(session, env, sym)?.value_owned();
    let alpha_id_val = crate::runtime::eval_inner(alpha_id, env, sym)?.value_owned();
    let bindings_val = crate::runtime::eval_inner(bindings, env, sym)?.value_owned();
    let sfact_val    = crate::runtime::eval_inner(sfact, env, sym)?.value_owned();
    let supporting   = crate::runtime::eval_inner(supporting, env, sym)?.value_owned();

    // ── Extract alpha_id ──────────────────────────────────────────────────────
    let alpha_id = match alpha_id_val {
        Value::i64(n) => n,
        other => return Err(RuntimeError::new(alpha_id_span, RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::i64 (alpha-id)",
            got: Box::new(ValueSnapshot::of(&other)),
        }).into()),
    };

    // ── Extract the token bindings ────────────────────────────────────────────
    // Both mouths take `PMap` (`DESIGN-STONE-token-bindings-promoting`): `resolve_operand` /
    // `PMap::get` here, and `eval_insert` / `build_insert_fact` on the RHS. No trie
    // materialisation on either path.
    let token_bindings: crate::value::pmap::PMap = match bindings_val {
        Value::wat__core__PersistentMap(ref m) => m.clone(),
        other => return Err(RuntimeError::new(bindings_span, RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::PersistentMap (token bindings)",
            got: Box::new(ValueSnapshot::of(&other)),
        }).into()),
    };

    // ── Extract the supporting fact (sfact) + its field names ────────────────
    let sfact = match fact_from_value(&sfact_val) {
        Some(f) => f,
        None => return Err(RuntimeError::new(sfact_span, RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::Record (supporting fact)",
            got: Box::new(ValueSnapshot::of(&sfact_val)),
        }).into()),
    };
    let sfact_field_names = class_field_names(sym, sfact.class_fqdn);

    let network = match session_network(&session_val) {
        Some(n) => n,
        None => return Err(RuntimeError::new(session_span, RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::rete::Session (named network field)",
            got: Box::new(ValueSnapshot::of(&session_val)),
        }).into()),
    };

    let cond_ast = match alpha_cond_of(network, alpha_id) {
        Some(c) => c,
        None => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "AlphaNode condition WatAST",
            got: Box::new(ValueSnapshot::of(&Value::i64(alpha_id))),
        }).into()),
    };

    let pat = match alpha_pattern(&cond_ast) {
        Some(p) => p,
        None => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "alpha condition form in AlphaNode.tests[0]",
            got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(cond_ast.clone())))),
        }).into()),
    };
    let pattern = pat.type_head.to_string();
    let clauses = pat.clauses;

    // ── Walk clauses: classify + build constraints + collect binder var names ─
    // Reuse the matcher's OWN classifier (same shape checks as alpha_match_inner).
    // Binder: (?v <- :field) — collect ?v name.
    // Constraint: (:op a b) — resolve operands via resolve_operand, rebuild as WatAST.
    let mut binder_vars: Vec<String> = Vec::new();
    let mut constraints_pv = crate::value::pvec::PVec::new();

    for clause in clauses {
        match classify_rete_clause(clause) {
            ReteClauseShape::Bind { var, .. } => {
                binder_vars.push(var.to_string());
            }
            ReteClauseShape::Constraint { op, lhs, rhs } => {
                if classify_constraint_head(op).is_none() {
                    continue;
                }
                let a_val = resolve_operand(lhs, sfact.fields, &sfact_field_names, &token_bindings);
                let b_val = resolve_operand(rhs, sfact.fields, &sfact_field_names, &token_bindings);
                let (Some(a_val), Some(b_val)) = (a_val, b_val) else { continue; };
                let (Some(a_ast), Some(b_ast)) = (value_to_ast_literal(a_val), value_to_ast_literal(b_val)) else { continue; };
                let substituted = WatAST::List(
                    vec![
                        WatAST::Keyword(op.to_string(), list_span.clone()),
                        a_ast,
                        b_ast,
                    ],
                    list_span.clone(),
                );
                constraints_pv.push_back_mut(Value::wat__WatAST(Arc::new(substituted)));
            }
            _ => continue,
        }
    }

    // ── Per-step bindings: project token bindings to binder_vars only ─────────
    let mut step_bindings_pairs: Vec<(Value, Value)> = Vec::new();
    for var_name in &binder_vars {
        let key = Value::String(Arc::new(var_name.clone()));
        if let Some(v) = token_bindings.get(&key) {
            step_bindings_pairs.push((key, v.clone()));
        }
    }
    let step_bindings_pm = crate::value::pmap::PMap::from_pairs(step_bindings_pairs);

    // ── Build DerivationStep record ───────────────────────────────────────────
    // Field order (declaration order in rete.wat):
    //   supporting(0) <- :wat::rete::DerivationNode
    //   pattern(1)    <- :wat::core::String
    //   bindings(2)   <- :wat::core::PersistentMap<String, Value>
    //   constraints(3)<- :wat::core::PersistentVector<WatAST>
    type ClassFqdn = Arc<String>;
    static STEP_CLASS_FQDN: std::sync::OnceLock<ClassFqdn> = std::sync::OnceLock::new();
    let step_class = STEP_CLASS_FQDN
        .get_or_init(|| Arc::new("wat::rete::DerivationStep".to_string()))
        .clone();

    Ok(Value::Aggregate(Arc::new(AggregateValue::record(
        (*step_class).clone(),
        derivation_step_names(),
        Arc::new(vec![
            supporting,                                              // supporting: DerivationNode
            Value::String(Arc::new(pattern)),                       // pattern: String (FQDN)
            Value::wat__core__PersistentMap(step_bindings_pm),      // bindings: PM<String, Value>
            Value::wat__core__PersistentVector(constraints_pv),     // constraints: PV<WatAST>
        ]),
    ))))
}

// Arc 296 G-1 — class C, missing from the brief's table: `DerivationStep` declared at
// `wat/rete.wat`.
::wat_source_derive::wat_field_names_from!(DERIVATION_STEP_FIELDS, "wat/rete.wat", ":wat::rete::DerivationStep");
fn derivation_step_names() -> FieldNames {
    static N: std::sync::OnceLock<FieldNames> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(DERIVATION_STEP_FIELDS)).clone()
}
