//! Explain `DerivationStep` payload — `eval_step_payload`.

use crate::ast::WatAST;
use crate::rete::kernel::{alpha_cond_of, session_network};
use crate::rete::clause::{classify_rete_clause, ReteClauseShape};
use crate::rete::matcher::{
    alpha_pattern, class_field_names, fact_from_value, resolve_operand, value_to_ast_literal,
    FieldNames,
};
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::span::Span;
use crate::value::value::AggregateValue;
use std::sync::Arc;

// ─── P12c: step-payload ───────────────────────────────────────────────────────

/// The head of the payload's ONE spelling for *"this constraint was satisfied and could not be
/// rendered"* — D6's cure for the silent `continue`.
///
/// **It is not, and must not become, a callable rete op.** A marker that evaluated would be a
/// second way to be silently wrong: a consumer could run it and get a verdict about a comparison
/// nobody performed. Because no `RETE_OPS` row bears this name, an attempt to evaluate the form
/// fails by name — loudly, at the point of the mistake.
pub(crate) const CONSTRAINT_NOT_RENDERED: &str = ":wat::rete::explain::constraint-not-rendered";

/// Resolve ONE operand of a satisfied inline constraint and spell it as a `WatAST` literal, or
/// say why it cannot be spelled.
///
/// The two failure modes are kept apart on purpose, because they predict different mechanisms:
/// an operand that produced no `Value` at all is a *resolution* gap (a `?var` absent from the
/// token's bindings, a keyword naming neither a declared field nor a known unit variant), while a
/// `Value` with no literal form is a *spelling* gap in
/// [`value_to_ast_literal`]. D6 was one of each, stacked: `sym: None` hid the second behind the
/// first, so curing only the first moved the drop one line down and changed nothing a user saw.
fn render_constraint_operand(
    operand: &WatAST,
    fact_fields: &[Value],
    field_names: &[String],
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<WatAST, String> {
    // `Some(sym)`, not `None`. With `None` an enum-variant keyword in direct operand position
    // (`:d6::Grade::Hi`) resolves to nothing, because `resolve_operand`'s keyword arm needs the
    // symbol table to tell a unit variant from a plain keyword — the FIRST of D6's two gates.
    let Some(v) = resolve_operand(operand, fact_fields, field_names, bindings, Some(sym)) else {
        return Err("operand resolved to no value (not a bound var, a declared field, or a literal)".to_string());
    };
    match value_to_ast_literal(v.clone()) {
        Some(ast) => Ok(ast),
        None => Err(match &v {
            // Guarded on `!fields.is_empty()`, not on `Value::Enum` alone: a UNIT variant reaching
            // here would mean `value_to_ast_literal` lost its enum arm, and calling that "tagged"
            // would send the reader to the wrong mechanism entirely. It falls to the generic arm
            // below instead, which describes what is true in both cases.
            Value::Enum(ev) if !ev.fields.is_empty() => format!(
                "a tagged enum variant ({}::{}, {} field(s)) has no literal spelling in the rete surface",
                ev.type_path,
                ev.variant_name,
                ev.fields.len()
            ),
            other => format!("a {} value has no literal spelling in the rete surface", other.type_name()),
        }),
    }
}

/// Build the omission marker that holds an unrenderable constraint's POSITION in `constraints`.
///
/// One entry per inline constraint clause is the property this buys: a caller can no longer
/// mistake an omission for a rule that genuinely had fewer constraints, which is exactly what the
/// bare `continue` produced.
fn constraint_not_rendered(op: &str, operand_index: i64, why: String, span: &Span) -> WatAST {
    WatAST::List(
        vec![
            WatAST::Keyword(CONSTRAINT_NOT_RENDERED.to_string(), span.clone()),
            WatAST::Keyword(op.to_string(), span.clone()),
            WatAST::IntLit(operand_index, span.clone()),
            WatAST::StringLit(why, span.clone()),
        ],
        span.clone(),
    )
}

/// `(:wat::rete::step-payload session alpha-id bindings sfact supporting) -> :wat::rete::DerivationStep`
///
/// Arc 278 Stone P12c — the explain payload builder. Given one (sfact, alpha-id) match edge
/// from a Token's matches chain, builds the full `DerivationStep` payload:
///
/// - **pattern**: the matched condition's fact-type FQDN (AlphaNode tests[0] head keyword).
/// - **bindings** (per-step): the binder-clause vars that THIS condition bound, projected
///   from the token's accumulated bindings.
/// - **constraints**: the condition's satisfied INLINE constraint clauses — the per-type
///   comparisons `classify_constraint_head` admits (`i64::<`, `enum::=`, …) — with bound values
///   substituted: `(:wat::rete::core::i64::< -5 0)` from `(:wat::rete::core::i64::< ?c 0)` with
///   `?c=-5`. **Exactly one entry per inline constraint clause, always** — see below.
///   `:where` fences, `not`/`exists` sub-conditions and predicate clauses are NOT in this field
///   and never were; they are separate `ReteClauseShape`s, not inline constraints.
///
/// **Faithfulness by construction**: `classify_rete_clause` + `resolve_operand` reconstruct
/// the matched clause for the payload. Native fire matches via `exec_compiled_with_key_ids`
/// (STOP-1), not `alpha_match_inner` (the oracle). Substituted values still cannot drift
/// from the classifier's spelling of what matched.
///
/// **⛔ An unrenderable constraint is NAMED, never dropped** (D6). Until this strike, a
/// constraint whose operand did not resolve, or whose resolved value had no literal spelling,
/// was skipped by a bare `continue` — and a caller cannot tell a shortened vector from a rule
/// that genuinely had fewer constraints. It now keeps its position as
///
/// ```text
/// (:wat::rete::explain::constraint-not-rendered <op-keyword> <operand-index> "<why>")
/// ```
///
/// so `constraints.length` still equals the condition's inline-constraint count. The head is
/// deliberately not a callable rete op: nothing can consume the marker as a satisfied predicate
/// and get a wrong answer — evaluating it fails by name. See
/// [`crate::rete::matcher::value_to_ast_literal`] for what is spellable today (a tagged enum
/// variant is the one live residue).
///
/// Arguments:
///   - `session`    — `:wat::rete::Session` (network via `session_network`)
///   - `alpha-id`   — `:wat::core::i64` (the AlphaNode id for this condition)
///   - `bindings`   — `:wat::core::PersistentMap` (the token's accumulated bindings)
///   - `sfact`      — `:wat::core::Record` (the supporting fact for this edge)
///   - `supporting` — `:wat::rete::DerivationNode` (the pre-computed recursive node)
///
/// Returns a `:wat::rete::DerivationStep` record.
pub(crate) fn eval_step_payload(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::step-payload"; 

    if args.len() != 5 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 5,
            got: args.len(),
        }).into());
    }

    // ── Evaluate all 5 arguments ──────────────────────────────────────────────
    let session_val  = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let alpha_id_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    let bindings_val = crate::runtime::eval_inner(&args[2], env, sym)?.value_owned();
    let sfact_val    = crate::runtime::eval_inner(&args[3], env, sym)?.value_owned();
    let supporting   = crate::runtime::eval_inner(&args[4], env, sym)?.value_owned();

    // ── Extract alpha_id ──────────────────────────────────────────────────────
    let alpha_id = match alpha_id_val {
        Value::i64(n) => n,
        other => return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
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
        other => return Err(RuntimeError::new(args[2].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::PersistentMap (token bindings)",
            got: Box::new(ValueSnapshot::of(&other)),
        }).into()),
    };

    // ── Extract the supporting fact (sfact) + its field names ────────────────
    let sfact = match fact_from_value(&sfact_val) {
        Some(f) => f,
        None => return Err(RuntimeError::new(args[3].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::Record (supporting fact)",
            got: Box::new(ValueSnapshot::of(&sfact_val)),
        }).into()),
    };
    let sfact_field_names = class_field_names(sym, sfact.class_fqdn);

    let network = match session_network(&session_val) {
        Some(n) => n,
        None => return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
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
                // ⚠ No `classify_constraint_head(op).is_none() → continue` guard here any more,
                // and its absence is not an oversight: `classify_rete_clause` produces
                // `Constraint` ONLY from the arm guarded by `classify_constraint_head(k).is_some()`
                // (`clause.rs`), so the re-check could never fire. It was one of the three
                // `continue`s D6 was drawn against and it was the dead one —
                // `clause.rs`'s `a_constraint_shape_implies_a_classifying_head` pins BOTH
                // directions of that, so if the classifier ever grows a second route to
                // `Constraint` it goes RED there rather than this going silently short.
                let a = render_constraint_operand(lhs, sfact.fields, &sfact_field_names, &token_bindings, sym);
                let b = render_constraint_operand(rhs, sfact.fields, &sfact_field_names, &token_bindings, sym);
                // Both failing reports operand 1 — the leftmost cause, so the message is stable
                // rather than dependent on evaluation order.
                let form = match (a, b) {
                    (Ok(a_ast), Ok(b_ast)) => WatAST::List(
                        vec![
                            WatAST::Keyword(op.to_string(), list_span.clone()),
                            a_ast,
                            b_ast,
                        ],
                        list_span.clone(),
                    ),
                    (Err(why), _) => constraint_not_rendered(op, 1, why, list_span),
                    (_, Err(why)) => constraint_not_rendered(op, 2, why, list_span),
                };
                constraints_pv.push_back_mut(Value::wat__WatAST(Arc::new(form)));
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
