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

// ─── Bindings — read-only accessor over either binding representation ─────────
//
// Arc 278 DESIGN-STONE-element-bindings-array + DESIGN-STONE-token-bindings-promoting.
// `Element.bindings` is `Arc<[(Value, Value)]>` (built once by `alpha_match_inner`,
// read/cloned/dropped forever after — never extended); `Token.bindings` is a `PMap` (array below
// `PROMOTION_THRESHOLD`, trie above it — a Token DOES extend, via `PMap::extend`, one clone of
// the backing storage rather than one clone per key). This matcher reads ALL THREE kinds —
// `resolve_operand` (token-side at `build_insert_fact`/`eval_step_payload`, element-side inside
// `eval_clause`'s own fold) and `eval_test_core` (token-side today; a `:test` clause can in
// principle sit after either side of a join) — and kernel.rs's join code reads both too (`key_of`
// walks a Token's `PMap` OR an Element's array depending on the caller). `Bindings` is the ONLY
// thing that lets those readers stay agnostic without converting one representation into another.
//
// The trait must NEVER grow an `insert`. The moment it does, the two representations are forced
// through one interface again and the array is made to pay for the trie's one winning operation.
pub(crate) trait Bindings {
    fn get(&self, k: &Value) -> Option<&Value>;
    fn iter(&self) -> impl Iterator<Item = (&Value, &Value)>;
}

impl Bindings for rpds::HashTrieMapSync<Value, Value> {
    fn get(&self, k: &Value) -> Option<&Value> {
        rpds::HashTrieMapSync::get(self, k)
    }
    fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        rpds::HashTrieMapSync::iter(self)
    }
}

impl Bindings for Arc<[(Value, Value)]> {
    fn get(&self, k: &Value) -> Option<&Value> {
        self.as_ref().iter().find(|(kk, _)| kk == k).map(|(_, v)| v)
    }
    fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.as_ref().iter().map(|(k, v)| (k, v))
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
/// bindings this way from a `:wat::rete::eval-step-payload'` caller. Delegates to `PMap`'s own
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

    // Evaluate fact: must be a record value (wat__core__Record, wat__holon__Record, or Struct).
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

    // Pure match: no environment, no eval, bindings as an array (element-side — see `Bindings`).
    let result = alpha_match_inner(&cond_ast, fact.class_fqdn, fact.fields, &field_names)
        .map(|b| attach_fact_bind(&cond_ast, &fact_val, b));
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

    let result = alpha_match_inner_seeded(
        &cond_ast,
        fact.class_fqdn,
        fact.fields,
        &field_names,
        &seed,
    )
    .map(|b| attach_fact_bind(&cond_ast, &fact_val, b));
    Ok(match result {
        Some(bindings) => {
            let pm = crate::value::pmap::PMap::from_pairs(
                bindings.iter().map(|(k, v)| (k.clone(), v.clone())),
            );
            Value::Option(Arc::new(Some(Value::wat__core__PersistentMap(pm))))
        }
        None => Value::Option(Arc::new(None)),
    })
}

// ─── Pure inner matcher ────────────────────────────────────────────────────────

/// The pure core: no `Environment`, no `eval_inner`. Returns the binding array or
/// `None` (Clara no-error: any mismatch is `None`, never a raise).
///
/// DESIGN-STONE-element-bindings-array: this is where an Element's bindings are BUILT — the
/// accumulator folds a plain `Vec<(Value, Value)>` (cheap: elements bind 1-2 vars in practice),
/// sealed into the `Arc<[(Value, Value)]>` `Element.bindings` wants via `.into()` at the end.
/// Building an array here instead of folding an `rpds` trie is most of this stone's win.
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
        WatAST::Symbol(s, _)
            if s.as_str().starts_with('?')
                && items.len() >= 3
                && matches!(&items[1], WatAST::Symbol(a, _) if a.as_str() == "<-") =>
        {
            let kw = match &items[2] {
                WatAST::Keyword(k, _) if k.contains("::") => k.as_str(),
                _ => return None,
            };
            Some(AlphaPattern {
                fact_var: Some(s.as_str()),
                type_head: kw.trim_start_matches(':'),
                clauses: &items[3..],
            })
        }
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

