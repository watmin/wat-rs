//! The rete wall's error surface — every `#wat.rete/*` a `defrule` can be refused with.
//!
//! ⛔ SPLIT OUT 2026-08-30 (`partire`'s second named cut). `validate.rs` was 2_452 lines with
//! THREE concerns and no seam drawn anywhere; this is the one with no inbound dependency at all.
//! rune:lint(cited-name-absent) validate.rs — the pre-split file; its three concerns are now `validate/mod.rs`,
//! `validate/typing.rs` and this file.
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
    /// A `::`-qualified keyword CONSTANT in a `:when` constraint whose PREFIX names a known enum
    /// but whose variant that enum does not declare — `:evt::G::Hii` where `:evt::G` has `Hi`/`Lo`.
    ///
    /// rune:lint(cited-name-absent) keyword_constant_segment — the retired classifier; its live successor is
    /// `classify_keyword_constant`, and this paragraph records the arm the old name carried.
    /// ⛔ **THIS IS THE THIRD FACT `keyword_constant_segment`'s `_ => "keyword"` ARM USED TO HOLD**
    /// (arc 278, driven 2026-08-31). D1 made the typo REFUSE; it refused by falling through to the
    /// keyword-constant route, where the operand names no declared field and so came out as a
    /// located `UnknownField`: *"`:evt::Req` has no field `:evt::G::Hii`; available fields:
    /// [k, grade]"*. Both halves of that remedy are wrong — the author did not mistype a FIELD, and
    /// no field name is the fix. **A confidently wrong remedy costs more than none** (R29
    /// `RVINA ERVDIT`: the ruin must teach). Core does not name it either — it refuses the same
    /// expression as a bare `TypeMismatch` with `:remedies []` — so agreement with core was never
    /// the target; naming the mistake is.
    ///
    /// SCOPE, and it is narrow by construction: the prefix must resolve to a `TypeDef::Enum`. A
    /// `::`-free constant (`:alpha`) and a `::` name whose prefix is not a registered enum are
    /// LEGITIMATE keyword constants and keep their existing route untouched — an over-wide arm
    /// here would refuse correct programs while every new probe went green.
    ///
    /// ⚠ A correctly-spelled but TAGGED variant (`:tg::P::Hi`, arity 1) does **NOT** reach this
    /// kind, and must not: that variant EXISTS, so "has no variant `Hi`; available variants: [Hi]"
    /// would be self-contradicting. It resolves through `enum_variant_ctor` and keeps D1's
    /// `UnknownField` route. That its message is ALSO wrong is a separate, still-open finding —
    /// `strike-variant-diagnostic/DESIGN.md` affirmatively cuts it from this strike.
    UnknownEnumVariant {
        rule: String,
        fact_type: String,
        /// The enum the prefix names, colon-stripped the way `UnknownField.fact_type` is.
        enum_path: String,
        /// The variant AS WRITTEN — the misspelling itself, so the reader sees their own typo.
        variant: String,
        /// The variants that DO exist, declaration order. The remedy `UnknownField` could not
        /// give: mirrors `available_fields`, and is the whole point of the kind.
        available_variants: Vec<String>,
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
    /// Arc 278 D10 — a `:then` field VALUE whose type is KNOWABLE and disagrees with the
    /// destination field's declared type.
    ///
    /// The hole this closes was driven (`wat-scripts/scratch-pad/d10-then-rhs-is-not-type-checked.wat`):
    /// the same construction that is a `#wat.check/TypeMismatch` everywhere else in the language
    /// was accepted inside a `:then`, so `#tr/Bad {:n "not-an-i64"}` entered the FACT SET — where
    /// joins, queries, the oracle and `explain` all trust the declared schema. The four RHS walls
    /// that already existed (`RhsArityMismatch`, `RhsMissingFields`,
    /// `RhsPositionalConstructionRetired`, `RhsUnresolvableOperand`) are every one of them
    /// STRUCTURAL; none typed a value.
    ///
    /// ⛔ **KNOWABLE, and only knowable.** The producer refuses exactly one shape:
    /// `OperandType::Resolved(a)` against a destination whose declared type also resolves to a
    /// rete segment, with `a` different. Every other answer the resolver can give —
    /// `UnboundInThisRule`, `ComputedNotDerivableHere`, `NotComparable`, `MistypedEnumVariant` —
    /// is NOT-KNOWABLE-HERE, which is not the same as wrong, and is passed. That distinction is
    /// the whole difficulty: refusing a not-knowable operand would reject a `?var` bound from a
    /// derived fact, a computed operand whose head is `Form`/`Redispatch`, or a type variable,
    /// while every new probe went green.
    ///
    /// ## Why BOTH `field_type` and `field_rete_type`, which no sibling kind carries
    ///
    /// The comparison is made at the rete SEGMENT the destination's declared type maps to
    /// (`rete_type_segment_of`), not at the declared path — `operand_type` is a segment because
    /// `resolve_operand_type` answers in segments. Reporting only the declared path would state a
    /// comparison that was not the one performed (two distinct enums both segment to `enum`, and
    /// this kind does NOT separate them); reporting only the segment would hide the `defrecord`
    /// line the author has to go and read. Both are carried so the message can be checked against
    /// the check that produced it — the granularity is part of the finding, not a footnote.
    RhsFieldTypeMismatch {
        rule: String,
        fact_type: String,
        /// The DESTINATION field, by name — `:then` fills fields by name, so the field is the
        /// thing to name first. `RhsArityMismatch` can only say "argument #n"; this can do better.
        field: String,
        /// That field's type AS DECLARED in the `defrecord`, rendered by `check::format_type`
        /// (the substrate's ONE TypeExpr renderer) so it reads exactly as the checker's messages.
        field_type: String,
        /// The rete segment `field_type` maps to — the LEFT side of the comparison actually made.
        field_rete_type: String,
        /// The offending value, rendered as wat source (`describe_operand`) — never Rust `Debug`,
        /// the same contract `RhsUnresolvableOperand.operand` states.
        operand: String,
        /// The rete segment the value resolves to — the RIGHT side of the comparison.
        operand_type: String,
    },
    /// Arc 278 BRIEF-construction-total-three-walls.md #1 — a NESTED surface aggregate-
    /// constructor operand (an operand's VALUE, not a `:then` item's own top-level head) written
    /// with MORE THAN ONE positional argument. Once #1 wires a nested constructor to actually
    /// reach `:wat::core::kwargs-construct`'s dispatch (`eval_kwargs_construct`, runtime.rs), that
    /// dispatch unconditionally retires multi-arg RAW POSITIONAL construction at a bare aggregate
    /// name (kwargs, or a single positional value, are the only two supported nested shapes) —
    ///
    /// ⛔⛔ **THAT LAST SENTENCE IS TRUE OF THE INTERPRETER AND FALSE OF RETE. Driven 2026-09-01.**
    /// Native rete fire never reaches that dispatch: `arm.rs`'s `rhs_must_compile` says outright
    /// *"Refuse — do not walk `build_insert_fact` on native fire"*, and the compiled path lowers
    /// through `eval_insert.rs`'s `rete_kwargs_value_asts`, whose non-kwargs arm returns the args
    /// verbatim — *"Positional is already declaration order BY DEFINITION"*. Driven at HEAD, a
    /// nested `(:T ?k 99)` COMPILED, FIRED, and derived a correctly-valued fact. So the retirement
    /// was never enforced on the rete path at all, and this variant firing from
    /// `walk_nested_constructors` is **new enforcement**, not restored parity — a deliberate
    /// alignment with the stated doctrine, taken on a corpus sweep showing zero uses
    /// (1650 `.wat`, 460 `:then` clauses). The claim above was written from the interpreter's
    /// behaviour and never checked against this path.
    ///
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
    /// Render one check error kind as the sentence a user reads.
    ///
    /// Every arm names the RULE and the FACT TYPE before the fault, because a check error with
    /// no such context sends the reader hunting for which of a dozen rules produced it. The arms
    /// are spelled out per kind rather than templated, for the same reason `into_eval` is: the
    /// kinds exist because the advice differs.
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
            ReteCheckErrorKind::UnknownEnumVariant {
                rule,
                fact_type,
                enum_path,
                variant,
                available_variants,
            } => write!(
                f,
                "defrule `{rule}` (`:{fact_type}`): `:{enum_path}` has no variant `{variant}`; \
                 available variants: [{}]",
                available_variants.join(", ")
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
            ReteCheckErrorKind::RhsFieldTypeMismatch {
                rule, fact_type, field, field_type, field_rete_type, operand, operand_type,
            } => write!(
                f,
                "defrule `{rule}`: `:then` insert of `:{fact_type}` fills field `:{field}`, declared \
                 `{field_type}` (rete `{field_rete_type}`), with operand `{operand}`, whose type is \
                 `{operand_type}` — the same construction written outside a rule is a TypeMismatch, \
                 and a `:then` value is checked the same way. Supply a value of type \
                 `{field_rete_type}`, or change `:{field}`'s declared type"
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
    /// The machine-readable form, splicing the SPAN into the kind's own map.
    ///
    /// A kind that renders as a non-map body is wrapped under `body` rather than dropped, so the
    /// span can always be attached — an error whose EDN form silently lost its location would be
    /// unusable by exactly the tooling this representation exists for.
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
