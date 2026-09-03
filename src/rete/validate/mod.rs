//! Arc 294 item 9a — the `defrule` freeze-time wall (DESIGN-rete-defrule-wall.md,
//! BRIEF-rete-defrule-wall.md).
//!
//! A `defrule`'s `:when` patterns and `:then` inserts are quoted DATA (`WatAST`) the
//! type-checker never sees as a construction — the runtime matcher (`crate::rete::matcher`)
//! classifies clauses by shape and treats any unrecognized shape / unknown field-ref as a
//! silent `None` (Clara's lenient no-match), and the RHS insert form takes kwargs
//! POSITIONALLY with no name-check or reorder. The 9a kwargs codemod corrupted a swath of
//! rule fixtures this way and NOTHING screamed — the floor just showed wrong derived counts.
//!
//! `validate_rete_rules` is the post-register freeze pass that closes the class: it walks
//! every `defrule`'s expanded `(:wat::rete::make-rule name (quote [:when…]) (quote [:then…]))`
//! call (reachable in `build_env`'s resolved user residue — see the `rete_wall_probe` in
//! `src/freeze/env.rs`), validates `:when` conditions against the type registry (unrecognized
//! clause shape / unknown field-ref → a LOCATED `#wat.rete/*` error), and validates + REORDERS
//! `:then` kwargs to declaration order (rewriting the quoted form in the residue in place, so
//! `build_insert_fact` receives declaration order at fire time).
//!
//! ## One grammar, shared (design call 1)
//!
//! Both the runtime matcher (`eval_clause`) and this validator classify rete-DSL shapes via
//! the SAME [`crate::rete::clause::classify_rete_clause`] (S1). A second hand-written
//! grammar here would be exactly the drift that bred the 9a corruption class in the first
//! place.
//!
//! ## Scope (design call 3)
//!
//! `:when` conditions get FULL validation (registered fact type, every clause a recognized
//! shape, every field-ref real) — including recursively through `not`/`exists` wrappers,
//! whose inner condition is a full alpha-matched condition just like a top-level one. A
//! `where` fence and an `accumulate`'s `:from` inner get OUTER-SHAPE + fact-type-head
//! validation ONLY — their arbitrary interior (a `where` predicate expr, an accumulate
//! reducer body, or the `:from` condition's own clauses) is explicitly OUT of this wall's
//! scope. Outer-shape only — not a half-wall on this corruption class.
//!
//! ## What this does NOT touch
//!
//! The wat oracle (`wat/rete/oracle/`) and the native kernel (`src/rete/kernel/`) are UNMOVED —
//! this is a freeze-time validator bolted on ahead of the engine, not an engine change.

// `partire`'s "self-contained one" — the operand typer. See `typing.rs`.
mod typing;
pub(crate) use typing::*;


// ⛔ `partire`'s second named cut, 2026-08-30. The error surface — every `#wat.rete/*` this wall
// can refuse with, plus its EDN faces — is `error.rs`. It has NO inbound dependency on the
// validators, while every validator names it; that asymmetry is why it came out first.
mod error;
pub(crate) use error::*;

use crate::ast::WatAST;
use crate::rete::clause::{classify_constraint_head, classify_rete_clause, ConstraintSpelling, ReteClauseShape};
use crate::span::Span;
use crate::types::{TypeDef, TypeEnv};

// ─── Error types (Pattern A: span at the outer struct, kind carries variant data) ────────────

/// `reorder_kwargs_by_field_name(field_order, kv_pairs, span) -> Vec<value_ast>` in declaration
/// order — ONE helper, single-sourced. The (C) spliced-construction reorder pass calls this
/// too (a separate strike; not wired here) — do NOT inline this at either call site.
///
/// The HELPER itself still does not require `kv_pairs` to cover every field in `field_order`
/// — any field with no matching pair is simply absent from the output; every SUPPLIED field
/// name, however, must be real: the first unknown name is returned as `Err(KwargsReorderError)` so the
/// caller can build its own contextual error. Full coverage is each CALLER's job, and both
/// callers now enforce it: `eval_kwargs_construct` (runtime.rs) relies on `construct_aggregate`'s
/// arity check plus check.rs's `infer_kwargs_construct_check` (the macro-expanded form); the
/// surface-form caller below, `validate_rule_when_and_reorder_then`, used to be the one caller that did
/// NOT — arc 278 BRIEF-construction-total-three-walls.md #2 closed that: STOP-A audited the
/// whole corpus for a `:then` that under-supplies (none found — the old "pre-existing,
/// unchanged" note here described an accident nobody depended on) and `validate_then_form`
/// now rejects an under-supplied kwargs RHS before ever reaching this reorder (`RhsMissingFields`).
pub(crate) fn reorder_kwargs_by_field_name(
    field_order: &[&str],
    kv_pairs: &[(&str, WatAST)],
    span: &Span,
) -> Result<Vec<WatAST>, KwargsReorderError> {
    for (field, _) in kv_pairs {
        if !field_order.contains(field) {
            return Err(KwargsReorderError {
                span: span.clone(),
                field: (*field).to_string(),
            });
        }
    }
    let mut out = Vec::with_capacity(kv_pairs.len());
    for f in field_order {
        if let Some((_, v)) = kv_pairs.iter().find(|(k, _)| k == f) {
            out.push(v.clone());
        }
    }
    Ok(out)
}

// ─── The validator (S2 + S3) ──────────────────────────────────────────────────────────────────

/// Post-register freeze pass: walk every live `make-rule` / `make-query` reachable in
/// `residue`, validate `:when`/`:then` (rules) and `:when` (queries) against `types`,
/// and rewrite `:then` kwargs to declaration order in place. Quoted / forms / literal
/// payloads are data (`Boundary::AllData`) and are not validated against this world's
/// type registry. Returns every finding batched (like `check_program`); an empty batch
/// is `Ok(())`.
///
/// Hook site: `src/freeze/env.rs::build_env`, immediately after `resolve_references` (step
/// 7) — the same seam the `rete_wall_probe` proves reachable, on the SAME resolved user
/// residue + fully-registered `types`.
pub(crate) fn validate_rete_rules(residue: &mut [WatAST], types: &TypeEnv) -> Result<(), ReteCheckErrors> {
    let mut errors: Vec<ReteCheckError> = Vec::new();
    walk_for_make_rule(residue, types, &mut errors);
    walk_for_make_query(residue, types, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ReteCheckErrors(errors))
    }
}

// ─── Registration (the pluggable extension point) ────────────────────────────────────────────
//
// The wall used to be hardcoded into `build_env` step 7.8 (`src/freeze/env.rs`); it is now the
// FIRST registered `FreezeValidator` (`crate::freeze::validator`), drained generically
// alongside any other crate's registration — mirrors `RestrictionEntry`'s
// `inventory::submit!` shape (`src/restriction_entry.rs`). `validate_rete_rules`'s signature
// and logic are UNCHANGED; only the caller (the drain in `build_env`) moved.
inventory::submit! {
    crate::freeze::validator::FreezeValidator {
        name: "wat.rete/defrule-wall",
        validate: |residue, types, _symbols| {
            validate_rete_rules(residue, types)
                .map_err(|e| Box::new(e) as Box<dyn crate::freeze::validator::FreezeValidatorError>)
        },
    }
}

/// True when this list is captured as data — `quote` / `forms` / `holon::literal`
/// (`Boundary::AllData`) or a quasiquote template. A `make-query` / `make-rule`
/// sitting inside is a payload for another world, not a live form in *this* freeze.
fn rete_walk_skips_children(items: &[WatAST]) -> bool {
    match items.first() {
        Some(WatAST::Keyword(k, _)) => matches!(
            crate::resolve::boundary::quote_boundary(k),
            crate::resolve::boundary::Boundary::AllData
                | crate::resolve::boundary::Boundary::Quasiquote
        ),
        _ => false,
    }
}

/// Recursive descent for `(:wat::rete::make-rule name (quote [:when…]) (quote [:then…]))`
/// calls — mirrors `find_make_rule` in the `rete_wall_probe` (`src/freeze/env.rs`), but
/// mutable (S3 rewrites `:then` in place) and exhaustive (every *live* rule in `forms`,
/// not just the first). Does not descend into AllData / quasiquote payloads.
fn walk_for_make_rule(forms: &mut [WatAST], types: &TypeEnv, errors: &mut Vec<ReteCheckError>) {
    for f in forms.iter_mut() {
        if let WatAST::List(items, _) = f {
            let is_make_rule =
                matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::rete::make-rule");
            if is_make_rule {
                validate_rule_when_and_reorder_then(items, types, errors);
                continue;
            }
            if rete_walk_skips_children(items) {
                continue;
            }
            walk_for_make_rule(items, types, errors);
        }
    }
}

/// `(:wat::rete::make-query name (quote [:params…]) (quote [:when…]))` — same `:when`
/// grammar as `make-rule`, no `:then` reorder. Live `defquery` / `make-query` is
/// validated; a quoted payload (the scratch-pad "ship the forms" probes) is not.
fn walk_for_make_query(forms: &[WatAST], types: &TypeEnv, errors: &mut Vec<ReteCheckError>) {
    for f in forms {
        if let WatAST::List(items, _) = f {
            let is_make_query =
                matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::rete::make-query");
            if is_make_query {
                validate_query_when(items, types, errors);
                continue;
            }
            if rete_walk_skips_children(items) {
                continue;
            }
            walk_for_make_query(items, types, errors);
        }
    }
}

fn validate_query_when(mq: &[WatAST], types: &TypeEnv, errors: &mut Vec<ReteCheckError>) {
    let qname = match mq.get(1) {
        Some(WatAST::StringLit(s, _)) => s.clone(),
        other => other.map(render_form).unwrap_or_else(|| "<unknown-query>".to_string()),
    };
    if let Some(when_conds) = quote_vector(mq.get(3)) {
        let binds = collect_rule_bind_types(when_conds, types);
        for cond in when_conds {
            validate_when_entry(cond, &qname, types, &binds, errors);
        }
        // A query has no `:then`, so a bind can only escape into a `:where`.
        validate_wrapper_binds(when_conds, &[], &qname, errors);
    }
}

