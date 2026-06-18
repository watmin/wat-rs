//! Arc 278 Stone 2a — `alpha-match`: the rete single-fact matcher.
//!
//! Given a condition form (DATA, a `:wat::WatAST`) and a fact (a `:wat::Record`
//! — either `Value::wat__Record` or `Value::wat__holon__Record`), return
//! `Some(bindings)` iff the fact's class matches the condition head AND every
//! clause holds; `None` otherwise.
//!
//! ## Why this is a PURE primitive (not a wrapper around `form::matches?`)
//!
//! `form::matches?` is a compile-time special form that binds into a live
//! `Environment` and returns `bool`. The rete engine needs data-in/data-out:
//! the condition arrives as runtime DATA (a quoted AST), the bindings must
//! be a returnable map, operands resolve from {bindings, field, literal} only
//! — never `eval_inner`. The structural difference is total; re-use was
//! scrutinized and rejected in the DESIGN-STONE-2a-alpha-match doc.
//!
//! ## Clause classifier (own grammar — NOT `classify_clause`)
//!
//! `classify_clause` in `form_match.rs` is `form::matches?`'s grammar (`:=`,
//! bare `<` heads, `where`). The rete DSL uses `<-` bind, FQDN ops
//! (`:wat::core::>`), and `:wat::rete::and/or/not` combinators. The matcher
//! classifies clause lists by SHAPE, not by shared grammar.
//!
//! ## Operand resolution (the purity crux)
//!
//! An operand is one of:
//! - `Symbol(?v)` → look up in the bindings map (absent = `None`; not yet bound
//!   within THIS condition = cross-condition join, handled by beta network in stone 3)
//! - `Keyword(:field)` → read the named field from the fact directly
//! - Literal (`IntLit`/`FloatLit`/`BoolLit`/`StringLit`) → its bare Value
//!
//! NEVER `eval_inner`. NEVER an `Environment`.
//!
//! ## Clara no-error semantics
//!
//! Wrong type / missing field / unbound var / failed constraint → `None`.
//! Errors are swallowed, never raised. Stone 6 (`where`) can raise; stone 2a
//! cannot.

use crate::ast::WatAST;
use crate::form_match::keyword_payload;
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::span::Span;
use std::sync::Arc;

// ─── Fact abstraction ─────────────────────────────────────────────────────────

/// The type name and ordered field values extracted from whichever record
/// variant arrived as the fact. Decouples the matcher from the storage variant.
struct Fact<'a> {
    /// Class FQDN without leading colon, e.g. `"user::Temp"`.
    class_fqdn: &'a str,
    /// Field values in declaration order.
    fields: &'a [Value],
}

/// Extract a [`Fact`] from either record variant. Returns `None` for
/// non-record Values (Clara semantics: wrong fact type → no match).
fn fact_from_value(v: &Value) -> Option<Fact<'_>> {
    match v {
        Value::wat__Record { class_fqdn, struct_form } => Some(Fact {
            class_fqdn: class_fqdn.as_str(),
            fields: struct_form.as_slice(),
        }),
        Value::wat__holon__Record { class_fqdn, struct_form, .. } => Some(Fact {
            class_fqdn: class_fqdn.as_str(),
            fields: struct_form.as_slice(),
        }),
        // Value::Struct also represents records in older substrate paths.
        Value::Struct(sv) => Some(Fact {
            class_fqdn: sv.type_name.trim_start_matches(':'),
            fields: sv.fields.as_slice(),
        }),
        _ => None,
    }
}

// ─── Public entry point ────────────────────────────────────────────────────────

