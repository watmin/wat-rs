//! Arc 278 Stone 2a — `alpha-match`: the rete single-fact matcher.
//!
//! Given a condition form (DATA, a `:wat::WatAST`) and a fact (a `:wat::core::Record`
//! — a `Value::Aggregate` whose `nature` is Record or HolonRecord), return
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
//! ## Clause classifier (consumes `clause.rs` — NOT `classify_clause`)
//!
//! `classify_clause` in `form_match.rs` is `form::matches?`'s grammar (`:=`,
//! bare `<` heads, `where`). The rete DSL uses `<-` bind, FQDN ops
//! (`:wat::rete::core::i64::>`), and `:wat::rete::and/or/not` combinators.
//! Generic `:wat::core::>` is freeze-walled (`NonReteConstraint`), not the rete spelling.
//! Shape recognition lives in `clause.rs`; this matcher is a consumer (`eval_clause`).
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
use crate::rete::clause::{classify_constraint_head, classify_rete_clause, CmpKind, ReteClauseShape};
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
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

/// Record/HolonRecord only. The one fact predicate: insert, compiled RHS,
/// and alpha all refuse Struct. `fact_from_value` is the match-shaped twin.
pub(crate) fn is_record_fact(v: &Value) -> bool {
    fact_from_value(v).is_some()
}

/// Extract a [`Fact`] from either record variant. Returns `None` for
/// non-record Values (Clara semantics: wrong fact type → no match).
pub(crate) fn fact_from_value(v: &Value) -> Option<Fact<'_>> {
    match v {
        // Record / HolonRecord only. Struct is not a fact — native fire
        // (`alpha_pass` / `alpha_activate_fact`) and insert (`require_record_fact`)
        // both exclude it.
        Value::Aggregate(a) if a.nature != crate::types::Nature::Struct => Some(Fact {
            class_fqdn: a.class.as_ref(),
            fields: a.fields.as_slice(),
        }),
        _ => None,
    }
}

/// Declared field names for a fact class (colon-free), read from the frozen type registry.
/// One reader of the registry — matcher, step_payload, alpha_tree, export, and arm compile.
///
/// That "one reader" claim used to be false: `validate.rs`'s `lookup_fields` reimplemented
/// the identical lookup — same key format, same registry `get`, same `TypeDef::Aggregate`
/// match, same collect — and is not in the list above. Both now go through
/// [`aggregate_field_names`], so the claim is enforced by there being one body rather than
/// asserted by a comment. (`solvere` raised this, then RETRACTED it; the retraction was
/// wrong and the two functions were byte-equivalent on the disk.)
pub(crate) fn class_field_names(sym: &SymbolTable, class: &str) -> Vec<String> {
    sym.types()
        .and_then(|t| aggregate_field_names(t, class))
        .unwrap_or_default()
}

/// THE registry read for "what fields does this aggregate declare?", keyed by `&TypeEnv`.
///
/// `class` is colon-free (`p::Rec`); the registry keys carry the leading colon, and that
/// `format!(":{class}")` is the one place the convention is written down. A caller holding a
/// `&SymbolTable` reaches it through [`class_field_names`]; a caller already holding a
/// `&TypeEnv` (the validator) calls this directly.
pub(crate) fn aggregate_field_names(
    types: &crate::types::TypeEnv,
    class: &str,
) -> Option<Vec<String>> {
    match types.get(&format!(":{class}")) {
        Some(crate::types::TypeDef::Aggregate(a)) => {
            Some(a.field_names().map(|s| s.to_string()).collect())
        }
        _ => None,
    }
}

/// Resolve a bare `{EnumPath}::{Variant}` head against the frozen registry.
///
/// ONE COPY. This resolution — `rsplit_once("::")`, registry `get`, match `TypeDef::Enum`,
/// find the variant by name across `Unit`/`Tagged` — was hand-written at THREE independent
/// sites: `purity.rs`'s `constructor_meta` (is this head pure/deterministic/total?),
/// `expr_ir.rs`'s lowerer (what arity must the call match?), and `validate.rs`'s
/// `walk_nested_constructors` (does the written arity agree?). `validate.rs`'s own comment
/// admitted it "mirrors `constructor_meta`'s own resolution".
///
/// Verified AGREEING before unification, unlike the `CallFallback` triplication earlier in
/// this arc, which looked identical and was a live native-vs-oracle divergence. Same shape,
/// different luck — which is the reason to collapse them rather than to trust the next one.
/// The three callers still differ in what they DO with the answer, and that part stays
/// theirs: purity returns an `OpMeta`, the lowerer raises `LowerError` on a mismatch, the
/// validator pushes a `ReteCheckError`.
/// Returns `(the enum, the variant name, its arity)` — all three, because the three callers
/// need different parts: the lowerer wants the `EnumDef` to reach `variant_names_arc`, the
/// validator wants the arity, and the purity classifier only wants to know it resolved.
pub(crate) fn enum_variant_ctor<'a>(
    types: &'a crate::types::TypeEnv,
    head: &'a str,
) -> Option<(&'a crate::types::EnumDef, &'a str, usize)> {
    let (enum_path, variant) = head.rsplit_once("::")?;
    let crate::types::TypeDef::Enum(e) = types.get(enum_path)? else {
        return None;
    };
    let arity = e.variants.iter().find_map(|v| match v {
        crate::types::EnumVariant::Unit(n) if n == variant => Some(0usize),
        crate::types::EnumVariant::Tagged { name, fields } if name == variant => {
            Some(fields.len())
        }
        _ => None,
    })?;
    Some((e, variant, arity))
}

