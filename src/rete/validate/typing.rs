//! The operand typer — what a `:when` operand IS, and whether it can resolve at all.
//!
//! ⛔ SPLIT OUT 2026-08-30. `partire` named this one on 2026-08-28 as *"the self-contained one"*
//! rune:lint(cited-name-absent) validate.rs — the pre-split file this module was cut out of; its three concerns now live in `validate/mod.rs`, `validate/typing.rs` and `validate/error.rs`.
//! of `validate.rs`'s three concerns, and the measurement agreed: 506 contiguous lines with no
//! other concern interleaved into them.
//!
//! It answers one question the rest of the wall asks repeatedly — **given a form in operand
//! position, what type does it denote, and does that type exist?** Field references, keyword
//! constants, the `i64`/`f64`/`String` segment a rete op is spelled under, and the
//! `OperandType` verdict itself. It moves when the TYPE LANGUAGE moves; the `:when` and `:then`
//! validators move when the rule GRAMMAR moves.
//!
//! Takes `ClauseCtx` from the parent rather than owning it: the context is the clause being
//! walked, which is the walker's business, not the typer's.

use super::*;

/// A constraint operand is schema-checked ONLY when it is a `:field` reference; a `?var`
/// (free or bound) and a literal are left alone (design: "the free `?v` stays free").
/// ★ THE ONE RULE, and it is the one `compiled_cond::bind_field_refs` has always used:
/// **a keyword operand is a FIELD REFERENCE if it names a declared field; otherwise it is a
/// CONSTANT.** rete has keyword-valued and enum-valued constants only, so at any other comparator
/// type there is no constant the keyword could be — and then "you meant a field" is both true and
/// the more actionable thing to say, so the located `UnknownField` is still what is reported.
///
/// ⚠ **THIS FUNCTION USED TO DEMAND THAT EVERY KEYWORD BE A FIELD**, which is why
/// `(keyword::= :v :alpha)` and `(enum::= :v :probe::E::A)` were refused for the life of the
/// engine — while the IDENTICAL comparison, nested one level as an operand of another call, fired
/// and answered correctly, because the nested path asked the question this one never did.
///
/// It can only ADMIT programs, never change one: a non-field keyword here was a hard freeze error,
/// so no program that compiles today contains one.
pub(crate) fn check_operand_field_ref(
    operand: &WatAST,
    clause: &WatAST,
    ctx: &ClauseCtx<'_>,
    op_type: Option<&str>,
    errors: &mut Vec<ReteCheckError>,
) {
    // Taken as the whole `ClauseCtx` rather than four loose borrows — the same shape
    // `check_constraint_head` already uses, and the reason is not tidiness: this function grew two
    // parameters on 2026-08-28 and clippy's arity ceiling caught it at 8, which is the ceiling
    // doing its job. Bundling is the fix; an `#[allow]` would have been the patch.
    let ClauseCtx { rule_name, fact_type, field_names, types, .. } = *ctx;
    if let WatAST::Keyword(k, _) = operand {
        let field = k.trim_start_matches(':');
        if field_names.iter().any(|f| f == field) {
            return; // a declared field — the field reference wins, exactly as before.
        }
        // ★ THE THIRD STATE, ahead of the comparator question and deliberately so. A `::` name
        // whose prefix is a known enum and whose variant does not exist is a MISTAKE at every
        // comparator — including `keyword::=`, where the old routing let it through as a
        // legitimate keyword constant — so the refusal cannot be conditioned on `op_type`.
        // It names the enum, the variant as written, and the variants that exist; it does NOT
        // fall through to `check_field_kw`, whose remedy (the record's field names) is the
        // confidently-wrong one this kind exists to delete.
        let constant = classify_keyword_constant(k, types);
        if let KeywordConstant::UnknownVariant { enum_path, variant, available } = constant {
            errors.push(ReteCheckError {
                span: clause.span().clone(),
                kind: ReteCheckErrorKind::UnknownEnumVariant {
                    rule: rule_name.to_string(),
                    fact_type: fact_type.to_string(),
                    enum_path: enum_path.trim_start_matches(':').to_string(),
                    variant: variant.to_string(),
                    available_variants: available,
                },
            });
            return;
        }
        // A usable constant AT THIS COMPARATOR is legitimate; say nothing.
        if op_type == Some(constant.segment()) {
            return;
        }
        // The OPERAND NODE, not `clause.span()`: the keyword IS the field reference, so its own
        // span is the only one this producer can be handed (see `check_field_kw`).
        check_field_kw(operand, rule_name, fact_type, field_names, errors);
    }
}

