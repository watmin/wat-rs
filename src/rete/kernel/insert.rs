//! Native `insert` / `insert-all`. Session overlay by field name.

use std::sync::Arc;

use crate::ast::WatAST;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::span::Span;
use crate::types::Nature;
use crate::value::value::AggregateValue;

// ── Public entry: native insert ───────────────────────────────────────────────

/// `facts` slot from the Aggregate's carried names (arc 296 G).
/// TypeEnv is not on this path (`DESIGN-STONE-insert-facts-from-names`).
/// `available` is allocated only on miss.
fn session_facts_idx(
    agg: &AggregateValue,
    list_span: &Span,
) -> Result<usize, EvalBreak> {
    match agg.names.iter().position(|n| n == "facts") {
        Some(i) => Ok(i),
        None => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::UnknownField {
                record_class: agg.class.to_string(),
                field: "facts".to_string(),
                available: agg.names.as_ref().clone(),
            },
        )
        .into()),
    }
}

/// `(:wat::rete::insert <session> <fact>) -> :wat::rete::Session`
///
/// Native dual of the wat oracle `insert$oracle` (`wat/rete/oracle/insert.wat`).
/// Stages `fact` into the Session's `facts` field. ZERO activation:
/// facts stay staged until `fire-rules`, so this touches no memory and walks no
/// network. The other Session fields carry through untouched.
///
/// ★ Contract: `facts` is resolved BY NAME from `agg.names` — never by positional
/// index (`DESIGN-STONE-insert-facts-from-names`). TypeEnv is not on the hot path.
pub(crate) fn eval_insert_native(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::insert";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }

    // Evaluate both arguments (mirrors eval_fire_rules_native's session eval).
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let fact = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    insert_one_on_session(session, fact, list_span)
}

/// Public `:wat::rete::insert` (`DESIGN-STONE-insert-prime-split`).
///
/// 2-ary is the streaming hot path — same native body as `insert`, not a
/// one-element PersistentVector through `insert-all`. 3+ sugar collects
/// the facts into one PV and rebuilds the Session once (`insert-all`).
/// The wat `defclause` remains the type surface; runtime dispatch takes
/// this arm first so the 2-ary body is not `apply_function`'d.
pub(crate) fn eval_insert_public(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::insert";
    match args.len() {
        2 => eval_insert_native(args, list_span, env, sym),
        0 | 1 => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into()),
        _ => {
            let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for a in &args[1..] {
                pv.push_back_mut(crate::runtime::eval_inner(a, env, sym)?.value_owned());
            }
            insert_facts_on_session(
                session,
                Value::wat__core__PersistentVector(pv),
                list_span,
            )
        }
    }
}

fn require_session_agg<'a>(
    session: &'a Value,
    op: &'static str,
    list_span: &Span,
) -> Result<&'a crate::value::value::AggregateValue, EvalBreak> {
    match session {
        Value::Aggregate(a)
            if a.nature != Nature::Struct && a.class.as_ref() == "wat::rete::Session" =>
        {
            Ok(a)
        }
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::rete::Session (a wat::core::Record)",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn require_record_fact(fact: &Value, op: &'static str, list_span: &Span) -> Result<(), EvalBreak> {
    match fact {
        Value::Aggregate(a) if a.nature != Nature::Struct => Ok(()),
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::Record (a fact)",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn insert_one_on_session(
    session: Value,
    fact: Value,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::insert";
    require_record_fact(&fact, OP, list_span)?;
    let agg = require_session_agg(&session, OP, list_span)?;

    let facts_idx = session_facts_idx(agg, list_span)?;

    // Conj the fact onto the resolved `facts` PersistentVector; every other field carries
    // through unchanged (structural clone).
    let facts_val = &agg.fields[facts_idx];
    let new_facts = crate::collection::eval::persistentvector_conj_inner(facts_val, &fact)?;

    let mut new_fields: Vec<Value> = agg.fields.as_ref().clone();
    new_fields[facts_idx] = new_facts;

    Ok(Value::Aggregate(Arc::new(AggregateValue::record_arc(
        agg.class.clone(),
        agg.names.clone(),
        Arc::new(new_fields),
    ))))
}

fn insert_facts_on_session(
    session: Value,
    new_facts_vec: Value,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::insert-all";
    let agg = require_session_agg(&session, OP, list_span)?;
    if let Value::wat__core__PersistentVector(pv) = &new_facts_vec {
        for f in pv.iter() {
            require_record_fact(f, OP, list_span)?;
        }
    }

    let facts_idx = session_facts_idx(agg, list_span)?;

    // Extend the resolved `facts` PersistentVector by every element of `new_facts_vec` in ONE
    // concat; every other field carries through unchanged (structural clone). This single
    // concat + single Session rebuild (below) is the whole win over N `insert` calls.
    let facts_val = &agg.fields[facts_idx];
    let new_facts = crate::collection::eval::vector_concat_inner(facts_val, &new_facts_vec)?;

    let mut new_fields: Vec<Value> = agg.fields.as_ref().clone();
    new_fields[facts_idx] = new_facts;

    Ok(Value::Aggregate(Arc::new(AggregateValue::record_arc(
        agg.class.clone(),
        agg.names.clone(),
        Arc::new(new_fields),
    ))))
}

// ── Public entry: native insert-all ────────────────────────────────────────────

/// `(:wat::rete::insert-all <session> <facts>) -> :wat::rete::Session`
///
/// The batch sibling of `insert` — native dual of the wat oracle `insert-all$oracle`
/// (`wat/rete/oracle/insert.wat`). Stages every element of `facts` (a `PersistentVector<Record>`) into the
/// Session's `facts` field in ONE rebuild, instead of N rebuilds (`insert` × N). ZERO
/// activation, same contract as `insert`: facts stay staged until `fire-rules`.
/// The other seven `Session` fields carry through untouched.
///
/// ★ This is the entire point of the stone: `insert` reconstructs the 8-field `Session` once
/// PER FACT (~1.03 µs of pure rebuild above a bare `conj`, measured in
/// `DESIGN-STONE-insert-all.md`); this extends the resolved `facts` PersistentVector by N
/// elements via one `Vector/concat` and rebuilds the `Session` exactly once.
///
/// ★ Contract: `facts` is resolved BY NAME from `agg.names` — never by positional
/// index — exactly mirroring `eval_insert_native` (`DESIGN-STONE-insert-facts-from-names`).
pub(crate) fn eval_insert_all_native(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::insert-all";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }

    // Evaluate both arguments (mirrors eval_insert_native's session/fact eval).
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let new_facts_vec = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    insert_facts_on_session(session, new_facts_vec, list_span)
}