// ─── Bindings — read-only accessor over either binding representation ─────────
//
// Native fire stores Element/Token bindings as `BindSpan` into `FireSession.bind_pool`;
// `BindView` (kernel/session.rs intern reader) is the borrowed reader. The
// oracle/differential still walks `PMap`. `Bindings` is the one trait that lets those
// readers stay agnostic without converting one representation into another.
//
// The trait must NEVER grow an `insert`. The moment it does, the two representations are forced
// through one interface again and the array is made to pay for the trie's one winning operation.
pub(crate) trait Bindings {
    fn get(&self, k: &Value) -> Option<&Value>;
    fn iter(&self) -> impl Iterator<Item = (&Value, &Value)>;
}

/// Fire-scoped bind view: key ids into `bind_keys`, filler ids into
/// `bind_vals` (`DESIGN-STONE-bind-key-intern`,
/// `DESIGN-STONE-bind-value-intern`).
pub(crate) type BindPairs = Option<Arc<[(Value, Value)]>>;
pub(crate) type FieldNames = Arc<Vec<String>>;
pub(crate) type BindAcc = Vec<(Value, Value)>;

impl Bindings for [(Value, Value)] {
    fn get(&self, k: &Value) -> Option<&Value> {
        <[(Value, Value)]>::iter(self)
            .find(|(kk, _)| kk == k)
            .map(|(_, v)| v)
    }
    fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        <[(Value, Value)]>::iter(self).map(|(k, v)| (k, v))
    }
}

impl Bindings for Vec<(Value, Value)> {
    fn get(&self, k: &Value) -> Option<&Value> {
        self.as_slice().iter().find(|(kk, _)| kk == k).map(|(_, v)| v)
    }
    fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.as_slice().iter().map(|(k, v)| (k, v))
    }
}

/// The user-facing `:wat::core::PersistentMap` surface — `eval_step_payload` receives token
/// bindings this way from a `:wat::rete::step-payload` caller. Delegates to `PMap`'s own
/// `get`/`iter`, which already dispatch on the array/trie arm internally; this impl adds nothing
/// beyond satisfying the trait, and per the trait's own rule above, it must never grow an `insert`.
impl Bindings for crate::value::pmap::PMap {
    fn get(&self, k: &Value) -> Option<&Value> {
        crate::value::pmap::PMap::get(self, k)
    }
    fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        crate::value::pmap::PMap::iter(self)
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
    eval_alpha_match_kind(args, list_span, env, sym, false)
}

pub(crate) fn eval_alpha_match_local(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_alpha_match_kind(args, list_span, env, sym, true)
}

/// `(:wat::rete::cond-has-deferred-constraint? cond) -> bool`
pub(crate) fn eval_cond_has_deferred_constraint(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::cond-has-deferred-constraint?";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let cond_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let cond_ast = match cond_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
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

fn eval_alpha_match_kind(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    local: bool,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::alpha-match";
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        }).into());
    }

    // Evaluate cond: must be Value::wat__WatAST wrapping a List.
    let cond_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let cond_ast = match cond_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST (condition form from quote)",
                got: Box::new(ValueSnapshot::of(&other)),
            }).into());
        }
    };

    // Evaluate fact: must be a record value (`Value::Aggregate`, nature Record/HolonRecord).
    let fact_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();

    // Resolve the fact's declared field names from the type registry.
    // The registry key carries the leading colon (e.g. ":user::Temp").
    // Falls back to an empty slice if the registry is absent (test harnesses
    // that bypass freeze) — binding clauses will return None at the first lookup.
    let fact = match fact_from_value(&fact_val) {
        Some(f) => f,
        None => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Record (a record fact)",
                got: Box::new(ValueSnapshot::of(&fact_val)),
            }).into());
        }
    };

    let field_names = class_field_names(sym, fact.class_fqdn);

    // Pure match: no environment, no eval, bindings as an array (element-side — see `Bindings`).
    let matched = if local {
        alpha_match_inner_local(Some(sym), &cond_ast, fact.class_fqdn, fact.fields, &field_names)
    } else {
        alpha_match_inner(Some(sym), &cond_ast, fact.class_fqdn, fact.fields, &field_names)
    };
    let result = matched.map(|b| attach_fact_bind(&cond_ast, &fact_val, b));
    pack_alpha_match_option(result)
}