/// `mr` = the full `make-rule` call's items: `[kw, name-lit, when-quote, then-quote]`.
fn validate_rule_when_and_reorder_then(
    mr: &mut [WatAST],
    types: &TypeEnv,
    errors: &mut Vec<ReteCheckError>,
) {
    let rule_name = match mr.get(1) {
        Some(WatAST::StringLit(s, _)) => s.clone(),
        other => other.map(render_form).unwrap_or_else(|| "<unknown-rule>".to_string()),
    };

    // ★ D10 — the bind map is HOISTED out of the `:when` block because the `:then` needs it too.
    // A `:then` operand's type is knowable exactly when the `?var` it names is bound by this
    // rule's `:when`, and that is the SAME rule-wide map the constraint typer already builds; a
    // second, `:then`-local collection would be a second place for a join variable to go missing.
    // Empty when `mr[2]` is not a `(quote [...])` — then every `?var` is `UnboundInThisRule`,
    // which is the honest answer, not a skipped check dressed as one.
    let mut binds: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    // :when (mr[2] = (quote [<cond>…])) — validate only, no rewrite.
    if let Some(when_conds) = quote_vector(mr.get(2)) {
        // ★ Binds collected across EVERY condition of the rule, before any is validated. A join
        // variable is bound in one pattern and compared in another, so a per-pattern map would
        // leave it unresolvable — and "unresolvable" was quietly meaning "skip the check". It is
        // knowable; it just is not knowable from one pattern.
        binds = collect_rule_bind_types(when_conds, types);
        for cond in when_conds {
            validate_when_entry(cond, &rule_name, types, &binds, errors);
        }
        // `:then` variable uses are read HERE, immutably, before the kwargs reorder takes `mr`
        // mutably below — a bind escaping into the RHS must count as an escape.
        let mut then_occ: Vec<String> = Vec::new();
        if let Some(then_forms) = quote_vector(mr.get(3)) {
            for t in then_forms {
                collect_var_occurrences(t, &mut then_occ);
            }
        }
        validate_wrapper_binds(when_conds, &then_occ, &rule_name, errors);
    }

    // :then (mr[3] = (quote [<fact-form>…])) — validate, then reorder kwargs. Arc 278 Stone A:
    // each member is a bare fact-form, no more `insert` wrapper.
    if let Some(WatAST::List(quote_items, _)) = mr.get_mut(3) {
        if let Some(WatAST::Vector(then_forms, _)) = quote_items.get_mut(1) {
            for fact_form in then_forms.iter_mut() {
                validate_then_form(fact_form, &rule_name, types, &binds, errors);
            }
        }
    }
}

/// `form` = `(:wat::core::quote <Vector>)` — return the Vector's items, or `None` if the
/// shape doesn't hold (defensive; the probe proves this shape survives resolve intact).
fn quote_vector(form: Option<&WatAST>) -> Option<&[WatAST]> {
    if let Some(WatAST::List(items, _)) = form {
        if let Some(WatAST::Vector(v, _)) = items.get(1) {
            return Some(v.as_slice());
        }
    }
    None
}

// ─── :when validation ─────────────────────────────────────────────────────────────────────────

/// Dispatch one top-level `:when`-entry — mirrors `compile-condition`'s own dispatch
/// (`wat/rete/compile.wat`: is-where / is-not / is-exists / is-accumulate / else-plain), via the
/// SHARED classifier so this never drifts into a second hand-rolled grammar.
fn validate_when_entry(
    cond: &WatAST,
    rule_name: &str,
    types: &TypeEnv,
    binds: &std::collections::HashMap<String, String>,
    errors: &mut Vec<ReteCheckError>,
) {
    match classify_rete_clause(cond) {
        // Design call 3 — a `where` fence's outer shape is already confirmed by the
        // classifier (2-item, `:wat::rete::where` head); its interior expr is out of scope.
        ReteClauseShape::Where(_) => {}
        // Design call 3 / brief S2 — `not`/`exists` recurse: their sub-condition gets the
        // SAME full validation (registered type + every clause + every field-ref) as any
        // top-level condition.
        ReteClauseShape::Not(inner) | ReteClauseShape::Exists(inner) => {
            // Inner is a full :when entry (fact, or `:and` of facts — Clara
            // `[:not [:and [Wind] [Temp]]]`), not only a plain pattern.
            validate_when_entry(inner, rule_name, types, binds, errors);
        }
        // Design call 3 — accumulate's `:from` inner gets fact-type-HEAD validation only;
        // its own clauses and the acc-form's reducer body are out of scope.
        ReteClauseShape::Accumulate { from, .. } => {
            validate_fact_type_head_only(from, rule_name, types, errors);
        }
        ReteClauseShape::FactBind { type_head, clauses, .. } => {
            validate_typed_clauses(cond, type_head, clauses, rule_name, types, binds, errors);
        }
        // Condition `:or` — or of activations (Clara `[:or [Temp] [Wind]]`). Each arm
        // is a full :when entry, not a fact named `or`. Empty `:or` is malformed.
        ReteClauseShape::Or(arms) => {
            if arms.is_empty() {
                errors.push(malformed(cond.span().clone(), rule_name, "", cond));
            }
            for arm in arms {
                validate_when_entry(arm, rule_name, types, binds, errors);
            }
        }
        // Condition `:and` — grouping (Clara `[:or [Temp] [:and [Temp] [Wind]]]`).
        // Sequential :when entries, not a fact named `and`. Empty `:and` is malformed.
        ReteClauseShape::And(arms) => {
            if arms.is_empty() {
                errors.push(malformed(cond.span().clone(), rule_name, "", cond));
            }
            for arm in arms {
                validate_when_entry(arm, rule_name, types, binds, errors);
            }
        }
        // Bind/Constraint/Unrecognized are not top-level :when entries — a
        // top-level entry that is not a wrapper must be `(:Type clause…)`.
        // `And` is matched above as a Clara grouping wrapper.
        _ => validate_plain_condition(cond, rule_name, types, binds, errors),
    }
}

/// Validate a plain `(:Type clause…)` condition: `Type` must be a registered aggregate, and
/// every clause must be a recognized shape whose field-refs name real fields.
fn validate_plain_condition(
    cond: &WatAST,
    rule_name: &str,
    types: &TypeEnv,
    binds: &std::collections::HashMap<String, String>,
    errors: &mut Vec<ReteCheckError>,
) {
    match crate::rete::matcher::alpha_pattern(cond) {
        Some(p) => validate_typed_clauses(
            cond, p.type_head, p.clauses, rule_name, types, binds, errors,
        ),
        None => errors.push(malformed(cond.span().clone(), rule_name, "", cond)),
    }
}

/// Check every clause of one condition against the fact type's DECLARED fields.
///
/// Resolving the type's field names and types happens once, here, and is then threaded into
/// each clause — a clause-level lookup would re-resolve the same type per clause and, worse,
/// could disagree with itself if the environment changed mid-walk. A type with no readable
/// fields is an error recorded and carried on from, not a bail: the point of a validator is to
/// report every fault in one pass, so the user fixes them together rather than one per run.
fn validate_typed_clauses(
    cond: &WatAST,
    fact_type: &str,
    clauses: &[WatAST],
    rule_name: &str,
    types: &TypeEnv,
    binds: &std::collections::HashMap<String, String>,
    errors: &mut Vec<ReteCheckError>,
) {
    let fact_type = fact_type.to_string();
    let field_types = lookup_field_types(types, &fact_type).unwrap_or_default();
    let field_names = match lookup_fields(types, &fact_type) {
        Some(f) => f,
        None => {
            errors.push(ReteCheckError {
                span: cond.span().clone(),
                kind: ReteCheckErrorKind::UnknownFactType { rule: rule_name.to_string(), fact_type },
            });
            return;
        }
    };
    for clause in clauses {
        validate_clause(
            clause,
            &ClauseCtx {
                rule_name,
                fact_type: &fact_type,
                field_names: &field_names,
                field_types: &field_types,
                binds,
                types,
            },
            errors,
        );
    }
}

/// Validate a single within-condition clause (recursing `and`/`or`/`not`), checking every
/// bind/constraint field-ref against `field_names`. The free `?var` side is never checked.
/// Everything a clause check needs about the CONDITION it sits in — invariant across the clause
/// walk, so it travels as one value rather than seven positional arguments. A struct over an alias
/// for the reason `AlphaTreeFixture` is one (`kernel/`): the call sites read by NAME, and the
/// alternative here was an `#[allow(clippy::too_many_arguments)]`, which silences the signal
/// instead of answering it.
pub(crate) struct ClauseCtx<'a> {
    rule_name: &'a str,
    fact_type: &'a str,
    field_names: &'a [String],
    field_types: &'a [String],
    /// `?var` -> declared field type, collected across the WHOLE rule (join vars included).
    binds: &'a std::collections::HashMap<String, String>,
    types: &'a TypeEnv,
}

/// Check one clause, dispatching on its `ReteClauseShape`.
///
/// The destructure takes only what THIS function reads — the rest travel onward inside `ctx`.
/// That is deliberate and the comment below records why: destructuring all of them and then not
/// using them produced three `unused_variables` warnings, and the fix is to TAKE LESS rather than
/// to `_`-prefix, which would silence the very gate that caught it (task #67).
fn validate_clause(
    clause: &WatAST,
    ctx: &ClauseCtx<'_>,
    errors: &mut Vec<ReteCheckError>,
) {
    // Only what THIS function reads. The other three travel onward inside `ctx` to
    // `check_constraint_head` — destructuring them here just to not use them is what produced
    // three `unused_variables` warnings, and the fix is to take less, not to `_`-prefix them
    // (that door is task #67: `_` silences the very gate that would have caught the mistake).
    let ClauseCtx { rule_name, fact_type, field_names, .. } = *ctx;
    match classify_rete_clause(clause) {
        ReteClauseShape::Bind { field_kw, .. } => {
            // The `:field` KEYWORD, not `clause`: this used to hand the whole `(?v <- :field)`
            // form's span to a producer whose doc promised the field's.
            check_field_kw(field_kw, rule_name, fact_type, field_names, errors);
        }
        // A boolean rete expression. Its operand TYPES are the expression's own business — there
        // is no per-type comparator to validate the way a `Constraint` has — and its field refs
        // are resolved by the lowering, which refuses (never silently fails) on an unknown one.
        //
        // ⚠ That refusal currently arrives as the generic "alpha N cond did not compile", which
        // teaches less than the located `UnknownField` a `Constraint` gets. Carrying a reason up
        // from `compile_seq` is tracked as its own strike; it is not made worse here.
        ReteClauseShape::Predicate(_) => {}
        ReteClauseShape::Constraint { op, lhs, rhs } => {
            // The comparator's own type, so a keyword operand that is not a field can be judged as
            // the CONSTANT it is. `None` for the core-generic spelling, which is refused by
            // `check_constraint_head` on its own grounds anyway.
            let op_type = match classify_constraint_head(op) {
                Some((_, ConstraintSpelling::Rete { ty })) => Some(ty),
                _ => None,
            };
            check_operand_field_ref(lhs, clause, ctx, op_type, errors);
            check_operand_field_ref(rhs, clause, ctx, op_type, errors);
            check_constraint_head(op, lhs, rhs, clause, ctx, errors);
        }
        ReteClauseShape::And(subs) | ReteClauseShape::Or(subs) => {
            for sub in subs {
                validate_clause(sub, ctx, errors);
            }
        }
        ReteClauseShape::Not(sub) => {
            validate_clause(sub, ctx, errors);
        }
        // Clause-level `where` is the stone-6 STOP arm (always `None` at fire time); its
        // interior is out of scope (design call 3) — nothing further to check.
        ReteClauseShape::Where(_) => {}
        // `exists`/`accumulate` never legitimately occur as within-condition clauses (they
        // are top-level-only wrappers, consumed before a condition's clause list is built).
        ReteClauseShape::Exists(_)
        | ReteClauseShape::Accumulate { .. }
        | ReteClauseShape::FactBind { .. }
        | ReteClauseShape::Unrecognized => {
            errors.push(malformed(clause.span().clone(), rule_name, fact_type, clause));
        }
    }
}


