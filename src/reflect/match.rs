//! Arc 109 Stone — the reflect home's MATCH role: form matching.
//!
//! Split by ROLE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-reflect-home.md`). `eval_form_matches`
//! is the `:wat::form::matches?` Clara-style structural matcher; `walk_match_clause` is
//! its per-clause recursive walker; `eval_forms` (the `:wat::core::forms` special form,
//! unrelated to pattern matching but small enough that DESIGN grouped it here with its
//! neighbours rather than opening a sixth file) rounds out the file. Moved verbatim out
//! of `src/runtime.rs` (arc 109 reflect stone). Behaviour is unchanged; only the location
//! moved.
//!
//! This file is declared `mod r#match;` in `src/lib.rs` — `match` is a Rust keyword, so
//! the module name needs the raw-identifier escape; the file itself is still `match.rs`
//! per the brief.
//!
//! All three items are `pub(crate)`: `eval_form_matches` carries `#[wat_intrinsic]` (every
//! such verb living outside `runtime.rs` in this codebase is `pub(crate)`, see
//! `crates/wat-macros/src/wat_intrinsic.rs`'s own doc example); `walk_match_clause` is
//! called cross-module from `runtime.rs`'s own `#[cfg(test)] mod tests` (the
//! `walk_compare_bool` helper, arc 300 stone C5b); `eval_forms` is called from
//! `runtime.rs`'s own `dispatch_keyword_head_value` (a special form with no
//! `#[wat_intrinsic]` entry). Each is a visibility bump forced by the new module
//! boundary, not a signature change.
//!
//! Siblings: `render.rs` (internal state → AST), `lookup.rs` (find a binding), `verbs.rs`
//! (the `*-of` API surface), `expand.rs` (macroexpand).

use crate::ast::WatAST;
use crate::span::Span;
use crate::types::Nature;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, TrackedValue, Value,
    ValueSnapshot,
};
use std::sync::Arc;
use wat_macros::wat_intrinsic;

// `eval_inner` (the evaluator's own entry point) and `values_equal` are genuinely defined
// in `crate::runtime` (not a facade re-export of a `crate::value` type — see STOP-2).
use crate::runtime::{eval_inner, values_equal};

/// Arc 098 — `:wat::form::matches?` runtime walker. Clara-style
/// single-item pattern matcher.
///
/// Shape:
///
/// ```text
/// (:wat::form::matches? SUBJECT
///   (:TYPE-NAME (= ?var :field) ... <constraint> ...))
/// ```
///
/// Returns `:bool`. Per the DESIGN's runtime semantics:
///
/// - Subject is `:None` / `(Some non-struct)` / non-Struct / a
///   Struct of a different class → `false` (no error; Clara
///   semantics).
/// - Subject is a `Value::Aggregate(Struct)` with the matching `class` → walk
///   clauses; AND every constraint result.
///
/// Bindings (`(= ?var :field)`) push `?var → field-value` into the
/// local environment for subsequent clauses (including `where`-
/// bodies and comparisons). Constraint clauses evaluate against the
/// accumulated scope; first failure short-circuits the walk.
///
/// Type-check side (`check.rs::infer_form_matches`) validates the
/// pattern grammar at expansion. The runtime trusts that input —
/// grammar errors at this layer are bugs in the type checker, not
/// user errors.
///
/// Homed to `#[wat_intrinsic]` arc 255 Stone P6-c-1 (one of the two proof verbs — the
/// ORDER shape: this handler declares its context tail `(list_span, env, sym)`, NOT the
/// 100-arm `(env, sym, span)` order the macro used to hardcode). `check.rs`'s own
/// `:wat::form::matches?` grammar check (line ~3933, `infer_form_matches`) matches on the
/// literal FQDN string ahead of any generic-apply/TypeScheme lookup and is untouched by
/// this move — the registry gains a dispatch entry, nothing about type-checking changes.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     args… :wat::core::Value SUBJECT (evaluated) followed by the pattern `(:TYPE-NAME clause ...)` (never evaluated — walked structurally)
/// @ret     :wat::core::bool whether SUBJECT structurally matches the pattern (Clara semantics: a non-matching class, non-Struct value, or `:None` subject is `false`, never an error)
/// @example (:wat::core::do (:wat::core::defstruct :probe::FormMatchSubject [amount <- :wat::core::i64]) (:wat::form::matches? (:probe::FormMatchSubject :amount 3) (:probe::FormMatchSubject (= ?a :amount)))) #=> true
#[wat_intrinsic(":wat::form::matches?")]
pub(crate) fn eval_form_matches(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::form::matches?";
    if args.len() != 2 {
        // arc 138: no span — leaf helper without list_span; threading
        // would require touching the entire dispatcher arm chain.
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

    // Eval the subject. Auto-unwrap one level of `Option(Some(_))`
    // so callers can write `(matches? maybe-event (:Foo ...))`
    // against `(Option :- [Value])` directly. None / non-Struct / wrong
    // type → false.
    let subject = eval_inner(&args[0], env, sym)?.value_owned();
    let subject = match subject {
        Value::Option(opt) => match (*opt).clone() {
            Some(v) => v,
            None => return Ok(Value::bool(false)),
        },
        other => other,
    };

    // Pattern shape: `(:TYPE-NAME clause ...)`. The type checker
    // rejected anything else; here we just destructure.
    let (type_name, clauses) = match &args[1] {
        WatAST::List(items, _) if !items.is_empty() => match &items[0] {
            WatAST::Keyword(k, _) => (k.as_str(), &items[1..]),
            other_head => {
                return Err(RuntimeError::new(
                    other_head.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "pattern head must be a struct type keyword".into(),
                    },
                )
                .into());
            }
        },
        other_pat => {
            return Err(RuntimeError::new(
                other_pat.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "pattern must be a list `(:TYPE-NAME clause ...)`".into(),
                },
            )
            .into());
        }
    };

    // Arc 293.R2.1 — Aggregate with nature==Struct; class is colon-free, type_name has ':'.
    let bare_type = type_name.strip_prefix(':').unwrap_or(type_name);
    let struct_value = match &subject {
        Value::Aggregate(a) if a.nature == Nature::Struct && a.class.as_ref() == bare_type => a.clone(),
        _ => return Ok(Value::bool(false)),
    };

    // Resolve the struct's declared field-name → index map.
    // Arc 293.2b/R2.1 — Struct aggregates (kind==Struct) replace TypeDef::Struct.
    let field_names: Vec<String> = sym
        .types()
        .and_then(|t| match t.get(type_name) {
            Some(crate::types::TypeDef::Aggregate(a))
                if a.nature == crate::types::Nature::Struct =>
            {
                Some(a.fields.iter().map(|(n, _)| n.clone()).collect())
            }
            _ => None,
        })
        .unwrap_or_default();

    // Walk clauses, threading env through bindings.
    let mut current_env = env.clone();
    for clause in clauses {
        let (passed, new_env) =
            walk_match_clause(clause, &field_names, &struct_value.fields, current_env, sym)?;
        if !passed {
            return Ok(Value::bool(false));
        }
        current_env = new_env;
    }
    Ok(Value::bool(true))
}