pub(crate) fn alpha_match_inner(
    cond: &WatAST,
    fact_class: &str,
    fact_fields: &[Value],
    field_names: &[String],
) -> Option<Arc<[(Value, Value)]>> {
    alpha_match_inner_seeded(cond, fact_class, fact_fields, field_names, &[])
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
    cond: &WatAST,
    fact_class: &str,
    fact_fields: &[Value],
    field_names: &[String],
    seed: &[(Value, Value)],
) -> Option<Arc<[(Value, Value)]>> {
    let pat = alpha_pattern(cond)?;
    crate::rete::kernel::census_count("match:calls");
    if pat.type_head != fact_class {
        crate::rete::kernel::census_count("match:head-miss");
        return None;
    }
    eval_clauses(pat.clauses, fact_fields, field_names, seed.to_vec()).map(Into::into)
}

/// Walk a slice of top-level condition clauses, threading bindings left→right.
/// Returns `None` on the first failure (short-circuit AND).
fn eval_clauses(
    clauses: &[WatAST],
    fact_fields: &[Value],
    field_names: &[String],
    bindings: Vec<(Value, Value)>,
) -> Option<Vec<(Value, Value)>> {
    let mut current = bindings;
    for clause in clauses {
        crate::rete::kernel::census_count("match:clause");
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
    /// `(:wat::rete::core::<ty>::<op> a b)` — a binary FQDN comparison; operands unresolved (the
    /// caller resolves each via `resolve_operand`). The generic `:wat::core::<op>` spelling also
    /// classifies here, deliberately — see [`classify_constraint_head`]; it is recognized so the
    /// diagnostic can name it, and refused by the validator.
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
    /// `(?p <- :ns::Type clause…)` — top-level fact bind (Clara `[?p <- Type]`).
    /// Discriminated from [`Self::Bind`] by a `::` in the type keyword; from
    /// [`Self::Accumulate`] by a keyword (not a list) after `<-`.
    FactBind {
        var: &'a str,
        type_head: &'a str,
        clauses: &'a [WatAST],
    },
    /// Not a recognized rete-DSL shape at any level. `eval_clause` maps this to `None`
    /// (Clara no-error); the freeze-time validator maps this to a located
    /// `#wat.rete/MalformedClause` error.
    Unrecognized,
}

/// The comparison an inline alpha constraint performs — independent of the type it is spelled at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CmpKind {
    Eq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
}

/// How a constraint head was SPELLED. Orthogonal to *which* comparison it is, and it is the law-A
/// axis: the spelling decides admissibility, the [`CmpKind`] decides behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConstraintSpelling {
    /// `:wat::rete::core::<ty>::<op>` — a rete primitive, monomorphic at `ty`. ADMISSIBLE.
    Rete { ty: &'static str },
    /// `:wat::core::<op>` — the generic core comparator. Recognized here **on purpose**, so the
    /// validator can name the head and point at its per-type twin (R29 `RVINA ERVDIT` — falling
    /// through to `Unrecognized`/`MalformedClause` would be a lie: the clause is well-formed, it
    /// is NON-RETE). REFUSED by `validate_clause`; never legitimately reaches evaluation.
    CoreGeneric,
}

