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

use std::fmt;

use wat_edn::{Keyword, OwnedValue, Tag};

use crate::ast::WatAST;
use crate::rete::clause::{classify_constraint_head, classify_rete_clause, ConstraintSpelling, ReteClauseShape};
use crate::span::Span;
use crate::types::{EnumVariant, TypeDef, TypeEnv};

// ─── Error types (Pattern A: span at the outer struct, kind carries variant data) ────────────

/// Variant data for [`ReteCheckError`]. Namespace `wat.rete` — see `src/error_ns.rs`.
#[derive(Debug, Clone, wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::RETE)]
pub enum ReteCheckErrorKind {
    /// A `:when` condition's fact-type head, or a `:then` fact-form's fact-type head, or an
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
    /// A positional `:then` fact-form's argument count does not match the fact type's declared
    /// field count. Arc 278 BRIEF-construction-total-three-walls.md #1/#3 — also reused for a
    /// NESTED constructor operand's own arity, closing the two counterparts the audit measured
    /// once #1 wired a nested surface constructor to actually evaluate: a single positional-value
    /// aggregate operand (`(:usr::Inner 1)`, mirroring `eval_kwargs_construct`'s `rest.len() <= 1`
    /// passthrough) and a bare `:Enum::Variant` call (`lookup_fields` only ever resolved
    /// `TypeDef::Aggregate`, so an enum-variant head was invisible to freeze-time validation
    /// entirely before this — `fact_type` carries the FULL `Enum::Variant` path for that case).
    RhsArityMismatch {
        rule: String,
        fact_type: String,
        expected: usize,
        got: usize,
    },
    /// A `:then` fact-form's VALUE-position operand can never resolve at fire time, whatever the
    /// bindings: a nested form, a `:field` keyword (a RHS has no current fact to read a field
    /// from), or a bare non-`?` symbol. `resolve_operand` returns `None` for all three and
    /// `build_insert_fact` then raises — but it raised *per derived fact, mid-fire*, for a
    /// property of the RULE that no fact can change.
    ///
    /// This wall is arc 278's third statement of one ruling: a negation cycle is a compile error
    /// (R18 `NEGATIO COMPLETVM POSCIT` — "the ill-defined program given no form"), a lying
    /// `extend-type` is a compile error (R28 `SOLVIMVS NE MENTIRETVR`), and `()` stopped being a
    /// value (arc 179) because a second spelling walks around every wall built on the first.
    /// `validate_then_form` already checked the fact-form's SHAPE — fact type, field
    /// names, positional arity — and stopped short of looking *inside* an argument, which is why
    /// `(:Out (:wat::core::+ ?a 1))` passed `--check` with arity 1 == 1 field and then exploded
    /// mid-fire.
    ///
    /// SCOPE, deliberately narrow: only operands that can NEVER resolve. An unbound `?var` is
    /// equally a compile-time property but needs binder analysis over `:when` (an `Or` arm binds
    /// conditionally, `exists` binds nothing outward, `accumulate` binds its result var) — and
    /// under-collecting that set would reject LEGAL rules, which is the one failure a wall must
    /// not have. This variant carries no binder claim.
    RhsUnresolvableOperand {
        rule: String,
        fact_type: String,
        /// The offending operand, rendered as wat source (`render_form`) — never Rust `Debug`.
        operand: String,
        /// What a RHS operand may be, so the error teaches rather than merely refusing (R29
        /// `RVINA ERVDIT`), mirroring `UnknownField`'s `available_fields`.
        accepted: Vec<String>,
    },
    /// Arc 278 BRIEF-construction-total-three-walls.md #2 — a kwargs `:then` RHS under-supplies
    /// `fact_type`'s declared fields. `reorder_kwargs_by_field_name`'s own doc used to call this
    /// "pre-existing behavior, unchanged" (a supplied-fewer-than-all kwargs RHS silently built a
    /// short record); STOP-A (the audit that grounded this wall) found no `:then` in the corpus
    /// that actually relies on it — the doc line was describing an accident nobody depended on.
    /// `build_insert_fact`'s kwargs fast path (`eval_insert.rs`) has no independent arity check, so
    /// the malformed record used to construct silently and raise only when something later read
    /// the missing field by name (`Record/field-at`, an index-out-of-bounds `TypeMismatch`) —
    /// this wall names the RULE, the TYPE, and the missing fields by NAME, at freeze, instead.
    RhsMissingFields {
        rule: String,
        fact_type: String,
        missing: Vec<String>,
    },
    /// Arc 278 BRIEF-construction-total-three-walls.md #1 — a NESTED surface aggregate-
    /// constructor operand (an operand's VALUE, not a `:then` item's own top-level head) written
    /// with MORE THAN ONE positional argument. Once #1 wires a nested constructor to actually
    /// reach `:wat::core::kwargs-construct`'s dispatch (`eval_kwargs_construct`, runtime.rs), that
    /// dispatch unconditionally retires multi-arg RAW POSITIONAL construction at a bare aggregate
    /// name (kwargs, or a single positional value, are the only two supported nested shapes) —
    /// regardless of whether `got` happens to equal the type's field count, so this is NOT an
    /// arity mismatch (a correct count would still be refused); walled here with its own message
    /// rather than borrowing `RhsArityMismatch`'s "expected N" framing, which would misstate it.
    RhsPositionalConstructionRetired {
        rule: String,
        fact_type: String,
        got: usize,
    },
    /// LAW A at the inline alpha constraint (`DESIGN-STONE-inline-constraint-admits-non-rete.md`).
    /// A constraint clause inside a fact pattern is spelled with the GENERIC core comparator
    /// (`:wat::core::>`), which is not a rete primitive — and, for the four orderings, is PARTIAL:
    /// it routes through `compare_values`, which errors on incomparable operands.
    ///
    /// This is its own variant rather than `MalformedClause` deliberately: the clause is
    /// well-formed, it is NON-RETE, and saying "malformed" would teach the wrong fix (R29
    /// `RVINA ERVDIT` — the ruin IS the lesson, so it must name the right one). `twin` carries the
    /// per-type spelling to use, which is the whole remedy.
    NonReteConstraint {
        rule: String,
        fact_type: String,
        head: String,
        twin: String,
    },
    /// A per-type rete constraint applied to a field of a DIFFERENT declared type —
    /// `(:wat::rete::core::i64::> :location 10)` where `:location` is `String`.
    ///
    /// This is the payoff of forcing the per-type spelling: monomorphising does not merely make
    /// the comparison faster, it moves the incomparable-operands case from a runtime question to
    /// a compile error. Under the generic spelling this clause was admitted and its runtime
    /// semantics were never proven; now it cannot be written.
    ConstraintTypeMismatch {
        rule: String,
        fact_type: String,
        head: String,
        field: String,
        /// The type the rete row is monomorphic at (`i64`, `string`, …).
        op_type: String,
        /// The field's declared type, rendered as wat source.
        field_type: String,
    },
    /// An inline constraint on an operand whose declared type has NO rete comparator at all — a
    /// record-valued field, a collection, an opaque. The rete equality surface is six modules
    /// (`i64 f64 string bool keyword enum`); there is no `record::=`.
    ///
    /// Two records ARE comparable at runtime (`values_equal` would do it) — this is not a runtime
    /// limitation but the CLOSED SET saying the form is not expressible. Its own variant rather
    /// than `ConstraintTypeMismatch`, because "use the comparator for X" is not the fix: there
    /// isn't one, and suggesting otherwise would teach a form that cannot be written.
    ConstraintTypeNotComparable {
        rule: String,
        fact_type: String,
        head: String,
        operand: String,
        field_type: String,
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
            ReteCheckErrorKind::RhsMissingFields { rule, fact_type, missing } => write!(
                f,
                "defrule `{rule}`: `:then` insert of `:{fact_type}` is missing required field(s): [{}]",
                missing.join(", ")
            ),
            ReteCheckErrorKind::RhsPositionalConstructionRetired { rule, fact_type, got } => write!(
                f,
                "defrule `{rule}`: `:then` insert nests a raw positional construction of `:{fact_type}` \
                 with {got} argument(s) — positional construction at a bare aggregate name is retired; \
                 use kwargs (`:field val …`) or a single positional value"
            ),
            ReteCheckErrorKind::NonReteConstraint { rule, fact_type, head, twin } => write!(
                f,
                "defrule `{rule}` (`:{fact_type}`): `{head}` is not a rete primitive — a rule condition \
                 admits only :wat::rete:: ops. Use the per-type spelling, e.g. `{twin}`: the rete \
                 surface is per-type so the comparison is TOTAL (the generic form has no answer for \
                 operands that are not comparable)"
            ),
            ReteCheckErrorKind::ConstraintTypeNotComparable {
                rule, fact_type, head, operand, field_type,
            } => write!(
                f,
                "defrule `{rule}` (`:{fact_type}`): `{head}` compares operand `{operand}`, declared \
                 `{field_type}`, for which rete has NO comparator — the rete equality surface is \
                 i64/f64/string/bool/keyword/enum. Compare a scalar FIELD of it instead"
            ),
            ReteCheckErrorKind::ConstraintTypeMismatch {
                rule, fact_type, head, field, op_type, field_type,
            } => write!(
                f,
                "defrule `{rule}` (`:{fact_type}`): `{head}` compares at `{op_type}`, but field \
                 `:{field}` is declared `{field_type}` — use the rete comparator for `{field_type}`"
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

/// Unknown field at kwargs reorder. Span is required at construction — a spanless
/// unknown-field is uncompilable (conformare Pattern A).
#[derive(Debug)]
pub(crate) struct KwargsReorderError {
    pub span: Span,
    pub field: String,
}

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

    // :when (mr[2] = (quote [<cond>…])) — validate only, no rewrite.
    if let Some(when_conds) = quote_vector(mr.get(2)) {
        // ★ Binds collected across EVERY condition of the rule, before any is validated. A join
        // variable is bound in one pattern and compared in another, so a per-pattern map would
        // leave it unresolvable — and "unresolvable" was quietly meaning "skip the check". It is
        // knowable; it just is not knowable from one pattern.
        let binds = collect_rule_bind_types(when_conds, types);
        for cond in when_conds {
            validate_when_entry(cond, &rule_name, types, &binds, errors);
        }
    }

    // :then (mr[3] = (quote [<fact-form>…])) — validate, then reorder kwargs. Arc 278 Stone A:
    // each member is a bare fact-form, no more `insert` wrapper.
    if let Some(WatAST::List(quote_items, _)) = mr.get_mut(3) {
        if let Some(WatAST::Vector(then_forms, _)) = quote_items.get_mut(1) {
            for fact_form in then_forms.iter_mut() {
                validate_then_form(fact_form, &rule_name, types, errors);
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

fn validate_clause(
    clause: &WatAST,
    ctx: &ClauseCtx<'_>,
    errors: &mut Vec<ReteCheckError>,
) {
    // Only what THIS function reads. The other three travel onward inside `ctx` to
    // `check_constraint_head` — destructuring them here just to not use them is what produced
    // three `unused_variable` warnings, and the fix is to take less, not to `_`-prefix them
    // (that door is task #67: `_` silences the very gate that would have caught the mistake).
    let ClauseCtx { rule_name, fact_type, field_names, .. } = *ctx;
    match classify_rete_clause(clause) {
        ReteClauseShape::Bind { field, .. } => {
            check_field(field, clause, rule_name, fact_type, field_names, errors);
        }
        ReteClauseShape::Constraint { op, lhs, rhs } => {
            check_operand_field_ref(lhs, clause, rule_name, fact_type, field_names, errors);
            check_operand_field_ref(rhs, clause, rule_name, fact_type, field_names, errors);
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
    let fact_type = match crate::rete::matcher::alpha_pattern(cond) {
        Some(p) => p.type_head.to_string(),
        None => {
            errors.push(malformed(cond.span().clone(), rule_name, "", cond));
            return;
        }
    };
    if lookup_fields(types, &fact_type).is_none() {
        errors.push(ReteCheckError {
            span: cond.span().clone(),
            kind: ReteCheckErrorKind::UnknownFactType { rule: rule_name.to_string(), fact_type },
        });
    }
}

/// Registry lookup — the SAME colon-prefixed key + `field_names()` accessor the runtime
/// matcher uses (proven reachable by the `rete_wall_probe`).
type FieldList = Vec<String>;

fn lookup_fields(types: &TypeEnv, fact_type: &str) -> Option<FieldList> {
    let type_key = format!(":{fact_type}");
    match types.get(&type_key) {
        Some(TypeDef::Aggregate(a)) => Some(a.field_names().map(|s| s.to_string()).collect()),
        _ => None,
    }
}

/// Sibling of `lookup_fields`, same key and same registry — the DECLARED TYPE of each field, in
/// declaration order, so a per-type rete constraint can be checked against the field it reads.
/// Kept beside its twin rather than folded into it: every existing caller wants only the names,
/// and widening the shared return would make them all pay for a walk they do not use.
fn lookup_field_types(types: &TypeEnv, fact_type: &str) -> Option<FieldList> {
    let type_key = format!(":{fact_type}");
    match types.get(&type_key) {
        Some(TypeDef::Aggregate(a)) => {
            // `check::format_type` is the substrate's ONE renderer for a TypeExpr — not a second
            // hand-rolled Display, so the message reads exactly as the checker's do.
            Some(a.field_types().map(crate::check::format_type).collect())
        }
        _ => None,
    }
}

/// The rete `ty` segment a declared field type is comparable at — `:wat::core::i64` -> `i64`.
///
/// The rete equality surface is SIX modules (verified against `RETE_OPS`): the five primitives
/// below plus `enum`, which needs the registry to recognise — an enum field's declared type is a
/// user path like `:my::Level`, not a fixed name. `:wat::rete::core::enum::=` exists and its own
/// note records why it works: head-substitution reaches core `=`, whose `values_equal` already
/// compares enum values.
///
/// ⚠ `None` is NOT "fine, skip it" — see `check_constraint_head`'s use. It means the substrate has
/// **no rete comparator for that type at all** (a record-valued field, a collection, an opaque),
/// so an inline constraint on it is not expressible. Two records ARE comparable at runtime
/// (`values_equal` would do it) but there is no `record::=` row, and minting one is its own ruling
/// — not something to smuggle in by making this function lenient.
///
/// A first draft of this file handled only the five primitives and let `None` mean "skip the
/// check", which silently admitted an enum-typed constraint — the exact vacuous-arm shape this arc
/// keeps pulling out. Caught by the builder: *"records may hold other records… and enums… and
/// whatever else we can express in rete's closed syntax."*
fn rete_type_segment_of(field_type: &str, types: &TypeEnv) -> Option<&'static str> {
    match field_type.trim_start_matches(':') {
        "wat::core::i64" => Some("i64"),
        "wat::core::f64" => Some("f64"),
        "wat::core::String" => Some("string"),
        "wat::core::bool" => Some("bool"),
        "wat::core::Keyword" => Some("keyword"),
        // An enum is named by a user path; the registry is the only way to know.
        other => match types.get(&format!(":{other}")) {
            Some(TypeDef::Enum(_)) => Some("enum"),
            _ => None,
        },
    }
}

/// LAW A + the per-type type check for an inline alpha constraint
/// (`DESIGN-STONE-inline-constraint-admits-non-rete.md`).
///
/// The grammar deliberately still ACCEPTS the generic core spelling so this function can name it
/// and point at its per-type twin — R29 `RVINA ERVDIT`: refusing it as `MalformedClause` would
/// teach the wrong fix, because the clause is well-formed, it is NON-RETE.
///
/// ★ NOTHING HERE IS GUESSED. An operand's type is always DERIVABLE, and a first draft of this
/// function defaulted to `i64` whenever it saw a `?var` — which was not a limitation of the
/// information available, it was the function not looking it up. The builder's cut: *"why is any
/// of this a guess? we know the type's value from the record def."* Correct — three exhaustive
/// sources, in order, and no fallback after them:
///   1. a `:field` operand   -> the field's DECLARED type
///   2. a `?var` operand     -> the field its `(?v <- :field)` bind names, then that field's type
///   3. a LITERAL operand    -> the literal's own type
///
/// If none resolves (a `?var` bound nowhere in the rule), the type is genuinely not knowable — and
/// then this reports the law-A violation WITHOUT a per-type suggestion rather than inventing one.
/// A wrong suggestion teaches a wrong fix.
fn check_constraint_head(
    op: &str,
    lhs: &WatAST,
    rhs: &WatAST,
    clause: &WatAST,
    ctx: &ClauseCtx<'_>,
    errors: &mut Vec<ReteCheckError>,
) {
    let ClauseCtx { rule_name, fact_type, field_names, field_types, binds, types } = *ctx;
    let Some((_, spelling)) = classify_constraint_head(op) else { return };

    let resolved: Vec<(&WatAST, OperandType)> = [lhs, rhs]
        .into_iter()
        .map(|o| (o, resolve_operand_type(o, field_names, field_types, binds, types)))
        .collect();

    // A type rete cannot compare is an ERROR on BOTH spellings — the form is not expressible.
    let mut not_comparable = false;
    for (operand, ty) in &resolved {
        if let OperandType::NotComparable(declared) = ty {
            not_comparable = true;
            errors.push(ReteCheckError {
                span: clause.span().clone(),
                kind: ReteCheckErrorKind::ConstraintTypeNotComparable {
                    rule: rule_name.to_string(),
                    fact_type: fact_type.to_string(),
                    head: op.to_string(),
                    operand: describe_operand(operand),
                    field_type: declared.clone(),
                },
            });
        }
    }
    if not_comparable {
        return;
    }

    match spelling {
        ConstraintSpelling::CoreGeneric => {
            let suffix = op.rsplit("::").next().unwrap_or(op);
            let twin = match resolved.iter().find_map(|(_, t)| match t {
                OperandType::Resolved(ty) => Some(*ty),
                _ => None,
            }) {
                Some(ty) => format!(":wat::rete::core::{ty}::{suffix}"),
                // Every operand is an unbound `?var`. Do NOT name a type the operands do not
                // justify — a wrong suggestion teaches a wrong fix.
                None => format!(":wat::rete::core::<type-of-the-operands>::{suffix}"),
            };
            errors.push(ReteCheckError {
                span: clause.span().clone(),
                kind: ReteCheckErrorKind::NonReteConstraint {
                    rule: rule_name.to_string(),
                    fact_type: fact_type.to_string(),
                    head: op.to_string(),
                    twin,
                },
            });
        }
        ConstraintSpelling::Rete { ty: op_type } => {
            for (operand, ty) in &resolved {
                match ty {
                    OperandType::Resolved(actual) if *actual != op_type => {
                        errors.push(ReteCheckError {
                            span: clause.span().clone(),
                            kind: ReteCheckErrorKind::ConstraintTypeMismatch {
                                rule: rule_name.to_string(),
                                fact_type: fact_type.to_string(),
                                head: op.to_string(),
                                field: describe_operand(operand),
                                op_type: op_type.to_string(),
                                field_type: (*actual).to_string(),
                            },
                        });
                    }
                    // Agrees — nothing to report.
                    OperandType::Resolved(_) => {}
                    // Reported above and returned before reaching here.
                    OperandType::NotComparable(_) => {}
                    // Explicitly out of scope (see the variant's doc), NOT a silent pass.
                    OperandType::UnboundInThisRule => {}
                }
            }
        }
    }
}

/// Collect every `(?v <- :field)` bind in the WHOLE rule, resolved to the field's declared type.
///
/// Rule-wide, not per-pattern, because a join variable is bound in one condition and compared in
/// another. `not`/`exists` wrappers are unwrapped so their inner pattern's binds count too.
fn collect_rule_bind_types(
    when_conds: &[WatAST],
    types: &TypeEnv,
) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for cond in when_conds {
        // Unwrap the wrappers that carry a plain pattern inside.
        match classify_rete_clause(cond) {
            ReteClauseShape::Or(arms) | ReteClauseShape::And(arms) => {
                let nested = collect_rule_bind_types(arms, types);
                out.extend(nested);
            }
            ReteClauseShape::Not(inner) | ReteClauseShape::Exists(inner) => {
                let nested = collect_rule_bind_types(std::slice::from_ref(inner), types);
                out.extend(nested);
            }
            ReteClauseShape::Accumulate { from, .. } => {
                let nested = collect_rule_bind_types(std::slice::from_ref(from), types);
                out.extend(nested);
            }
            _other => {
                let Some(pat) = crate::rete::matcher::alpha_pattern(cond) else { continue };
                if let Some(var) = pat.fact_var {
                    out.insert(var.to_string(), format!(":{}", pat.type_head));
                }
                let (Some(names), Some(tys)) =
                    (lookup_fields(types, pat.type_head), lookup_field_types(types, pat.type_head))
                else {
                    continue;
                };
                for clause in pat.clauses {
                    if let ReteClauseShape::Bind { var, field } = classify_rete_clause(clause) {
                        if let Some(idx) = names.iter().position(|f| f == field) {
                            if let Some(t) = tys.get(idx) {
                                out.insert(var.to_string(), t.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// What an operand's type resolution actually yielded. THREE outcomes, all explicit — there is no
/// `Option` here on purpose.
///
/// The first draft returned `Option<&str>` and `None` meant "skip the check". The builder's cut:
/// *"'none means skip' feels like a catastrophic bug?"* — it is, and it is this arc's most-repeated
/// class: an arm that discards while looking like diligence. `None` was silently covering two
/// completely different situations (a type rete genuinely cannot compare, and a variable we simply
/// had not bothered to resolve), and passing both.
enum OperandType {
    /// Resolved to a rete module segment — checkable.
    Resolved(&'static str),
    /// A real declared type for which rete has NO comparator (a record, a collection, an opaque).
    /// An ERROR, never a pass.
    NotComparable(String),
    /// A `?var` bound NOWHERE in this rule. Not a type question — an unbound-variable question,
    /// which needs the binder analysis `RhsUnresolvableOperand`'s doc already scopes out (an `Or`
    /// arm binds conditionally, `exists` binds nothing outward). Named so it is visibly out of
    /// scope rather than indistinguishable from a pass.
    UnboundInThisRule,
}

/// An operand's type — field ref, then bound `?var` (rule-wide), then literal.
fn resolve_operand_type(
    operand: &WatAST,
    field_names: &[String],
    field_types: &[String],
    binds: &std::collections::HashMap<String, String>,
    types: &TypeEnv,
) -> OperandType {
    let declared: String = match operand {
        // 1. `:field` — the declared type.
        WatAST::Keyword(k, _) => {
            let field = k.trim_start_matches(':');
            match field_names.iter().position(|f| f == field).and_then(|i| field_types.get(i)) {
                Some(t) => t.clone(),
                // An unknown field is already reported by `check_operand_field_ref`; do not
                // double-report it here as a type problem.
                None => return OperandType::UnboundInThisRule,
            }
        }
        // 2. `?var` — the field its bind names, anywhere in the rule.
        WatAST::Symbol(sym, _) if sym.as_str().starts_with('?') => {
            match binds.get(sym.as_str()) {
                Some(t) => t.clone(),
                None => return OperandType::UnboundInThisRule,
            }
        }
        // 3. a literal — its own type, known outright.
        WatAST::IntLit(..) => return OperandType::Resolved("i64"),
        WatAST::FloatLit(..) => return OperandType::Resolved("f64"),
        WatAST::StringLit(..) => return OperandType::Resolved("string"),
        WatAST::BoolLit(..) => return OperandType::Resolved("bool"),
        _ => return OperandType::UnboundInThisRule,
    };
    match rete_type_segment_of(&declared, types) {
        Some(seg) => OperandType::Resolved(seg),
        None => OperandType::NotComparable(declared),
    }
}

/// How to name an operand in a diagnostic: a field by its name, anything else by its source form.
fn describe_operand(operand: &WatAST) -> String {
    match operand {
        WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
        WatAST::Symbol(s, _) => s.as_str().to_string(),
        other => format!("{other:?}"),
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
///   - a bare aggregate-type keyword (`:usr::Inner`) — validated with the SAME kwargs-coverage /
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
fn walk_nested_constructors(
    operand: &WatAST,
    rule_name: &str,
    types: &TypeEnv,
    errors: &mut Vec<ReteCheckError>,
) {
    let WatAST::List(items, span) = operand else { return };
    if items.is_empty() {
        return;
    }
    if let WatAST::Keyword(head, _) = &items[0] {
        let args = &items[1..];
        // Bare aggregate-type constructor head.
        if let Some(TypeDef::Aggregate(_)) = types.get(head) {
            let nested_type = head.trim_start_matches(':').to_string();
            let field_names = lookup_fields(types, &nested_type).unwrap_or_default();
            let is_kwargs = crate::rete::eval_insert::rete_is_kwargs(args);
            if is_kwargs {
                let mut supplied: Vec<String> = Vec::with_capacity(args.len() / 2);
                for pair in args.chunks(2) {
                    let field = match &pair[0] {
                        WatAST::Keyword(k, _) => k.trim_start_matches(':').to_string(),
                        _ => unreachable!("is_kwargs confirmed a Keyword at every even index"),
                    };
                    if !field_names.iter().any(|f| f == &field) {
                        errors.push(ReteCheckError {
                            span: span.clone(),
                            kind: ReteCheckErrorKind::UnknownField {
                                rule: rule_name.to_string(),
                                fact_type: nested_type.clone(),
                                field: field.clone(),
                                available_fields: field_names.clone(),
                            },
                        });
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
                walk_nested_constructors(arg, rule_name, types, errors);
            }
            return;
        }
        // Bare enum-variant constructor head (`{EnumPath}::{Variant}`) — mirrors
        // `constructor_meta`'s own resolution (`purity.rs`).
        if let Some((enum_path, variant)) = head.rsplit_once("::") {
            if let Some(TypeDef::Enum(e)) = types.get(enum_path) {
                let expected = e.variants.iter().find_map(|v| match v {
                    EnumVariant::Unit(n) if n == variant => Some(0usize),
                    EnumVariant::Tagged { name, fields } if name == variant => Some(fields.len()),
                    _ => None,
                });
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
                        walk_nested_constructors(arg, rule_name, types, errors);
                    }
                    return;
                }
            }
        }
    }
    // Not a recognized constructor head — recurse into every item anyway (a plain call's
    // arguments, e.g. `(:wat::core::+ (:usr::Inner 1) ?a)`, may still nest a constructor deeper).
    for item in items {
        walk_nested_constructors(item, rule_name, types, errors);
    }
}

/// Validates a `:then` fact-form: fact-type head and, for kwargs, every `:field` name.
/// `reorder_then_kwargs` then rewrites kwargs args to declaration order in place.
fn validate_then_form(
    fact_form: &mut WatAST,
    rule_name: &str,
    types: &TypeEnv,
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
        // whole static validate.rs call tree is the param cascade `BRIEF-then-user-forms.md`'s
        // STOP-1 forbids). The wat-side fence (`wat/rete/compile.wat`'s `then-item-fence`, wired into
        // `compile-rule`) takes over enforcing head-legality, the three axes, and
        // "returns-a-fact" for this item — at rule-COMPILE time, same as `where`'s fence. A
        // genuinely unknown/malformed head still surfaces there, just not from this function.
        None => return,
    };

    // Arc 294 item 9a — the SAME kwargs-shape test `build_insert_fact` uses:
    // even arity, ≥2 args, a keyword at every even index.
    let args = &fact_items[1..];
    let is_kwargs = crate::rete::eval_insert::rete_is_kwargs(args);

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
        // Arc 278 #1/#3 — recurse for a NESTED constructor operand (e.g. `:inner (:usr::Inner
        // :x 1)`); the top-level shape above only covers THIS item's own head.
        for v in &kwargs_values {
            walk_nested_constructors(v, rule_name, types, errors);
        }

        reorder_then_kwargs(fact_items, &field_names, &kv_pairs, &fact_span, rule_name, &fact_type, errors);
    } else {
        // The wall, positional side. Independent of the arity verdict below: a rule can be both
        // wrong-arity AND carry an unresolvable operand, and batching every finding is this
        // validator's whole contract (`validate_rete_rules` returns them all, not the first).
        check_rhs_operands(args, rule_name, &fact_type, errors);
        // Arc 278 #1/#3 — recurse for a NESTED constructor operand, same as the kwargs branch.
        for a in args {
            walk_nested_constructors(a, rule_name, types, errors);
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

fn reorder_then_kwargs(
    fact_items: &mut Vec<WatAST>,
    field_names: &[String],
    kv_pairs: &[(String, WatAST)],
    fact_span: &crate::span::Span,
    rule_name: &str,
    fact_type: &str,
    errors: &mut Vec<ReteCheckError>,
) {
    let field_order: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();
    let kv_ref: Vec<(&str, WatAST)> = kv_pairs.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    match reorder_kwargs_by_field_name(&field_order, &kv_ref, fact_span) {
        Ok(reordered) => {
            fact_items.truncate(1);
            fact_items.extend(reordered);
        }
        Err(bad) => {
            errors.push(ReteCheckError {
                span: bad.span,
                kind: ReteCheckErrorKind::UnknownField {
                    rule: rule_name.to_string(),
                    fact_type: fact_type.to_string(),
                    field: bad.field,
                    available_fields: field_names.to_vec(),
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
}