/// `(:wat::rete::alpha-match cond fact) -> Option<PersistentMap<String, Value>>`
///
/// Entry point dispatched by `dispatch_keyword_head_value` in `runtime.rs`.
/// Evaluates both arguments, extracts the WatAST condition and record fact,
/// then delegates to the pure inner matcher.
pub(crate) fn eval_alpha_match(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::alpha-match";
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        } }.into());
    }

    // Evaluate cond: must be Value::wat__WatAST wrapping a List.
    let cond_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let cond_ast = match cond_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST (condition form from quote)",
                got: Box::new(ValueSnapshot::of(&other)),
            } }.into());
        }
    };

    // Evaluate fact: must be a record value (wat__Record, wat__holon__Record, or Struct).
    let fact_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();

    // Resolve the fact's declared field names from the type registry.
    // The registry key carries the leading colon (e.g. ":user::Temp").
    // Falls back to an empty slice if the registry is absent (test harnesses
    // that bypass freeze) — binding clauses will return None at the first lookup.
    let fact = match fact_from_value(&fact_val) {
        Some(f) => f,
        None => {
            return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::Record (a record fact)",
                got: Box::new(ValueSnapshot::of(&fact_val)),
            } }.into());
        }
    };

    let type_key = format!(":{}", fact.class_fqdn);
    let field_names: Vec<String> = sym
        .types()
        .and_then(|t| match t.get(&type_key) {
            Some(crate::types::TypeDef::Record(rd)) => {
                Some(rd.field_names.clone())
            }
            Some(crate::types::TypeDef::Struct(sd)) => {
                Some(sd.fields.iter().map(|(n, _)| n.clone()).collect())
            }
            _ => None,
        })
        .unwrap_or_default();

    // Pure match: no environment, no eval, bindings as a persistent map.
    let result = alpha_match_inner(&cond_ast, fact.class_fqdn, fact.fields, &field_names);
    Ok(match result {
        Some(bindings) => Value::Option(Arc::new(Some(Value::wat__core__PersistentMap(bindings)))),
        None => Value::Option(Arc::new(None)),
    })
}

// ─── Pure inner matcher ────────────────────────────────────────────────────────

/// The pure core: no `Environment`, no `eval_inner`. Returns the binding map or
/// `None` (Clara no-error: any mismatch is `None`, never a raise).
fn alpha_match_inner(
    cond: &WatAST,
    fact_class: &str,
    fact_fields: &[Value],
    field_names: &[String],
) -> Option<rpds::HashTrieMapSync<Value, Value>> {
    // Condition must be a List whose head is a keyword naming the expected type.
    let items = match cond {
        WatAST::List(items, _) if !items.is_empty() => items,
        _ => return None,
    };

    // Head keyword is the fact-type selector. Both sides strip the leading `:`.
    let cond_head = match &items[0] {
        WatAST::Keyword(k, _) => k.trim_start_matches(':'),
        _ => return None,
    };
    // fact_class is already colon-free (extracted by fact_from_value).
    if cond_head != fact_class {
        return None;
    }

    // Fold the remaining clauses left→right, threading the bindings map.
    // Empty bindings = an rpds empty map (structural sharing; cheap insert).
    let empty: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    let clauses = &items[1..];
    eval_clauses(clauses, fact_fields, field_names, empty)
}

/// Walk a slice of top-level condition clauses, threading bindings left→right.
/// Returns `None` on the first failure (short-circuit AND).
fn eval_clauses(
    clauses: &[WatAST],
    fact_fields: &[Value],
    field_names: &[String],
    bindings: rpds::HashTrieMapSync<Value, Value>,
) -> Option<rpds::HashTrieMapSync<Value, Value>> {
    let mut current = bindings;
    for clause in clauses {
        current = eval_clause(clause, fact_fields, field_names, current)?;
    }
    Some(current)
}

