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
    ///
    /// `field_kw` is the `:field` KEYWORD NODE, carried alongside its own colon-stripped text.
    /// The classifier held that node all along and dropped it (`keyword_payload(&items[2])`), so
    /// the wall's `UnknownField` for a bad bind had no field span to report and fell back to the
    /// whole clause's. `check_field_kw` takes the node, not a `Span` — see its doc — so this is
    /// the field that makes the right caret reachable from the bind path at all.
    Bind { var: &'a str, field: &'a str, field_kw: &'a WatAST },
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
        #[allow(dead_code)] // grammar payload; consumers walk `from` only
        var: &'a str,
        // rune:purgare(trait-contract) — same grammar payload; fire reads acc-form
        // off the AccumulateNode, not this parse field.
        #[allow(dead_code)] // grammar payload; fire reads acc-form off AccumulateNode
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
    /// **A BOOLEAN-VALUED RETE EXPRESSION, written where a constraint goes.**
    ///
    /// Admitted 2026-08-28. Before it, the inline position took a fixed SHAPE SET and refused
    /// everything else, so `(String/empty? :v)`, `(String/contains? :v "x")`,
    /// `(PersistentVector/contains? …)`, `core::and/or/not`, `cond`, `let` and `match` were all
    /// `MalformedClause` inline while working inside a `where` fence — the identical predicate,
    /// two answers by position.
    ///
    /// ⛔ **THE STATED REASON FOR THAT SPLIT WAS FALSE, AND IT WAS MINE.** I argued it as
    /// indexability: shapes let the alpha discrimination tree pre-filter, expressions do not. The
    /// disk refutes it — `alpha_tree.rs` indexes ONLY provable equality discriminators, and says
    /// so: *"(`< > <= >=`) contribute no discriminator and ride the wildcard edge"*. Ordering
    /// comparisons are admitted inline TODAY and already ride that edge. None of the 14 refused
    /// shapes could ever have been an equality discriminator either, so admitting them costs
    /// nothing that admitted forms are not already costing.
    ///
    /// What was left was a rule no reader could infer: `(i64::> :v 5)` in, `(String/empty? :v)`
    /// out, both boolean predicates over a field, neither indexed. One rule replaces the list —
    /// *an inline constraint is any rete expression returning bool*, which is what the fence
    /// already says.
    ///
    /// Carries the WHOLE clause: the compiler lowers it through the one expression core and the
    /// interpreter evaluates it the same way, so there is no second grammar to keep in step.
    Predicate(&'a WatAST),
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
/// Is this rete expression's result PROVABLY boolean, by shape alone?
///
/// ⛔ **"Provably" is the load-bearing word, and getting it wrong would re-open fix-list F.** A
/// clause admitted here is required to evaluate TRUE; a predicate returning a non-bool would
/// simply fail that comparison and the rule would silently never fire — the exact
/// silent-wrong-answer shape this arc has now closed three times. So the test is static
/// knowledge, never optimism.
///
/// ⚠ **THIS USED TO ASK ONLY THE HEAD, AND SO REFUSED FORMS IT COULD HAVE PROVEN.** It read
/// `row.ret` and stopped — which is a fact for `Alias`/`Fallback` rows and a PLACEHOLDER for
/// `Form`/`Redispatch` ones, so `cond`/`let`/`match`/`if` were refused with the stated reason
/// *"polymorphic in their body's type… the inline position has no type check that could demand
/// bool of them."* **That reason was wrong.** Polymorphic-in-the-body means the type is a
/// function OF THE BODY — and the body is right there in the AST. The builder's ruling,
/// 2026-08-28: *"we very carefully crafted rete's DSL to ensure every form a user can express can
/// be compiled… I see no reason why forms like what we're describing cannot be supported… we just
/// inappropriately denied access, poorly, to tooling we fully intended to support."*
///
/// **Why the proof is decidable HERE, in a pass with no env.** Rete's vocabulary is closed, and
/// every row is `pure · deterministic · total` (`every_rete_row_is_total` makes a non-total row a
/// red build). Totality means every op is defined on its whole domain, so no supported expression
/// can fail to have a value; purity and determinism mean that value depends only on the inputs, so
/// the TYPE is a function of the subexpression types — all of which are in this AST. Nothing here
/// needs a registry, which is what keeps `classify_rete_clause`'s "by SHAPE alone" contract intact.
///
/// The rules, each one the type rule of the form it names:
///   · `and` / `or` / `not`        — boolean by definition.
///   · an `Alias`/`Fallback` row   — its REAL `ret`, a fact about the row.
///   · `if`                        — bool iff BOTH branches are. A `Form` row's `ret` is a
///                                   placeholder and is never read.
///   · `let`                       — its BODY's type.
///   · `match`                     — bool iff EVERY arm's body is.
///   · a `bool` literal            — itself.
///
/// `cond` needs no arm: it is `RETE_OPS`' one MACRO-BACKED row and expands to nested
/// `:wat::rete::core::if` (`vocabulary.rs`) before any classification runs, so the `if` rule
/// covers it. Anything this cannot prove stays REFUSED — a diagnostic, never accept-and-go-quiet.
fn expr_is_provably_boolean(ast: &WatAST) -> bool {
    match ast {
        WatAST::BoolLit(..) => true,
        WatAST::List(items, _) => {
            let Some(WatAST::Keyword(head, _)) = items.first() else { return false };
            match head.as_str() {
                ":wat::rete::core::and" | ":wat::rete::core::or" | ":wat::rete::core::not" => true,
                // `(if c then else)` — both branches, or the form is not provably anything.
                ":wat::rete::core::if" => {
                    items.len() == 4
                        && expr_is_provably_boolean(&items[2])
                        && expr_is_provably_boolean(&items[3])
                }
                // `(let [binds…] body)` — the BODY is the type. `last()` rather than `[2]` so a
                // multi-form body answers on the form whose value the `let` actually yields.
                ":wat::rete::core::let" => {
                    items.len() >= 3 && items.last().is_some_and(expr_is_provably_boolean)
                }
                // `(match subject (pattern body)…)` — every arm, because any one of them can be
                // the one that runs. An arm that is not a `(pattern body)` list is not provable.
                ":wat::rete::core::match" => {
                    items.len() >= 3
                        && items[2..].iter().all(|arm| match arm {
                            WatAST::List(a, _) if a.len() >= 2 => {
                                a.last().is_some_and(expr_is_provably_boolean)
                            }
                            _ => false,
                        })
                }
                // Every other head: the row's DECLARED `ret`, believed as it stands.
                //
                // ⛔ THERE IS NO `class` TEST HERE ANY MORE, AND ITS ABSENCE IS THE POINT. This
                // used to read `class is Alias|Fallback AND ret is Bool`, because on a
                // `Form`/`Redispatch` row `ret: ParamType::Bool` was a PLACEHOLDER meaning "no
                // scheme". That convention refused `:wat::rete::holon::coincident?` as an inline
                // constraint — a row that genuinely returns bool, refused because the
                // representation could not say so without also saying "placeholder" (2026-08-28;
                // it worked in a `where` fence the whole time, which is how it stayed invisible).
                //
                // `Ret::NoScheme` now carries the placeholder, so `Ret::Is(Bool)` is a FACT from
                // any class and this test asks the one question it means to ask. Widening the old
                // guard to admit `Redispatch` instead was tried and is UNSOUND — it lets
                // `Tuple/first` (an `i64`, whose row also said `ret: Bool`) through as a
                // constraint that silently matches nothing.
                other => match crate::rete::vocabulary::rete_op_for(other) {
                    Some(row) => matches!(
                        row.ret,
                        crate::rete::vocabulary::Ret::Is(crate::rete::vocabulary::ParamType::Bool)
                    ),
                    None => false,
                },
            }
        }
        _ => false,
    }
}

/// THE grammar: one clause form → one `ReteClauseShape`. The single parser every rete consumer
/// shares.
///
/// Compilation, validation, stratification and purity all route through here rather than
/// re-deriving "what shape is this form" from the raw `WatAST` — that is the point of the shape
/// enum, and a second parser anywhere would be two grammars that can disagree about the same
/// program.
///
/// A non-list or empty form is `Unrecognized` rather than a panic or a guess. That arm is
/// load-bearing: a bare keyword is exactly the injected-`:celsius` corruption the wall exists to
/// catch, and classifying it as "not a shape" is what lets the caller report it instead of
/// matching it as something.
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
                            return ReteClauseShape::Bind { var: var_name, field, field_kw: &items[2] };
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
            // Any other RETE-VOCABULARY head is a PREDICATE — a boolean-valued expression,
            // admitted here exactly as inside a `where` fence. Reached only AFTER every
            // structural shape above has declined, so this is strictly additive: a constraint is
            // still a Constraint, a combinator still a combinator.
            //
            // A head outside the vocabulary still falls to `Unrecognized`, so Law A holds: the
            // rete query language is composed from rete primitives, and a core-spelled head is
            // refused with the diagnostic that names its per-type twin.
            _ if expr_is_provably_boolean(clause) => ReteClauseShape::Predicate(clause),
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

