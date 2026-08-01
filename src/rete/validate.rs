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
//! `build_insert_fact` receives declaration order at fire time — retiring the
//! `matcher.rs:451`-era follow-up).
//!
//! ## One grammar, shared (design call 1)
//!
//! Both the runtime matcher (`eval_clause`) and this validator classify rete-DSL shapes via
//! the SAME [`crate::rete::matcher::classify_rete_clause`] (S1). A second hand-written
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
//! scope; a wider wall is a named follow-on, not a half-wall on this corruption class.
//!
//! ## What this does NOT touch
//!
//! The wat oracle (`wat/rete.wat`) and the native kernel (`src/rete/kernel.rs`) are UNMOVED —
//! this is a freeze-time validator bolted on ahead of the engine, not an engine change.

use std::fmt;

use wat_edn::{Keyword, OwnedValue, Tag};

use crate::ast::WatAST;
use crate::rete::matcher::{classify_rete_clause, ReteClauseShape};
use crate::span::Span;
use crate::types::{TypeDef, TypeEnv};

// ─── Error types (Pattern A: span at the outer struct, kind carries variant data) ────────────

/// Variant data for [`ReteCheckError`]. Namespace `wat.rete` — see `src/error_ns.rs`.
#[derive(Debug, Clone, wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::RETE)]
pub enum ReteCheckErrorKind {
    /// A `:when` condition's fact-type head, or a `:then` insert's fact-type head, or an
    /// accumulate's `:from` fact-type head, is not a registered aggregate type.
    UnknownFactType { rule: String, fact_type: String },
    /// A `:when` clause (or a `:when`-entry wrapper) does not match any recognized rete-DSL
    /// shape — the SAME grammar the runtime matcher classifies at fire time
    /// (`classify_rete_clause`). An unrecognized clause here would have silently `None`'d at
    /// fire time (Clara no-error, `src/rete/matcher.rs`); the wall makes it a located error.
    MalformedClause {
        rule: String,
        fact_type: String,
        clause: String,
    },
    /// A `(?v <- :field)` bind clause, a constraint operand's `:field` reference, or a
    /// `:then` kwargs field name does not name a real field of `fact_type`. The free `?v`
    /// side of a bind/constraint is never checked here — only the `:field` side.
    UnknownField {
        rule: String,
        fact_type: String,
        field: String,
        available_fields: Vec<String>,
    },
    /// A positional `:then` insert's argument count does not match the fact type's declared
    /// field count.
    RhsArityMismatch {
        rule: String,
        fact_type: String,
        expected: usize,
        got: usize,
    },
    /// A `:then` insert's VALUE-position operand can never resolve at fire time, whatever the
    /// bindings: a nested form, a `:field` keyword (a RHS has no current fact to read a field
    /// from), or a bare non-`?` symbol. `resolve_operand` returns `None` for all three and
    /// `build_insert_fact` then raises — but it raised *per derived fact, mid-fire*, for a
    /// property of the RULE that no fact can change.
    ///
    /// This wall is arc 278's third statement of one ruling: a negation cycle is a compile error
    /// (R18 `NEGATIO COMPLETVM POSCIT` — "the ill-defined program given no form"), a lying
    /// `extend-type` is a compile error (R28 `SOLVIMVS NE MENTIRETVR`), and `()` stopped being a
    /// value (arc 179) because a second spelling walks around every wall built on the first.
    /// `validate_and_reorder_then` already checked the insert's SHAPE — head, fact type, field
    /// names, positional arity — and stopped short of looking *inside* an argument, which is why
    /// `(:Out (:wat::core::+ ?a 1))` passed `--check` with arity 1 == 1 field and then exploded
    /// mid-fire.
    ///
    /// SCOPE, deliberately narrow: only operands that can NEVER resolve. An unbound `?var` is
    /// equally a compile-time property but needs binder analysis over `:when` (an `Or` arm binds
    /// conditionally, `exists` binds nothing outward, `accumulate` binds its result var) — and
    /// under-collecting that set would reject LEGAL rules, which is the one failure a wall must
    /// not have. Tracked separately; this variant carries no binder claim.
    RhsUnresolvableOperand {
        rule: String,
        fact_type: String,
        /// The offending operand, rendered as wat source (`render_form`) — never Rust `Debug`.
        operand: String,
        /// What a RHS operand may be, so the error teaches rather than merely refusing (R29
        /// `RVINA ERVDIT`), mirroring `UnknownField`'s `available_fields`.
        accepted: Vec<String>,
    },
}