/// ★ **THE ONE PRODUCER of a check-time `UnknownField`, and it takes the KEYWORD NODE.**
///
/// Record an `UnknownField` unless the keyword names a field `fact_type` declares. Returns
/// `true` when the field IS declared (nothing recorded), so a caller batching several kwargs can
/// fold the verdicts without re-deriving the lookup.
///
/// rune:lint(cited-name-absent) check_field_at — the span-taking predecessor renamed to `check_field_kw`; the whole
/// paragraph is about the name being gone, so nothing bears it today.
/// ⛔ **It does not take a `Span`, and that is the whole point.** This function's predecessor
/// (`check_field_at`) took `span: Span` under a doc promising *"the span of the FIELD rather than
/// the clause so the caret lands on the offending keyword"* — and BOTH its callers passed
/// `clause.span()`, while two more sites open-coded the same error against an enclosing form's
/// span. A `Span` parameter accepts the clause's, the fact's and the field's with equal ease, so
/// the promise had no way to be wrong out loud: three docs stated the behaviour and three sites
/// did otherwise for the life of the wall. Taking the NODE makes the wrong span unwritable at the
/// call — there is nothing to pass but the keyword the author actually mistyped.
///
/// The error carries the available field names alongside the bad one — a did-you-mean the reader
/// can act on without going to look up the record definition.
pub(crate) fn check_field_kw(
    field_kw: &WatAST,
    rule_name: &str,
    fact_type: &str,
    field_names: &[String],
    errors: &mut Vec<ReteCheckError>,
) -> bool {
    let WatAST::Keyword(k, span) = field_kw else {
        // ⛔ LOUD, NOT SILENT — and the choice is the cure probing its own shape. Every span this
        // strike deleted was an enclosing FORM's (`clause`, the nested constructor's, the fact's),
        // and a form is a `List`. So the exact mistake the old code made now arrives HERE, and a
        // `return true` would answer it by reporting NOTHING: the wrong caret would become a
        // vanished refusal, which is worse in kind than the defect being fixed. A wrong span is
        // visible in a golden; an absent error is visible in nothing.
        //
        // It cannot fire today. Each of the four callers arrives from a grammar position already
        // proven to be a keyword: `check_operand_field_ref`'s own `if let Keyword`,
        // `classify_rete_clause`'s `Bind` (whose `field_kw` is the node `keyword_payload` returned
        // `Some` for — `Some` for `Keyword` alone), and the two kwargs walks, where
        // `rete_is_kwargs` has confirmed a keyword at every even index and an even arity. This is
        // the same guarantee those two walks already spell `unreachable!` three lines away.
        unreachable!(
            "check_field_kw takes the FIELD-NAMING KEYWORD; every caller reaches it from a \
             position the grammar has already proven to be one. Got: {field_kw:?}"
        )
    };
    let field = k.trim_start_matches(':');
    if field_names.iter().any(|f| f == field) {
        return true;
    }
    errors.push(ReteCheckError {
        // The KEYWORD's own span. Not reachable from anywhere else in this function.
        span: span.clone(),
        kind: ReteCheckErrorKind::UnknownField {
            rule: rule_name.to_string(),
            fact_type: fact_type.to_string(),
            field: field.to_string(),
            available_fields: field_names.to_vec(),
        },
    });
    false
}