/// ★ ONE DOOR for an inline alpha-constraint head — the single place the constraint vocabulary is
/// written down.
///
/// Before this existed, the six generic core spellings were matched by literal string in FOUR
/// independent places (this grammar, `eval_clause`, `compiled_cond::compile_clause`,
/// `alpha_tree::collect_equalities`), each re-asserting a closed set nothing enforced. That is the
/// arc's recurring defect class — a match on a literal STRING no exhaustiveness check can see —
/// and it is why law A never reached this surface. All four now read this function.
///
/// **Why the per-type rows and not a generic rete comparator:** generic `>` is PARTIAL — it routes
/// through `compare_values`, which errors on incomparable operands. `i64::>` has no such case.
/// Monomorphising *deletes* the domain hole rather than handling it, which is the standing ruling
/// ("the rete surface is per-type, period") and the reason zero generic rete comparators exist.
///
/// The table is held honest by `every_constraint_head_is_a_real_rete_row` (below), which checks each
/// `Rete` name against `RETE_OPS` — a name that drifts is a red build, not a silent no-match.
pub(crate) fn classify_constraint_head(head: &str) -> Option<(CmpKind, ConstraintSpelling)> {
    use CmpKind::{Eq, Ge, Gt, Le, Lt, NotEq};
    use ConstraintSpelling::{CoreGeneric, Rete};

    // The generic core spellings — recognized to be REFUSED with a teaching diagnostic.
    let core = match head {
        ":wat::core::=" => Some(Eq),
        ":wat::core::not=" => Some(NotEq),
        ":wat::core::<" => Some(Lt),
        ":wat::core::>" => Some(Gt),
        ":wat::core::<=" => Some(Le),
        ":wat::core::>=" => Some(Ge),
        _ => None,
    };
    if let Some(k) = core {
        return Some((k, CoreGeneric));
    }

    // The admissible per-type rete rows. Orderings exist only where the type totally orders;
    // equality exists for every comparable type.
    let (ty, op) = head.strip_prefix(":wat::rete::core::")?.rsplit_once("::")?;
    let kind = match (ty, op) {
        ("i64" | "f64", "<") => Lt,
        ("i64" | "f64", ">") => Gt,
        ("i64" | "f64", "<=") => Le,
        ("i64" | "f64", ">=") => Ge,
        ("i64" | "f64" | "string" | "bool" | "keyword" | "enum", "=") => Eq,
        ("i64" | "f64" | "string" | "bool" | "keyword" | "enum", "not=") => NotEq,
        _ => return None,
    };
    // Re-borrow `ty` as 'static by matching it back to the literal set above — the strip/rsplit
    // borrowed from `head`, whose lifetime the caller does not control.
    let ty: &'static str = match ty {
        "i64" => "i64",
        "f64" => "f64",
        "string" => "string",
        "bool" => "bool",
        "keyword" => "keyword",
        "enum" => "enum",
        _ => return None,
    };
    Some((kind, Rete { ty }))
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
            // Fact-bind: (?p <- :ns::Type clause…) — type keyword contains `::`.
            // Field-bind: (?v <- :field) — bare field keyword, exactly 3 items.
            if items.len() >= 3 {
                let is_arrow = matches!(&items[1], WatAST::Symbol(s, _) if s.as_str() == "<-");
                if is_arrow {
                    if let Some(kw) = keyword_payload(&items[2]) {
                        if kw.contains("::") {
                            return ReteClauseShape::FactBind {
                                var: var_name,
                                type_head: kw.trim_start_matches(':'),
                                clauses: &items[3..],
                            };
                        }
                        if items.len() == 3 {
                            let field = kw.strip_prefix(':').unwrap_or(kw);
                            return ReteClauseShape::Bind { var: var_name, field };
                        }
                    }
                }
                if items.len() == 3 {
                    return ReteClauseShape::Unrecognized;
                }
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
            // ── constraint: (:wat::rete::core::<ty>::<op> a b), or the core generic it replaces ──
            // Vocabulary via the ONE DOOR (`classify_constraint_head`), never a literal list here.
            k if classify_constraint_head(k).is_some() => {
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
    bindings: Vec<(Value, Value)>,
) -> Option<Vec<(Value, Value)>> {
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

        // ── constraint: (:wat::core::<op> a b) ───────────────────────────────
        // FQDN comparison ops; operands resolved from {bindings, field, literal}.
        ReteClauseShape::Constraint { op, lhs, rhs } => {
            let a = resolve_operand(lhs, fact_fields, field_names, &bindings)?;
            let b = resolve_operand(rhs, fact_fields, field_names, &bindings)?;
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
pub(crate) fn resolve_operand<B: Bindings>(
    operand: &WatAST,
    fact_fields: &[Value],
    field_names: &[String],
    bindings: &B,
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
/// Arc 278 Stone A (DESIGN-STONE-then-is-a-vector-of-singular-facts.md): `:then` is now a
/// vector of BARE fact-forms — the `(:wat::rete::insert …)` RHS marker wrapper is gone (the
/// engine is inserts-only by doctrine, so naming it per entry said nothing). `fact_form` IS
/// the fact-form directly: `(:RecordType arg…)`.
///
/// Given `fact_form` and the token `bindings`, validates the form, resolves each fact-arg via
/// `resolve_operand` (empty fact-fields/names: RHS has no current fact), and builds the
/// `Value::wat__core__Record`.
///
/// Called from `eval_insert` (after arg evaluation) and from the production pass in
/// `kernel.rs` (which already has the form + bindings and calls this directly).
///
/// Arc 278 Stone B (DESIGN-STONE-then-is-a-vector-of-singular-facts.md § "Stone B") — takes
/// `sym` now: widening (a) means `fact_items[0]` may name a plain fn instead of a fact-type
/// constructor, and only `sym.types()`/`sym.functions` can tell the two apart at fire time (the
/// freeze-time wat fence, `then-item-fence`, already proved whichever this is is legal — this is
/// the SAME registry read, once more, to pick the execution shape). See
/// [`build_insert_fact_call`] for the fn-call branch.
///
/// Raises `RuntimeError` on malformed form or unresolved operand. Never panics.
pub(crate) fn build_insert_fact(
    fact_form: &WatAST,
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-insert";
    // Arc 278 — splitting the production pass (34.9ms, 34% of a fact-heavy fire) into its parts
    // BEFORE drawing a stone against it. Coarse marks only (~52ns/pair x 4 x 40,000 derived facts):
    // read as PROPORTIONS, and read the enclosing `production` total against its un-instrumented
    // 34.963ms to see the tax. Allocation COUNTS use counters (~1-2ns) — the house method for a
    // level where a timer would tax the thing it measures.
    let __pv = crate::rete::kernel::phase_start();

    // Validate the fact form: must be a List `(:RecordType arg…)` with a keyword head.
    // Borrow (do NOT clone) — this runs once per derived fact; cloning the form AST per fact was
    // pure waste (the fan-out residual). We only read items[0]/len here.
    let fact_items = match fact_form {
        WatAST::List(items, _) if !items.is_empty() => items.as_slice(),
        _ => {
            return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "fact-form List (:RecordType arg…)",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(fact_form.clone())))),
            }).into());
        }
    };
    // Head of fact-form must be a keyword naming the record type.
    let type_keyword = match &fact_items[0] {
        WatAST::Keyword(k, _) => k.as_str(),
        other => {
            return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "keyword (record type) as fact-form head",
                got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{other:?}"))))),
            }).into());
        }
    };
    // Arc 278 Stone B, widening (a) — the item head may now be EITHER a fact-type constructor
    // (the fast path below, UNCHANGED) OR a fn whose declared return type is a fact type ("has
    // its own argument convention" — plain positional call args, not field values;
    // `BRIEF-then-user-forms.md` § "(a) THE ITEM HEAD"). `sym.types()` is the SAME registry
    // `validate_and_reorder_then`'s `lookup_fields` reads at freeze time (Rust-side); here it
    // disambiguates at FIRE time. The freeze-time wat fence (`then-item-fence`, wired into
    // `compile-rule`) already proved this item legal before this ever runs — this check only
    // picks which of the two (already-proven-safe) execution shapes to take.
    let names = match sym.types().and_then(|t| t.get(type_keyword)) {
        Some(crate::types::TypeDef::Aggregate(a)) => a.names_arc(),
        _ => return build_insert_fact_call(fact_form, type_keyword, &fact_items[1..], bindings, sym),
    };

    // class = keyword stripped of leading ':' (Arc 293.R2.1: colon-free).
    // A String allocated per derived fact for a class name fixed at compile time — NOT counted by
    // `match:key-alloc`, which arms only the two resolve_operand sites.
    crate::rete::kernel::census_count("prod:class-alloc");
    let class = type_keyword.strip_prefix(':').unwrap_or(type_keyword).to_string();
    crate::rete::kernel::phase_end("  ├ prod:validate", __pv);
    let __ps = crate::rete::kernel::phase_start();

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
    crate::rete::kernel::census_count_n("prod:vec-alloc", 2); // value_asts + fields
    let mut fields: Vec<Value> = Vec::with_capacity(value_asts.len());
    crate::rete::kernel::phase_end("  ├ prod:shape", __ps);
    let __pr = crate::rete::kernel::phase_start();
    for arg in value_asts {
        match resolve_rhs_value(arg, bindings, sym)? {
            Some(v) => fields.push(v),
            None => {
                return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "resolvable operand (?var, literal, or a fenced expression) in RHS fact-form",
                    got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{arg:?}"))))),
                }).into());
            }
        }
    }

    crate::rete::kernel::phase_end("  ├ prod:resolve", __pr);
    let __pc = crate::rete::kernel::phase_start();
    crate::rete::kernel::census_count_n("prod:record-alloc", 2); // AggregateValue + the fields Arc
    let out = Value::Aggregate(Arc::new(AggregateValue::record(class, names, Arc::new(fields))));
    crate::rete::kernel::phase_end("  ├ prod:construct", __pc);
    Ok(out)
}

