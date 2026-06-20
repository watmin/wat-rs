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
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, TrackedValue, Value, ValueSnapshot};
use crate::span::Span;
use std::sync::Arc;

// ─── Fact abstraction ─────────────────────────────────────────────────────────

/// The type name and ordered field values extracted from whichever record
/// variant arrived as the fact. Decouples the matcher from the storage variant.
pub(crate) struct Fact<'a> {
    /// Class FQDN without leading colon, e.g. `"user::Temp"`.
    pub(crate) class_fqdn: &'a str,
    /// Field values in declaration order.
    pub(crate) fields: &'a [Value],
}

/// Extract a [`Fact`] from either record variant. Returns `None` for
/// non-record Values (Clara semantics: wrong fact type → no match).
pub(crate) fn fact_from_value(v: &Value) -> Option<Fact<'_>> {
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
pub(crate) fn alpha_match_inner(
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
pub(crate) fn resolve_operand(
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

/// `build_insert_fact` — the pure inner of `eval_insert`.
///
/// Given the outer `insert-form` AST (a `(:wat::rete::insert (:RecordType arg…))` list)
/// and the token `bindings`, validates the form, resolves each fact-arg via `resolve_operand`
/// (empty fact-fields/names: RHS has no current fact), and builds the `Value::wat__Record`.
///
/// Called from `eval_insert` (after arg evaluation) and from the production pass in
/// `kernel.rs` (which already has the form + bindings and calls this directly).
///
/// Raises `RuntimeError` on malformed form or unresolved operand. Never panics.
pub(crate) fn build_insert_fact(
    insert_form: &WatAST,
    bindings: &rpds::HashTrieMapSync<Value, Value>,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-insert";

    // Validate the insert form: must be a List `(:wat::rete::insert <fact-form>)`.
    // Borrow (do NOT clone) — this runs once per derived fact; cloning the form AST per fact was
    // pure waste (the fan-out residual). We only read items[0]/[1]/len here.
    let insert_items = match insert_form {
        WatAST::List(items, _) if !items.is_empty() => items,
        _ => {
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "List (:wat::rete::insert <fact-form>)",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(insert_form.clone())))),
            } }.into());
        }
    };
    // Head must be the keyword :wat::rete::insert.
    let insert_head = match &insert_items[0] {
        WatAST::Keyword(k, _) if k.as_str() == ":wat::rete::insert" => k.as_str(),
        other => {
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "keyword :wat::rete::insert as form head",
                got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{other:?}"))))),
            } }.into());
        }
    };
    let _ = insert_head; // validated; not used further
    // Exactly 2 children: the :wat::rete::insert keyword + <fact-form>.
    if insert_items.len() != 2 {
        return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ArityMismatch {
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
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
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
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
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
        match resolve_operand(arg, &[], &[], bindings) {
            Some(v) => struct_form.push(v),
            None => {
                return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
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

/// `(:wat::rete::eval-insert <insert-form: :wat::WatAST> <bindings: :wat::core::PersistentMap>)
/// -> :wat::Record`
///
/// The RHS dual of `eval_alpha_match`: where alpha-match is `(cond, fact) → Option<bindings>`,
/// eval-insert is `(insert-form, bindings) → fact`. Both sides reuse `resolve_operand`.
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

    // Delegate to the pure inner (the production pass calls this directly with the form + bindings).
    build_insert_fact(&form_ast, &bindings)
}

// ─── Field read ───────────────────────────────────────────────────────────────

/// Read a named field from a fact's ordered field slice via the class's field
/// name list. The registry provides names in declaration order, matching the
/// `struct_form` / `fields` Vec positionally.
///
/// A name not found → `None` (Clara semantics: missing field = no match).
pub(crate) fn read_fact_field(
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

// ─── P12c: step-payload ───────────────────────────────────────────────────────

/// Convert a resolved primitive `Value` to a literal `WatAST` node.
///
/// Used when rebuilding substituted constraint forms: each resolved operand
/// (always a primitive at this point) must be expressed as a literal AST node
/// so the resulting `(:op a' b')` list prints as `(:wat::core::< -5 0)` (the
/// substituted form — not the unsubstituted `(:wat::core::< ?c 0)`).
///
/// Panics/returns None for non-primitive values (should not occur in a
/// well-formed rete condition's operand position).
fn value_to_ast_literal(v: Value) -> Option<WatAST> {
    match v {
        Value::i64(n) => Some(WatAST::IntLit(n, Span::unknown())),
        Value::f64(x) => Some(WatAST::FloatLit(x, Span::unknown())),
        Value::bool(b) => Some(WatAST::BoolLit(b, Span::unknown())),
        Value::String(s) => Some(WatAST::StringLit((*s).clone(), Span::unknown())),
        Value::wat__core__keyword(k) => Some(WatAST::Keyword((*k).clone(), Span::unknown())),
        Value::Unit => Some(WatAST::NilLit(Span::unknown())),
        _ => None,
    }
}

/// `(:wat::rete::step-payload' session alpha-id bindings sfact supporting) -> :wat::rete::DerivationStep`
///
/// Arc 278 Stone P12c — the explain payload builder. Given one (sfact, alpha-id) match edge
/// from a Token's matches chain, builds the full `DerivationStep` payload:
///
/// - **pattern**: the matched condition's fact-type FQDN (AlphaNode tests[0] head keyword).
/// - **bindings** (per-step): the binder-clause vars that THIS condition bound, projected
///   from the token's accumulated bindings.
/// - **constraints**: the rule's satisfied predicates with bound values substituted:
///   `(:wat::core::< -5 0)` from `(:wat::core::< ?c 0)` with `?c=-5`.
///
/// **Faithfulness by construction**: both `resolve_operand` and the clause classifier are
/// REUSED directly from matcher.rs (this file) — they are the same paths that fired during
/// `alpha_match_inner`. The substituted constraint values cannot drift from what actually
/// matched.
///
/// Arguments:
///   - `session`    — `:wat::rete::Session` (carries `network` at struct_form[0])
///   - `alpha-id`   — `:wat::core::i64` (the AlphaNode id for this condition)
///   - `bindings`   — `:wat::core::PersistentMap` (the token's accumulated bindings)
///   - `sfact`      — `:wat::Record` (the supporting fact for this edge)
///   - `supporting` — `:wat::rete::DerivationNode` (the pre-computed recursive node)
///
/// Returns a `:wat::rete::DerivationStep` record.
pub(crate) fn eval_step_payload(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::step-payload'";

    if args.len() != 5 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 5,
            got: args.len(),
        } }.into());
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
        other => return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::i64 (alpha-id)",
            got: Box::new(ValueSnapshot::of(&other)),
        } }.into()),
    };

    // ── Extract the token bindings ────────────────────────────────────────────
    let token_bindings: rpds::HashTrieMapSync<Value, Value> = match bindings_val {
        Value::wat__core__PersistentMap(ref m) => m.clone(),
        other => return Err(RuntimeError { span: args[2].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::PersistentMap (token bindings)",
            got: Box::new(ValueSnapshot::of(&other)),
        } }.into()),
    };

    // ── Extract the supporting fact (sfact) + its field names ────────────────
    let sfact = match fact_from_value(&sfact_val) {
        Some(f) => f,
        None => return Err(RuntimeError { span: args[3].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::Record (supporting fact)",
            got: Box::new(ValueSnapshot::of(&sfact_val)),
        } }.into()),
    };
    let type_key = format!(":{}", sfact.class_fqdn);
    let sfact_field_names: Vec<String> = sym
        .types()
        .and_then(|t| match t.get(&type_key) {
            Some(crate::types::TypeDef::Record(rd)) => Some(rd.field_names.clone()),
            Some(crate::types::TypeDef::Struct(sd)) => {
                Some(sd.fields.iter().map(|(n, _)| n.clone()).collect())
            }
            _ => None,
        })
        .unwrap_or_default();

    // ── Get Session.network (struct_form[0]) + look up AlphaNode ─────────────
    let network = match &session_val {
        Value::wat__Record { struct_form, .. } => struct_form.get(0).cloned(),
        _ => None,
    };
    let network = match network {
        Some(n) => n,
        None => return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::rete::Session (record with network at field 0)",
            got: Box::new(ValueSnapshot::of(&session_val)),
        } }.into()),
    };

    // Reuse kernel's get_node to look up the AlphaNode by id.
    let alpha_node_val = match &network {
        Value::wat__core__PersistentMap(m) => m.get(&Value::i64(alpha_id)).cloned(),
        _ => None,
    };
    let alpha_node_val = match alpha_node_val {
        Some(n) => n,
        None => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "AlphaNode in network",
            got: Box::new(ValueSnapshot::of(&Value::i64(alpha_id))),
        } }.into()),
    };

    // ── Extract AlphaNode.tests[0] — the full condition WatAST ───────────────
    // AlphaNode struct_form: [id(0), tests(1), children(2)].
    // tests is PV<WatAST>; tests[0] is `(:FactType clause…)`.
    let cond_ast: WatAST = match &alpha_node_val {
        Value::wat__Record { struct_form, .. } => {
            match struct_form.get(1) {
                Some(Value::wat__core__PersistentVector(pv)) => {
                    match pv.first() {
                        Some(Value::wat__WatAST(ast)) => (**ast).clone(),
                        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: ":wat::WatAST in AlphaNode.tests[0]",
                            got: Box::new(ValueSnapshot::of(other.unwrap_or(&Value::Unit))),
                        } }.into()),
                    }
                }
                other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "PersistentVector at AlphaNode.tests (struct_form[1])",
                    got: Box::new(ValueSnapshot::of(other.unwrap_or(&Value::Unit))),
                } }.into()),
            }
        }
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::rete::AlphaNode (record)",
            got: Box::new(ValueSnapshot::of(other)),
        } }.into()),
    };

    // ── Classify the condition's clauses (REUSE the matcher's classifier) ─────
    // cond_ast = (:FactType clause…); items[0] = the head keyword (type FQDN).
    // items[1..] = the clauses — binders `(?v <- :field)` and constraints `(:op a b)`.
    let (cond_head, clauses) = match &cond_ast {
        WatAST::List(items, _) if !items.is_empty() => {
            let head = match &items[0] {
                WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
                other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "keyword head in condition form",
                    got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{other:?}"))))),
                } }.into()),
            };
            (head, items[1..].to_vec())
        }
        _ => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "List (condition form) in AlphaNode.tests[0]",
            got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(cond_ast)))),
        } }.into()),
    };

    // pattern = the type FQDN (head keyword without leading ':').
    let pattern = cond_head;

    // ── Walk clauses: classify + build constraints + collect binder var names ─
    // Reuse the matcher's OWN classifier (same shape checks as alpha_match_inner).
    // Binder: (?v <- :field) — collect ?v name.
    // Constraint: (:op a b) — resolve operands via resolve_operand, rebuild as WatAST.
    let mut binder_vars: Vec<String> = Vec::new();
    let mut constraints_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();

    for clause in &clauses {
        let items = match clause {
            WatAST::List(items, _) if !items.is_empty() => items.as_slice(),
            _ => continue, // non-list clause: skip (not a recognised shape)
        };

        match &items[0] {
            // ── Binder clause: (?v <- :field) ────────────────────────────────
            // REUSES the matcher's binder classification: Symbol(?v), arrow, Keyword(:field).
            WatAST::Symbol(head_ident, _) => {
                let var_name = head_ident.as_str();
                if !var_name.starts_with('?') { continue; }
                if items.len() != 3 { continue; }
                let is_arrow = matches!(&items[1], WatAST::Symbol(s, _) if s.as_str() == "<-");
                if !is_arrow { continue; }
                // Third element must be a field keyword — confirmed it's a binder.
                if keyword_payload(&items[2]).is_none() { continue; }
                binder_vars.push(var_name.to_string());
            }

            // ── Constraint clause: (:op a b) ─────────────────────────────────
            // REUSES resolve_operand for each operand — the same resolver that fired.
            // Rebuilds (:op a' b') as a WatAST with the resolved literal nodes.
            WatAST::Keyword(head_kw, _) => {
                let op_str = head_kw.as_str();
                // Only the comparison operators (not combinators/where).
                match op_str {
                    ":wat::core::=" | ":wat::core::not=" |
                    ":wat::core::<" | ":wat::core::>" |
                    ":wat::core::<=" | ":wat::core::>=" => {}
                    _ => continue, // combinators/where: skip for now
                }
                if items.len() != 3 { continue; }
                // resolve_operand REUSED directly — the same call the match used.
                let a_val = resolve_operand(&items[1], sfact.fields, &sfact_field_names, &token_bindings);
                let b_val = resolve_operand(&items[2], sfact.fields, &sfact_field_names, &token_bindings);
                let (Some(a_val), Some(b_val)) = (a_val, b_val) else { continue; };
                let (Some(a_ast), Some(b_ast)) = (value_to_ast_literal(a_val), value_to_ast_literal(b_val)) else { continue; };
                // Rebuild (:op a' b') as a WatAST — the substituted constraint form.
                let substituted = WatAST::List(
                    vec![
                        WatAST::Keyword(op_str.to_string(), Span::unknown()),
                        a_ast,
                        b_ast,
                    ],
                    Span::unknown(),
                );
                constraints_pv = constraints_pv.push_back(Value::wat__WatAST(Arc::new(substituted)));
            }

            _ => continue, // non-symbol/non-keyword head: skip
        }
    }

    // ── Per-step bindings: project token bindings to binder_vars only ─────────
    let mut step_bindings_pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for var_name in &binder_vars {
        let key = Value::String(Arc::new(var_name.clone()));
        if let Some(v) = token_bindings.get(&key) {
            step_bindings_pm = step_bindings_pm.insert(key, v.clone());
        }
    }

    // ── Build DerivationStep record ───────────────────────────────────────────
    // Field order (declaration order in rete.wat):
    //   supporting(0) <- :wat::rete::DerivationNode
    //   pattern(1)    <- :wat::core::String
    //   bindings(2)   <- :wat::core::PersistentMap<String, Value>
    //   constraints(3)<- :wat::core::PersistentVector<WatAST>
    static STEP_CLASS_FQDN: std::sync::OnceLock<Arc<String>> = std::sync::OnceLock::new();
    let step_class = STEP_CLASS_FQDN
        .get_or_init(|| Arc::new("wat::rete::DerivationStep".to_string()))
        .clone();

    Ok(Value::wat__Record {
        class_fqdn: step_class,
        struct_form: Arc::new(vec![
            supporting,                                              // supporting: DerivationNode
            Value::String(Arc::new(pattern)),                       // pattern: String (FQDN)
            Value::wat__core__PersistentMap(step_bindings_pm),      // bindings: PM<String, Value>
            Value::wat__core__PersistentVector(constraints_pv),     // constraints: PV<WatAST>
        ]),
    })
}