impl fmt::Display for ReteCheckErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReteCheckErrorKind::UnknownFactType { rule, fact_type } => write!(
                f,
                "defrule `{rule}`: `:{fact_type}` is not a registered fact type"
            ),
            ReteCheckErrorKind::MalformedClause { rule, fact_type, clause } => write!(
                f,
                "defrule `{rule}` (`:{fact_type}`): malformed rete clause `{clause}` — not a recognized :when shape"
            ),
            ReteCheckErrorKind::UnknownField { rule, fact_type, field, available_fields } => write!(
                f,
                "defrule `{rule}`: `:{fact_type}` has no field `:{field}`; available fields: [{}]",
                available_fields.join(", ")
            ),
            ReteCheckErrorKind::RhsArityMismatch { rule, fact_type, expected, got } => write!(
                f,
                "defrule `{rule}`: `:then` insert of `:{fact_type}` expects {expected} positional argument(s); got {got}"
            ),
            ReteCheckErrorKind::RhsUnresolvableOperand { rule, fact_type, operand, accepted } => write!(
                f,
                "defrule `{rule}`: `:then` insert of `:{fact_type}` has operand `{operand}`, which can \
                 never resolve at fire time — a RHS operand must be {}",
                accepted.join(", or ")
            ),
        }
    }
}

/// Pattern A (mirrors `crate::check::error::CheckError`): span at the outer struct, kind
/// carries variant data.
#[derive(Clone)]
pub struct ReteCheckError {
    pub span: Span,
    pub kind: ReteCheckErrorKind,
}

impl fmt::Debug for ReteCheckError {
    // Stone B convention: Debug emits EDN, not Rust struct layout.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::to_edn::to_wire_edn(self))
    }
}

impl fmt::Display for ReteCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::to_edn::to_wire_edn(self))
    }
}

impl std::error::Error for ReteCheckError {}

/// Aggregated errors — `validate_rete_rules` returns every finding together (one batch, like
/// `check_program`'s `CheckErrors`).
pub struct ReteCheckErrors(pub Vec<ReteCheckError>);

impl fmt::Debug for ReteCheckErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::to_edn::to_wire_edn(self))
    }
}

impl fmt::Display for ReteCheckErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::to_edn::to_wire_edn(self))
    }
}

impl std::error::Error for ReteCheckErrors {}

// ─── ToEdn + WatError impls (mirrors src/check/error_edn.rs) ─────────────────────────────────

impl crate::to_edn::ToEdn for ReteCheckError {
    fn to_edn(&self) -> OwnedValue {
        use crate::to_edn::edn_kw;
        let kind_val = self.kind.to_edn();
        match kind_val {
            OwnedValue::Tagged(tag, body) => {
                let mut fields = match *body {
                    OwnedValue::Map(f) => f,
                    other => vec![(edn_kw("body"), other)],
                };
                fields.push((edn_kw("span"), self.span.to_edn()));
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(fields)))
            }
            other => other,
        }
    }
}

impl crate::to_edn::WatError for ReteCheckError {
    fn message(&self) -> String {
        crate::to_edn::first_line(self.kind.to_string())
    }
    fn location(&self) -> OwnedValue {
        crate::to_edn::location_from_span(&self.span)
    }
    fn causes(&self) -> OwnedValue {
        OwnedValue::Vector(vec![])
    }
    fn variant(&self) -> OwnedValue {
        use crate::to_edn::ToEdn;
        crate::to_edn::strip_span_from_tagged(self.to_edn())
    }
}