/// Arc 278 Stone B, widening (a) — the FN-CALL branch of [`build_insert_fact`]: `head` does not
/// name a known aggregate type, so (by the freeze-time `then-item-fence`'s own proof) it names a
/// user fn whose declared return type is a fact type. Its "arguments" are the fn's OWN positional
/// parameters — a DIFFERENT convention from a constructor's field values, so no kwargs detection
/// applies here (`BRIEF-then-user-forms.md` § "(a) THE ITEM HEAD": *"the kwargs
/// reorder-to-declaration-order logic … applies to a constructor, not to a fn call, which has its
/// own argument convention"*).
///
/// Resolves each arg via [`resolve_rhs_value`] (widening (b) applies to a fn call's args too),
/// applies the fn, and checks the result is a fact (an `Aggregate`) — defensively: the
/// freeze-time fence already proved the fn's DECLARED return type is a fact type, so reaching a
/// non-`Aggregate` result here would mean the fence was bypassed or a checker gap let a
/// mistyped fn through, never an expected path. Never panics, never silently drops.
fn build_insert_fact_call(
    fact_form: &WatAST,
    head: &str,
    args: &[WatAST],
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-insert";
    let func = match sym.get(head) {
        Some(f) => f.clone(),
        None => {
            return Err(RuntimeError::new(fact_form.span().clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "':then' item head '{head}' names neither a known fact-type constructor nor \
                     a registered fn — the rule-compile fence should have refused this"
                ),
            }).into());
        }
    };
    let mut vals: Vec<Value> = Vec::with_capacity(args.len());
    for arg in args {
        match resolve_rhs_value(arg, bindings, sym)? {
            Some(v) => vals.push(v),
            None => {
                return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "resolvable operand (?var, literal, or a fenced expression) in a RHS fn-call arg",
                    got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{arg:?}"))))),
                }).into());
            }
        }
    }
    let result = crate::runtime::apply_function(func, vals, sym, fact_form.span().clone())
        .map_err(EvalBreak::from)?;
    match result {
        Value::Aggregate(_) => Ok(result),
        other => Err(RuntimeError::new(fact_form.span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "the fn to return a fact (a Record/Struct) — the rule-compile fence should \
                       have refused a non-fact return type",
            got: Box::new(ValueSnapshot::of(&other)),
        }).into()),
    }
}