// ─── The `:not` bind wall ────────────────────────────────────────────────────────────────────
//
// A bind under a `:not` must be consumed under that `:not`. The rete twin of arc 109's
// `DESIGN-STONE-a-param-spec-must-be-consumed`, and refused for the same stated reason: an unused
// declaration changes no answer, but a reader cannot tell a deliberate one from a leftover edit.
//
// ★ HOW THIS AVOIDS THE TRAP `RhsUnresolvableOperand` NAMED. That variant deliberately declines
// binder analysis, because under-collecting the bound set would reject LEGAL rules — the one
// failure a wall must not have. This check never asks whether a variable IS bound. It asks a
// purely syntactic question: are ALL of this variable's declarations inside one negation? If it
// is declared anywhere else it is a correlation (`(Station (?loc <- :loc))` then
// `(:not (Reading (?loc <- :loc)))`, which is "no Reading AT THIS loc") and is left alone. Only a
// variable whose sole declaration site is under the negation can be judged, and for that one the
// answer needs no binder analysis at all.
//
// ★ WHY `:exists` IS NOT HERE, THOUGH CLARA TRAPS BOTH. This wall covered `:exists` for exactly
// one build, and it was WRONG: in wat `:exists` binds OUTWARD, and `leading-exists.wat` — a live
// accuracy axis — reads its `:exists`-bound `?loc` back out of the query rows by string key to
// build `:derived`. A consumer in HOST CODE is invisible to any syntactic check, so no such check
// may judge an outward-binding construct. The sweep over every grid axis is what caught it, one
// step before it shipped; the prior warning it would have violated is quoted above. `:not` is
// safe to judge because it binds nothing outward BY CONSTRUCTION — it admits a token precisely
// when no fact matched, so there is no value to carry, and the runtime says so out loud
// (`unbound symbol` at fire).

/// One bind declared INSIDE a `:not`, attributed to its INNERMOST enclosing negation — so `not`
/// of `not` judges the declaration once, at the negation that actually traps it.
struct WrapperBind<'a> {
    var: &'a str,
    fact_type: &'a str,
    span: &'a Span,
    /// The negation's inner form: the scope the variable must be consumed within.
    scope: &'a WatAST,
}

/// Every `?var` OCCURRENCE in a form, at any depth — a raw symbol walk, deliberately NOT
/// classifier-driven.
///
/// Consumption can happen anywhere a symbol can appear, including inside a `:where` predicate
/// whose interior this validator otherwise holds out of scope (design call 3). Out of scope for
/// JUDGING is not out of scope for COUNTING: a `:where` is exactly where an escaped bind gets
/// referenced, and missing it would turn an escape into a false "unconsumed".
///
/// The match is exhaustive with no wildcard, so a new `WatAST` container is a compile error here
/// rather than a silent blind spot — under-counting occurrences is precisely how this wall would
/// come to reject a legal rule.
fn collect_var_occurrences(form: &WatAST, out: &mut Vec<String>) {
    match form {
        WatAST::Symbol(sym, _) => {
            let name = sym.as_str();
            if name.starts_with('?') {
                out.push(name.to_string());
            }
        }
        WatAST::List(xs, _) | WatAST::Vector(xs, _) | WatAST::Set(xs, _) => {
            for x in xs {
                collect_var_occurrences(x, out);
            }
        }
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                collect_var_occurrences(k, out);
                collect_var_occurrences(v, out);
            }
        }
        // Scalars carry no variable. Enumerated rather than wildcarded — see the doc comment.
        WatAST::IntLit(..)
        | WatAST::FloatLit(..)
        | WatAST::RationalLit(..)
        | WatAST::BigIntLit(..)
        | WatAST::BoolLit(..)
        | WatAST::StringLit(..)
        | WatAST::NilLit(..)
        | WatAST::Keyword(..) => {}
    }
}

/// Every variable DECLARED by a bind, fact-bind or accumulate result, at any depth and under any
/// wrapper. Used only to answer "is this variable declared anywhere ELSE?", which is what
/// separates a correlation from a trapped declaration.
fn collect_all_declarations<'a>(cond: &'a WatAST, out: &mut Vec<&'a str>) {
    match classify_rete_clause(cond) {
        ReteClauseShape::Not(inner) | ReteClauseShape::Exists(inner) => {
            collect_all_declarations(inner, out);
        }
        // A predicate is an EXPRESSION: it reads bindings, it never declares one.
        ReteClauseShape::Predicate(_) => {}
        ReteClauseShape::And(arms) | ReteClauseShape::Or(arms) => {
            for a in arms {
                collect_all_declarations(a, out);
            }
        }
        ReteClauseShape::Accumulate { var, from, .. } => {
            out.push(var);
            collect_all_declarations(from, out);
        }
        ReteClauseShape::Bind { var, .. } => out.push(var),
        ReteClauseShape::FactBind { var, clauses, .. } => {
            out.push(var);
            for c in clauses {
                collect_all_declarations(c, out);
            }
        }
        // A `where` fence declares nothing, and a constraint only USES. Named rather than left to
        // the fallback so neither is walked as if it were a pattern.
        ReteClauseShape::Where(_) | ReteClauseShape::Constraint { .. } => {}
        ReteClauseShape::Unrecognized => {
            if let Some(pat) = crate::rete::matcher::alpha_pattern(cond) {
                if let Some(v) = pat.fact_var {
                    out.push(v);
                }
                for c in pat.clauses {
                    collect_all_declarations(c, out);
                }
            }
        }
    }
}

/// Record binds that sit under a `:not`. `wrapper` is the innermost enclosing negation's inner
/// form (`None` outside any negation); `type_head` is the nearest enclosing pattern's fact type,
/// so the error can say which type's field the dead bind reads.
fn collect_wrapper_binds<'a>(
    cond: &'a WatAST,
    wrapper: Option<&'a WatAST>,
    type_head: &'a str,
    out: &mut Vec<WrapperBind<'a>>,
) {
    match classify_rete_clause(cond) {
        // An expression binds nothing — it only READS what earlier clauses bound.
        ReteClauseShape::Predicate(_) => {}
        ReteClauseShape::Not(inner) => {
            collect_wrapper_binds(inner, Some(inner), type_head, out);
        }
        // `:exists` does NOT open a scope — it binds outward (see the banner). It passes the
        // ENCLOSING wrapper through, so a bind under `not(exists(…))` is still trapped by the
        // `:not` and still judged.
        ReteClauseShape::Exists(inner) => {
            collect_wrapper_binds(inner, wrapper, type_head, out);
        }
        ReteClauseShape::And(arms) | ReteClauseShape::Or(arms) => {
            for a in arms {
                collect_wrapper_binds(a, wrapper, type_head, out);
            }
        }
        // An accumulate's RESULT var binds OUTWARD and is not this wall's business; its `:from`
        // inner is an ordinary pattern and keeps whatever wrapper it sits under.
        ReteClauseShape::Accumulate { from, .. } => {
            collect_wrapper_binds(from, wrapper, type_head, out);
        }
        ReteClauseShape::Bind { var, .. } => {
            if let Some(scope) = wrapper {
                out.push(WrapperBind { var, fact_type: type_head, span: cond.span(), scope });
            }
        }
        ReteClauseShape::FactBind { var, type_head: th, clauses } => {
            if let Some(scope) = wrapper {
                out.push(WrapperBind { var, fact_type: th, span: cond.span(), scope });
            }
            for c in clauses {
                collect_wrapper_binds(c, wrapper, th, out);
            }
        }
        ReteClauseShape::Where(_) | ReteClauseShape::Constraint { .. } => {}
        ReteClauseShape::Unrecognized => {
            if let Some(pat) = crate::rete::matcher::alpha_pattern(cond) {
                for c in pat.clauses {
                    collect_wrapper_binds(c, wrapper, pat.type_head, out);
                }
            }
        }
    }
}

/// The wall. `extra_occurrences` carries the rule's `:then` variable uses (a query has none), so
/// a bind escaping into the RHS is caught the same as one escaping into a `:where`.
fn validate_wrapper_binds(
    when_conds: &[WatAST],
    extra_occurrences: &[String],
    rule_name: &str,
    errors: &mut Vec<ReteCheckError>,
) {
    let mut binds: Vec<WrapperBind<'_>> = Vec::new();
    for cond in when_conds {
        collect_wrapper_binds(cond, None, "", &mut binds);
    }
    if binds.is_empty() {
        return;
    }

    let count = |hay: &[String], needle: &str| hay.iter().filter(|v| v.as_str() == needle).count();

    let mut whole_occ: Vec<String> = extra_occurrences.to_vec();
    let mut all_decls: Vec<&str> = Vec::new();
    for cond in when_conds {
        collect_var_occurrences(cond, &mut whole_occ);
        collect_all_declarations(cond, &mut all_decls);
    }

    for b in &binds {
        let declared_everywhere = all_decls.iter().filter(|v| **v == b.var).count();
        let mut scope_decls: Vec<&str> = Vec::new();
        collect_all_declarations(b.scope, &mut scope_decls);
        let declared_in_scope = scope_decls.iter().filter(|v| **v == b.var).count();
        // Declared outside this wrapper too ⇒ a correlation, not a trapped declaration. Untouched,
        // and this is the clause that makes the wall unable to reject a legal rule.
        if declared_everywhere > declared_in_scope {
            continue;
        }

        let mut scope_occ: Vec<String> = Vec::new();
        collect_var_occurrences(b.scope, &mut scope_occ);
        let inside = count(&scope_occ, b.var);
        let outside = count(&whole_occ, b.var).saturating_sub(inside);

        let kind = if outside > 0 {
            ReteCheckErrorKind::EscapedWrapperBind {
                rule: rule_name.to_string(),
                var: b.var.to_string(),
                fact_type: b.fact_type.to_string(),
            }
        } else if inside <= 1 {
            ReteCheckErrorKind::UnconsumedWrapperBind {
                rule: rule_name.to_string(),
                var: b.var.to_string(),
                fact_type: b.fact_type.to_string(),
            }
        } else {
            continue;
        };
        errors.push(ReteCheckError { span: b.span.clone(), kind });
    }
}