impl crate::to_edn::ToEdn for ReteCheckErrors {
    fn to_edn(&self) -> OwnedValue {
        let items: Vec<OwnedValue> = self.0.iter().map(|e| e.to_edn()).collect();
        tagged("ReteCheckErrors", OwnedValue::Map(vec![(kw("errors"), OwnedValue::Vector(items))]))
    }
}

impl crate::to_edn::WatError for ReteCheckErrors {
    fn message(&self) -> String {
        let n = self.0.len();
        format!("{} rete rule validation error{}", n, if n == 1 { "" } else { "s" })
    }
    fn location(&self) -> OwnedValue {
        OwnedValue::Nil
    }
    fn causes(&self) -> OwnedValue {
        OwnedValue::Vector(vec![])
    }
    fn variant(&self) -> OwnedValue {
        let items: Vec<OwnedValue> = self.0.iter().map(|e| e.error_edn()).collect();
        tagged("ReteCheckErrors", OwnedValue::Map(vec![(kw("errors"), OwnedValue::Vector(items))]))
    }
}

fn tagged(variant: &str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns(crate::error_ns::RETE, variant), Box::new(body))
}

fn kw(name: &str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

/// Render a clause/form for a diagnostic message — the same structural pretty-printer
/// `:wat::core::write-forms` uses (`crate::wat_edn_bridge::watast_to_edn` + `wat_edn::write`),
/// so a `#wat.rete/MalformedClause` names the offending form exactly as a wat reader would.
fn render_form(ast: &WatAST) -> String {
    wat_edn::write(&crate::wat_edn_bridge::watast_to_edn(ast))
}

// ─── The shared reorder helper (S3, design call 2) ───────────────────────────────────────────

/// `reorder_kwargs_by_field_name(field_order, kv_pairs) -> Vec<value_ast>` in declaration
/// order — ONE helper, single-sourced. The (C) spliced-construction reorder pass calls this
/// too (a separate strike; not wired here) — do NOT inline this at either call site.
///
/// `kv_pairs` need not cover every field in `field_order` (a `:then` RHS may under-supply
/// fields today — pre-existing behavior, unchanged by this pass, see `matcher.rs`'s
/// `build_insert_fact`); any field in `field_order` with no matching pair is simply absent
/// from the output. Every SUPPLIED field name, however, must be real: the first unknown
/// name is returned as `Err(field_name)` so the caller can build its own contextual error.
pub(crate) fn reorder_kwargs_by_field_name(
    field_order: &[&str],
    kv_pairs: &[(&str, WatAST)],
) -> Result<Vec<WatAST>, String> {
    for (field, _) in kv_pairs {
        if !field_order.contains(field) {
            return Err((*field).to_string());
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

/// Post-register freeze pass: walk every `defrule`'s expanded `make-rule` call reachable in
/// `residue`, validate its `:when` conditions and `:then` inserts against `types`, and
/// REWRITE `:then` kwargs to declaration order in place. Returns every finding batched
/// (like `check_program`); an empty batch is `Ok(())`.
///
/// Hook site: `src/freeze/env.rs::build_env`, immediately after `resolve_references` (step
/// 7) — the same seam the `rete_wall_probe` proves reachable, on the SAME resolved user
/// residue + fully-registered `types`.
pub(crate) fn validate_rete_rules(residue: &mut [WatAST], types: &TypeEnv) -> Result<(), ReteCheckErrors> {
    let mut errors: Vec<ReteCheckError> = Vec::new();
    walk_for_make_rule(residue, types, &mut errors);
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

/// Recursive descent for `(:wat::rete::make-rule name (quote [:when…]) (quote [:then…]))`
/// calls — mirrors `find_make_rule` in the `rete_wall_probe` (`src/freeze/env.rs`), but
/// mutable (S3 rewrites `:then` in place) and exhaustive (every rule in `forms`, not just
/// the first).
fn walk_for_make_rule(forms: &mut [WatAST], types: &TypeEnv, errors: &mut Vec<ReteCheckError>) {
    for f in forms.iter_mut() {
        if let WatAST::List(items, _) = f {
            let is_make_rule =
                matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::rete::make-rule");
            if is_make_rule {
                validate_and_reorder_rule(items, types, errors);
                continue;
            }
            walk_for_make_rule(items, types, errors);
        }
    }
}

/// `mr` = the full `make-rule` call's items: `[kw, name-lit, when-quote, then-quote]`.
fn validate_and_reorder_rule(mr: &mut [WatAST], types: &TypeEnv, errors: &mut Vec<ReteCheckError>) {
    let rule_name = match mr.get(1) {
        Some(WatAST::StringLit(s, _)) => s.clone(),
        other => other.map(render_form).unwrap_or_else(|| "<unknown-rule>".to_string()),
    };

    // :when (mr[2] = (quote [<cond>…])) — validate only, no rewrite.
    if let Some(when_conds) = quote_vector(mr.get(2)) {
        for cond in when_conds {
            validate_when_entry(cond, &rule_name, types, errors);
        }
    }

    // :then (mr[3] = (quote [<insert>…])) — validate + reorder in place.
    if let Some(WatAST::List(quote_items, _)) = mr.get_mut(3) {
        if let Some(WatAST::Vector(then_forms, _)) = quote_items.get_mut(1) {
            for insert_form in then_forms.iter_mut() {
                validate_and_reorder_then(insert_form, &rule_name, types, errors);
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
/// (`wat/rete.wat`: is-where / is-not / is-exists / is-accumulate / else-plain), via the
/// SHARED classifier so this never drifts into a second hand-rolled grammar.
fn validate_when_entry(cond: &WatAST, rule_name: &str, types: &TypeEnv, errors: &mut Vec<ReteCheckError>) {
    match classify_rete_clause(cond) {
        // Design call 3 — a `where` fence's outer shape is already confirmed by the
        // classifier (2-item, `:wat::rete::where` head); its interior expr is out of scope.
        ReteClauseShape::Where(_) => {}
        // Design call 3 / brief S2 — `not`/`exists` recurse: their sub-condition gets the
        // SAME full validation (registered type + every clause + every field-ref) as any
        // top-level condition.
        ReteClauseShape::Not(inner) | ReteClauseShape::Exists(inner) => {
            validate_plain_condition(inner, rule_name, types, errors);
        }
        // Design call 3 — accumulate's `:from` inner gets fact-type-HEAD validation only;
        // its own clauses and the acc-form's reducer body are out of scope.
        ReteClauseShape::Accumulate { from, .. } => {
            validate_fact_type_head_only(from, rule_name, types, errors);
        }
        // Every other shape (including Bind/Constraint/And/Or/Unrecognized, none of which are
        // legitimate TOP-level :when entries) falls to the plain-condition path: a top-level
        // entry that is not a wrapper must be `(:Type clause…)`.
        _ => validate_plain_condition(cond, rule_name, types, errors),
    }
}

/// Validate a plain `(:Type clause…)` condition: `Type` must be a registered aggregate, and
/// every clause must be a recognized shape whose field-refs name real fields.
fn validate_plain_condition(cond: &WatAST, rule_name: &str, types: &TypeEnv, errors: &mut Vec<ReteCheckError>) {
    let items = match cond {
        WatAST::List(items, _) if !items.is_empty() => items,
        _ => {
            errors.push(malformed(cond.span().clone(), rule_name, "", cond));
            return;
        }
    };
    let head_kw = match &items[0] {
        WatAST::Keyword(k, _) => k.clone(),
        _ => {
            errors.push(malformed(cond.span().clone(), rule_name, "", cond));
            return;
        }
    };
    let fact_type = head_kw.trim_start_matches(':').to_string();
    let field_names = match lookup_fields(types, &fact_type) {
        Some(f) => f,
        None => {
            errors.push(ReteCheckError {
                span: cond.span().clone(),
                kind: ReteCheckErrorKind::UnknownFactType { rule: rule_name.to_string(), fact_type },
            });
            return; // no schema to validate clauses against
        }
    };
    for clause in &items[1..] {
        validate_clause(clause, rule_name, &fact_type, &field_names, errors);
    }
}

/// Validate a single within-condition clause (recursing `and`/`or`/`not`), checking every
/// bind/constraint field-ref against `field_names`. The free `?var` side is never checked.
fn validate_clause(
    clause: &WatAST,
    rule_name: &str,
    fact_type: &str,
    field_names: &[String],
    errors: &mut Vec<ReteCheckError>,
) {
    match classify_rete_clause(clause) {
        ReteClauseShape::Bind { field, .. } => {
            check_field(field, clause, rule_name, fact_type, field_names, errors);
        }
        ReteClauseShape::Constraint { lhs, rhs, .. } => {
            check_operand_field_ref(lhs, clause, rule_name, fact_type, field_names, errors);
            check_operand_field_ref(rhs, clause, rule_name, fact_type, field_names, errors);
        }
        ReteClauseShape::And(subs) | ReteClauseShape::Or(subs) => {
            for sub in subs {
                validate_clause(sub, rule_name, fact_type, field_names, errors);
            }
        }
        ReteClauseShape::Not(sub) => {
            validate_clause(sub, rule_name, fact_type, field_names, errors);
        }
        // Clause-level `where` is the stone-6 STOP arm (always `None` at fire time); its
        // interior is out of scope (design call 3) — nothing further to check.
        ReteClauseShape::Where(_) => {}
        // `exists`/`accumulate` never legitimately occur as within-condition clauses (they
        // are top-level-only wrappers, consumed before a condition's clause list is built).
        ReteClauseShape::Exists(_) | ReteClauseShape::Accumulate { .. } | ReteClauseShape::Unrecognized => {
            errors.push(malformed(clause.span().clone(), rule_name, fact_type, clause));
        }
    }
}

/// A constraint operand is schema-checked ONLY when it is a `:field` reference; a `?var`
/// (free or bound) and a literal are left alone (design: "the free `?v` stays free").
fn check_operand_field_ref(
    operand: &WatAST,
    clause: &WatAST,
    rule_name: &str,
    fact_type: &str,
    field_names: &[String],
    errors: &mut Vec<ReteCheckError>,
) {
    if let WatAST::Keyword(k, _) = operand {
        let field = k.trim_start_matches(':');
        check_field_at(field, clause.span().clone(), rule_name, fact_type, field_names, errors);
    }
}

fn check_field(
    field: &str,
    clause: &WatAST,
    rule_name: &str,
    fact_type: &str,
    field_names: &[String],
    errors: &mut Vec<ReteCheckError>,
) {
    check_field_at(field, clause.span().clone(), rule_name, fact_type, field_names, errors);
}

fn check_field_at(
    field: &str,
    span: Span,
    rule_name: &str,
    fact_type: &str,
    field_names: &[String],
    errors: &mut Vec<ReteCheckError>,
) {
    if !field_names.iter().any(|f| f == field) {
        errors.push(ReteCheckError {
            span,
            kind: ReteCheckErrorKind::UnknownField {
                rule: rule_name.to_string(),
                fact_type: fact_type.to_string(),
                field: field.to_string(),
                available_fields: field_names.to_vec(),
            },
        });
    }
}

/// Design call 3 — accumulate's `:from` inner and (by extension) any bare fact-type-head-only
/// check: registered-type validation ONLY, no clause walk.
fn validate_fact_type_head_only(cond: &WatAST, rule_name: &str, types: &TypeEnv, errors: &mut Vec<ReteCheckError>) {
    let items = match cond {
        WatAST::List(items, _) if !items.is_empty() => items,
        _ => {
            errors.push(malformed(cond.span().clone(), rule_name, "", cond));
            return;
        }
    };
    let head_kw = match &items[0] {
        WatAST::Keyword(k, _) => k,
        _ => {
            errors.push(malformed(cond.span().clone(), rule_name, "", cond));
            return;
        }
    };
    let fact_type = head_kw.trim_start_matches(':').to_string();
    if lookup_fields(types, &fact_type).is_none() {
        errors.push(ReteCheckError {
            span: cond.span().clone(),
            kind: ReteCheckErrorKind::UnknownFactType { rule: rule_name.to_string(), fact_type },
        });
    }
}

/// Registry lookup — the SAME colon-prefixed key + `field_names()` accessor the runtime
/// matcher uses (`matcher.rs:126-135`, proven reachable by the `rete_wall_probe`).
fn lookup_fields(types: &TypeEnv, fact_type: &str) -> Option<Vec<String>> {
    let type_key = format!(":{fact_type}");
    match types.get(&type_key) {
        Some(TypeDef::Aggregate(a)) => Some(a.field_names().map(|s| s.to_string()).collect()),
        _ => None,
    }
}

fn malformed(span: Span, rule_name: &str, fact_type: &str, clause: &WatAST) -> ReteCheckError {
    ReteCheckError {
        span,
        kind: ReteCheckErrorKind::MalformedClause {
            rule: rule_name.to_string(),
            fact_type: fact_type.to_string(),
            clause: render_form(clause),
        },
    }
}

// ─── :then validation + reorder (S3) ─────────────────────────────────────────────────────────

/// `insert_form` = `(:wat::rete::insert (:Type arg…))`. Validates the fact-type head and,
/// for a kwargs RHS, every `:field` name — then REWRITES the fact-form's args to declaration
/// order in place (mutating `residue`, so `build_insert_fact` sees declaration order at fire
/// time). A positional RHS is checked for arity only (unchanged shape; no rewrite needed).
/// A `:then` value-position operand that can NEVER resolve at fire time — whatever the bindings.
///
/// Mirrors `resolve_operand`'s accepted set (`matcher.rs`) exactly, minus the `?var` case whose
/// boundness this stone does not judge: a literal resolves, a `?var` MAY resolve, and everything
/// else — a nested form, a `:field` keyword (a RHS has no current fact), a bare non-`?` symbol —
/// resolves to `None` for every possible token. Purely syntactic, so it cannot reject a legal rule.
fn rhs_operand_can_never_resolve(arg: &WatAST) -> bool {
    !matches!(
        arg,
        WatAST::IntLit(_, _)
            | WatAST::FloatLit(_, _)
            | WatAST::BoolLit(_, _)
            | WatAST::StringLit(_, _)
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

fn validate_and_reorder_then(
    insert_form: &mut WatAST,
    rule_name: &str,
    types: &TypeEnv,
    errors: &mut Vec<ReteCheckError>,
) {
    let outer_span = insert_form.span().clone();
    let items = match insert_form {
        WatAST::List(items, _) if items.len() == 2 => items,
        WatAST::List(_, _) => {
            errors.push(malformed(outer_span, rule_name, "", insert_form));
            return;
        }
        other => {
            errors.push(malformed(outer_span, rule_name, "", other));
            return;
        }
    };
    let is_insert_head = matches!(&items[0], WatAST::Keyword(k, _) if k.as_str() == ":wat::rete::insert");
    if !is_insert_head {
        let form_copy = insert_form.clone();
        errors.push(malformed(outer_span, rule_name, "", &form_copy));
        return;
    }

    let fact_span = items[1].span().clone();
    let fact_items = match &mut items[1] {
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
        None => {
            errors.push(ReteCheckError {
                span: fact_span,
                kind: ReteCheckErrorKind::UnknownFactType { rule: rule_name.to_string(), fact_type },
            });
            return;
        }
    };

    // Arc 294 item 9a — the SAME kwargs-shape test `build_insert_fact` uses
    // (`matcher.rs:454-456`): even arity, ≥2 args, a keyword at every even index.
    let args = &fact_items[1..];
    let is_kwargs = args.len() >= 2
        && args.len() % 2 == 0
        && args.iter().step_by(2).all(|a| matches!(a, WatAST::Keyword(_, _)));

    if is_kwargs {
        let mut kv_pairs: Vec<(String, WatAST)> = Vec::with_capacity(args.len() / 2);
        for pair in args.chunks(2) {
            let field = match &pair[0] {
                WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
                _ => unreachable!("is_kwargs confirmed a Keyword at every even index"),
            };
            kv_pairs.push((field, pair[1].clone()));
        }
        let mut all_known = true;
        for (field, _) in &kv_pairs {
            if !field_names.iter().any(|f| f == field) {
                errors.push(ReteCheckError {
                    span: fact_span.clone(),
                    kind: ReteCheckErrorKind::UnknownField {
                        rule: rule_name.to_string(),
                        fact_type: fact_type.clone(),
                        field: field.clone(),
                        available_fields: field_names.clone(),
                    },
                });
                all_known = false;
            }
        }
        if !all_known {
            return; // do not rewrite a form already flagged invalid
        }
        // The wall, kwargs side. Checked BEFORE the reorder rewrites `fact_items` in place, so
        // the operand reported is the one the author wrote, at the span they wrote it at.
        let kwargs_values: Vec<WatAST> = kv_pairs.iter().map(|(_, v)| v.clone()).collect();
        check_rhs_operands(&kwargs_values, rule_name, &fact_type, errors);

        let field_order: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();
        let kv_ref: Vec<(&str, WatAST)> = kv_pairs.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        match reorder_kwargs_by_field_name(&field_order, &kv_ref) {
            Ok(reordered) => {
                fact_items.truncate(1); // keep the type keyword; replace the args
                fact_items.extend(reordered);
            }
            Err(bad_field) => {
                // Unreachable given the all_known check above; stay honest if it ever isn't.
                errors.push(ReteCheckError {
                    span: fact_span,
                    kind: ReteCheckErrorKind::UnknownField {
                        rule: rule_name.to_string(),
                        fact_type,
                        field: bad_field,
                        available_fields: field_names,
                    },
                });
            }
        }
    } else {
        // The wall, positional side. Independent of the arity verdict below: a rule can be both
        // wrong-arity AND carry an unresolvable operand, and batching every finding is this
        // validator's whole contract (`validate_rete_rules` returns them all, not the first).
        check_rhs_operands(args, rule_name, &fact_type, errors);

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
  (:wat::rete::insert (:alert::Unattended :location ?loc)))
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
  (:wat::rete::insert (:alert::Unattended :location ?loc)))
"#;
        let forms = crate::parse_all!(src).expect("parse");
        // build_env hooks validate_rete_rules internally (step 7.8) — a well-formed rule
        // must not turn a clean build_env into an error.
        build_env(forms).expect("a well-formed rule freezes clean");
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
  (:wat::rete::insert (:alert::Unattended :location ?loc)))
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
  (:wat::rete::insert (:alert2::Cold :celsius ?c :location ?loc)))
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
        let insert_form = &then_vec[0];
        let insert_items = match insert_form {
            WatAST::List(items, _) => items,
            other => panic!("insert form is a List; got {other:?}"),
        };
        let fact_items = match &insert_items[1] {
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
        let out = reorder_kwargs_by_field_name(&order, &pairs).expect("both fields known");
        assert_eq!(out, vec![a, b]);
    }

    #[test]
    fn reorder_helper_rejects_unknown_field_name() {
        use crate::scope::Identifier;
        let v = WatAST::Symbol(Identifier::bare("v"), crate::rust_caller_span!());
        let pairs = vec![("nope", v)];
        let order = ["x", "y"];
        let err = reorder_kwargs_by_field_name(&order, &pairs).expect_err("unknown field must error");
        assert_eq!(err, "nope");
    }
}