/// Arc 278 Stone B — the RHS-only operand resolver: tries the plain [`resolve_operand`] first
/// (unchanged fast path — `?var` / `:field` / literal), and if that returns `None` AND `arg` is a
/// call form (a `List`), falls through to a FENCED evaluation (widening (b)). Lives here, NOT
/// inside `resolve_operand` itself, so `:when`'s LHS matching (which shares that fn) is untouched
/// (`BRIEF-then-user-forms.md` STOP-5: "Do NOT touch `:when`").
///
/// The freeze-time wat fence (`then-item-fence`) has already proven any `List` reaching here is
/// pure ∧ deterministic and rete-namespaced-or-composed — the SAME warrant `eval_test_core`
/// already relies on for a `where` predicate. [`eval_rhs_expr`] is that same evaluation, reused,
/// not a second implementation.
pub(crate) fn resolve_rhs_value(
    arg: &WatAST,
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<Option<Value>, EvalBreak> {
    if let Some(v) = resolve_operand(arg, &[], &[], bindings) {
        return Ok(Some(v));
    }
    match arg {
        WatAST::List(..) => Ok(Some(eval_rhs_expr(arg, bindings, sym)?)),
        _ => Ok(None),
    }
}

/// Arc 278 Stone B — evaluate a fenced `:then` expression (an operand, or — via
/// `compiled_rhs::RhsOp::Expr` — the compiled path's own third op) against one token's bindings.
/// Shared by the interpreted path ([`resolve_rhs_value`]) and the compiled path
/// (`compiled_rhs::exec_compiled_rhs`) so the two can never independently drift — the "shared
/// kernel, two surfaces" law (`DESIGN-STONE-where-admits-only-rete-ops.md` § "the implementation
/// law"). Mirrors [`build_test_env`]'s own child-`Environment`-over-`bindings` construction
/// exactly (the same one `eval_test_core` uses for a `where` predicate): a fresh base
/// `Environment` is correct here for the same reason it is there — the only names a fenced
/// `:then` expression may reference are its `?vars` and `sym`'s registered functions.
pub(crate) fn eval_rhs_expr(
    expr: &WatAST,
    bindings: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let expr_env = build_test_env(bindings, &Environment::new());
    Ok(crate::runtime::eval_inner(expr, &expr_env, sym)?.value_owned())
}

/// `(:wat::rete::eval-insert <fact-form: :wat::WatAST> <bindings: :wat::core::PersistentMap>)
/// -> :wat::core::Record`
///
/// The RHS dual of `eval_alpha_match`: where alpha-match is `(cond, fact) → Option<bindings>`,
/// eval-insert is `(fact-form, bindings) → fact`. Both sides reuse `resolve_operand`. Arc 278
/// Stone A: `fact-form` is a bare `(:Type arg…)` — the `insert` RHS-marker wrapper is gone.
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
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        }).into());
    }

    // Evaluate arg[0]: must be Value::wat__WatAST wrapping a List.
    let form_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let form_ast = match form_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST (fact form from quote)",
                got: Box::new(ValueSnapshot::of(&other)),
            }).into());
        }
    };

    // Evaluate arg[1]: must be Value::wat__core__PersistentMap (token bindings). `build_insert_fact`
    // is now typed to `PMap` directly (DESIGN-STONE-token-bindings-promoting) — no trie
    // materialisation at this boundary; the value IS the field.
    let bindings_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    let bindings: crate::value::pmap::PMap = match bindings_val {
        Value::wat__core__PersistentMap(ref m) => m.clone(),
        other => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::PersistentMap (token bindings)",
                got: Box::new(ValueSnapshot::of(&other)),
            }).into());
        }
    };

    // Delegate to the pure inner (the production pass calls this directly with the form + bindings).
    build_insert_fact(&form_ast, &bindings, sym)
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
    // `resolve_operand` and `PMap::get` below are both generic-over/native-to `PMap` — no trie
    // materialisation needed here, unlike `eval_insert`'s boundary into the trie-typed
    // `build_insert_fact`.
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
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.first().cloned(),
        _ => None,
    };
    let network = match network {
        Some(n) => n,
        None => return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::rete::Session (record with network at field 0)",
            got: Box::new(ValueSnapshot::of(&session_val)),
        }).into()),
    };

    // Reuse kernel's get_node to look up the AlphaNode by id.
    let alpha_node_val = match &network {
        Value::wat__core__PersistentMap(m) => m.get(&Value::i64(alpha_id)).cloned(),
        _ => None,
    };
    let alpha_node_val = match alpha_node_val {
        Some(n) => n,
        None => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "AlphaNode in network",
            got: Box::new(ValueSnapshot::of(&Value::i64(alpha_id))),
        }).into()),
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
                        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: ":wat::WatAST in AlphaNode.tests[0]",
                            got: Box::new(ValueSnapshot::of(other.unwrap_or(&Value::Unit))),
                        }).into()),
                    }
                }
                other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "PersistentVector at AlphaNode.tests (struct_form[1])",
                    got: Box::new(ValueSnapshot::of(other.unwrap_or(&Value::Unit))),
                }).into()),
            }
        }
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::rete::AlphaNode (record)",
            got: Box::new(ValueSnapshot::of(other)),
        }).into()),
    };

    // ── Classify the condition's clauses (REUSE the matcher's classifier) ─────
    // cond_ast = (:FactType clause…); items[0] = the head keyword (type FQDN).
    // items[1..] = the clauses — binders `(?v <- :field)` and constraints `(:op a b)`.
    let (cond_head, clauses) = match &cond_ast {
        WatAST::List(items, _) if !items.is_empty() => {
            let head = match &items[0] {
                WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
                other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "keyword head in condition form",
                    got: Box::new(ValueSnapshot::of(&Value::String(Arc::new(format!("{other:?}"))))),
                }).into()),
            };
            (head, items[1..].to_vec())
        }
        _ => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "List (condition form) in AlphaNode.tests[0]",
            got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(cond_ast)))),
        }).into()),
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
                // Only the comparison operators (not combinators/where) — via the ONE DOOR,
                // `classify_constraint_head`, never a second literal list.
                //
                // ⚠ THIS WAS THE FIFTH SITE, and it is the one the four-site census MISSED. It
                // re-listed the six core spellings by hand while its own comment claimed to
                // "REUSE the matcher's OWN classifier" — so once the corpus migrated to the
                // per-type rete names, every constraint fell to the `continue` below and each
                // DerivationStep carried ZERO constraints. EXPLAIN went quietly empty; the P12c
                // payload tests caught it. A `continue` is the same discard as `_ => {}`, and a
                // comment claiming reuse is not reuse.
                if classify_constraint_head(op_str).is_none() {
                    continue; // combinators/where: not a constraint clause
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
                constraints_pv.push_back_mut(Value::wat__WatAST(Arc::new(substituted)));
            }

            _ => continue, // non-symbol/non-keyword head: skip
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
    static STEP_CLASS_FQDN: std::sync::OnceLock<Arc<String>> = std::sync::OnceLock::new();
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
// `wat/rete.wat:233`.
::wat_source_derive::wat_field_names_from!(DERIVATION_STEP_FIELDS, "wat/rete.wat", ":wat::rete::DerivationStep");
fn derivation_step_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(DERIVATION_STEP_FIELDS)).clone()
}