// ─── :then validation + reorder (S3) ─────────────────────────────────────────────────────────

/// A `:then` value-position operand that can NEVER resolve at fire
/// time — whatever the bindings.
///
/// Mirrors `resolve_operand`'s accepted set (`matcher.rs`) exactly, minus the `?var` case whose
/// boundness this stone does not judge: a literal resolves, a `?var` MAY resolve, and everything
/// else — a `:field` keyword (a RHS has no current fact), a bare non-`?` symbol, a Vector/Map/Set
/// literal — resolves to `None` for every possible token. Purely syntactic, so it cannot reject a
/// legal rule.
///
/// Arc 278 Stone B, widening (b) — a `List` (a call form) is REMOVED from the never-resolves set.
/// It is no longer necessarily dead: it may be a fenced expression composed of
/// `:wat::rete::`-namespaced ops (or a user fn bottoming out in them), which the wat-side fence
/// (`then-item-fence`) proves legal — or refuses, naming the offending head + axis — at
/// rule-compile time. This function does not have `sym`, so it cannot judge a List itself; it
/// only stops flagging a shape that is no longer categorically illegal.
fn rhs_operand_can_never_resolve(arg: &WatAST) -> bool {
    !matches!(
        arg,
        WatAST::IntLit(_, _)
            | WatAST::FloatLit(_, _)
            | WatAST::BoolLit(_, _)
            | WatAST::StringLit(_, _)
            | WatAST::List(_, _)
    ) && !matches!(arg, WatAST::Symbol(ident, _) if ident.as_str().starts_with('?'))
}

/// Flag every value-position operand of a `:then` insert that can never resolve.
fn check_rhs_operands(
    value_args: &[WatAST],
    rule_name: &str,
    fact_type: &str,
    errors: &mut Vec<ReteCheckError>,
) {
    for arg in value_args {
        if rhs_operand_can_never_resolve(arg) {
            errors.push(ReteCheckError {
                span: arg.span().clone(),
                kind: ReteCheckErrorKind::RhsUnresolvableOperand {
                    rule: rule_name.to_string(),
                    fact_type: fact_type.to_string(),
                    operand: render_form(arg),
                    accepted: vec![
                        "a ?var bound by this rule's :when".to_string(),
                        "an integer / float / boolean / string literal".to_string(),
                    ],
                },
            });
        }
    }
}

