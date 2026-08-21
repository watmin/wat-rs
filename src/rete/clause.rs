//! Rete-DSL clause grammar — shapes compile, validate, stratify, and the
//! oracle matcher all consume.
//!
//! One grammar, many consumers. `eval_clause` (`matcher.rs`) is a consumer, not
//! a second spelling. `alpha_pattern` stays on the matcher: it is the inner
//! condition-head parse, not this classifier.
//!
//! Independent of Fact, BindView, FireSession, TypeEnv.

use crate::ast::WatAST;
use crate::form_match::keyword_payload;

/// Arc 294 item 9a (DESIGN-rete-defrule-wall.md, design call 1 — "one grammar, shared") —
/// the rete-DSL clause/condition-wrapper shape space, recognized identically whether the
/// caller is the runtime matcher (`matcher::eval_clause`) or the freeze-time validator
/// (`crate::rete::validate::validate_rete_rules`). A single source for "what shape is
/// this form" closes the drift hole that let the 9a codemod's injected bare-keyword
/// clauses classify as `Unrecognized` (silently `None`'d at fire time) instead of a
/// located freeze error.
///
/// Covers BOTH grammar levels the rete DSL actually has:
/// - within-condition CLAUSES (`Bind`, `Constraint`, `And`, `Or`, `Not`, `Where`) — the
///   shapes `eval_clause` classifies today (this extraction is behavior-identical).
/// - top-level `:when`-entry WRAPPERS (`Not`, `Exists`, `Where`, `Accumulate`) — shapes
///   `eval_clause` never actually receives (compile-condition, `wat/rete/compile.wat`, consumes
///   them into NegationNode/ExistsNode/AccumulateNode/TestNode topology before alpha-match
///   ever runs), but the validator's top-level `:when` walk needs to recognize them too, via
///   this SAME function, rather than a second hand-rolled keyword-matcher (the drift risk
///   design call 1 rules out). `eval_clause`'s new dispatch maps `Exists`/`Accumulate` to
///   `None` — identical to the pre-extraction default arm, since those shapes never reach it.
///
/// `Where` payload is read by `compile_cond_driver`. `Accumulate.from` is read by
/// stratify / validate. `var` / `acc_form` ride the shape so the classifier is total
/// over the grammar (callers use `..`).
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
        // rune:purgare(trait-contract) — classifier names the full accumulate shape
        // (`?var <- acc-form :from inner`); current consumers only walk `from`.
        #[allow(dead_code)]
        var: &'a str,
        // rune:purgare(trait-contract) — same grammar payload; fire reads acc-form
        // off the AccumulateNode, not this parse field.
        #[allow(dead_code)]
        acc_form: &'a WatAST,
        from: &'a WatAST,
    },
    /// `(?p <- :ns::Type clause…)` — top-level fact bind (Clara `[?p <- Type]`).
    /// Discriminated from [`Self::Bind`] by a `::` in the type keyword; from
    /// [`Self::Accumulate`] by a keyword (not a list) after `<-`.
    FactBind {
        // rune:purgare(trait-contract) — classifier names the bound `?p`; alpha_pattern
        // still owns `fact_var` for the keyword-headed twin until that consumer switches.
        #[allow(dead_code)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
/// The table is held honest by `every_constraint_head_is_a_real_rete_row`, which checks each
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