/// Design call 3 — accumulate's `:from` inner and (by extension) any bare fact-type-head-only
/// check: registered-type validation ONLY, no clause walk.
pub(crate) fn validate_fact_type_head_only(cond: &WatAST, rule_name: &str, types: &TypeEnv, errors: &mut Vec<ReteCheckError>) {
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

/// Reads the registry through `matcher::aggregate_field_names` — the same body
/// `class_field_names` uses. This used to be a byte-equivalent second copy, while
/// `matcher.rs`'s doc claimed to be the registry's one reader and did not list this file.
pub(crate) fn lookup_fields(types: &TypeEnv, fact_type: &str) -> Option<FieldList> {
    crate::rete::matcher::aggregate_field_names(types, fact_type)
}

/// Sibling of `lookup_fields`, same key and same registry — the DECLARED TYPE of each field, in
/// declaration order, so a per-type rete constraint can be checked against the field it reads.
/// Kept beside its twin rather than folded into it: every existing caller wants only the names,
/// and widening the shared return would make them all pay for a walk they do not use.
pub(crate) fn lookup_field_types(types: &TypeEnv, fact_type: &str) -> Option<FieldList> {
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
        // ⛔ BOTH SPELLINGS, and the lower-case one is the ONLY inhabitable half.
        //
        // This line read `"wat::core::Keyword"` alone until 2026-08-28, and that capital is a type
        // NO VALUE CAN HAVE: `(:wat::core::defrecord :R [v <- :wat::core::Keyword])` declares
        // clean and every construction of it is a TypeMismatch (proven in
        // `docs/arc/2026/04/109-kill-std/NOTE-keyword-is-two-disjoint-type-names-…md`). So the map
        // recognised the uninhabitable spelling and missed the real one, which fell through to the
        // enum-registry lookup, missed there too, and returned `None` —
        // `ConstraintTypeNotComparable`, refusing keyword equality as an inline constraint for
        // the life of the engine.
        //
        // The diagnostic was self-contradicting and said so: it lists `keyword` as part of the
        // equality surface in the same sentence that refuses a keyword.
        //
        // Found by arc 278's § 4.1 reachability ledger, root named by arc 109's NOTE. The capital
        // stays mapped because removing a dead type NAME is arc 109's ground, not this file's.
        "wat::core::keyword" | "wat::core::Keyword" => Some("keyword"),
        // An enum is named by a user path; the registry is the only way to know.
        other => match types.get(&format!(":{other}")) {
            Some(TypeDef::Enum(_)) => Some("enum"),
            _ => None,
        },
    }
}

/// A keyword operand that names no declared field — i.e. one the ONE RULE reads as a CONSTANT.
///
/// Used only to suppress a second, misleading diagnostic: when such a keyword does not type-check
/// at the comparator, `check_operand_field_ref` has already reported the located `UnknownField`,
/// which is the actionable message. This predicate is deliberately NOT the rule itself — the rule
/// lives in `check_operand_field_ref` and in `compiled_cond::bind_field_refs`; this only answers
/// "has that already been reported?".
fn is_non_field_keyword(operand: &WatAST, field_names: &[String]) -> bool {
    match operand {
        WatAST::Keyword(k, _) => {
            let field = k.trim_start_matches(':');
            !field_names.iter().any(|f| f == field)
        }
        _ => false,
    }
}

