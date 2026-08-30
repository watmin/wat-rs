//! The operand typer — what a `:when` operand IS, and whether it can resolve at all.
//!
//! ⛔ SPLIT OUT 2026-08-30. `partire` named this one on 2026-08-28 as *"the self-contained one"*
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
        // A usable constant AT THIS COMPARATOR is legitimate; say nothing.
        if op_type == Some(keyword_constant_segment(k, types)) {
            return;
        }
        check_field_at(field, clause.span().clone(), rule_name, fact_type, field_names, errors);
    }
}

pub(crate) fn check_field(
    field: &str,
    clause: &WatAST,
    rule_name: &str,
    fact_type: &str,
    field_names: &[String],
    errors: &mut Vec<ReteCheckError>,
) {
    check_field_at(field, clause.span().clone(), rule_name, fact_type, field_names, errors);
}

/// Record an `UnknownField` unless `field` is one the fact type declares.
///
/// Takes the span of the FIELD rather than the clause so the caret lands on the offending
/// keyword. The error carries the available field names alongside the bad one — a
/// did-you-mean the reader can act on without going to look up the record definition.
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

/// The rete type a bare keyword CONSTANT carries: `enum` when its prefix names a registered enum,
/// else `keyword`.
///
/// `:probe::E::A` -> prefix `:probe::E` -> a `TypeDef::Enum` -> `enum`. Note this keyword could
/// never have been a field reference in the first place: it carries `::`, and a field name is a
/// bare identifier (`available fields: [k, v]`). The engine refused it as an unknown field anyway.
fn keyword_constant_segment(k: &str, types: &TypeEnv) -> &'static str {
    if let Some((type_path, _variant)) = k.rsplit_once("::") {
        if matches!(types.get(type_path), Some(TypeDef::Enum(_))) {
            return "enum";
        }
    }
    "keyword"
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
                None => return OperandType::Resolved(keyword_constant_segment(k, types)),
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