/// Walk a single clause. Returns `(passed, env)` — `passed` is the
/// clause's truth value at runtime; `env` is the environment that
/// subsequent clauses see (bindings flow forward).
///
/// For bindings, `passed` is always `true` and `env` carries the new
/// `?var → field-value` mapping. For constraints, `passed` reflects
/// the comparison/where result and `env` is unchanged.
pub(crate) fn walk_match_clause(
    clause: &WatAST,
    field_names: &[String],
    struct_fields: &[Value],
    env: Environment,
    sym: &SymbolTable,
) -> Result<(bool, Environment), EvalBreak> {
    use crate::form_match::{
        classify_clause, keyword_payload, logic_var_name, CompareOp, RawClause,
    };

    let raw = classify_clause(clause).map_err(|e| {
        RuntimeError::new(
            clause.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::form::matches?".into(),
                reason: format!("classifier: {:?}", e),
            },
        )
    })?;

    match raw {
        RawClause::Eq { left, right } => {
            // Disambiguate binding vs equality by LHS shape and
            // whether the variable is already in scope.
            if let Some(var) = logic_var_name(left) {
                if env.lookup(var, left.span()).is_none() {
                    // Fresh ?var — binding.
                    let field_kw = keyword_payload(right).ok_or_else(|| {
                        RuntimeError::new(
                            right.span().clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: ":wat::form::matches?".into(),
                                reason: format!("binding RHS for {} must be a field keyword", var),
                            },
                        )
                    })?;
                    let field_lookup = field_kw.strip_prefix(':').unwrap_or(field_kw);
                    let idx = field_names.iter().position(|n| n == field_lookup);
                    let value = match idx {
                        Some(i) if i < struct_fields.len() => struct_fields[i].clone(),
                        _ => {
                            // Type-check should have caught this;
                            // hitting it at runtime means the
                            // registry was missing or the pattern
                            // skipped check. Return false rather
                            // than error — Clara-style.
                            return Ok((false, env));
                        }
                    };
                    let new_env = env
                        .child()
                        .bind_unknown_span(var.to_string(), TrackedValue::from(value))
                        .build();
                    return Ok((true, new_env));
                }
                // ?var already bound — fall through to comparison.
            }
            // Equality comparison. eval both sides; structural equality.
            let a = eval_inner(left, &env, sym)?.value_owned();
            let b = eval_inner(right, &env, sym)?.value_owned();
            let eq = values_equal(&a, &b).unwrap_or(false);
            Ok((eq, env))
        }
        RawClause::Compare { op, left, right } => {
            let a = eval_inner(left, &env, sym)?.value_owned();
            let b = eval_inner(right, &env, sym)?.value_owned();
            match op {
                CompareOp::NotEq => {
                    let eq = values_equal(&a, &b).unwrap_or(false);
                    Ok((!eq, env))
                }
                _ => {
                    // Arc 300 stone C5b — the i64<->f64 cross arms route through the
                    // one exact ordering door instead of coercing i64 down to f64
                    // (lossy above 2^53). Policy for this caller (Clara no-error
                    // semantics): Incomparable (NaN) -> Equal, preserving today's
                    // behaviour byte-for-byte; NotNumeric can't occur for these two
                    // type-guaranteed arms but falls to the `_ => Ok((false, env))`
                    // silent-false below if it ever did. No BigInt/Rational arms are
                    // added here — this table only ever knew i64/u8/f64, and adding
                    // them is a widening this stone does not ask for.
                    let order = match (&a, &b) {
                        (Value::i64(x), Value::i64(y)) => x.cmp(y),
                        (Value::u8(x), Value::u8(y)) => x.cmp(y),
                        (Value::f64(x), Value::f64(y)) => {
                            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        (Value::i64(_), Value::f64(_)) | (Value::f64(_), Value::i64(_)) => {
                            match crate::value::numeric_order::numeric_order(&a, &b) {
                                crate::value::numeric_order::NumOrd::Ord(o) => o,
                                crate::value::numeric_order::NumOrd::Incomparable => {
                                    std::cmp::Ordering::Equal
                                }
                                crate::value::numeric_order::NumOrd::NotNumeric => {
                                    return Ok((false, env));
                                }
                            }
                        }
                        (Value::String(x), Value::String(y)) => x.cmp(y),
                        (Value::bool(x), Value::bool(y)) => x.cmp(y),
                        (Value::wat__core__keyword(x), Value::wat__core__keyword(y)) => x.cmp(y),
                        _ => return Ok((false, env)),
                    };
                    let pred = match op {
                        CompareOp::Lt => order == std::cmp::Ordering::Less,
                        CompareOp::Gt => order == std::cmp::Ordering::Greater,
                        CompareOp::Le => order != std::cmp::Ordering::Greater,
                        CompareOp::Ge => order != std::cmp::Ordering::Less,
                        // Eq + NotEq handled above.
                        CompareOp::Eq | CompareOp::NotEq => unreachable!(),
                    };
                    Ok((pred, env))
                }
            }
        }
        RawClause::And(subs) => {
            let mut e = env;
            for sub in subs {
                let (p, e2) = walk_match_clause(sub, field_names, struct_fields, e, sym)?;
                if !p {
                    return Ok((false, e2));
                }
                e = e2;
            }
            Ok((true, e))
        }
        RawClause::Or(subs) => {
            // For `or`, evaluate each branch with the env as it was
            // entering the `or`. Bindings introduced inside an
            // `or` branch don't survive past the `or` — they'd be
            // ambiguous (which branch's bindings won?) and the
            // type checker doesn't carry per-branch bindings into
            // the post-`or` scope either.
            let entry_env = env;
            for sub in subs {
                let (p, _) =
                    walk_match_clause(sub, field_names, struct_fields, entry_env.clone(), sym)?;
                if p {
                    return Ok((true, entry_env));
                }
            }
            Ok((false, entry_env))
        }
        RawClause::Not(sub) => {
            // `not` flips the result. Sub-clause bindings (if any)
            // don't survive the `not` — they only mean something
            // when the sub-clause matched, which `not` rejects.
            let entry_env = env;
            let (p, _) =
                walk_match_clause(sub, field_names, struct_fields, entry_env.clone(), sym)?;
            Ok((!p, entry_env))
        }
        RawClause::Where(body) => {
            let v = eval_inner(body, &env, sym)?.value_owned();
            match v {
                Value::bool(b) => Ok((b, env)),
                other => Err(RuntimeError::new(
                    body.span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::form::matches?".into(),
                        expected: "bool from where-body",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into()),
            }
        }
    }
}

/// `(:wat::core::forms f1 f2 ... fn)` → `(:wat::core::Vector :- [wat::WatAST])`.
///
/// Variadic sibling of `quote`. Takes N unevaluated forms and returns
/// a Vec of `:wat::WatAST` values — one per form, each captured as
/// data. Semantically equivalent to
/// `(vec :wat::WatAST (quote f1) (quote f2) ... (quote fn))` but
/// without the per-form quote ceremony.
///
/// Use case: building program-as-data payloads for
/// `:wat::kernel::run-sandboxed-ast`, `:wat::eval-ast!`, or
/// any consumer of AST sequences. The test stdlib's `:wat::test::
/// program` macro expands directly to this.
///
/// Like `quote`, this is a special form — arguments are NOT
/// evaluated. The type checker returns `(:wat::core::Vector :- [wat::WatAST])`
/// unconditionally; see `check.rs::infer_list` for the handling.
pub(crate) fn eval_forms(
    args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — infallible — no error path (always `Ok`)
) -> Result<Value, EvalBreak> {
    let items: Vec<Value> = args
        .iter()
        .map(|a| Value::wat__WatAST(Arc::new(a.clone())))
        .collect();
    Ok(Value::Vec(Arc::new(items)))
}