// ─── Arc 278 Stone 6b-i: eval-test ────────────────────────────────────────────

/// Core evaluator for a `where` predicate — callable directly from the native kernel
/// without going through the `eval-test` dispatch wrapper.
///
/// Builds a CHILD `Environment` from `bindings` (keys are `Value::String("?x")`),
/// evaluates `expr` in it, and requires `Value::bool`. Called by:
/// - `eval_test` (the dispatch wrapper for the `(:wat::rete::eval-test …)` surface), and
/// - `fire_fixpoint_delta`'s test-pass (stone 6b-ii-b), which already holds a
///   native `Token`'s `PMap` bindings and a `WatAST` from the TestNode.
///
/// A fresh `env` (typically `&Environment::new()`) should be passed — the only names
/// a `where` expression may reference are its `?vars` (from `bindings`) and
/// `sym`'s registered user functions.
///
/// Generic over [`Bindings`] — today's callers always pass a Token's `PMap` (a `:test` clause
/// evaluates after a join in the fixtures exercised so far), but a `:test` clause may in
/// principle sit right after a single condition (element-side), so the reader stays agnostic
/// rather than assuming a representation.
/// Build the CHILD `Environment` a `where` predicate is evaluated in — one binding per `?var` the
/// token carries.
///
/// Extracted from [`eval_test_core`] (which is its only production caller) so that
/// DESIGN-STONE-compiled-where's **Step 0** can time this block ALONE against the block plus the
/// `eval_inner` walk, without duplicating it in a test where it would drift from the real path
/// (`[[feedback_feasibility_probe_must_exercise_the_exact_mechanism]]` — a probe that does not walk
/// the exact substrate path production uses proves nothing). Pure extraction: no behaviour change.
///
/// COUNTED, because this is the hot path: it runs for EVERY token × EVERY TestNode — 10,000 times
/// on node-share `[50 200]`, of which 98% are about to FAIL — and each pass allocates a child
/// `Environment` (`Arc<EnvCell>` + a `HashMap`) plus, per binding, a fresh `String` (`.to_string()`
/// on a key FIXED at rule-compile time), a `Span`, and a `Value` clone. Exactly the waste
/// `compiled_cond` was built to remove from the alpha path: *"two heap allocations rebuilding the
/// constant binding key on every call, including every call that is about to fail."*
///
/// Measured (Step 0, 2026-08-01): **122.5 ns/eval — 22.7% of a `where` evaluation.** The other
/// 77.3% is the `eval_inner` walk, which is why the stone is a full expression IR and not just
/// this block.
pub(crate) fn build_test_env<B: Bindings>(bindings: &B, env: &Environment) -> Environment {
    crate::rete::kernel::census_count("filter:test-env-builds");
    let mut b = env.child();
    for (k, v) in bindings.iter() {
        let name = match k {
            Value::String(s) => s.as_str().to_string(),
            _ => continue, // non-string key: skip (should not occur in well-formed bindings)
        };
        crate::rete::kernel::census_count("filter:test-key-alloc");
        b = b.bind_unknown_span(name, TrackedValue::from(v.clone()));
    }
    b.build()
}

