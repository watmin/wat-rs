//! The rete wall's error surface — every `#wat.rete/*` a `defrule` can be refused with.
//!
//! ⛔ SPLIT OUT 2026-08-30 (`partire`'s second named cut). `validate.rs` was 2_452 lines with
//! THREE concerns and no seam drawn anywhere; this is the one with no inbound dependency at all.
//! Every other part of the wall names these types; this file names nothing back, which is what
//! makes it the cut to take first.
//!
//! It moves for ONE reason: a new way for a rule to be wrong. The validators move when the rules
//! they walk change shape. Those are different days.

use std::fmt;

use wat_edn::{Keyword, OwnedValue, Tag};

use crate::ast::WatAST;
use crate::span::Span;

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
    /// conditionally, `accumulate` binds its result var) — and under-collecting that set would
    /// reject LEGAL rules, which is the one failure a wall must not have. This variant carries no
    /// binder claim.
    ///
    /// ⚠ CORRECTED 2026-08-26. This list used to include "`exists` binds nothing outward". **That
    /// is false in wat, and it was nearly acted on.** `wat-scripts/perf/grid/leading-exists.wat`
    /// is an accuracy axis built entirely on the opposite: its query is a LEADING `:exists`
    /// binding `?loc`, and the axis reads that binding back out of the query rows
    /// (`PersistentMap/get p "?loc"`) to produce `:derived`. So a bind under `:exists` not only
    /// escapes, its consumer can be HOST CODE addressing the row by string key — which no
    /// syntactic check can see. `:not` is the opposite and is proven so: reference a `:not`-bound
    /// variable outside and fire reports `unbound symbol`. The asymmetry is real, it is not
    /// symmetric with Clara (which traps both), and it is why the wrapper wall below is scoped to
    /// `:not` alone.
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
    /// A `(?v <- :field)` bind inside a `:not` whose variable is consumed NOWHERE — not by a
    /// constraint inside the negation, not anywhere else in the rule.
    ///
    /// This is the rete twin of `TypeErrorKind::UnconsumedTypeParam`
    /// (`DESIGN-STONE-a-param-spec-must-be-consumed`, arc 109) and it is refused for the SAME
    /// reason, which is readability rather than soundness: an unused bind changes no answer, but
    /// a reader cannot tell a deliberate one from a leftover edit unless every bind is written
    /// into the shape somewhere. Declared must be consumed.
    ///
    /// SCOPE — `:not` ONLY, and the exclusions are each proven rather than assumed:
    /// - **`:exists` is NOT covered**, though Clara traps both. In wat `:exists` binds OUTWARD,
    ///   and its consumer may be host code reading the query row by string key — see the
    ///   correction on `RhsUnresolvableOperand`. A wall over `:exists` would reject
    ///   `leading-exists.wat`, a live accuracy axis.
    /// - **An ordinary join bind is not covered.** Its consumer may legitimately be another
    ///   condition, another rule's `:then`, or host code; judging those needs the binder analysis
    ///   `RhsUnresolvableOperand` deliberately refused.
    ///
    /// A `:not` is the one wrapper where the question is answerable without binder analysis,
    /// because it binds nothing outward BY CONSTRUCTION: it admits a token precisely when no fact
    /// matched, so there is no value to carry. And this check still never asks whether a variable
    /// is bound — only whether every DECLARATION of it sits inside one negation. A variable
    /// declared anywhere else is a correlation and is untouched.
    UnconsumedWrapperBind {
        rule: String,
        /// The offending variable, as written (`?s`).
        var: String,
        /// The fact type the bind reads a field of, so the error names where to look.
        fact_type: String,
    },
    /// A `(?v <- :field)` bind inside a `:not` whose variable is referenced OUTSIDE the negation,
    /// where it provably cannot have a value.
    ///
    /// Worse than [`Self::UnconsumedWrapperBind`], and the reason this wall is at declaration
    /// time rather than at `compile-all`: the reference DOES fail at fire — `#wat.runtime/
    /// UnboundSymbol` — but only along the path where the wrapper PASSES. A `:where` after a
    /// `:not` runs only when the negation admits a token, so the rule compiles, fires, and
    /// answers cleanly on any data where a matching fact happens to exist, then dies the first
    /// time none does. Measured, same binary, same rule: fact present → `n=0`, exit 0; fact
    /// absent → UnboundSymbol, exit 1. A failure whose visibility depends on the data is the
    /// one this wall exists to convert into a build error.
    ///
    /// Clara 0.24.0 refuses the same shape at session construction: *"Using variable that is not
    /// previously bound ... variables used in negations are not bound for subsequent rules since
    /// the negation can never match."*
    EscapedWrapperBind {
        rule: String,
        var: String,
        fact_type: String,
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
                "defrule `{rule}` (`:{fact_type}`): `{head}` compares at `{op_type}`, but operand \
                 `{field}` has type `{field_type}` — use the rete comparator for `{field_type}`"
            ),
            ReteCheckErrorKind::UnconsumedWrapperBind { rule, var, fact_type } => write!(
                f,
                "defrule `{rule}`: `{var}` is bound inside a `:wat::rete::not` of `:{fact_type}` \
                 and consumed nowhere — a `:not` binds nothing outward, so a bind under it is \
                 meaningful only where it is written, as a constraint on the negated fact: \
                 `(:wat::rete::not (:{fact_type} ({var} <- :field) \
                 (:wat::rete::core::i64::< {var} 10)))`. An unused bind changes no answer, but a \
                 reader cannot tell a deliberate one from a leftover edit. Drop it — \
                 `(:{fact_type})` negates the whole class — or consume it"
            ),
            ReteCheckErrorKind::EscapedWrapperBind { rule, var, fact_type } => write!(
                f,
                "defrule `{rule}`: `{var}` is bound inside a `:wat::rete::not` of `:{fact_type}` \
                 and referenced OUTSIDE it, where it can never have a value — a `:not` admits a \
                 token precisely because NO fact matched, so there is nothing to bind. At fire \
                 this is `unbound symbol: {var}`, but ONLY along the path where the `:not` \
                 passes: the rule answers cleanly on data where a matching `:{fact_type}` exists \
                 and dies the first time none does. Bind `{var}` in a positive condition, or drop \
                 the reference"
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
///
/// `pub(crate)` since 2026-08-27: `eval_insert` and `compiled_rhs` rendered their `got` operand
/// with Rust `Debug` — a user who wrote an unbound `?var` in a `:then` was shown
/// `Symbol(Identifier { name: "?nope", scopes: {} }, Span { file: … })` instead of `?nope`. They
/// now call THIS, rather than growing a second renderer, which is the whole reason it is exported
/// instead of inlined.
pub(crate) fn render_form(ast: &WatAST) -> String {
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

pub(crate) fn malformed(span: Span, rule_name: &str, fact_type: &str, clause: &WatAST) -> ReteCheckError {
    ReteCheckError {
        span,
        kind: ReteCheckErrorKind::MalformedClause {
            rule: rule_name.to_string(),
            fact_type: fact_type.to_string(),
            clause: render_form(clause),
        },
    }
}