/// Arc 278 BRIEF-construction-total-three-walls.md #1/#3 — walk a `:then` item's value-position
/// operand recursively for a NESTED constructor call, and validate it. Mirrors the runtime's own
/// recursive evaluation: `dispatch_keyword_head_value`/`eval_kwargs_construct` reach a nested
/// aggregate or enum-variant constructor exactly the way a top-level `:then` item is reached,
/// just one (or more) `eval_inner` deeper (`eval_list` walks every argument of every call form,
/// unconditionally — the SAME shape `classify_expr`'s general-list arm uses for `pure`,
/// `purity.rs:862-884`). So this walk is unbounded-depth too, not a single peek one level down.
///
/// Two constructor heads are recognized (both invisible to `lookup_fields`, which resolves only
/// `TypeDef::Aggregate` by NAME, not by asking "is this keyword any kind of constructor head"):
///   - an aggregate-type keyword in the CONSTRUCTOR-TYPE SLOT — which after `defrecord`'s
///     pre-freeze lowering is index 1 of `(:wat::core::kwargs-construct :usr::Inner …)`, not the
///     head; see the ★ note in the body, and do not re-key this off `items[0]` — validated with
///     the SAME kwargs-coverage /
///     positional-shape rules `validate_then_form` gives a `:then` item's own top-level
///     shape (#2's fix, generalized: unknown field, missing field, or — the shape #2's top-level
///     branch never has to consider, since `build_insert_fact`'s top-level fast path supports
///     legacy positional unconditionally — a RETIRED multi-arg positional call, since a NESTED
///     constructor reaches `eval_kwargs_construct`'s dispatch, not `build_insert_fact`'s).
///   - a bare `:Enum::Variant` keyword — arity-checked against the variant's declared field count
///     (#3: no freeze wall resolved this at all before now).
///
/// No AST rewrite here (unlike the top-level kwargs branch): a nested operand's kwargs are
/// reordered again at FIRE time by `eval_kwargs_construct` regardless of what freeze validated
/// (arc 278 #1), so this pass only has to prove the shape is constructible, never to reorder it.
///
/// ## ★ D11 — and it TYPES the values now, not only the shape
///
/// Everything above is STRUCTURAL: a field name, an arity, a missing field, a retired spelling.
/// None of it typed a single value, because the walker had no `binds` and `resolve_operand_type`
/// cannot answer for a `?var` without one. D10 closed exactly this hole at the top level and the
/// nested one survived it by one commit: at `f87bb070b`, `:then [(:nh::Outer :i (:nh::Inner :n
/// ?s))]` with `?s : String` into an `i64` field compiled, fired, and put
/// `#nh/Outer {:i #nh/Inner {:n "nested-string"}}` into the FACT SET — where joins, queries, the
/// oracle and `explain` all trust the declared schema.
///
/// The aggregate branch now pairs each nested field with its DECLARED type
/// (`lookup_field_types`, the sibling of the `lookup_fields` it already called) and hands the pair
/// to D10's own producer, `check_then_field_type` — unchanged, and reusing `RhsFieldTypeMismatch`
/// unchanged, because a nested occurrence is the same claim at a different position. The
/// invariant is therefore **at ANY depth**, which is what the recursion above was always for:
/// `tests/rete/probe_arc278_D11_nested_then_field_types.rs` drives it at depth 2 and inside a
/// `match` arm BODY, alongside the five constructed not-knowable operands that say the wall still
/// stands down where the type is merely unknown.
///
/// ⛔ The ENUM-VARIANT branch below is NOT typed, deliberately. `enum_variant_ctor` answers with
/// an arity and nothing else; getting a variant's per-field declared types is a different registry
/// read and its own ruling. That branch keeps the arity diagnostic it had.
fn walk_nested_constructors(
    operand: &WatAST,
    rule_name: &str,
    types: &TypeEnv,
    // D11 — the rule's `?var` -> declared-type map, threaded so this walker can TYPE a nested
    // field value and not merely count and name it. `resolve_operand_type` needs it for source 2
    // (`?var`); without it the walker could only ever check names, arity and missing fields, and
    // `(:nh::Outer :i (:nh::Inner :n ?s))` with `?s : String` into an `i64` field compiled, fired
    // and put `#nh/Outer {:i #nh/Inner {:n "nested-string"}}` in the FACT SET (driven at
    // `f87bb070b`, one commit after D10 closed the identical hole at the top level).
    binds: &std::collections::HashMap<String, String>,
    errors: &mut Vec<ReteCheckError>,
) {
    let WatAST::List(items, span) = operand else { return };
    if items.is_empty() {
        return;
    }
    // ── `match` — A PATTERN IS NOT A CALL (arc 278 strike-match-arm-is-not-a-call, D5).
    //
    // A match form is `(HEAD scrutinee arm…)` and an arm is `(pattern body…)`. Without this arm the
    // generic fallthrough at the bottom recursed into the ARM ITSELF, whose `items[0]` is the
    // PATTERN — for a bare enum-variant pattern that is a keyword `enum_variant_ctor` resolves, so
    // the arity branch below fired the variant's 0 declared fields against the arm's length 1 and
    // refused a legal program with `RhsArityMismatch` naming a `:then` INSERT of `:probe::E::A`
    // that appears nowhere in the source. It survived only by coincidence of spelling:
    // `((:E::A) true)` hides the keyword one level down, so the same expression compiled — and the
    // byte-identical expression was accepted unchanged in the `where` fence.
    //
    // Scrutinee (`items[1]`) and every arm BODY (`arm[1..]`) ARE walked: a body can legitimately
    // nest a constructor, and the four kinds strike-nested-wall wired must keep reaching there.
    // Only `arm[0]` is skipped — a bare variant keyword, a destructuring List, or a literal, none
    // of which is a constructor call in that position. This mirrors `purity.rs`'s `match` arm
    // (`classify_expr`, "skip pattern (element 0); check body forms (1..)") exactly, including its
    // one indirection through `resolve_core_name` (STOP-4: never a second arm keyed on the rete
    // name). BOTH spellings were MEASURED to reach this walker un-lowered, by instrumenting it:
    // `:wat::rete::core::match` (the `RETE_OPS` row in `vocabulary.rs`) and `:wat::core::match`
    // each arrive verbatim in a `:then` operand, and each reproduced the false refusal at HEAD.
    //
    // ⛔ `let` / `fn` / `cond` deliberately get NO arm here, and the omission is measured, not
    // assumed: `let` and `fn` bind in a **Vector**, so `walk_nested_constructors` returns at the
    // `WatAST::List` bind above before ever reaching a pattern; a `cond` clause is a List but its
    // `items[0]` is a call form, so keyword extraction fails and it falls through harmlessly. An
    // arm for any of them would be a dead branch no mutation could redden.
    if let Some(WatAST::Keyword(head, _)) = items.first() {
        if crate::rete::vocabulary::resolve_core_name(head) == ":wat::core::match" {
            if let Some(scrutinee) = items.get(1) {
                walk_nested_constructors(scrutinee, rule_name, types, binds, errors);
            }
            for arm in items.iter().skip(2) {
                // A non-List arm is malformed; shape is not this walker's diagnostic (the freeze
                // checker and `classify_expr` both raise on it), and recursing into it would be a
                // no-op anyway — the bind at the top of this function returns on any non-List.
                if let WatAST::List(parts, _) = arm {
                    for body_form in parts.iter().skip(1) {
                        walk_nested_constructors(body_form, rule_name, types, binds, errors);
                    }
                }
            }
            return;
        }
    }
    // ★ Arc 278 strike-nested-wall — READ THE FORM AS IT EXISTS AT THE WALL, NOT AS IT WAS
    // WRITTEN. `defrecord`'s companion macro lowers EVERY record-constructor call before freeze
    // (`macros/parse.rs:343`, `(:wat::core::kwargs-construct ~_kc-type ~@call-args)`), so the head
    // this walker sees is the MACRO's and the type is at INDEX 1:
    //
    //     (:fsn::Inner :nope ?k)  ->  (:wat::core::kwargs-construct :fsn::Inner :nope ?k)
    //
    // Written against `items[0]` alone, `types.get(head)` was `None` for every record-constructor
    // spelling and the aggregate branch below never opened — leaving `UnknownField`,
    // `RhsMissingFields`, `RhsArityMismatch` and `RhsPositionalConstructionRetired` unreachable
    // HERE. That is ORPHANING, not oversight: the walker was correct when written, and the fix
    // that made a nested constructor WORK introduced the lowering that darkened it. Its three
    // siblings were all re-pointed and this one was not — `purity.rs:349`, `purity.rs:829`,
    // `kernel/stratify.rs:517`, `expr_ir/mod.rs:547`. This is the fourth site, same idiom.
    //
    // ⛔ `:wat::core::aggregate-new` is deliberately NOT recognised here, and its absence is
    // driven rather than assumed. `purity.rs` and `stratify.rs` pair the two verbs, so the reflex
    // is to mirror them — but (a) no surface spelling lowers to `aggregate-new` at this wall (the
    // kwargs sugar, both positional spellings and a positionally-written outer item ALL arrive as
    // `kwargs-construct`; the positional prime `:T'` arrives UN-lowered under its own primed head,
    // which `types.get` does not resolve), and (b) `aggregate-new` IS the positional route, so
    // `RhsPositionalConstructionRetired` — which exists to refuse positional construction at the
    // bare kwargs-macro name — would be an actively WRONG refusal there, not merely a dead one.
    // Recognising it would need its own rules, which is a different strike.
    //
    // The enum-variant sibling branch below keeps reading the same slot: an enum variant is NOT
    // lowered, and when the lowered head is present the slot holds a record type, on which
    // `enum_variant_ctor` is `None` anyway.
    let type_idx = match &items[0] {
        WatAST::Keyword(h, _) if h == ":wat::core::kwargs-construct" => 1,
        _ => 0,
    };
    // STOP-3: the type slot is NOT assumed to be a keyword. The macro always emits one, but a
    // hand-written `(:wat::core::kwargs-construct x 1)` over a non-keyword is expressible (it is
    // the shape `is_declaration_derived_construction`'s own gate refuses, `purity.rs:829`). Such a
    // form falls through to the generic recursion below rather than widening the match blind.
    if let Some(WatAST::Keyword(head, _)) = items.get(type_idx) {
        let args = &items[type_idx + 1..];
        // Bare aggregate-type constructor head.
        if let Some(TypeDef::Aggregate(_)) = types.get(head) {
            let nested_type = head.trim_start_matches(':').to_string();
            let field_names = lookup_fields(types, &nested_type).unwrap_or_default();
            // D11 — the DECLARED type of each nested field, index-aligned with `field_names`, the
            // same pairing `validate_then_form` makes at the top level. Both accessors read the
            // same `TypeDef::Aggregate`, so a `lookup_fields` hit implies a `lookup_field_types`
            // hit; the `unwrap_or_default` is the belt, and an empty vector makes every `get(i)`
            // miss and every per-field type check SKIP rather than mis-index.
            let field_types = lookup_field_types(types, &nested_type).unwrap_or_default();
            let is_kwargs = crate::rete::eval_insert::rete_is_kwargs(args);
            if is_kwargs {
                let mut supplied: Vec<String> = Vec::with_capacity(args.len() / 2);
                for pair in args.chunks(2) {
                    let field = match &pair[0] {
                        WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
                        _ => unreachable!("is_kwargs confirmed a Keyword at every even index"),
                    };
                    // Through the ONE producer, taking the key KEYWORD. This site used to
                    // open-code the same error against `span` — the whole nested constructor
                    // form — which is how a promise made in three docs was broken at three
                    // sites: an inline `ReteCheckError { span, .. }` accepts any span in scope.
                    check_field_kw(&pair[0], rule_name, &nested_type, &field_names, errors);
                    // ★ D11 — the TYPE wall, kwargs side, at DEPTH. Same producer, same error
                    // kind, one level down: a nested occurrence is the same claim at a different
                    // position, so `RhsFieldTypeMismatch` is reused unchanged.
                    //
                    // An unknown field name needs no guard of its own: `position` misses, the
                    // `and_then` yields `None`, and the pair is skipped — so `check_field_kw`'s
                    // `UnknownField` stands alone, exactly as the top level's early return
                    // arranges. What is DELIBERATELY not mirrored is that return's OTHER half:
                    // a `RhsMissingFields` on a SIBLING field does not suppress a type finding
                    // here. The top level returns there to avoid `reorder_then_kwargs` rewriting
                    // a form already flagged invalid; this walker performs no rewrite (a nested
                    // operand's kwargs are reordered at FIRE time regardless — see this
                    // function's header), so the only effect of copying the return would be to
                    // drop a real, separate finding about a DIFFERENT field.
                    //
                    // ⚠ The `rhs_operand_can_never_resolve` skip is copied from the top level but
                    // its REASON does not carry: up there it suppresses a second ruin over an
                    // operand `check_rhs_operands` has already flagged, and `check_rhs_operands`
                    // is NOT called at depth. Kept anyway, deliberately and narrowly: of the
                    // shapes it excludes, `resolve_operand_type` answers `UnboundInThisRule` for
                    // every one (non-`?` `Symbol`, `RationalLit`, `BigIntLit`, `NilLit`, `Vector`,
                    // `Map`, `Set`) EXCEPT `Keyword`, which it types as a constant. So the skip
                    // costs exactly one class — a keyword constant in a nested value position —
                    // and typing that class would be NEW enforcement with no top-level twin
                    // (up there such an operand is refused as unresolvable instead), which is a
                    // different ruling from this strike's.
                    if !rhs_operand_can_never_resolve(&pair[1]) {
                        if let Some(declared) = field_names
                            .iter()
                            .position(|f| *f == field)
                            .and_then(|i| field_types.get(i))
                        {
                            check_then_field_type(
                                &field,
                                declared,
                                &pair[1],
                                rule_name,
                                &nested_type,
                                binds,
                                types,
                                errors,
                            );
                        }
                    }
                    supplied.push(field);
                }
                let missing: Vec<String> =
                    field_names.iter().filter(|f| !supplied.contains(f)).cloned().collect();
                if !missing.is_empty() {
                    errors.push(ReteCheckError {
                        span: span.clone(),
                        kind: ReteCheckErrorKind::RhsMissingFields {
                            rule: rule_name.to_string(),
                            fact_type: nested_type.clone(),
                            missing,
                        },
                    });
                }
            } else if args.len() <= 1 {
                // Single-arg / zero-arg positional passthrough — mirrors `eval_kwargs_construct`'s
                // own `rest.len() <= 1` passthrough straight to `construct_aggregate`.
                if args.len() != field_names.len() {
                    errors.push(ReteCheckError {
                        span: span.clone(),
                        kind: ReteCheckErrorKind::RhsArityMismatch {
                            rule: rule_name.to_string(),
                            fact_type: nested_type.clone(),
                            expected: field_names.len(),
                            got: args.len(),
                        },
                    });
                } else {
                    // ★ D11 — the TYPE wall, positional side, at DEPTH. Positional args ARE
                    // declaration order by definition, so arg `i` fills field `i` — but ONLY when
                    // the counts agree, which is what the `else` states: under a mismatch
                    // `RhsArityMismatch` above is the finding and inventing an alignment would
                    // report a type fault against a field the author never addressed. Same
                    // ruling, same words, as the top level's positional branch.
                    //
                    // This arm is narrow BY CONSTRUCTION, not by oversight: it is only reached
                    // for `args.len() <= 1`, so an equal count means a one-field record given one
                    // positional arg (or a zero-field record given none). Every WIDER positional
                    // spelling is already refused above as `RhsPositionalConstructionRetired`,
                    // which `eval_kwargs_construct` retires unconditionally at fire time — so
                    // there is no second positional shape here left to type.
                    for (i, arg) in args.iter().enumerate() {
                        if rhs_operand_can_never_resolve(arg) {
                            continue;
                        }
                        let (Some(field), Some(declared)) =
                            (field_names.get(i), field_types.get(i))
                        else {
                            continue;
                        };
                        check_then_field_type(
                            field,
                            declared,
                            arg,
                            rule_name,
                            &nested_type,
                            binds,
                            types,
                            errors,
                        );
                    }
                }
            } else {
                // Multi-arg, not kwargs — `eval_kwargs_construct` retires this shape
                // unconditionally at fire time; wall it here with its own message.
                errors.push(ReteCheckError {
                    span: span.clone(),
                    kind: ReteCheckErrorKind::RhsPositionalConstructionRetired {
                        rule: rule_name.to_string(),
                        fact_type: nested_type.clone(),
                        got: args.len(),
                    },
                });
            }
            for arg in args {
                walk_nested_constructors(arg, rule_name, types, binds, errors);
            }
            return;
        }
        // Bare enum-variant constructor head (`{EnumPath}::{Variant}`) — mirrors
        // `constructor_meta`'s own resolution (`purity.rs`).
        // Resolution through `matcher::enum_variant_ctor` — the one registry read. What to DO
        // with the answer stays here: the validator's job is the arity diagnostic.
        {
            {
                let expected =
                    crate::rete::matcher::enum_variant_ctor(types, head).map(|(_, _, n)| n);
                if let Some(expected) = expected {
                    let got = args.len();
                    if got != expected {
                        errors.push(ReteCheckError {
                            span: span.clone(),
                            kind: ReteCheckErrorKind::RhsArityMismatch {
                                rule: rule_name.to_string(),
                                fact_type: head.trim_start_matches(':').to_string(),
                                expected,
                                got,
                            },
                        });
                    }
                    for arg in args {
                        walk_nested_constructors(arg, rule_name, types, binds, errors);
                    }
                    return;
                }
            }
        }
    }
    // Not a recognized constructor head — recurse into every item anyway (a plain call's
    // arguments, e.g. `(:wat::core::+ (:usr::Inner 1) ?a)`, may still nest a constructor deeper).
    for item in items {
        walk_nested_constructors(item, rule_name, types, binds, errors);
    }
}

