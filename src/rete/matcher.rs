//! Arc 278 Stone 2a — `alpha-match`: the rete single-fact matcher.
//!
//! Given a condition form (DATA, a `:wat::WatAST`) and a fact (a `:wat::core::Record`
//! — either `Value::wat__core__Record` or `Value::wat__holon__Record`), return
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
use crate::value::value::AggregateValue;
use crate::types::Nature;
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
        // Arc 293.R2.1 — Aggregate covers Record, HolonRecord, and Struct.
        Value::Aggregate(a) => Some(Fact {
            class_fqdn: a.class.as_str(),
            fields: a.fields.as_slice(),
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

    // Evaluate fact: must be a record value (wat__core__Record, wat__holon__Record, or Struct).
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
                expected: ":wat::core::Record (a record fact)",
                got: Box::new(ValueSnapshot::of(&fact_val)),
            } }.into());
        }
    };

    // Arc 293.2b — Aggregate(kind!=Struct) = record, Aggregate(kind==Struct) = struct.
    let type_key = format!(":{}", fact.class_fqdn);
    let field_names: Vec<String> = sym
        .types()
        .and_then(|t| match t.get(&type_key) {
            Some(crate::types::TypeDef::Aggregate(a)) => {
                Some(a.field_names().map(|s| s.to_string()).collect())
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

/// Arc 294 item 9a (DESIGN-rete-defrule-wall.md, design call 1 — "one grammar, shared") —
/// the rete-DSL clause/condition-wrapper shape space, recognized identically whether the
/// caller is the runtime matcher (`eval_clause`, below) or the freeze-time validator
/// (`crate::rete::validate::validate_rete_rules`). A single source for "what shape is
/// this form" closes the drift hole that let the 9a codemod's injected bare-keyword
/// clauses classify as `Unrecognized` (silently `None`'d at fire time) instead of a
/// located freeze error.
///
/// Covers BOTH grammar levels the rete DSL actually has:
/// - within-condition CLAUSES (`Bind`, `Constraint`, `And`, `Or`, `Not`, `Where`) — the
///   shapes `eval_clause` classifies today (this extraction is behavior-identical).
/// - top-level `:when`-entry WRAPPERS (`Not`, `Exists`, `Where`, `Accumulate`) — shapes
///   `eval_clause` never actually receives (compile-condition, `wat/rete.wat`, consumes
///   them into NegationNode/ExistsNode/AccumulateNode/TestNode topology before alpha-match
///   ever runs), but the validator's top-level `:when` walk needs to recognize them too, via
///   this SAME function, rather than a second hand-rolled keyword-matcher (the drift risk
///   design call 1 rules out). `eval_clause`'s new dispatch maps `Exists`/`Accumulate` to
///   `None` — identical to the pre-extraction default arm, since those shapes never reach it.
///
/// `#[allow(dead_code)]`: `Where`'s payload and `Accumulate`'s `var`/`acc_form` are held for
/// shape-completeness (a future consumer — e.g. a wider validator scope, room 7's own
/// enumeration — reads them) even though today's two consumers (`eval_clause`, which always
/// maps these shapes to `None`/skips the fields, and the validator, which only reads
/// `Accumulate.from`) don't read every field.
#[allow(dead_code)]
pub(crate) enum ReteClauseShape<'a> {
    /// `(?v <- :field)` — a fresh/cross-condition-join bind.
    Bind { var: &'a str, field: &'a str },
    /// `(:wat::core::<op> a b)` — a binary FQDN comparison; operands unresolved (the
    /// caller resolves each via `resolve_operand`).
    Constraint {
        op: &'a str,
        lhs: &'a WatAST,
        rhs: &'a WatAST,
    },
    /// `(:wat::rete::and c1 c2 …)` — clause-level conjunction (within one condition).
    And(&'a [WatAST]),
    /// `(:wat::rete::or c1 c2 …)` — clause-level disjunction (within one condition).
    Or(&'a [WatAST]),
    /// `(:wat::rete::not inner)` — dual duty: a clause-level negated sub-clause (within one
    /// condition, `eval_clause` consumes this) OR a top-level negated condition wrapper (the
    /// validator's `:when`-entry walk consumes this) — same 2-item shape, disambiguated by
    /// the caller's own position in the walk, not by this classifier.
    Not(&'a WatAST),
    /// `(:wat::rete::exists inner)` — top-level-only existential condition wrapper.
    Exists(&'a WatAST),
    /// `(:wat::rete::where expr)` — dual duty like `Not`: a clause-level STOP arm (`eval_clause`
    /// always `None`s it — stone 6 territory) or the top-level `where` fence.
    Where(&'a WatAST),
    /// `(?result-var <- (<acc-form>) :from (<inner>))` — top-level-only accumulate wrapper.
    Accumulate {
        var: &'a str,
        acc_form: &'a WatAST,
        from: &'a WatAST,
    },
    /// Not a recognized rete-DSL shape at any level. `eval_clause` maps this to `None`
    /// (Clara no-error); the freeze-time validator maps this to a located
    /// `#wat.rete/MalformedClause` error.
    Unrecognized,
}

/// Classify a single rete-DSL form (a `:when` clause OR a top-level `:when`-entry wrapper)
/// by SHAPE alone — no fact/registry access, no bindings. See [`ReteClauseShape`].
pub(crate) fn classify_rete_clause(clause: &WatAST) -> ReteClauseShape<'_> {
    let items = match clause {
        WatAST::List(items, _) if !items.is_empty() => items.as_slice(),
        // Not a non-empty list — cannot be any recognized shape (e.g. a bare keyword,
        // the exact injected-`:celsius` corruption the wall exists to catch).
        _ => return ReteClauseShape::Unrecognized,
    };

    match &items[0] {
        // ── symbol-headed: bind or accumulate ────────────────────────────────
        WatAST::Symbol(head_ident, _) => {
            let var_name = head_ident.as_str();
            if !var_name.starts_with('?') {
                return ReteClauseShape::Unrecognized;
            }
            // Bind: (?v <- :field) — [Symbol(?v), Symbol(<-), Keyword(:field)].
            if items.len() == 3 {
                let is_arrow = matches!(&items[1], WatAST::Symbol(s, _) if s.as_str() == "<-");
                if is_arrow {
                    if let Some(field_kw) = keyword_payload(&items[2]) {
                        let field = field_kw.strip_prefix(':').unwrap_or(field_kw);
                        return ReteClauseShape::Bind { var: var_name, field };
                    }
                }
                return ReteClauseShape::Unrecognized;
            }
            // Accumulate: (?result <- (acc-form) :from (inner)) — 5 items, `:from` at [3].
            if items.len() == 5 {
                let is_arrow = matches!(&items[1], WatAST::Symbol(s, _) if s.as_str() == "<-");
                let is_from = matches!(&items[3], WatAST::Keyword(k, _) if k.as_str() == ":from");
                if is_arrow && is_from {
                    return ReteClauseShape::Accumulate {
                        var: var_name,
                        acc_form: &items[2],
                        from: &items[4],
                    };
                }
            }
            ReteClauseShape::Unrecognized
        }

        // ── keyword-headed clause ─────────────────────────────────────────────
        WatAST::Keyword(head_kw, _) => match head_kw.as_str() {
            // ── constraint: (:wat::core::<op> a b) ───────────────────────────
            ":wat::core::=" | ":wat::core::not=" | ":wat::core::<" | ":wat::core::>"
            | ":wat::core::<=" | ":wat::core::>=" => {
                if items.len() == 3 {
                    ReteClauseShape::Constraint { op: head_kw.as_str(), lhs: &items[1], rhs: &items[2] }
                } else {
                    ReteClauseShape::Unrecognized
                }
            }
            // ── combinators ──────────────────────────────────────────────────
            ":wat::rete::and" => ReteClauseShape::And(&items[1..]),
            ":wat::rete::or" => ReteClauseShape::Or(&items[1..]),
            ":wat::rete::not" => {
                if items.len() == 2 { ReteClauseShape::Not(&items[1]) } else { ReteClauseShape::Unrecognized }
            }
            ":wat::rete::exists" => {
                if items.len() == 2 { ReteClauseShape::Exists(&items[1]) } else { ReteClauseShape::Unrecognized }
            }
            ":wat::rete::where" => {
                if items.len() == 2 { ReteClauseShape::Where(&items[1]) } else { ReteClauseShape::Unrecognized }
            }
            // Unknown head keyword → unrecognised clause shape.
            _ => ReteClauseShape::Unrecognized,
        },

        // Non-symbol, non-keyword head → unrecognised clause shape.
        _ => ReteClauseShape::Unrecognized,
    }
}

/// Classify and evaluate a single clause. Returns `Some(updated_bindings)` on
/// success, `None` on mismatch or unresolvable operand.
///
/// Arc 294 item 9a — re-pointed at the shared [`classify_rete_clause`] (S1 extraction).
/// BEHAVIOR-IDENTICAL to the pre-extraction hand-rolled match: `Exists`/`Accumulate` never
/// actually reach this fn at fire time (compile-condition consumes them earlier), so mapping
/// them to `None` here matches the prior default-arm outcome exactly.
fn eval_clause(
    clause: &WatAST,
    fact_fields: &[Value],
    field_names: &[String],
    bindings: rpds::HashTrieMapSync<Value, Value>,
) -> Option<rpds::HashTrieMapSync<Value, Value>> {
    match classify_rete_clause(clause) {
        // ── bind clause: (?v <- :field) ──────────────────────────────────────
        ReteClauseShape::Bind { var, field } => {
            let field_value = read_fact_field(fact_fields, field_names, field)?;
            // Bind ?v → field value. If ?v was already bound in this condition,
            // treat it as a constraint: the bound value must equal the field value.
            let key = Value::String(Arc::new(var.to_string()));
            match bindings.get(&key) {
                Some(existing) if existing != &field_value => None, // conflict
                Some(_) => Some(bindings),                          // already bound, equal
                None => Some(bindings.insert(key, field_value)),    // fresh binding
            }
        }

        // ── constraint: (:wat::core::<op> a b) ───────────────────────────────
        // FQDN comparison ops; operands resolved from {bindings, field, literal}.
        ReteClauseShape::Constraint { op, lhs, rhs } => {
            let a = resolve_operand(lhs, fact_fields, field_names, &bindings)?;
            let b = resolve_operand(rhs, fact_fields, field_names, &bindings)?;
            let holds = match op {
                ":wat::core::=" => a == b,
                ":wat::core::not=" => a != b,
                ":wat::core::<" => compare_values(&a, &b)? == std::cmp::Ordering::Less,
                ":wat::core::>" => compare_values(&a, &b)? == std::cmp::Ordering::Greater,
                ":wat::core::<=" => compare_values(&a, &b)? != std::cmp::Ordering::Greater,
                ":wat::core::>=" => compare_values(&a, &b)? != std::cmp::Ordering::Less,
                // classify_rete_clause only ever produces Constraint for the 6 ops above.
                _ => unreachable!("classify_rete_clause: Constraint op outside the recognized set"),
            };
            if holds { Some(bindings) } else { None }
        }

        // ── combinators ──────────────────────────────────────────────────────
        // :wat::rete::and — every sub-clause holds (thread bindings left→right).
        ReteClauseShape::And(subs) => eval_clauses(subs, fact_fields, field_names, bindings),
        // :wat::rete::or — ≥1 sub-clause holds. Bindings from a branch
        // do NOT survive past the `or` (which branch won is ambiguous).
        ReteClauseShape::Or(subs) => {
            let entry = bindings;
            for sub in subs {
                if eval_clause(sub, fact_fields, field_names, entry.clone()).is_some() {
                    return Some(entry);
                }
            }
            None
        }
        // :wat::rete::not — the sub-clause must NOT hold. Bindings from
        // the negated branch are discarded (no values to bind from a failed match).
        ReteClauseShape::Not(sub) => {
            let sub_matched = eval_clause(sub, fact_fields, field_names, bindings.clone()).is_some();
            if sub_matched { None } else { Some(bindings) }
        }

        // ── STOP: :wat::rete::where is stone 6 ───────────────────────────────
        // Arbitrary-expression eval belongs in a TestNode (stone 6), not here.
        // Reaching this arm means the caller used a `where` clause in a v1 condition.
        // Return None (Clara no-error: unhandled clause = no match).
        ReteClauseShape::Where(_) => None,

        // `exists`/`accumulate` are top-level `:when`-entry wrappers, consumed entirely by
        // compile-condition (wat/rete.wat) before alpha-match runs — they never legitimately
        // reach a condition's clause list. Matches the pre-extraction default-arm outcome.
        ReteClauseShape::Exists(_) | ReteClauseShape::Accumulate { .. } => None,

        // Unrecognised clause shape → None.
        ReteClauseShape::Unrecognized => None,
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

// ─── Public entry point: RHS insert-form evaluator ───────────────────────────

/// `build_insert_fact` — the pure inner of `eval_insert`.
///
/// Given the outer `insert-form` AST (a `(:wat::rete::insert (:RecordType arg…))` list)
/// and the token `bindings`, validates the form, resolves each fact-arg via `resolve_operand`
/// (empty fact-fields/names: RHS has no current fact), and builds the `Value::wat__core__Record`.
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
            return Err(RuntimeError { span: crate::rust_caller_span!(), kind: RuntimeErrorKind::TypeMismatch {
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
            return Err(RuntimeError { span: crate::rust_caller_span!(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "keyword :wat::rete::insert as form head",
                got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{other:?}"))))),
            } }.into());
        }
    };
    let _ = insert_head; // validated; not used further
    // Exactly 2 children: the :wat::rete::insert keyword + <fact-form>.
    if insert_items.len() != 2 {
        return Err(RuntimeError { span: crate::rust_caller_span!(), kind: RuntimeErrorKind::ArityMismatch {
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
            return Err(RuntimeError { span: crate::rust_caller_span!(), kind: RuntimeErrorKind::TypeMismatch {
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
            return Err(RuntimeError { span: crate::rust_caller_span!(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "keyword (record type) as fact-form head",
                got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{other:?}"))))),
            } }.into());
        }
    };
    // class = keyword stripped of leading ':' (Arc 293.R2.1: colon-free).
    let class = type_keyword.strip_prefix(':').unwrap_or(type_keyword).to_string();

    // Arc 294 item 9a — a defrule :then RHS fact-form may be written in KWARGS form
    // `(:Type :field1 v1 :field2 v2)` (the flip's encouraged form, symmetric with the
    // field-named :when patterns) or the legacy positional `(:Type v1 v2)`. build_insert_fact
    // is a pure fire-time fn with no type registry, so for the kwargs form it takes the
    // VALUES in written order, skipping the :field keywords — fields are authored in the
    // type's declaration order (both the kwargs migration and the macro companion emit
    // declaration order). Follow-up: compile-time reorder-by-name (field-names-of) would make
    // an out-of-declaration-order kwargs RHS correct rather than positionally mapped.
    let args = &fact_items[1..];
    let is_kwargs = args.len() >= 2
        && args.len() % 2 == 0
        && args.iter().step_by(2).all(|a| matches!(a, WatAST::Keyword(_, _)));
    let value_asts: Vec<&WatAST> = if is_kwargs {
        args.iter().skip(1).step_by(2).collect()
    } else {
        args.iter().collect()
    };
    // Resolve each value via resolve_operand with empty fact-fields/names.
    // RHS has no current fact — only ?var + literal resolve; None → malformed rule.
    let mut fields: Vec<Value> = Vec::with_capacity(value_asts.len());
    for arg in value_asts {
        match resolve_operand(arg, &[], &[], bindings) {
            Some(v) => fields.push(v),
            None => {
                return Err(RuntimeError { span: crate::rust_caller_span!(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "resolvable operand (?var or literal) in RHS fact-form",
                    got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{arg:?}"))))),
                } }.into());
            }
        }
    }

    Ok(Value::Aggregate(Arc::new(AggregateValue::record(class, Arc::new(fields)))))
}

/// `(:wat::rete::eval-insert <insert-form: :wat::WatAST> <bindings: :wat::core::PersistentMap>)
/// -> :wat::core::Record`
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
        Value::i64(n) => Some(WatAST::IntLit(n, crate::rust_caller_span!())),
        Value::f64(x) => Some(WatAST::FloatLit(x, crate::rust_caller_span!())),
        Value::bool(b) => Some(WatAST::BoolLit(b, crate::rust_caller_span!())),
        Value::String(s) => Some(WatAST::StringLit((*s).clone(), crate::rust_caller_span!())),
        Value::wat__core__keyword(k) => Some(WatAST::Keyword((*k).clone(), crate::rust_caller_span!())),
        Value::Unit => Some(WatAST::NilLit(crate::rust_caller_span!())),
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
    const OP: &str = ":wat::rete::step-payload'";  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed

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
            expected: ":wat::core::Record (supporting fact)",
            got: Box::new(ValueSnapshot::of(&sfact_val)),
        } }.into()),
    };
    let type_key = format!(":{}", sfact.class_fqdn);
    // Arc 293.2b — Aggregate covers both record and struct field name lookup.
    let sfact_field_names: Vec<String> = sym
        .types()
        .and_then(|t| match t.get(&type_key) {
            Some(crate::types::TypeDef::Aggregate(a)) => Some(a.field_names().map(|s| s.to_string()).collect()),
            _ => None,
        })
        .unwrap_or_default();

    // ── Get Session.network (fields[0]) + look up AlphaNode ─────────────
    let network = match &session_val {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.get(0).cloned(),
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
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            match a.fields.get(1) {
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
                        WatAST::Keyword(op_str.to_string(), list_span.clone()),
                        a_ast,
                        b_ast,
                    ],
                    list_span.clone(),
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

    Ok(Value::Aggregate(Arc::new(AggregateValue::record(
        (*step_class).clone(),
        Arc::new(vec![
            supporting,                                              // supporting: DerivationNode
            Value::String(Arc::new(pattern)),                       // pattern: String (FQDN)
            Value::wat__core__PersistentMap(step_bindings_pm),      // bindings: PM<String, Value>
            Value::wat__core__PersistentVector(constraints_pv),     // constraints: PV<WatAST>
        ]),
    ))))
}

// ─── Arc 278 Stone 6b-i: eval-test ────────────────────────────────────────────

/// Core evaluator for a `where` predicate — callable directly from the native kernel
/// without going through the `eval-test` dispatch wrapper.
///
/// Builds a CHILD `Environment` from `bindings` (keys are `Value::String("?x")`),
/// evaluates `expr` in it, and requires `Value::bool`. Called by:
/// - `eval_test` (the dispatch wrapper for the `(:wat::rete::eval-test …)` surface), and
/// - `fire_fixpoint_delta`'s test-pass (stone 6b-ii-b), which already holds a
///   native `rpds::HashTrieMapSync<Value,Value>` and a `WatAST` from the TestNode.
///
/// A fresh `env` (typically `&Environment::new()`) should be passed — the only names
/// a `where` expression may reference are its `?vars` (from `bindings`) and
/// `sym`'s registered user functions.
pub(crate) fn eval_test_core(
    expr: &WatAST,
    bindings: &rpds::HashTrieMapSync<Value, Value>,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<bool, EvalBreak> {
    const OP: &str = ":wat::rete::eval-test";

    // Build a CHILD Environment binding each ?var → value.
    let mut b = env.child();
    for (k, v) in bindings.iter() {
        let name = match k {
            Value::String(s) => s.as_str().to_string(),
            _ => continue, // non-string key: skip (should not occur in well-formed bindings)
        };
        b = b.bind_unknown_span(name, TrackedValue::from(v.clone()));
    }
    let test_env = b.build();

    // Evaluate the predicate expr in the test env; result MUST be bool.
    match crate::runtime::eval_inner(expr, &test_env, sym)?.value_owned() {
        Value::bool(x) => Ok(x),
        other => Err(RuntimeError {
            span: expr.span().clone(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::bool (a where predicate must return bool)",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        }
        .into()),
    }
}

/// `(:wat::rete::eval-test <quoted-expr: :wat::WatAST> <bindings: :wat::core::PersistentMap>) -> :wat::core::bool`
///
/// Dispatch wrapper: evaluates the two args, extracts the `WatAST` and `PersistentMap`,
/// then delegates to `eval_test_core`. No behavior change from the previous monolithic
/// implementation — the core extraction is a refactor only.
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

    // Delegate to the core evaluator (a fresh env: where sees only ?vars + sym's user fns).
    Ok(Value::bool(eval_test_core(&expr_ast, &map, env, sym)?))
}