/// ⚠ **THE CLASSIFIER TYPED BY PREFIX ALONE AND NEVER CHECKED THE VARIANT EXISTED** (vigilia Class D1,
/// driven 2026-08-31). `rsplit_once("::")` + "is that path a `TypeDef::Enum`" typed `:evt::G::Hii`
/// — a variant the enum does not declare — as `"enum"`, the checker saw enum-vs-enum and passed,
/// and the RUNTIME then resolved the same keyword through `expr_ir::keyword_value` ->
/// `sym.unit_variant`, an EXACT lookup, got `None`, and fell back to a plain keyword. `enum::=`
/// compared Enum vs keyword: always false. **The rule compiled, fired, and matched nothing, with
/// no diagnostic** — while CORE refuses the identical expression at check time
/// (`:wat::core::=: parameter #2 expects :wat::core::keyword; got :evt::G`). Agreement with core
/// is the contract; "it didn't match" is the easiest wrong answer to ship.
///
/// TWO conditions, and the second is the one the obvious fix misses:
///   1. `matcher::enum_variant_ctor` — the ONE resolution, the same one the lowerer, the purity
///      classifier and `walk_nested_constructors` already use. `typing.rs` was the fourth,
///      hand-written, site and the only one that disagreed with the runtime.
///   2. **arity == 0.** `enum_variant_ctor` resolves Unit **and** Tagged; `sym.unit_variant` is
///      UNIT-ONLY. A tagged variant has no bare value form — `(:tg::P::Hi 7)` is the only way to
///      write one, which is why core refuses that too (`expects [:wat::core::i64 :-> :tg::P]`) —
///      so resolving is not enough. Without this clause the tagged arm stays broken.
///
/// ⛔ **AND THE PARAGRAPH THAT STOOD HERE WAS THE NEXT DEFECT, IN THIS FILE, IN ITS OWN DOC.** It
/// read: *"Everything else falls to `keyword`, where the existing `UnknownField` /
/// `ConstraintTypeMismatch` machinery produces the located diagnostic."* True about the LOCATION
/// and silent about the MESSAGE — that route reports the constant as an unknown FIELD and offers
/// the record's field names as the remedy, so `:evt::G::Hii` was refused with *"`:evt::Req` has no
/// field `:evt::G::Hii`; available fields: [k, grade]"*: a reader sent hunting for a field when
/// they mistyped a variant. Its own next sentence had the disproof in it — *"such a keyword could
/// never have been a field reference: it carries `::`"* — and still concluded the field diagnostic
/// was the right one. Split out below (`UnknownEnumVariant`, 2026-08-31).
///
/// What a bare keyword CONSTANT in operand position actually is — **THREE states, not two.**
///
/// rune:lint(cited-name-absent) keyword_constant_segment — the retired classifier this superseded; its live successor
/// is `classify_keyword_constant`, and the paragraph records the arm the old name carried.
/// ⛔ `keyword_constant_segment`'s `_ => "keyword"` arm held two facts (arc 278, the fifth
/// catch-all of this class after A2b's `Option`, D3's missing arity, A6's `None => true` and A5's
/// `Ok(())`): *"this is a genuine keyword constant"* **and** *"this is a `::`-qualified name whose
/// prefix is a known enum and whose variant does not exist."* The second is a diagnosable mistake
/// being typed as the first, and the cost was not a missing refusal — D1 already made it refuse —
/// it was the refusal naming the WRONG THING: the keyword route reports the operand as an unknown
/// FIELD and offers the record's field names as the remedy. Same cure as the other four: climb to
/// the type, and let each state carry what only it knows.
///
/// The `'a` lifetime is the KEYWORD's, not the `TypeEnv`'s: `enum_path`/`variant` are slices of the
/// constant as the author wrote it, so the diagnostic shows their own text back to them.
enum KeywordConstant<'a> {
    /// A UNIT variant that EXISTS. Types as `enum` — D1's arity-0 clause, unchanged.
    UnitVariant,
    /// A genuine keyword constant: no `::`, or a `::` name whose prefix is not a registered enum,
    /// or (see `UnknownEnumVariant`'s doc) a correctly-spelled TAGGED variant, which resolves.
    Keyword,
    /// The third state. The prefix names a `TypeDef::Enum`; the enum does not declare the variant.
    UnknownVariant { enum_path: &'a str, variant: &'a str, available: Vec<String> },
}

impl KeywordConstant<'_> {
    /// The rete module segment the constant denotes. `UnknownVariant` answers `keyword` because
    /// that is what the RUNTIME does with it (`sym.unit_variant` misses and falls back) — but
    /// nothing type-checks on that answer: `check_operand_field_ref` refuses the constant before
    /// any comparator comparison, and `resolve_operand_type` routes it to
    /// `OperandType::MistypedEnumVariant` rather than through here.
    fn segment(&self) -> &'static str {
        match self {
            KeywordConstant::UnitVariant => "enum",
            KeywordConstant::Keyword | KeywordConstant::UnknownVariant { .. } => "keyword",
        }
    }
}