pub(crate) fn eval_test_core<B: Bindings>(
    expr: &WatAST,
    bindings: &B,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<bool, EvalBreak> {
    const OP: &str = ":wat::rete::eval-test";

    // Build a CHILD Environment binding each ?var → value. The cost this carries, and why
    // DESIGN-STONE-compiled-where targets it, is documented on `build_test_env` itself — where
    // Step 0's timing arm calls the same body, so the two cannot drift.
    let test_env = build_test_env(bindings, env);

    // Evaluate the predicate expr in the test env; result MUST be bool.
    match crate::runtime::eval_inner(expr, &test_env, sym)?.value_owned() {
        Value::bool(x) => Ok(x),
        other => Err(RuntimeError::new(expr.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::bool (a where predicate must return bool)",
                got: Box::new(ValueSnapshot::of(&other)),
            })
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
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            })
        .into());
    }

    // Arg 0: evaluate → must be Value::wat__WatAST (a quoted expr from :wat::core::quote).
    let expr_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let expr_ast = match expr_val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                })
            .into());
        }
    };

    // Arg 1: evaluate → must be Value::wat__core__PersistentMap.
    let bindings_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    let map = match bindings_val {
        Value::wat__core__PersistentMap(ref m) => m.clone(),
        other => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::core::PersistentMap (the token's merged bindings)",
                    got: Box::new(ValueSnapshot::of(&other)),
                })
            .into());
        }
    };

    // Delegate to the core evaluator (a fresh env: where sees only ?vars + sym's user fns).
    Ok(Value::bool(eval_test_core(&expr_ast, &map, env, sym)?))
}