/// Classify and evaluate a single clause. Returns `Some(updated_bindings)` on
/// success, `None` on mismatch or unresolvable operand.
fn eval_clause(
    clause: &WatAST,
    fact_fields: &[Value],
    field_names: &[String],
    bindings: rpds::HashTrieMapSync<Value, Value>,
) -> Option<rpds::HashTrieMapSync<Value, Value>> {
    let items = match clause {
        WatAST::List(items, _) if !items.is_empty() => items.as_slice(),
        // A clause that is not a non-empty list cannot be classified → None.
        _ => return None,
    };

    match &items[0] {
        // ── bind clause: (?v <- :field) ──────────────────────────────────────
        // Shape: [Symbol(?v), Symbol(<-), Keyword(:field)]
        WatAST::Symbol(head_ident, _) => {
            let var_name = head_ident.as_str();
            // A symbol-headed clause must look like `(?v <- :field)`.
            // The ?-prefix signals a logic variable; anything else is unrecognised → None.
            if !var_name.starts_with('?') {
                return None;
            }
            if items.len() != 3 {
                return None;
            }
            // Second element must be the arrow symbol `<-`.
            let is_arrow = matches!(&items[1], WatAST::Symbol(s, _) if s.as_str() == "<-");
            if !is_arrow {
                return None;
            }
            // Third element must be a field keyword `:field`.
            let field_kw = keyword_payload(&items[2])?;
            let field_name = field_kw.strip_prefix(':').unwrap_or(field_kw);
            let field_value = read_fact_field(fact_fields, field_names, field_name)?;
            // Bind ?v → field value. If ?v was already bound in this condition,
            // treat it as a constraint: the bound value must equal the field value.
            let key = Value::String(Arc::new(var_name.to_string()));
            match bindings.get(&key) {
                Some(existing) if existing != &field_value => None, // conflict
                Some(_) => Some(bindings),                          // already bound, equal
                None => Some(bindings.insert(key, field_value)),    // fresh binding
            }
        }

        // ── keyword-headed clause ─────────────────────────────────────────────
        WatAST::Keyword(head_kw, _) => match head_kw.as_str() {
            // ── constraint: (:wat::core::<op> a b) ───────────────────────────
            // FQDN comparison ops; operands resolved from {bindings, field, literal}.
            ":wat::core::=" => {
                let (a, b) = resolve_binary_operands(items, fact_fields, field_names, &bindings)?;
                if a == b { Some(bindings) } else { None }
            }
            ":wat::core::not=" => {
                let (a, b) = resolve_binary_operands(items, fact_fields, field_names, &bindings)?;
                if a != b { Some(bindings) } else { None }
            }
            ":wat::core::<" => {
                let (a, b) = resolve_binary_operands(items, fact_fields, field_names, &bindings)?;
                if compare_values(&a, &b)? == std::cmp::Ordering::Less { Some(bindings) } else { None }
            }
            ":wat::core::>" => {
                let (a, b) = resolve_binary_operands(items, fact_fields, field_names, &bindings)?;
                if compare_values(&a, &b)? == std::cmp::Ordering::Greater { Some(bindings) } else { None }
            }
            ":wat::core::<=" => {
                let (a, b) = resolve_binary_operands(items, fact_fields, field_names, &bindings)?;
                if compare_values(&a, &b)? != std::cmp::Ordering::Greater { Some(bindings) } else { None }
            }
            ":wat::core::>=" => {
                let (a, b) = resolve_binary_operands(items, fact_fields, field_names, &bindings)?;
                if compare_values(&a, &b)? != std::cmp::Ordering::Less { Some(bindings) } else { None }
            }

            // ── combinators ──────────────────────────────────────────────────
            // :wat::rete::and — every sub-clause holds (thread bindings left→right).
            ":wat::rete::and" => {
                eval_clauses(&items[1..], fact_fields, field_names, bindings)
            }
            // :wat::rete::or — ≥1 sub-clause holds. Bindings from a branch
            // do NOT survive past the `or` (which branch won is ambiguous).
            ":wat::rete::or" => {
                let entry = bindings;
                for sub in &items[1..] {
                    if eval_clause(sub, fact_fields, field_names, entry.clone()).is_some() {
                        return Some(entry);
                    }
                }
                None
            }
            // :wat::rete::not — the sub-clause must NOT hold. Bindings from
            // the negated branch are discarded (no values to bind from a failed match).
            ":wat::rete::not" => {
                if items.len() != 2 {
                    return None;
                }
                let sub_matched = eval_clause(&items[1], fact_fields, field_names, bindings.clone()).is_some();
                if sub_matched { None } else { Some(bindings) }
            }

            // ── STOP: :wat::rete::where is stone 6 ───────────────────────────
            // Arbitrary-expression eval belongs in a TestNode (stone 6), not here.
            // Reaching this arm means the caller used a `where` clause in a v1 condition.
            // Return None (Clara no-error: unhandled clause = no match).
            ":wat::rete::where" => None,

            // Unknown head keyword → unrecognised clause shape → None.
            _ => None,
        },

        // Non-symbol, non-keyword head → unrecognised clause shape → None.
        _ => None,
    }
}