/// The ONE classification. Asks `matcher::enum_variant_ctor` — the same single resolver the
/// lowerer, the purity classifier and `walk_nested_constructors` use, and the one D1 routed this
/// file through — and then asks it a SECOND question, not a different one: when it declines, was
/// the prefix an enum anyway?
///
/// ⚠ ORDER IS LOAD-BEARING. `enum_variant_ctor` resolves Unit **and** Tagged, so a resolved
/// non-unit variant falls to `Keyword`, NOT to `UnknownVariant` — `:tg::P::Hi` (arity 1) exists,
/// and telling its author the enum "has no variant `Hi`" would be a false statement in a
/// diagnostic built to stop false statements in diagnostics. A guard placed after the arity-0 arm
/// (rather than on the `None` case specifically) would swallow it; that is the trap this order
/// closes.
fn classify_keyword_constant<'a>(k: &'a str, types: &TypeEnv) -> KeywordConstant<'a> {
    match crate::rete::matcher::enum_variant_ctor(types, k) {
        Some((_, _, 0)) => KeywordConstant::UnitVariant,
        // Resolved but not arity 0: a TAGGED variant, which EXISTS. D1's route, deliberately kept.
        Some(_) => KeywordConstant::Keyword,
        None => match k.rsplit_once("::") {
            Some((enum_path, variant)) => match types.get(enum_path) {
                Some(TypeDef::Enum(e)) => KeywordConstant::UnknownVariant {
                    enum_path,
                    variant,
                    available: e
                        .variants
                        .iter()
                        .map(|v| match v {
                            crate::types::EnumVariant::Unit(n)
                            | crate::types::EnumVariant::Tagged { name: n, .. } => n.clone(),
                        })
                        .collect(),
                },
                // The prefix is not a registered enum — a legitimate `::`-bearing keyword.
                _ => KeywordConstant::Keyword,
            },
            // No `::` at all: `:alpha`. Always a keyword, and the anti-vacuity control.
            None => KeywordConstant::Keyword,
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
/// of this a guess? we know the type's value from the record def."* Correct — FOUR exhaustive
/// sources, in order, and no fallback after them:
///   1. a `:field` operand      -> the field's DECLARED type
///   2. a `?var` operand        -> the field its `(?v <- :field)` bind names, then the field's type
///   3. a LITERAL operand       -> the literal's own type
///   4. a nested CALL operand   -> its head row's declared `ret` (`Alias`/`Fallback` only)
///
/// ⚠ **SOURCE 4 WAS MISSING AND THIS DOC CLAIMED THE LIST WAS EXHAUSTIVE ANYWAY.** The three above
/// were written before fix-list F made a nested call a legal operand, and nothing came back to
/// re-read them; a computed operand fell to a `_` arm meaning "an unbound `?var`" and so skipped
/// the type check outright. Measured 2026-08-28: `(string::= :v "x")` is CAUGHT, and wrapping the
/// same operand in a call — `(string::= (i64::+ :v 0 :undefined 0) "x")` — was NOT, after which the
/// rule compiled, fired and matched nothing. The builder's cut applies verbatim to the fourth
/// source: every rete row is `total`, so its `ret` is a FACT about the row, never a guess.
///
/// If none resolves (a `?var` bound nowhere in the rule), the type is genuinely not knowable — and
/// then this reports the law-A violation WITHOUT a per-type suggestion rather than inventing one.
/// A wrong suggestion teaches a wrong fix. A nested call whose head is `Form`/`Redispatch`, or
/// whose `ret` is a type variable, is not-knowable-HERE rather than not-knowable — a distinction
/// `OperandType::ComputedNotDerivableHere` carries so the two cannot be confused again.
pub(crate) fn check_constraint_head(
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
            let suffix = wat_reader::identifier::leaf(op);
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
                    // ⛔ DO NOT DOUBLE-REPORT. A keyword that names no field has already been
                    // reported by `check_operand_field_ref` as the unknown field it almost
                    // certainly is — and a second error here would teach the WRONG fix, telling
                    // the author to switch comparator (`use the rete comparator for keyword`)
                    // when they actually mistyped a field name. R29 `RVINA ERVDIT`: the ruin must
                    // teach, and two ruins pointing opposite ways teach worse than one.
                    OperandType::Resolved(_) if is_non_field_keyword(operand, field_names) => {}
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
                    // Also out of scope, for a DIFFERENT reason the variant's doc names: this pass
                    // holds a `TypeEnv`, not the checker. Same action, separate name — the whole
                    // point of the variant.
                    OperandType::ComputedNotDerivableHere => {}
                    // Refused by NAME in `check_operand_field_ref` (`UnknownEnumVariant`). Silent
                    // here for the same reason the keyword arm at the top of this match is: two
                    // ruins pointing opposite ways teach worse than one, and the located
                    // variant-not-found message is the one that names the actual mistake.
                    OperandType::MistypedEnumVariant => {}
                }
            }
        }
    }
}