fn pack_alpha_match_option(result: BindPairs) -> Result<Value, EvalBreak> {
    Ok(match result {
        // wat-contract boundary: this primitive's surface is `Option<PersistentMap>` — build one
        // from the array here (not the matcher hot path; this is a primitive dispatch, not
        // per-element construction inside `alpha_pass`/`alpha_match_inner`'s own fold).
        Some(bindings) => {
            let pm = crate::value::pmap::PMap::from_pairs(
                bindings.iter().map(|(k, v)| (k.clone(), v.clone())),
            );
            Value::Option(Arc::new(Some(Value::wat__core__PersistentMap(pm))))
        }
        None => Value::Option(Arc::new(None)),
    })
}

/// `(:wat::rete::alpha-match-under cond fact bindings) -> Option<PersistentMap>`
///
/// Same matcher as `alpha-match`, but `bindings` (a token's left-accumulated
/// `?vars`) seed the clause fold. Used by the oracle `:not` / `:exists` filter
/// so a constraint that names a left-bound var (`?v < ?m`) is a beta check,
/// not a silent alpha miss.
pub(crate) fn eval_alpha_match_under(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::alpha-match-under";
    if args.len() != 3 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 3,
            got: args.len(),
        }).into());
    }

    let cond_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let cond_ast = match cond_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST (condition form from quote)",
                got: Box::new(ValueSnapshot::of(&other)),
            }).into());
        }
    };

    let fact_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    let fact = match fact_from_value(&fact_val) {
        Some(f) => f,
        None => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Record (a record fact)",
                got: Box::new(ValueSnapshot::of(&fact_val)),
            }).into());
        }
    };

    let binds_val = crate::runtime::eval_inner(&args[2], env, sym)?.value_owned();
    let seed: Vec<(Value, Value)> = match &binds_val {
        Value::wat__core__PersistentMap(pm) => pm
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        other => {
            return Err(RuntimeError::new(args[2].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::PersistentMap (token bindings)",
                got: Box::new(ValueSnapshot::of(other)),
            }).into());
        }
    };

    let field_names = class_field_names(sym, fact.class_fqdn);

    let result = alpha_match_inner_seeded(
        Some(sym),
        &cond_ast,
        fact.class_fqdn,
        fact.fields,
        &field_names,
        &seed,
    )
    .map(|b| attach_fact_bind(&cond_ast, &fact_val, b));
    pack_alpha_match_option(result)
}

// ─── Pure inner matcher ────────────────────────────────────────────────────────

/// A top-level alpha condition: `(:Type clause…)` or `(?p <- :Type clause…)`.
pub(crate) struct AlphaPattern<'a> {
    pub fact_var: Option<&'a str>,
    pub type_head: &'a str,
    pub clauses: &'a [WatAST],
}

/// Parse the B-form `(?p <- :ns::Type …)` or the field-only `(:Type …)`.
pub(crate) fn alpha_pattern(cond: &WatAST) -> Option<AlphaPattern<'_>> {
    let items = match cond {
        WatAST::List(items, _) if !items.is_empty() => items.as_slice(),
        _ => return None,
    };
    match &items[0] {
        WatAST::Keyword(k, _) => Some(AlphaPattern {
            fact_var: None,
            type_head: k.trim_start_matches(':'),
            clauses: &items[1..],
        }),
        WatAST::Symbol(_, _) => match crate::rete::clause::classify_rete_clause(cond) {
            crate::rete::clause::ReteClauseShape::FactBind {
                var,
                type_head,
                clauses,
            } => Some(AlphaPattern {
                fact_var: Some(var),
                type_head,
                clauses,
            }),
            _ => None,
        },
        _ => None,
    }
}

/// Put the matched fact on `?p` when `cond` is `(?p <- :Type …)`.
pub(crate) fn attach_fact_bind(
    cond: &WatAST,
    fact: &Value,
    bindings: Arc<[(Value, Value)]>,
) -> Arc<[(Value, Value)]> {
    match alpha_pattern(cond).and_then(|p| p.fact_var) {
        Some(var) => {
            let mut out: Vec<(Value, Value)> = Vec::with_capacity(bindings.len() + 1);
            out.push((Value::String(Arc::new(var.to_string())), fact.clone()));
            out.extend(bindings.iter().map(|(k, v)| (k.clone(), v.clone())));
            out.into()
        }
        None => bindings,
    }
}

/// The pure core: no `Environment`, no `eval_inner`. Returns the binding array or
/// `None` (Clara no-error: any mismatch is `None`, never a raise).
///
/// Oracle / differential matcher. Native fire materializes via
/// `exec_compiled_with_key_ids` / `materialize_into` and seals `BindSpan`s there.
/// DESIGN-STONE-element-bindings-array: this path still returns a packed
/// `Vec<(Value, Value)>` (cheap: elements bind 1-2 vars in practice) rather than
/// folding an `rpds` trie — most of that stone's win on the oracle side.
pub(crate) fn alpha_match_inner(
    sym: Option<&SymbolTable>,
    cond: &WatAST,
    fact_class: &str,
    fact_fields: &[Value],
    field_names: &[String],
) -> BindPairs {
    alpha_match_inner_opts(sym, cond, fact_class, fact_fields, field_names, &[], false)
}