// ─── Operand resolution ────────────────────────────────────────────────────────

/// Resolve one operand from `{bindings, fact-field, literal}`. NEVER eval_inner.
///
/// - `Symbol(?v)` → bindings[?v] (None if unbound — a ?v unbound in THIS condition
///   is a cross-condition join key, handled by the beta network in stone 3)
/// - `Keyword(:field)` → the named field of the fact
/// - Literal → its bare Value
fn resolve_operand(
    operand: &WatAST,
    fact_fields: &[Value],
    field_names: &[String],
    bindings: &rpds::HashTrieMapSync<Value, Value>,
) -> Option<Value> {
    match operand {
        WatAST::Symbol(ident, _) => {
            let name = ident.as_str();
            if name.starts_with('?') {
                // Logic variable: look up in bindings accumulated so far in this condition.
                let key = Value::String(Arc::new(name.to_string()));
                bindings.get(&key).cloned()
            } else {
                // A bare (non-?-prefix) symbol at operand position is not a
                // recognised operand form in the rete DSL.
                None
            }
        }
        WatAST::Keyword(k, _) => {
            // Field reference: :field-name → read from the fact.
            let field_name = k.strip_prefix(':').unwrap_or(k.as_str());
            read_fact_field(fact_fields, field_names, field_name)
        }
        // Literals: direct Value construction — no eval, no environment.
        WatAST::IntLit(n, _) => Some(Value::i64(*n)),
        WatAST::FloatLit(x, _) => Some(Value::f64(*x)),
        WatAST::BoolLit(b, _) => Some(Value::bool(*b)),
        WatAST::StringLit(s, _) => Some(Value::String(Arc::new(s.clone()))),
        // Anything else (nested list, vector, nil, …) is not a supported
        // v1 operand. These are `where`-territory (stone 6).
        _ => None,
    }
}

/// Resolve both operands of a binary clause `[head a b]`.
fn resolve_binary_operands(
    items: &[WatAST],
    fact_fields: &[Value],
    field_names: &[String],
    bindings: &rpds::HashTrieMapSync<Value, Value>,
) -> Option<(Value, Value)> {
    if items.len() != 3 {
        return None;
    }
    let a = resolve_operand(&items[1], fact_fields, field_names, bindings)?;
    let b = resolve_operand(&items[2], fact_fields, field_names, bindings)?;
    Some((a, b))
}

// ─── Public entry point: RHS insert-form evaluator ───────────────────────────