/// Collect every `(?v <- :field)` bind in the WHOLE rule, resolved to the field's declared type.
///
/// Rule-wide, not per-pattern, because a join variable is bound in one condition and compared in
/// another. `not`/`exists` wrappers are unwrapped so their inner pattern's binds count too.
pub(crate) fn collect_rule_bind_types(
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
                    if let ReteClauseShape::Bind { var, field, .. } = classify_rete_clause(clause) {
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
    /// A nested CALL operand whose return type THIS PASS cannot derive — not one that has no type.
    ///
    /// ⛔ **This variant exists because the alternative was a lie.** Until 2026-08-28 a nested call
    /// fell to `_ => UnboundInThisRule` — the variant one line above, whose whole doc says it means
    /// "an unbound `?var`". A computed operand is not an unbound variable, and routing it there
    /// meant `(string::= (i64::+ :v 0 :undefined 0) "x")` skipped the type check ENTIRELY and then
    /// compiled, fired and matched nothing. Wrapping an operand in a call made its type error
    /// disappear (measured). The three sources this function documents as "exhaustive" were written
    /// before fix-list F made a nested call a legal operand, and nothing re-read them.
    ///
    /// Two cases reach here, and NEITHER is derivable from `RETE_OPS` alone:
    ///   · a `Form`/`Redispatch` head — its `ret` is a PLACEHOLDER (`vocabulary.rs`: those rows
    ///     carry no `TypeScheme` at all), so reading it would assert a type the table does not
    ///     know. The real answer is `check.rs`'s `infer_rete_form`, which runs LATER — this wall
    ///     is hooked into `build_env` and holds only a `TypeEnv`.
    ///   · a `ret` that is a type VARIABLE or a container (`PersistentVector/first : PV<T> -> T`).
    ///     The row states a relation, not a type; the type comes from the arguments.
    ///
    /// The caller's action is the same as for `UnboundInThisRule` — report nothing — but the REASON
    /// is different, and collapsing two reasons into one outcome is the exact conflation that has
    /// now cost this arc four separate defects.
    ComputedNotDerivableHere,
    /// A keyword constant whose `::`-prefix names a known enum and whose variant that enum does
    /// not declare — already refused, by NAME, in `check_operand_field_ref`.
    ///
    /// ⛔ Not `Resolved("keyword")`, which is what it used to be. That answer is what the runtime
    /// does with the constant, not what it IS, and returning it here put the refused constant back
    /// into the same catch-all the strike split — where the CoreGeneric branch would then have
    /// read a type off a rejected operand to build its did-you-mean. Same ruling as
    /// `ComputedNotDerivableHere` one line above: the caller's ACTION is "report nothing", the
    /// REASON is its own, and collapsing reasons into outcomes is what cost this arc four defects.
    MistypedEnumVariant,
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
                // Not a declared field -> a CONSTANT, and its type is the constant's own. This
                // used to return `UnboundInThisRule`, which is why `(keyword::= :v :alpha)` had
                // no type at all and `:alpha` could only ever be reported as a missing field.
                None => {
                    return match classify_keyword_constant(k, types) {
                        KeywordConstant::UnknownVariant { .. } => OperandType::MistypedEnumVariant,
                        other => OperandType::Resolved(other.segment()),
                    };
                }
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
        // 4. a nested CALL — its head row's DECLARED return type.
        //
        // ★ THE FOURTH SOURCE, and the reason the list above stopped being exhaustive. Rete's ops
        // are `pure · deterministic · total` — gated for every row by `every_rete_row_is_total` —
        // so an `Alias`/`Fallback` row's `ret` is a FACT about the row, not a guess. That is the
        // same standard the three sources above already meet, applied to the operand shape
        // fix-list F made legal and nobody came back to type.
        WatAST::List(items, _) => {
            let Some(WatAST::Keyword(head, _)) = items.first() else {
                return OperandType::ComputedNotDerivableHere;
            };
            let Some(row) = crate::rete::vocabulary::rete_op_for(head) else {
                // A non-rete head in operand position is LAW A's finding, reported by its own
                // path. Not a type question, and not this function's to answer.
                return OperandType::ComputedNotDerivableHere;
            };
            // The row's own answer, believed as it stands. This used to test `class` first —
            // "`Form`/`Redispatch` carry `ret` as a PLACEHOLDER" — which was true of the old bare
            // `ParamType` and is now impossible to express: a row with nothing to state says
            // `Ret::NoScheme`. Same outcome for `let`/`match`/`fn` (which genuinely have no
            // statable return), and `coincident?` stops being collateral damage.
            //
            // A `Var` ret is a type VARIABLE resolved from the ARGUMENTS, never a type this row
            // states. Rejected BEFORE the path mapping so it cannot accidentally resolve against a
            // user type that happens to share the variable's spelling.
            let crate::rete::vocabulary::Ret::Is(declared) = row.ret else {
                return OperandType::ComputedNotDerivableHere;
            };
            if matches!(declared, crate::rete::vocabulary::ParamType::Var(_)) {
                return OperandType::ComputedNotDerivableHere;
            }
            // ONE mapping, not a second copy: the row's `ret` becomes a `TypeExpr` by the same
            // `to_type_expr` the checker registers schemes with, and the path goes through this
            // file's own `rete_type_segment_of`. A private ParamType->segment table here would be
            // a second place for the keyword bug of 2026-08-28 to live.
            return match declared.to_type_expr() {
                crate::types::TypeExpr::Path(p) => match rete_type_segment_of(&p, types) {
                    Some(seg) => OperandType::Resolved(seg),
                    None => OperandType::ComputedNotDerivableHere,
                },
                // Parametric — a container. Rete has no comparator for one, which is what
                // `NotComparable` says for a FIELD; here the operand is computed, so the honest
                // answer is that this pass cannot derive it.
                _ => OperandType::ComputedNotDerivableHere,
            };
        }
        // ⛔ THE WILDCARD IS DELETED. `_ => UnboundInThisRule` is what swallowed the nested call
        // above. Every remaining variant is named, so a new `WatAST` variant is a compile error
        // here rather than a silent skip of the type check.
        WatAST::Symbol(..)
        | WatAST::RationalLit(..)
        | WatAST::BigIntLit(..)
        | WatAST::NilLit(..)
        | WatAST::Vector(..)
        | WatAST::Map(..)
        | WatAST::Set(..) => return OperandType::UnboundInThisRule,
    };
    match rete_type_segment_of(&declared, types) {
        Some(seg) => OperandType::Resolved(seg),
        None => OperandType::NotComparable(declared),
    }
}

/// How to name an operand in a diagnostic: a field by its name, anything else by its source form.
///
/// The fallback goes through [`render_form`], not Rust `Debug` — this file's own
/// contract (`RhsUnresolvableOperand.operand`: "rendered as wat source
/// (`render_form`) — never Rust `Debug`"). It used to be `{other:?}`, so a
/// diagnostic about a literal operand printed Rust struct and span noise at the
/// exact moment the reader needed to see their own source.
fn describe_operand(operand: &WatAST) -> String {
    match operand {
        // The operand's REAL spelling, colon included. It used to be stripped here and re-added by
        // one caller's format string — which was fine while every operand was a field keyword, and
        // rendered `:(:wat.rete.core.i64/+ :v 0 …)` the moment a nested CALL could reach the same
        // message (2026-08-28). A diagnostic that misspells the form it is quoting cannot be
        // pasted back into the source, which is the whole job of quoting it.
        WatAST::Keyword(k, _) => k.clone(),
        WatAST::Symbol(s, _) => s.as_str().to_string(),
        other => render_form(other),
    }
}