/// Empty-seed match that **defers** a constraint whose `?var` is not bound in
/// this condition (`?v < ?m` after an accum). Those facts still enter alpha;
/// `:not` / `:exists` re-check the full cond with `alpha-match-under` at beta.
/// Join alphas must not use this — a deferred join constraint would be lost
/// at `token_element_compatible`.
pub(crate) fn alpha_match_inner_local(
    sym: Option<&SymbolTable>,
    cond: &WatAST,
    fact_class: &str,
    fact_fields: &[Value],
    field_names: &[String],
) -> BindPairs {
    alpha_match_inner_opts(sym, cond, fact_class, fact_fields, field_names, &[], true)
}

/// Alpha-match with a seed binding map (token bindings already accumulated on the left).
///
/// A `?var` in an inline constraint that is not bound by THIS condition is a
/// cross-condition join key (`resolve_operand` says so). Empty-seed alpha
/// therefore drops every fact whose `:not` / `:exists` inner mentions a
/// left-bound var (`?v < ?m` after an accum). Clara evaluates that constraint
/// against the token. Seed the left bindings so the same cond is honest at
/// beta time. `alpha_match_inner` stays the empty-seed path.
pub(crate) fn alpha_match_inner_seeded(
    sym: Option<&SymbolTable>,
    cond: &WatAST,
    fact_class: &str,
    fact_fields: &[Value],
    field_names: &[String],
    seed: &[(Value, Value)],
) -> BindPairs {
    alpha_match_inner_opts(sym, cond, fact_class, fact_fields, field_names, seed, false)
}

fn alpha_match_inner_opts(
    sym: Option<&SymbolTable>,
    cond: &WatAST,
    fact_class: &str,
    fact_fields: &[Value],
    field_names: &[String],
    seed: &[(Value, Value)],
    defer_unbound: bool,
) -> BindPairs {
    let pat = alpha_pattern(cond)?;
    crate::rete::kernel::census_count("match:calls");
    if pat.type_head != fact_class {
        crate::rete::kernel::census_count("match:head-miss");
        return None;
    }
    eval_clauses(
        sym,
        pat.clauses,
        fact_fields,
        field_names,
        seed.to_vec(),
        defer_unbound,
    )
    .map(Into::into)
}

/// True when an inline constraint names a `?var` this condition does not bind.
/// Those constraints are beta (`alpha-match-under`), not empty-seed alpha.
pub(crate) fn cond_has_deferred_constraint(cond: &WatAST) -> bool {
    let Some(pat) = alpha_pattern(cond) else {
        return false;
    };
    let mut bound = std::collections::HashSet::new();
    collect_bind_vars(pat.clauses, &mut bound);
    clause_has_unbound_qvar(pat.clauses, &bound)
}

fn collect_bind_vars(clauses: &[WatAST], out: &mut std::collections::HashSet<String>) {
    for clause in clauses {
        match classify_rete_clause(clause) {
            ReteClauseShape::Bind { var, .. } => {
                out.insert(var.to_string());
            }
            ReteClauseShape::And(subs) => collect_bind_vars(subs, out),
            _ => {}
        }
    }
}