// ─── Arc 278 Stone 6b-i: eval-test ────────────────────────────────────────────

/// `(:wat::rete::eval-test <quoted-expr: :wat::WatAST> <bindings: :wat::core::PersistentMap>) -> :wat::core::bool`
///
/// Evaluates a boolean predicate expression against a token's merged bindings
/// (`?var → value`). Builds a CHILD `Environment` binding each `?var` to its
/// value, then calls `eval_inner(expr, &test_env, sym)`. The result MUST be
/// `Value::bool` — a `where` clause is a predicate; any other result is a
/// `TypeMismatch`.
///
/// Because the 6a fence (pure ∧ deterministic) proves safety at compile time,
/// no runtime purity mode is needed here.
pub(crate) fn eval_test(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-test";

    // Arity: exactly 2 args.
    if args.len() != 2 {
        return Err(RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        }
        .into());
    }

    // Arg 0: evaluate → must be Value::wat__WatAST (a quoted expr from :wat::core::quote).
    let expr_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let expr_ast = match expr_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError {
                span: args[0].span().clone(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            }
            .into());
        }
    };

    // Arg 1: evaluate → must be Value::wat__core__PersistentMap.
    let bindings_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    let map = match bindings_val {
        Value::wat__core__PersistentMap(ref m) => m.clone(),
        other => {
            return Err(RuntimeError {
                span: args[1].span().clone(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::core::PersistentMap (the token's merged bindings)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            }
            .into());
        }
    };

    // Build a CHILD Environment binding each ?var → value.
    // Keys are Value::String("?x"); env_key for a bare ?x symbol is "?x" directly.
    let mut b = env.child();
    for (k, v) in map.iter() {
        let name = match k {
            Value::String(s) => s.as_str().to_string(),
            _ => continue, // non-string key: skip (should not occur in a well-formed bindings map)
        };
        b = b.bind_unknown_span(name, TrackedValue::from(v.clone()));
    }
    let test_env = b.build();

    // Evaluate the predicate expr in the test env; result MUST be bool.
    match crate::runtime::eval_inner(&expr_ast, &test_env, sym)?.value_owned() {
        Value::bool(x) => Ok(Value::bool(x)),
        other => Err(RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::bool (a where predicate must return bool)",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        }
        .into()),
    }
}