/// Validates a `:then` fact-form: fact-type head and, for kwargs, every `:field` name.
/// `reorder_then_kwargs` then rewrites kwargs args to declaration order in place.
fn validate_then_form(
    fact_form: &mut WatAST,
    rule_name: &str,
    types: &TypeEnv,
    binds: &std::collections::HashMap<String, String>,
    errors: &mut Vec<ReteCheckError>,
) {
    let fact_span = fact_form.span().clone();
    let fact_items = match fact_form {
        WatAST::List(fi, _) if !fi.is_empty() => fi,
        other => {
            let form_copy = other.clone();
            errors.push(malformed(fact_span, rule_name, "", &form_copy));
            return;
        }
    };
    let type_kw = match &fact_items[0] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            let form_copy = other.clone();
            errors.push(malformed(fact_span, rule_name, "", &form_copy));
            return;
        }
    };
    let fact_type = type_kw.trim_start_matches(':').to_string();
    let field_names = match lookup_fields(types, &fact_type) {
        Some(f) => f,
        // Arc 278 Stone B (DESIGN-STONE-then-is-a-vector-of-singular-facts.md § "Stone B") —
        // RELAXES rather than tightens: a head this validator cannot resolve in `types` is no
        // longer an error HERE. It is no longer necessarily a constructor at all — Stone B
        // widens an item's head to "a fn whose declared return type is a fact type," and this
        // validator carries `types: &TypeEnv` but not `sym` (no `sym.functions`, so it cannot
        // classify a fn head or its transitively-composed body — threading `sym` through the
        // whole static validate/mod.rs call tree is the param cascade `BRIEF-then-user-forms.md`'s
        // STOP-1 forbids). The wat-side fence (`wat/rete/compile.wat`'s `then-item-fence`, wired into
        // `compile-rule`) takes over enforcing head-legality, the three axes, and
        // "returns-a-fact" for this item — at rule-COMPILE time, same as `where`'s fence. A
        // genuinely unknown/malformed head still surfaces there, just not from this function.
        None => return,
    };
    // D10 — the DECLARED type of each field, index-aligned with `field_names` (both read the same
    // `TypeDef::Aggregate`, so a `lookup_fields` hit implies a `lookup_field_types` hit; the
    // `unwrap_or_default` is the belt, and an empty vector makes every `get(i)` miss and every
    // per-field type check skip rather than mis-index).
    let field_types = lookup_field_types(types, &fact_type).unwrap_or_default();

    // Arc 294 item 9a — the SAME kwargs-shape test `build_insert_fact` uses:
    // even arity, ≥2 args, a keyword at every even index.
    let args = &fact_items[1..];
    let is_kwargs = crate::rete::eval_insert::rete_is_kwargs(args);

    if is_kwargs {
        let mut kv_pairs: Vec<(String, WatAST)> = Vec::with_capacity(args.len() / 2);
        // ONE walk, because the check needs the key NODE and the reorder needs the key TEXT.
        // The old shape built `kv_pairs` first and then checked the names off it — by which point
        // the keyword's span had been thrown away, so the error could only be located at
        // `fact_span` (the whole fact form) while `check_field_at`'s doc promised the field's own.
        // rune:lint(cited-name-absent) check_field_at — the span-taking predecessor, renamed to `check_field_kw` when it
        // was made to take the keyword node instead; nothing bears the old name.
        // `&=` and not `&&`: no short-circuit, because batching every finding is this validator's
        // contract.
        let mut all_known = true;
        for pair in args.chunks(2) {
            let field = match &pair[0] {
                WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
                _ => unreachable!("is_kwargs confirmed a Keyword at every even index"),
            };
            all_known &= check_field_kw(&pair[0], rule_name, &fact_type, &field_names, errors);
            kv_pairs.push((field, pair[1].clone()));
        }
        // Arc 278 BRIEF-construction-total-three-walls.md #2 — every declared field must be
        // supplied. STOP-A audited the corpus first (every kwargs `:then` found fully supplies
        // its type's fields; none rely on the old under-supply); closing this is free. Checked
        // even when `all_known` is false — batch every finding, this validator's own contract.
        let missing: Vec<String> = field_names
            .iter()
            .filter(|f| !kv_pairs.iter().any(|(k, _)| k == *f))
            .cloned()
            .collect();
        let has_missing = !missing.is_empty();
        if has_missing {
            errors.push(ReteCheckError {
                span: fact_span.clone(),
                kind: ReteCheckErrorKind::RhsMissingFields {
                    rule: rule_name.to_string(),
                    fact_type: fact_type.clone(),
                    missing,
                },
            });
        }
        if !all_known || has_missing {
            return; // do not rewrite a form already flagged invalid
        }
        // The wall, kwargs side. Checked BEFORE the reorder rewrites `fact_items` in place, so
        // the operand reported is the one the author wrote, at the span they wrote it at.
        let kwargs_values: Vec<WatAST> = kv_pairs.iter().map(|(_, v)| v.clone()).collect();
        check_rhs_operands(&kwargs_values, rule_name, &fact_type, errors);
        // ★ D10 — the TYPE wall, kwargs side. `kv_pairs` has paired the destination field with
        // its value AST all along; this is the call that was missing. Checked BEFORE the reorder,
        // for the same reason `check_rhs_operands` above is: the operand named in the diagnostic
        // must be the one the author wrote, at the span they wrote it at.
        //
        // An operand `check_rhs_operands` has already flagged is SKIPPED — `rhs_operand_can_never_resolve`
        // is the same predicate that produced that finding. Reporting both would tell the author
        // to fix the type of an operand whose real fault is that it can never resolve at all, and
        // two ruins pointing opposite ways teach worse than one (R29 `RVINA ERVDIT`).
        for (field, value) in &kv_pairs {
            if rhs_operand_can_never_resolve(value) {
                continue;
            }
            let Some(declared) =
                field_names.iter().position(|f| f == field).and_then(|i| field_types.get(i))
            else {
                continue;
            };
            check_then_field_type(
                field, declared, value, rule_name, &fact_type, binds, types, errors,
            );
        }
        // Arc 278 #1/#3 — recurse for a NESTED constructor operand (e.g. `:inner (:usr::Inner
        // :x 1)`); the top-level shape above only covers THIS item's own head.
        for v in &kwargs_values {
            walk_nested_constructors(v, rule_name, types, binds, errors);
        }

        reorder_then_kwargs(fact_items, &field_names, &kv_pairs, &fact_span);
    } else {
        // The wall, positional side. Independent of the arity verdict below: a rule can be both
        // wrong-arity AND carry an unresolvable operand, and batching every finding is this
        // validator's whole contract (`validate_rete_rules` returns them all, not the first).
        check_rhs_operands(args, rule_name, &fact_type, errors);
        // ★ D10 — the TYPE wall, positional side. Positional args ARE declaration order by
        // definition (`eval_insert.rs`'s `rete_kwargs_value_asts` says so), so arg `i` fills field
        // `i` — but ONLY when the counts agree. Under a count mismatch there is no defensible
        // pairing, `RhsArityMismatch` below is the finding, and inventing an alignment would
        // report a type fault against a field the author never addressed.
        if args.len() == field_names.len() {
            for (i, arg) in args.iter().enumerate() {
                if rhs_operand_can_never_resolve(arg) {
                    continue;
                }
                let (Some(field), Some(declared)) = (field_names.get(i), field_types.get(i)) else {
                    continue;
                };
                check_then_field_type(
                    field, declared, arg, rule_name, &fact_type, binds, types, errors,
                );
            }
        }
        // Arc 278 #1/#3 — recurse for a NESTED constructor operand, same as the kwargs branch.
        for a in args {
            walk_nested_constructors(a, rule_name, types, binds, errors);
        }

        // Positional: arg count must equal the type's field count.
        if args.len() != field_names.len() {
            errors.push(ReteCheckError {
                span: fact_span,
                kind: ReteCheckErrorKind::RhsArityMismatch {
                    rule: rule_name.to_string(),
                    fact_type,
                    expected: field_names.len(),
                    got: args.len(),
                },
            });
        }
    }
}

/// Rewrite a kwargs-style fact form into positional field order, in place.
///
/// `fact_items.truncate(1)` keeps the head and drops the arguments before re-extending, so the
/// rewrite is a replacement rather than an append — re-running it cannot accumulate duplicates.
///
/// ⛔ **THIS FUNCTION NO LONGER REPORTS AN UNKNOWN FIELD, AND ITS DOC USED TO BE THE ONLY PLACE
/// THE CONTRACT WAS STATED.** It read: *"An unknown field name is reported against ITS OWN span
/// (`bad.span`), not the fact's, so the caret lands on the offending keyword rather than the whole
/// form."* That was the truest sentence in the file and it described **dead code** — the only one
/// of four `UnknownField` producers that pointed at the right token was the one that could not
/// run, while the three live ones pointed at an enclosing form. Keeping an unreachable arm because
/// it documents better behaviour is a graveyard that reads like a spec. The contract now lives on
/// `check_field_kw`, which is the ONE producer, takes the keyword NODE, and runs.
///
/// The `Err` arm is unreachable **by the caller's guard, not by hope**: the sole caller
/// (`validate_then_form`) returns at `!all_known || has_missing` before reaching here, and
/// `all_known` is false on exactly the condition `reorder_kwargs_by_field_name` errors on — a
/// supplied kwarg naming no declared field. Both quantify over the same `kv_pairs` against the
/// same `field_names`. Driven, not merely read: with this `unreachable!` in place, a `:then`
/// naming an unknown kwarg field is reported by the check above and never arrives here.
fn reorder_then_kwargs(
    fact_items: &mut Vec<WatAST>,
    field_names: &[String],
    kv_pairs: &[(String, WatAST)],
    fact_span: &crate::span::Span,
) {
    let field_order: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();
    let kv_ref: Vec<(&str, WatAST)> = kv_pairs.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    match reorder_kwargs_by_field_name(&field_order, &kv_ref, fact_span) {
        Ok(reordered) => {
            fact_items.truncate(1);
            fact_items.extend(reordered);
        }
        Err(bad) => unreachable!(
            "`validate_then_form` returns at `!all_known` before calling this, and `all_known` is \
             false on exactly this condition — a kwarg naming no declared field, already reported \
             by `check_field_kw` against the keyword's own span. Field: {}",
            bad.field
        ),
    }
}