fn clause_has_unbound_qvar(clauses: &[WatAST], bound: &std::collections::HashSet<String>) -> bool {
    for clause in clauses {
        match classify_rete_clause(clause) {
            ReteClauseShape::Constraint { lhs, rhs, .. } => {
                if operand_is_unbound_qvar(lhs, bound) || operand_is_unbound_qvar(rhs, bound) {
                    return true;
                }
            }
            ReteClauseShape::And(subs) => {
                if clause_has_unbound_qvar(subs, bound) {
                    return true;
                }
            }
            ReteClauseShape::Or(subs) => {
                if clause_has_unbound_qvar(subs, bound) {
                    return true;
                }
            }
            ReteClauseShape::Not(inner)
                if clause_has_unbound_qvar(std::slice::from_ref(inner), bound) =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn operand_is_unbound_qvar(operand: &WatAST, bound: &std::collections::HashSet<String>) -> bool {
    match operand {
        WatAST::Symbol(ident, _) => {
            let name = ident.as_str();
            name.starts_with('?') && !bound.contains(name)
        }
        _ => false,
    }
}

fn operand_is_qvar(operand: &WatAST) -> bool {
    matches!(operand, WatAST::Symbol(ident, _) if ident.as_str().starts_with('?'))
}

/// Walk a slice of top-level condition clauses, threading bindings left→right.
/// Returns `None` on the first failure (short-circuit AND).
fn eval_clauses(
    sym: Option<&SymbolTable>,
    clauses: &[WatAST],
    fact_fields: &[Value],
    field_names: &[String],
    bindings: BindAcc,
    defer_unbound: bool,
) -> Option<BindAcc> {
    let mut current = bindings;
    for clause in clauses {
        crate::rete::kernel::census_count("match:clause");
        current = eval_clause(sym, clause, fact_fields, field_names, current, defer_unbound)?;
    }
    Some(current)
}

/// Classify and evaluate a single clause. Returns `Some(updated_bindings)` on
/// success, `None` on mismatch or unresolvable operand.
///
/// Arc 294 item 9a — re-pointed at the shared [`classify_rete_clause`] (S1 extraction).
/// BEHAVIOR-IDENTICAL to the pre-extraction hand-rolled match: `Exists`/`Accumulate` never
/// actually reach this fn at fire time (compile-condition consumes them earlier), so mapping
/// them to `None` here matches the prior default-arm outcome exactly.
fn eval_clause(
    sym: Option<&SymbolTable>,
    clause: &WatAST,
    fact_fields: &[Value],
    field_names: &[String],
    bindings: BindAcc,
    defer_unbound: bool,
) -> Option<BindAcc> {
    match classify_rete_clause(clause) {
        // ── bind clause: (?v <- :field) ──────────────────────────────────────
        ReteClauseShape::Bind { var, field } => {
            let field_value = read_fact_field(fact_fields, field_names, field)?;
            // Bind ?v → field value. If ?v was already bound in this condition,
            // treat it as a constraint: the bound value must equal the field value.
            // Linear scan (not `Bindings::get` — this is the concrete array accumulator
            // itself, not a generic reader; see the array's one losing op in the
            // DESIGN-STONE, accepted because elements bind 1-2 vars in practice).
            // Arc 278 DESIGN-STONE-compiled-conditions.md, row 2 of the scorecard — this
            // allocation (rebuilding the constant `"?var"` key on every call, including every
            // failing one) is exactly what the compiled executor eliminates. Counted, not timed,
            // so the differential can assert it at zero for the compiled path and non-zero here.
            crate::rete::kernel::census_count("match:key-alloc");
            let key = Value::String(Arc::new(var.to_string()));
            let existing = bindings.iter().find_map(|(k, v)| (*k == key).then(|| v.clone()));
            match existing {
                Some(v) if v != field_value => None, // conflict
                Some(_) => Some(bindings),            // already bound, equal
                None => {
                    // STOP-5: a fresh key only ever reaches `push` here — the arms above
                    // already handle "key present" (equal or conflicting), so no duplicate
                    // key can land in the array. This is the trie's free dedupe, reproduced.
                    crate::rete::kernel::census_count("match:bind-insert");
                    let mut bindings = bindings;
                    bindings.push((key, field_value));
                    Some(bindings)
                }
            }
        }

        // ── constraint: (:wat::rete::core::<ty>::<op> a b) ───────────────────
        // Admissible spelling is the per-type rete primitive. CoreGeneric
        // (`:wat::core::<op>`) is freeze-walled and compile_condition_local-refused.
        // Operands resolved from {bindings, field, literal}.
        ReteClauseShape::Constraint { op, lhs, rhs } => {
            let a = resolve_operand(lhs, fact_fields, field_names, &bindings, sym)
                .or_else(|| eval_computed_operand(sym, lhs, fact_fields, field_names, &bindings));
            let b = resolve_operand(rhs, fact_fields, field_names, &bindings, sym)
                .or_else(|| eval_computed_operand(sym, rhs, fact_fields, field_names, &bindings));
            let (a, b) = match (a, b) {
                (Some(a), Some(b)) => (a, b),
                _ if defer_unbound && (operand_is_qvar(lhs) || operand_is_qvar(rhs)) => {
                    return Some(bindings);
                }
                _ => return None,
            };
            // The ONE DOOR again — `classify_rete_clause` produced this `Constraint`, so the head
            // is in the table by construction. A `None` here means the two disagree, which is a
            // bug in this file, not a user error.
            let (kind, _spelling) = classify_constraint_head(op)
                .unwrap_or_else(|| unreachable!("classify_rete_clause admitted a Constraint head the ONE DOOR rejects: {op}"));
            let holds = match kind {
                CmpKind::Eq => a == b,
                CmpKind::NotEq => a != b,
                CmpKind::Lt => compare_values(&a, &b)? == std::cmp::Ordering::Less,
                CmpKind::Gt => compare_values(&a, &b)? == std::cmp::Ordering::Greater,
                CmpKind::Le => compare_values(&a, &b)? != std::cmp::Ordering::Greater,
                CmpKind::Ge => compare_values(&a, &b)? != std::cmp::Ordering::Less,
            };
            if holds { Some(bindings) } else { None }
        }

        // ── combinators ──────────────────────────────────────────────────────
        // :wat::rete::and — every sub-clause holds (thread bindings left→right).
        ReteClauseShape::And(subs) => {
            eval_clauses(sym, subs, fact_fields, field_names, bindings, defer_unbound)
        }
        // :wat::rete::or — ≥1 sub-clause holds. Bindings from a branch
        // do NOT survive past the `or` (which branch won is ambiguous).
        ReteClauseShape::Or(subs) => {
            let entry = bindings;
            for sub in subs {
                if eval_clause(
                    sym,
                    sub,
                    fact_fields,
                    field_names,
                    entry.clone(),
                    defer_unbound,
                )
                .is_some()
                {
                    return Some(entry);
                }
            }
            None
        }
        // :wat::rete::not — the sub-clause must NOT hold. Bindings from
        // the negated branch are discarded (no values to bind from a failed match).
        ReteClauseShape::Not(sub) => {
            let sub_matched =
                eval_clause(sym, sub, fact_fields, field_names, bindings.clone(), defer_unbound)
                    .is_some();
            if sub_matched {
                None
            } else {
                Some(bindings)
            }
        }

        // ── STOP: :wat::rete::where is stone 6 ───────────────────────────────
        // Arbitrary-expression eval belongs in a TestNode (stone 6), not here.
        // Reaching this arm means the caller used a `where` clause in a v1 condition.
        // Return None (Clara no-error: unhandled clause = no match).
        // A boolean rete expression written where a constraint goes. Evaluated through the SAME
        // core the operands use, and required to answer TRUE.
        //
        // A non-bool cannot reach here: `head_is_boolean_rete_predicate` admits only rows whose
        // `ret` is genuinely known, so `false` below means the predicate answered false — never
        // "it answered something else and we quietly treated that as no-match", which is the shape
        // fix-list F was.
        ReteClauseShape::Predicate(expr) => {
            match eval_computed_operand(sym, expr, fact_fields, field_names, &bindings) {
                Some(Value::bool(true)) => Some(bindings),
                _ => None,
            }
        }

        ReteClauseShape::Where(_) => None,

        // `exists`/`accumulate` are top-level `:when`-entry wrappers, consumed entirely by
        // compile-condition (wat/rete/compile.wat) before alpha-match runs — they never legitimately
        // reach a condition's clause list. Matches the pre-extraction default-arm outcome.
        ReteClauseShape::Exists(_)
        | ReteClauseShape::Accumulate { .. }
        | ReteClauseShape::FactBind { .. } => None,

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
///
/// Generic over [`Bindings`] — called with the element-side array accumulator (from
/// `eval_clause`'s `Constraint` arm, mid-fold), the token-side trie (`build_insert_fact`,
/// `eval_step_payload`), or in principle either (see the `Bindings` doc). Monomorphised per
/// call site — no vtable, no dispatch cost.
/// Evaluate an operand that is a NESTED CALL, through the one expression core.
///
/// ⛔ **THE ORACLE HAD FIX-LIST ENTRY F TOO, AND THAT IS WHY THE FUZZERS WERE BLIND.** Entry F was
/// fixed first in the COMPILED path (`compiled_cond`'s `Op::Eval`), and the new grid axis
/// immediately showed the `$oracle` still answering `n=0` where Clara and native both said 111.
/// Native and the oracle did not merely agree — they SHARED the defect, which is FM 28 in its
/// purest form: two engines agreeing proves nothing when the thing they agree on is the bug.
/// Clara broke the tie, which is the whole reason a third reference exists.
///
/// This is the interpreted twin of the compiled fix and it reuses the SAME core rather than
/// growing a second evaluator: field refs become reserved `?`-names bound to their values, the
/// operand lowers through `expr_ir::lower`, and `exec_value` runs it against those bindings —
/// the identical `Expr::Call`, opcode and `RETE_OPS` table the fence and the compiled path use.
///
/// `None` when there is no `SymbolTable` (no caller lacks one today) or when the operand is not a
/// call at all — in which case `resolve_operand` has already answered and this is never reached.
fn eval_computed_operand(
    sym: Option<&SymbolTable>,
    operand: &WatAST,
    fact_fields: &[Value],
    field_names: &[String],
    bindings: &BindAcc,
) -> Option<Value> {
    let WatAST::List(..) = operand else { return None };
    let sym = sym?;
    let mut binds: BindAcc = bindings.clone();
    let rewritten = rewrite_field_refs(operand, fact_fields, field_names, &mut binds)?;
    let program = crate::rete::expr_ir::lower(&rewritten, sym).ok()?;
    crate::rete::expr_ir::exec_value(&program, &binds, sym, operand.span()).ok()
}

/// Rewrite each FIELD-naming keyword in operand position into a reserved `?`-name, binding it to
/// that field's value. Only non-head positions, and only declared fields — so a call's head and a
/// marker like `:undefined` reach the core untouched.
fn rewrite_field_refs(
    ast: &WatAST,
    fact_fields: &[Value],
    field_names: &[String],
    binds: &mut BindAcc,
) -> Option<WatAST> {
    match ast {
        WatAST::Keyword(k, span) => {
            let name = k.strip_prefix(':').unwrap_or(k.as_str());
            let Some(idx) = field_names.iter().position(|n| n == name) else {
                return Some(ast.clone());
            };
            let v = fact_fields.get(idx)?.clone();
            // `%` cannot start a user symbol, so this cannot collide with a real `?var`.
            let reserved = format!("?%fld%{name}");
            binds.push((Value::String(Arc::new(reserved.clone())), v));
            Some(WatAST::Symbol(crate::scope::Identifier::bare(reserved), span.clone()))
        }
        WatAST::List(items, span) => {
            let mut out = Vec::with_capacity(items.len());
            for (i, it) in items.iter().enumerate() {
                out.push(if i == 0 {
                    it.clone()
                } else {
                    rewrite_field_refs(it, fact_fields, field_names, binds)?
                });
            }
            Some(WatAST::List(out, span.clone()))
        }
        // The oracle's twin of `compiled_cond::bind_field_refs`'s `Vector` arm — a vector has no
        // head, so every element is an operand position. Both engines shared the missing arm, which
        // is why five fuzzers and 5612 shapes were blind: they did not merely agree, they agreed on
        // the SAME wrong answer, exactly as in fix-list F.
        WatAST::Vector(items, span) => {
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                out.push(rewrite_field_refs(it, fact_fields, field_names, binds)?);
            }
            Some(WatAST::Vector(out, span.clone()))
        }
        // Unreachable — `:wat::rete::lower` refuses both literals — and named rather than swept up
        // for the same reason as the compiled twin. ⚠ ASYMMETRY, recorded rather than papered over:
        // a `None` here becomes NO MATCH at `eval_clause` (line ~688), while the compiled path
        // turns its `None` into a REFUSAL. If a map or set ever becomes lowerable, that difference
        // is a live divergence and this arm is where to fix it.
        WatAST::Map(..) | WatAST::Set(..) => None,
        // ⛔ THE WILDCARD IS DELETED ON PURPOSE — see the compiled twin. `other => clone` conflated
        // "leaf, leave it alone" with "no arm for this node"; naming every variant makes a new
        // `WatAST` variant a compile error here rather than a silent never-match.
        WatAST::IntLit(..)
        | WatAST::FloatLit(..)
        | WatAST::RationalLit(..)
        | WatAST::BigIntLit(..)
        | WatAST::BoolLit(..)
        | WatAST::StringLit(..)
        | WatAST::NilLit(..)
        | WatAST::Symbol(..) => Some(ast.clone()),
    }
}

pub(crate) fn resolve_operand<B: Bindings>(
    operand: &WatAST,
    fact_fields: &[Value],
    field_names: &[String],
    bindings: &B,
    sym: Option<&SymbolTable>,
) -> Option<Value> {
    match operand {
        WatAST::Symbol(ident, _) => {
            let name = ident.as_str();
            if name.starts_with('?') {
                // Logic variable: look up in bindings accumulated so far in this condition.
                // Arc 278 DESIGN-STONE-compiled-conditions.md, row 2 — second heap allocation
                // rebuilding the same constant key (see the `Bind` arm above); counted for the
                // same differential.
                crate::rete::kernel::census_count("match:key-alloc");
                let key = Value::String(Arc::new(name.to_string()));
                bindings.get(&key).cloned()
            } else {
                // A bare (non-?-prefix) symbol at operand position is not a
                // recognised operand form in the rete DSL.
                None
            }
        }
        WatAST::Keyword(k, _) => {
            // THE ONE RULE: a declared field is a field reference; anything else is a CONSTANT.
            //
            // ⚠ The `or_else` is new (2026-08-28) and is the oracle's half of admitting
            // `(keyword::= :v :alpha)` / `(enum::= :v :probe::E::A)` in DIRECT operand position.
            // Without it the two engines disagree — native resolves the constant and matches, the
            // oracle reads no field and returns `None`, which `eval_clause` maps to "no match".
            // Caught by driving both before shipping, which is the only way this class is ever
            // caught: `$native` and `$oracle` quietly disagreeing is the same instrument failure
            // as the two of them quietly agreeing.
            //
            // `keyword_value` is `expr_ir`'s — the same resolver the nested-operand path has
            // always used, so an enum unit variant becomes an enum value and everything else a
            // plain keyword. One question, one answer, one function.
            let field_name = k.strip_prefix(':').unwrap_or(k.as_str());
            read_fact_field(fact_fields, field_names, field_name)
                .or_else(|| sym.map(|s| crate::rete::expr_ir::keyword_value(k, s)))
        }
        // Literals: direct Value construction — no eval, no environment.
        other => ast_literal_value(other),
    }
}

/// WatAST literal → Value (Int/Float/Bool/String). Keyword-as-field stays out:
/// in operand position a keyword is a field reference, never a keyword value.
pub(crate) fn ast_literal_value(ast: &WatAST) -> Option<Value> {
    match ast {
        WatAST::IntLit(n, _) => Some(Value::i64(*n)),
        WatAST::FloatLit(x, _) => Some(Value::f64(*x)),
        WatAST::BoolLit(b, _) => Some(Value::bool(*b)),
        WatAST::StringLit(s, _) => Some(Value::String(Arc::new(s.clone()))),
        _ => None,
    }
}

/// Value → WatAST literal for explain substitution (`step_payload`).
/// Keywords and Unit are included here; [`ast_literal_value`] excludes them
/// because a keyword in operand position is a field ref, not a value.
// rune:solvere(load-bearing-coupling) — encode includes keyword/unit; decode
// for operands must not, or field refs become values.
pub(crate) fn value_to_ast_literal(v: Value) -> Option<WatAST> {
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
/// Routes the numeric arms through the one exact ordering door, `crate::value::
/// numeric_order` — arc 300 stone C5b — instead of the i64<->f64 coerce-to-f64 this
/// used to hand-roll (lossy above 2^53). Policy here DIFFERS from `walk_match_clause`'s
/// `RawClause::Compare` deliberately: NaN (`Incomparable`) maps to `None`, not `Equal`
/// — that disagreement between the two tables is real and preserved, per the stone.
/// Still no `Environment`, no `EvalBreak`, and still returns `None` for incompatible
/// types (Clara no-error: type mismatch = constraint fails = `None`). This table only
/// ever knew i64/u8/f64; no BigInt/Rational arms are added — see
/// `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-C5b-exact-mixed-numeric-order.md`
/// for why that widening does not apply here (`check_comparison` already rejects
/// mixed-numeric rete clauses before they can reach this function).
pub(crate) fn compare_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    use crate::value::numeric_order::{numeric_order, NumOrd};
    match (a, b) {
        (Value::i64(x), Value::i64(y)) => Some(x.cmp(y)),
        (Value::u8(x), Value::u8(y)) => Some(x.cmp(y)),
        (Value::f64(x), Value::f64(y)) => x.partial_cmp(y),
        (Value::i64(_), Value::f64(_)) | (Value::f64(_), Value::i64(_)) => {
            match numeric_order(a, b) {
                NumOrd::Ord(o) => Some(o),
                NumOrd::Incomparable => None,
                NumOrd::NotNumeric => None,
            }
        }
        (Value::String(x), Value::String(y)) => Some(x.as_ref().cmp(y.as_ref())),
        (Value::bool(x), Value::bool(y)) => Some(x.cmp(y)),
        (Value::wat__core__keyword(x), Value::wat__core__keyword(y)) => Some(x.as_ref().cmp(y.as_ref())),
        // Incompatible types: ordering undefined → None (Clara no-error).
        _ => None,
    }
}

#[cfg(test)]
mod c5b_compare_values_gate_tests {
    use super::*;

    /// Arc 300 stone C5b — `compare_values` is unreachable with mixed-numeric operands
    /// through the checked rete path (`check_comparison` unifies operand types before
    /// either rete comparison table is reached; see the design stone's reachability
    /// ruling). It has no wat-surface entry point, so it is exercised directly here —
    /// this is its only executable regression coverage for the fix.
    #[test]
    fn i64_f64_boundary_is_exact_not_lossy() {
        let above = Value::i64(9007199254740993); // 2^53 + 1
        let at_limit = Value::f64(9007199254740992.0); // 2^53, last exact f64 integer
        // RED at HEAD: coercing 2^53+1 down to f64 rounded it onto 2^53, comparing Equal.
        assert_eq!(compare_values(&at_limit, &above), Some(std::cmp::Ordering::Less));
        assert_eq!(compare_values(&above, &at_limit), Some(std::cmp::Ordering::Greater));
    }

    /// This caller's NaN policy DIFFERS from `values_compare`'s and `walk_match_clause`'s
    /// deliberately (table 3 disagreed before the collapse and must keep disagreeing):
    /// NaN maps to `None` here, not `Some(Equal)`. Losing this divergence would be STOP-2.
    #[test]
    fn nan_is_none_not_equal() {
        assert_eq!(compare_values(&Value::i64(1), &Value::f64(f64::NAN)), None);
        assert_eq!(compare_values(&Value::f64(f64::NAN), &Value::i64(1)), None);
    }

    /// Same-type fast paths are untouched (STOP-1 regression guard).
    #[test]
    fn same_type_fast_paths_unaffected() {
        assert_eq!(compare_values(&Value::i64(3), &Value::i64(5)), Some(std::cmp::Ordering::Less));
        assert_eq!(compare_values(&Value::u8(5), &Value::u8(3)), Some(std::cmp::Ordering::Greater));
    }

    /// Incompatible types stay `None` (Clara no-error), unaffected by the numeric fix.
    #[test]
    fn incompatible_types_stay_none() {
        assert_eq!(compare_values(&Value::i64(1), &Value::bool(true)), None);
    }
}