#[cfg(test)]
mod constraint_head_tests {
    use super::*;
    use crate::rete::vocabulary::RETE_OPS;

    /// ★ THE ANTI-DRIFT GATE. Every `Rete` spelling `classify_constraint_head` admits must be a
    /// real `RETE_OPS` row.
    ///
    /// The failure this exists to prevent is the arc's recurring one: a match on a literal STRING
    /// that no exhaustiveness check can see. If a vocabulary row is renamed, this table silently
    /// stops matching — the constraint is refused as `Unrecognized`, which reads to a user as
    /// "malformed clause" for a form that is perfectly well spelled. Freeze the NAMES, not a count
    /// (`[[feedback_a_gate_freezes_names_never_a_count]]`): the failure message names the offender.
    #[test]
    fn every_constraint_head_is_a_real_rete_row() {
        let known: std::collections::HashSet<&str> =
            RETE_OPS.iter().map(|op| op.rete_name).collect();

        let mut admitted = Vec::new();
        for ty in ["i64", "f64", "string", "bool", "keyword", "enum"] {
            for op in ["=", "not=", "<", ">", "<=", ">="] {
                let head = format!(":wat::rete::core::{ty}::{op}");
                if classify_constraint_head(&head).is_some() {
                    admitted.push(head);
                }
            }
        }

        // Non-vacuity FIRST: a table that admitted nothing would satisfy the check below trivially.
        assert!(
            admitted.len() >= 12,
            "classify_constraint_head admitted only {} per-type heads — the table looks empty, so \
             the membership check below would pass vacuously. Admitted: {admitted:#?}",
            admitted.len()
        );

        let phantom: Vec<&String> = admitted.iter().filter(|h| !known.contains(h.as_str())).collect();
        assert!(
            phantom.is_empty(),
            "classify_constraint_head admits {} head(s) with NO matching RETE_OPS row — a renamed \
             row would silently stop matching and the clause would be refused as `Unrecognized` \
             (which teaches the wrong fix). Offenders: {phantom:#?}",
            phantom.len()
        );
    }

    /// The generic core spellings must stay RECOGNIZED (not `None`), because the validator needs to
    /// name them and point at the per-type twin. Dropping them from the table would silently
    /// downgrade law A's teaching diagnostic to `MalformedClause` — R29's exact failure.
    #[test]
    fn the_generic_core_spellings_are_recognized_so_the_refusal_can_teach() {
        for op in [
            ":wat::core::=",
            ":wat::core::not=",
            ":wat::core::<",
            ":wat::core::>",
            ":wat::core::<=",
            ":wat::core::>=",
        ] {
            assert_eq!(
                classify_constraint_head(op).map(|(_, s)| s),
                Some(ConstraintSpelling::CoreGeneric),
                "{op} must classify as CoreGeneric — recognized here, refused by the validator"
            );
        }
    }

    /// A head that is neither is not a constraint at all — the door must not over-admit.
    #[test]
    fn unrelated_heads_are_not_constraints() {
        for op in [
            ":wat::rete::fire-rules",
            ":wat::rete::core::i64::+",
            ":wat::rete::core::vector::=",
            ":wat::core::foldl",
        ] {
            assert!(
                classify_constraint_head(op).is_none(),
                "{op} must NOT classify as a constraint head"
            );
        }
    }
}

#[cfg(test)]
mod one_core_vocabulary_tests {
    use super::*;

    /// Third row of `tests/rete/probe_arc278_49_one_core_covers_the_surfaces.rs`, living here
    /// because `CmpKind` is crate-private.
    ///
    /// `CmpKind` is ALREADY shared by the grammar, the interpreter, `compiled_cond` and the
    /// validator (#84's ONE DOOR). That is the one-core claim already true in miniature, on disk:
    /// a change to this vocabulary is a change to every surface at once, which is precisely the
    /// property `DESIGN-STONE-compiled-where.md`'s "ONE CORE, THREE ADJACENT FLIPS" is claiming.
    /// Pinned so a regression that re-forks the comparison vocabulary is caught.
    #[test]
    fn the_comparison_vocabulary_is_already_one_door() {
        let all = [CmpKind::Eq, CmpKind::NotEq, CmpKind::Lt, CmpKind::Gt, CmpKind::Le, CmpKind::Ge];
        assert_eq!(
            all.len(),
            6,
            "CmpKind is the shared comparison vocabulary across four consumers; a change in its \
             arity changes every surface at once"
        );
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                assert_eq!(
                    i == j,
                    a == b,
                    "CmpKind variants must be pairwise distinct: {a:?} vs {b:?}"
                );
            }
        }
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