// ─── Tests (S2/S3 gate) ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    /// The wall's rendered error, PARSED: assert the first error's variant tag and read a
    /// named field. The boxed error has no `Any` bound (see the note on the tests below), so
    /// the wire EDN is the only face available — but a SUBSTRING search over it is a loose
    /// check the value does not deserve: it is deterministic, and `contains` would pass on
    /// reordered fields or appended garbage. A whole-blob golden is the wrong tool too: each
    /// error carries a `:span` into THIS file's own inline `r#"…"#` source, so the golden
    /// would go stale on any edit above the test. Parsing pins the tag + named fields exactly
    /// and leaves the span's VALUE free — while `rete_error_is_located` still proves it is
    /// THERE, which is the wall's actual claim.
    fn rete_error(edn: &str, variant: &str) -> Vec<(wat_edn::OwnedValue, wat_edn::OwnedValue)> {
        use wat_edn::{Keyword, OwnedValue, Tag};
        let parsed = wat_edn::parse_owned(edn).expect("the wall's error face must be EDN");
        let errors = match parsed {
            OwnedValue::Tagged(tag, body) => {
                assert_eq!(tag, Tag::ns("wat.rete", "ReteCheckErrors"), "outer batch tag");
                match *body {
                    OwnedValue::Map(m) => m
                        .into_iter()
                        .find(|(k, _)| *k == OwnedValue::Keyword(Keyword::new("errors")))
                        .map(|(_, v)| v)
                        .expect("the batch must carry :errors"),
                    other => panic!("expected a map body; got {other:?}"),
                }
            }
            other => panic!("expected a tagged #wat.rete/ReteCheckErrors batch; got {other:?}"),
        };
        let first = match errors {
            OwnedValue::Vector(mut xs) if !xs.is_empty() => xs.remove(0),
            other => panic!("expected a non-empty :errors vector; got {other:?}"),
        };
        match first {
            OwnedValue::Tagged(tag, body) => {
                assert_eq!(tag, Tag::ns("wat.rete", variant), "error variant tag");
                match *body {
                    OwnedValue::Map(m) => m,
                    other => panic!("expected a map body; got {other:?}"),
                }
            }
            other => panic!("expected a tagged error; got {other:?}"),
        }
    }

    /// Read one field of a parsed error as a String.
    fn field_str(fields: &[(wat_edn::OwnedValue, wat_edn::OwnedValue)], name: &str) -> String {
        use wat_edn::{Keyword, OwnedValue};
        let v = fields
            .iter()
            .find(|(k, _)| *k == OwnedValue::Keyword(Keyword::new(name)))
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("the error must carry :{name}"));
        match v {
            OwnedValue::String(s) => s.to_string(),
            other => panic!(":{name} must be a String; got {other:?}"),
        }
    }

    /// The wall's claim is a LOCATED error — prove the span is present without pinning it.
    fn rete_error_is_located(fields: &[(wat_edn::OwnedValue, wat_edn::OwnedValue)]) -> bool {
        use wat_edn::{Keyword, OwnedValue};
        fields
            .iter()
            .any(|(k, v)| *k == OwnedValue::Keyword(Keyword::new("span")) && *v != OwnedValue::Nil)
    }

    use super::*;
    use crate::freeze::env::build_env;

    /// Test helper: find the first `:wat::rete::make-rule` form anywhere in a parse tree.
    ///
    /// Recurses into every list rather than checking the top level, because the tests that use it
    /// build rules nested inside `let`/`do` wrappers and should not have to know the nesting
    /// depth their fixture happens to produce.
    fn find_make_rule(forms: &[WatAST]) -> Option<&Vec<WatAST>> {
        for f in forms {
            if let WatAST::List(items, _) = f {
                if let Some(WatAST::Keyword(k, _)) = items.first() {
                    if k == ":wat::rete::make-rule" {
                        return Some(items);
                    }
                }
                if let Some(found) = find_make_rule(items) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// The corrupt fixture from the disconfirming probe (`src/freeze/env.rs`'s
    /// `rete_wall_probe`) — an injected bare-keyword `:celsius` clause — must now freeze
    /// with a LOCATED error naming the rule, instead of silently passing through.
    ///
    /// `build_env` hooks the `defrule` wall via the generic `FreezeValidator` inventory drain
    /// now (`src/freeze/env.rs` step 7.8), so the located error surfaces as `build_env`'s own
    /// `Err(StartupError::Validator(..))` — the end-to-end proof the drain fires, not just the
    /// bare fn in isolation. The boxed error carries no `Any` bound (a multi-consumer registry
    /// has no reason to let a caller downcast back to one specific validator's concrete error
    /// type), so the located-error shape is asserted on the wire EDN text — this is also the
    /// namespace-preservation proof: a corrupt rule STILL tags `#wat.rete/MalformedClause`
    /// through the box, not a generic tag.
    #[test]
    fn corrupt_when_clause_is_a_located_error() {
        let src = r#"
(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :alert::Unattended    [location <- :wat::core::String])
(:wat::rete::defrule :alert::unattended
  :when
  [(:weather::Temperature :celsius (?loc <- :location) :location (?c <- :celsius))]
  :then
  [(:alert::Unattended :location ?loc)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        let boxed = match build_env(forms) {
            Err(crate::freeze::StartupError::Validator(e)) => e,
            Err(other) => panic!("expected StartupError::Validator; got {other:?}"),
            Ok(_) => panic!("the injected bare-keyword clause must be a located freeze error"),
        };
        let edn = wat_edn::write(&boxed.to_edn());
        let e = rete_error(&edn, "MalformedClause");
        assert_eq!(field_str(&e, "rule"), "alert::unattended", "the error must name the offending rule");
        assert!(rete_error_is_located(&e), "the wall's errors are LOCATED; got: {edn}");
    }

    /// A correct defrule (no corruption) freezes clean.
    #[test]
    fn correct_defrule_validates_clean() {
        let src = r#"
(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :alert::Unattended    [location <- :wat::core::String])
(:wat::rete::defrule :alert::unattended
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius))]
  :then
  [(:alert::Unattended :location ?loc)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        // build_env hooks validate_rete_rules internally (step 7.8) — a well-formed rule
        // must not turn a clean build_env into an error.
        build_env(forms).expect("a well-formed rule freezes clean");
    }

    // ─── The `:not` bind wall ────────────────────────────────────────────────────────────────

    /// The rete twin of `UnconsumedTypeParam`: a bind under a `:not`, consumed nowhere.
    #[test]
    fn an_unconsumed_bind_inside_a_not_is_refused() {
        let src = r#"
(:wat::core::defrecord :w::S2  [k <- :wat::core::i64])
(:wat::core::defrecord :w::Hit [k <- :wat::core::i64])
(:wat::rete::defrule :w::r
  :when [(:wat::rete::not (:w::S2 (?s <- :k)))]
  :then [(:w::Hit :k 1)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        let boxed = match build_env(forms) {
            Err(crate::freeze::StartupError::Validator(e)) => e,
            Err(other) => panic!("expected StartupError::Validator; got {other:?}"),
            Ok(_) => panic!("a bind under `:not` consumed nowhere must be a located freeze error"),
        };
        let edn = wat_edn::write(&boxed.to_edn());
        let e = rete_error(&edn, "UnconsumedWrapperBind");
        assert_eq!(field_str(&e, "rule"), "w::r");
        assert_eq!(field_str(&e, "var"), "?s");
        assert!(rete_error_is_located(&e), "the wall's errors are LOCATED; got: {edn}");
    }

    /// The escape, and the reason this wall is at DECLARATION time rather than at `compile-all`:
    /// the reference fails at fire with `unbound symbol`, but only along the path where the
    /// `:not` PASSES — so on data where a matching fact exists the rule answers cleanly and the
    /// defect stays invisible. Measured, same binary, same rule: fact present → `n=0`, exit 0;
    /// fact absent → UnboundSymbol, exit 1.
    #[test]
    fn a_bind_escaping_a_not_is_refused_at_declaration_time() {
        let src = r#"
(:wat::core::defrecord :w::S2  [k <- :wat::core::i64])
(:wat::core::defrecord :w::Hit [k <- :wat::core::i64])
(:wat::rete::defrule :w::r
  :when [(:wat::rete::not (:w::S2 (?s <- :k)))
         (:wat::rete::where (:wat::rete::core::i64::>= ?s 0))]
  :then [(:w::Hit :k 1)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        let boxed = match build_env(forms) {
            Err(crate::freeze::StartupError::Validator(e)) => e,
            Err(other) => panic!("expected StartupError::Validator; got {other:?}"),
            Ok(_) => panic!("a `:not`-bound variable referenced outside must not reach fire"),
        };
        let edn = wat_edn::write(&boxed.to_edn());
        let e = rete_error(&edn, "EscapedWrapperBind");
        assert_eq!(field_str(&e, "var"), "?s");
        assert!(rete_error_is_located(&e), "the wall's errors are LOCATED; got: {edn}");
    }

    /// ⛔ THE REGRESSION GUARD FOR THE FALSE POSITIVE THIS WALL NEARLY SHIPPED.
    ///
    /// The wall covered `:exists` for exactly one build. That was WRONG: in wat `:exists` binds
    /// OUTWARD, and `wat-scripts/perf/grid/leading-exists.wat` — a live accuracy axis — reads its
    /// `:exists`-bound `?loc` back out of the query rows by string key
    /// (`PersistentMap/get p "?loc"`) to build `:derived`. A consumer in HOST CODE is invisible to
    /// any syntactic check, so no such check may judge an outward-binding construct.
    ///
    /// This shape is textually identical to the refused one above except for the wrapper. If it
    /// ever starts failing, the wall has re-grown over `:exists` and `leading-exists` is about to
    /// break — fix the wall, never this test.
    #[test]
    fn an_unconsumed_bind_inside_exists_is_left_alone_because_exists_binds_outward() {
        let src = r#"
(:wat::core::defrecord :w::Wind [loc <- :wat::core::String])
(:wat::core::defrecord :w::Hit  [k <- :wat::core::i64])
(:wat::rete::defrule :w::r
  :when [(:wat::rete::exists (:w::Wind (?loc <- :loc)))]
  :then [(:w::Hit :k 1)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        build_env(forms).expect("`:exists` binds outward — its binds are NOT this wall's to judge");
    }

    /// A CORRELATION: `?loc` is bound by an earlier condition, so inside the `:not` it is a USE,
    /// not a declaration — "no Reading AT THIS loc". The single most important legal shape, and
    /// the one a naive "no binds in negations" rule would destroy.
    #[test]
    fn a_correlated_bind_inside_a_not_is_legal() {
        let src = r#"
(:wat::core::defrecord :w::Station [loc <- :wat::core::String])
(:wat::core::defrecord :w::Reading [loc <- :wat::core::String])
(:wat::core::defrecord :w::Hit     [loc <- :wat::core::String])
(:wat::rete::defrule :w::r
  :when [(:w::Station (?loc <- :loc))
         (:wat::rete::not (:w::Reading (?loc <- :loc)))]
  :then [(:w::Hit :loc ?loc)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        build_env(forms).expect("a correlated bind inside `:not` is a use, not a declaration");
    }

    /// A FRESH bind consumed by a constraint INSIDE the same `:not` — "there is no Temp under
    /// 20". In wat the bind is load-bearing: constraints reference variables, so this is the only
    /// way to say it (Clara needs no variable because its constraints name the field directly).
    #[test]
    fn a_bind_consumed_inside_the_not_is_legal() {
        let src = r#"
(:wat::core::defrecord :w::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :w::Hit  [k <- :wat::core::i64])
(:wat::rete::defrule :w::r
  :when [(:wat::rete::not (:w::Temp (?c <- :c) (:wat::rete::core::i64::< ?c 20)))]
  :then [(:w::Hit :k 1)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        build_env(forms).expect("a bind consumed by a constraint inside the `:not` is legal");
    }

    /// A quoted `make-query` is payload data (`Boundary::AllData`), not a live query
    /// in this freeze. The scratch-pad "ship the forms" probes freeze a quoted
    /// evaluand whose fact types exist only on the far side.
    #[test]
    fn quoted_make_query_of_unregistered_type_is_data_not_a_freeze_error() {
        let src = r#"
(:wat::core::defn :probe::evaluand [] -> :wat::WatAST
  (:wat::core::quote
    (:wat::rete::make-query "usr::Hot"
      (:wat::core::quote [])
      (:wat::core::quote [(:usr::Hot)]))))
"#;
        let forms = crate::parse_all!(src).expect("parse");
        build_env(forms).expect("quoted make-query is data; unknown :usr::Hot must not freeze-fail");
    }

    /// A *live* `defquery` of an unregistered fact type is the wall's job.
    #[test]
    fn live_defquery_of_unregistered_type_is_unknown_fact_type() {
        let src = r#"
(:wat::rete::defquery :usr::hot-q
  :params []
  :when [(:usr::Hot)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        let boxed = match build_env(forms) {
            Err(crate::freeze::StartupError::Validator(e)) => e,
            Err(other) => panic!("expected StartupError::Validator; got {other:?}"),
            Ok(_) => panic!("live defquery of :usr::Hot must be a freeze error"),
        };
        let edn = wat_edn::write(&boxed.to_edn());
        let e = rete_error(&edn, "UnknownFactType");
        assert_eq!(field_str(&e, "fact-type"), "usr::Hot");
        assert!(rete_error_is_located(&e), "the wall's errors are LOCATED; got: {edn}");
    }

    /// An unknown field-ref in a bind clause is a located `UnknownField` error naming the
    /// type, the bad field, and the available fields. Surfaces via `build_env`'s own error
    /// (the hook, not a bare separate call).
    #[test]
    fn unknown_field_ref_is_located() {
        let src = r#"
(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :alert::Unattended    [location <- :wat::core::String])
(:wat::rete::defrule :alert::unattended
  :when
  [(:weather::Temperature (?loc <- :location) (?bad <- :not-a-field))]
  :then
  [(:alert::Unattended :location ?loc)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        let boxed = match build_env(forms) {
            Err(crate::freeze::StartupError::Validator(e)) => e,
            Err(other) => panic!("expected StartupError::Validator; got {other:?}"),
            Ok(_) => panic!("bad field-ref must be a located freeze error"),
        };
        let edn = wat_edn::write(&boxed.to_edn());
        let e = rete_error(&edn, "UnknownField");
        assert_eq!(field_str(&e, "rule"), "alert::unattended", "the error must name the offending rule");
        assert_eq!(field_str(&e, "field"), "not-a-field", "the error must name the bad field-ref");
        assert!(rete_error_is_located(&e), "the wall's errors are LOCATED; got: {edn}");
    }

    /// S3 — a `:then` kwargs RHS written OUT of declaration order gets REWRITTEN in the
    /// residue in place, so `build_insert_fact` at fire time receives declaration order.
    #[test]
    fn out_of_order_then_kwargs_are_reordered_in_residue() {
        // Cold declares [location, celsius] — the :then below writes celsius BEFORE location.
        let src = r#"
(:wat::core::defrecord :weather2::Temp [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :alert2::Cold   [location <- :wat::core::String  celsius <- :wat::core::i64])
(:wat::rete::defrule :alert2::mark-cold
  :when
  [(:weather2::Temp (?c <- :celsius) (?loc <- :location))]
  :then
  [(:alert2::Cold :celsius ?c :location ?loc)])
"#;
        let forms = crate::parse_all!(src).expect("parse");
        // build_env hooks validate_rete_rules internally (step 7.8) — the reorder already
        // happened by the time build_env returns; read the residue it produced directly.
        let env = build_env(forms).expect("well-formed rule, just out-of-order kwargs");

        let mr = find_make_rule(&env.residue).expect("make-rule reachable");
        let then_items = match &mr[3] {
            WatAST::List(items, _) => items,
            other => panic!("then quote is a List; got {other:?}"),
        };
        let then_vec = match &then_items[1] {
            WatAST::Vector(v, _) => v,
            other => panic!("then payload is a Vector; got {other:?}"),
        };
        let fact_form = &then_vec[0];
        let fact_items = match fact_form {
            WatAST::List(items, _) => items,
            other => panic!("fact form is a List; got {other:?}"),
        };
        // Rewritten to declaration order [location, celsius] → positional [?loc, ?c].
        assert_eq!(fact_items.len(), 3, "type keyword + 2 positional values");
        assert!(matches!(&fact_items[1], WatAST::Symbol(s, _) if s.as_str() == "?loc"), "field 0 (location) now carries ?loc; got {:?}", fact_items[1]);
        assert!(matches!(&fact_items[2], WatAST::Symbol(s, _) if s.as_str() == "?c"), "field 1 (celsius) now carries ?c; got {:?}", fact_items[2]);
    }

    /// `reorder_kwargs_by_field_name` in isolation — the S3 shared helper.
    #[test]
    fn reorder_helper_maps_supplied_pairs_to_declaration_order() {
        use crate::scope::Identifier;
        let a = WatAST::Symbol(Identifier::bare("a"), crate::rust_caller_span!());
        let b = WatAST::Symbol(Identifier::bare("b"), crate::rust_caller_span!());
        let pairs = vec![("y", b.clone()), ("x", a.clone())];
        let order = ["x", "y"];
        let out = reorder_kwargs_by_field_name(&order, &pairs, &crate::rust_caller_span!())
            .expect("both fields known");
        assert_eq!(out, vec![a, b]);
    }

    #[test]
    fn reorder_helper_rejects_unknown_field_name() {
        use crate::scope::Identifier;
        let v = WatAST::Symbol(Identifier::bare("v"), crate::rust_caller_span!());
        let pairs = vec![("nope", v)];
        let order = ["x", "y"];
        let err = reorder_kwargs_by_field_name(&order, &pairs, &crate::rust_caller_span!())
            .expect_err("unknown field must error");
        assert_eq!(err.field, "nope");
    }

    /// ★★ A COMPUTED OPERAND IS TYPED LIKE ANY OTHER — the fourth source, gated.
    ///
    /// ⚠ **WRAPPING AN OPERAND IN A CALL USED TO MAKE ITS TYPE ERROR DISAPPEAR.** Measured
    /// 2026-08-28: `(string::= :v "x")` on an i64 field was CAUGHT, and
    /// `(string::= (i64::+ :v 0 :undefined 0) "x")` — the same mismatch, same field, one call
    /// deeper — was NOT. The rule then compiled, fired and matched nothing, silently.
    ///
    /// The mechanism was `resolve_operand_type`'s `_ => UnboundInThisRule` arm. That variant's own
    /// doc says it means "a `?var` bound NOWHERE in this rule", and it was written to be *"visibly
    /// out of scope rather than indistinguishable from a pass"* — then a `WatAST::List` fell into
    /// it and became exactly the indistinguishable pass the doc warns against. The three sources
    /// the function documents as exhaustive were written before fix-list F made a nested call a
    /// legal operand, and nothing came back to re-read them.
    ///
    /// **Why the type is knowable, which is the whole argument for source 4.** Every `RETE_OPS`
    /// row is `pure · deterministic · total` — `every_rete_row_is_total` makes a non-total row a
    /// red build. Totality means an op is defined on its whole domain, so an `Alias`/`Fallback`
    /// row's `ret` is a FACT about the row, exactly as a field's declared type is a fact about
    /// the record. The builder's cut against the first draft of this function applies verbatim:
    /// *"why is any of this a guess? we know the type's value from the record def."*
    #[test]
    fn a_computed_operand_is_typed_like_any_other() {
        const MISMATCH: &str = r#"
(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::core::i64])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])
(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k)
     (:wat::rete::core::string::= (:wat::rete::core::i64::+ :v 0 :undefined 0) "x"))]
  :then
  [(:probe::Out :k ?k)])
"#;
        let forms = crate::parse_all!(MISMATCH).expect("parse");
        let boxed = match build_env(forms) {
            Err(crate::freeze::StartupError::Validator(e)) => e,
            Err(other) => panic!("expected StartupError::Validator; got {other:?}"),
            Ok(_) => panic!(
                "an i64-returning call compared by `string::=` must be REFUSED. This compiled, \
                 fired and matched nothing for the life of the engine"
            ),
        };
        let edn = wat_edn::write(&boxed.to_edn());
        let e = rete_error(&edn, "ConstraintTypeMismatch");
        assert_eq!(
            field_str(&e, "field-type"),
            "i64",
            "the operand's type comes from the HEAD ROW's `ret`, which is what source 4 adds"
        );
        // The diagnostic must quote the CALL, not a field name — R29 `RVINA ERVDIT`. Asserted
        // EXACTLY, not by `contains`: this string is `render_form` over a fixed AST, so it is
        // fully deterministic and a loose check would pass on a mangled rendering. It also pins
        // the rendering fix — `describe_operand` used to strip a keyword's colon and one caller
        // re-added it, so a nested call came out as `:(:wat.rete.core.i64/+ …)`, un-pasteable.
        assert_eq!(
            field_str(&e, "field"),
            "(:wat.rete.core.i64/+ :v 0 :undefined 0)",
            "the message must quote the offending CALL verbatim so it can be pasted back"
        );
        assert!(rete_error_is_located(&e), "the wall's errors are LOCATED; got: {edn}");

        // ⛔ THE OVER-REFUSAL CONTROL. A change that refused every computed operand would satisfy
        // the assertions above and be catastrophically wrong — and this arc has shipped exactly
        // that mistake before (a termination verifier once refused a legal fn-headed `:then`).
        // The identical call compared by its CORRECT comparator must validate clean.
        let ok = MISMATCH
            .replace(":wat::rete::core::string::=", ":wat::rete::core::i64::=")
            .replace(r#" "x"))]"#, " 10))]");
        assert_ne!(MISMATCH, ok, "the rewrite must change the comparator");
        let forms = crate::parse_all!(&ok).expect("parse");
        assert!(
            build_env(forms).is_ok(),
            "`i64::=` over an i64-returning call is well typed and must pass — source 4 types the \
             operand, it does not refuse it"
        );

        // ⛔ THE VACUITY CONTROL. The plain-field spelling of the same mismatch was ALWAYS caught,
        // so pinning it proves this gate measures the new source rather than the old one.
        let plain = MISMATCH.replace("(:wat::rete::core::i64::+ :v 0 :undefined 0)", ":v");
        assert_ne!(MISMATCH, plain, "the rewrite must remove the nesting");
        let forms = crate::parse_all!(&plain).expect("parse");
        assert!(
            build_env(forms).is_err(),
            "the un-nested mismatch was caught before this strike and must stay caught"
        );
    }
}