/// `(:wat::rete::eval-insert <insert-form: :wat::WatAST> <bindings: :wat::core::PersistentMap>)
/// -> :wat::Record`
///
/// The RHS dual of `eval_alpha_match`: where alpha-match is `(cond, fact) → Option<bindings>`,
/// eval-insert is `(insert-form, bindings) → fact`. Both sides reuse `resolve_operand`.
///
/// Entry point dispatched by `dispatch_keyword_head_value` in `runtime.rs`.
/// Evaluates both arguments, validates the insert form, resolves each fact-arg
/// via `resolve_operand` (empty fact-fields/names: RHS has no current fact),
/// and returns a `Value::wat__Record`.
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
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        } }.into());
    }

    // Evaluate arg[0]: must be Value::wat__WatAST wrapping a List.
    let form_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let form_ast = match form_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST (insert form from quote)",
                got: Box::new(ValueSnapshot::of(&other)),
            } }.into());
        }
    };

    // Evaluate arg[1]: must be Value::wat__core__PersistentMap (token bindings).
    let bindings_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    let bindings: rpds::HashTrieMapSync<Value, Value> = match bindings_val {
        Value::wat__core__PersistentMap(ref m) => m.clone(),
        other => {
            return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::PersistentMap (token bindings)",
                got: Box::new(ValueSnapshot::of(&other)),
            } }.into());
        }
    };

    // Validate the insert form: must be a List `(:wat::rete::insert <fact-form>)`.
    let insert_items = match &form_ast {
        WatAST::List(items, _) if !items.is_empty() => items.clone(),
        _ => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "List (:wat::rete::insert <fact-form>)",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(form_ast)))),
            } }.into());
        }
    };
    // Head must be the keyword :wat::rete::insert.
    let insert_head = match &insert_items[0] {
        WatAST::Keyword(k, _) if k.as_str() == ":wat::rete::insert" => k.as_str(),
        other => {
            return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "keyword :wat::rete::insert as form head",
                got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{other:?}"))))),
            } }.into());
        }
    };
    let _ = insert_head; // validated; not used further
    // Exactly 2 children: the :wat::rete::insert keyword + <fact-form>.
    if insert_items.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: insert_items.len(),
        } }.into());
    }

    // Extract the fact-form: <fact-form> = (:RecordType arg…) — a List with a keyword head.
    let fact_form_ast = &insert_items[1];
    let fact_items = match fact_form_ast {
        WatAST::List(items, _) if !items.is_empty() => items.as_slice(),
        _ => {
            return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "fact-form List (:RecordType arg…)",
                got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{fact_form_ast:?}"))))),
            } }.into());
        }
    };
    // Head of fact-form must be a keyword naming the record type.
    let type_keyword = match &fact_items[0] {
        WatAST::Keyword(k, _) => k.as_str(),
        other => {
            return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "keyword (record type) as fact-form head",
                got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{other:?}"))))),
            } }.into());
        }
    };
    // class_fqdn = keyword stripped of leading ':' (mirrors eval_record_of:12798).
    let class_fqdn = Arc::new(type_keyword.strip_prefix(':').unwrap_or(type_keyword).to_string());

    // Resolve each fact-form arg via resolve_operand with empty fact-fields/names.
    // RHS has no current fact — :field references are malformed; only ?var + literal resolve.
    // None → unresolved operand → RuntimeError (a malformed rule, not a silent drop).
    let mut struct_form: Vec<Value> = Vec::with_capacity(fact_items.len().saturating_sub(1));
    for arg in &fact_items[1..] {
        match resolve_operand(arg, &[], &[], &bindings) {
            Some(v) => struct_form.push(v),
            None => {
                return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "resolvable operand (?var or literal) in RHS fact-form",
                    got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{arg:?}"))))),
                } }.into());
            }
        }
    }

    Ok(Value::wat__Record {
        class_fqdn,
        struct_form: Arc::new(struct_form),
    })
}

// ─── Field read ───────────────────────────────────────────────────────────────

/// Read a named field from a fact's ordered field slice via the class's field
/// name list. The registry provides names in declaration order, matching the
/// `struct_form` / `fields` Vec positionally.
///
/// A name not found → `None` (Clara semantics: missing field = no match).
fn read_fact_field(
    fact_fields: &[Value],
    field_names: &[String],
    field_name: &str,
) -> Option<Value> {
    let idx = field_names.iter().position(|n| n == field_name)?;
    fact_fields.get(idx).cloned()
}

// ─── Value comparison ─────────────────────────────────────────────────────────

/// Pure ordering comparison for the numeric/string/bool types that the rete DSL's
/// `<`, `>`, `<=`, `>=` operators support.
///
/// Mirrors the Compare arm in `walk_match_clause` (`runtime.rs` ~:10615) but as a
/// pure value-level function: no `Environment`, no `EvalBreak`. Returns `None` for
/// incompatible types (Clara no-error: type mismatch = constraint fails = `None`).
fn compare_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::i64(x), Value::i64(y)) => Some(x.cmp(y)),
        (Value::u8(x), Value::u8(y)) => Some(x.cmp(y)),
        (Value::f64(x), Value::f64(y)) => x.partial_cmp(y),
        (Value::i64(x), Value::f64(y)) => (*x as f64).partial_cmp(y),
        (Value::f64(x), Value::i64(y)) => x.partial_cmp(&(*y as f64)),
        (Value::String(x), Value::String(y)) => Some(x.as_ref().cmp(y.as_ref())),
        (Value::bool(x), Value::bool(y)) => Some(x.cmp(y)),
        (Value::wat__core__keyword(x), Value::wat__core__keyword(y)) => Some(x.as_ref().cmp(y.as_ref())),
        // Incompatible types: ordering undefined → None (Clara no-error).
        _ => None,
    }
}
