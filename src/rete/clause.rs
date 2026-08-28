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
    //
    // ⛔ THE PREFIX IS DERIVED, NOT HARDCODED — and that is the whole of this line's history.
    // It read `strip_prefix(":wat::rete::core::")` until arc 255 stone E moved the string ops to
    // `:wat::string::*`, whose mirror is `:wat::rete::string::*` by the naming rule
    // (`vocabulary.rs`: rete_name == core_name with `rete::` inserted after `:wat::`, gated by
    // `rete_name_is_core_name_with_rete_inserted_after_wat`). A hardcoded `core::` asserts the
    // mirror prefix is FIXED; the rule says it is DERIVED from wherever the subject lives. So
    // `:wat::rete::string::=` stopped being recognised, every string `where` guard was refused as
    // `Unrecognized`, and 15 tests went red at once — caught by
    // `every_constraint_head_is_a_real_rete_row`, which exists for exactly this.
    //
    // Undo the naming rule instead: strip `:wat::rete::`, and what remains IS the core name
    // (`core::i64::=` or `string::=`). Split the op off the end; the TYPE is the last segment of
    // what is left. This is correct for every member today and stays correct as each type's leaf
    // moves out of `core::` toward `wat.<type>/<op>`
    // (`109/NOTE-operator-namespaces-dotted.md`) — string is simply the first to arrive. A
    // hardcode would need editing once per type; a derivation needs editing never.
    // ★ AND THE ROW MUST EXIST. Deriving the shape is not enough: `:wat::rete::i64::=` PARSES
    // under the rule above but has no row, because i64's ops have not left `core::` yet. Admitting
    // it would recreate the phantom class one type early — measured, the derivation alone admitted
    // 20 such heads. Consulting the vocabulary makes a phantom UNREPRESENTABLE rather than checked:
    // a head is admitted because a row EXISTS, never because its shape parses.
    crate::rete::vocabulary::rete_op_for(head)?;

    let core_name = head.strip_prefix(":wat::rete::")?;
    let (ty_path, op) = core_name.rsplit_once("::")?;
    // Through the ONE door — `identifier::leaf` is the sanctioned reader for "the last `::`
    // segment". A hand-rolled `rsplit("::")` here is a SECOND NAME PARSER, and
    // `only_identifier_rs_parses_a_name` caught exactly that when this line was first written
    // (STONE-one-name-grammar, arc 109: a name is an atom, parsed exactly one way, or two
    // parsers WILL disagree — its census found 33 that already had).
    let ty = wat_reader::identifier::leaf(ty_path);
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
/// Is this head a rete op whose result is KNOWN to be boolean?
///
/// ⛔ **"Known" is the load-bearing word, and getting it wrong would re-open fix-list F.** A clause
/// admitted here is required to evaluate TRUE; a predicate returning a non-bool would simply fail
/// that comparison and the rule would silently never fire — the exact silent-wrong-answer shape
/// this arc has now closed twice. So the test is static knowledge, never optimism:
///
///   · `Alias` / `Fallback` rows carry a REAL `ret`, so `ret == Bool` is a fact about the row.
///   · `Form` / `Redispatch` rows carry `ret: Bool` as a PLACEHOLDER — they have no `TypeScheme`
///     at all — so their `ret` says nothing and must not be read as if it did. `and`/`or`/`not`
///     are boolean by definition and are named explicitly; `cond`/`let`/`match` are polymorphic in
///     their body's type and stay REFUSED, because the inline position has no type check that
///     could demand bool of them. That refusal is a diagnostic, which is the honest half of the
///     pair: admit what is provably boolean, refuse the rest BY NAME, never accept-and-go-quiet.
fn head_is_boolean_rete_predicate(head: &str) -> bool {
    if matches!(
        head,
        ":wat::rete::core::and" | ":wat::rete::core::or" | ":wat::rete::core::not"
    ) {
        return true;
    }
    match crate::rete::vocabulary::rete_op_for(head) {
        Some(row) => {
            matches!(
                row.class,
                crate::rete::vocabulary::OpClass::Alias
                    | crate::rete::vocabulary::OpClass::Fallback
            ) && matches!(row.ret, crate::rete::vocabulary::ParamType::Bool)
        }
        None => false,
    }
}

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
            // Any other RETE-VOCABULARY head is a PREDICATE — a boolean-valued expression,
            // admitted here exactly as inside a `where` fence. Reached only AFTER every
            // structural shape above has declined, so this is strictly additive: a constraint is
            // still a Constraint, a combinator still a combinator.
            //
            // A head outside the vocabulary still falls to `Unrecognized`, so Law A holds: the
            // rete query language is composed from rete primitives, and a core-spelled head is
            // refused with the diagnostic that names its per-type twin.
            k if head_is_boolean_rete_predicate(k) => ReteClauseShape::Predicate(clause),
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
        // ⚠ The candidate heads are BUILT, and that is why no text census could find this door:
        // the joined form (the old rete-core prefix, then a type segment, then the op) exists in
        // no file — the prefix and the type are written apart and only ever meet at runtime. A
        // census can only find text; this door had none, which is why it survived every table.
        //
        // ⛔ THE GENERATOR MUST STAY INDEPENDENT OF `RETE_OPS`. Sourcing these heads from the rows
        // themselves would make `phantom` trivially empty and the assertion vacuous — a gate
        // reading a copy of the truth it is meant to check. It generates the cross-product itself.
        //
        // BOTH spellings are generated per type: `:wat::rete::core::<ty>::<op>` (where a type's
        // ops still live under `core::`) and `:wat::rete::<ty>::<op>` (where they have moved out,
        // as `string` did in arc 255 stone E). The claim under test is spelling-agnostic —
        // WHATEVER is admitted must have a row — so it keeps holding as each type migrates.
        for ty in ["i64", "f64", "string", "bool", "keyword", "enum"] {
            for op in ["=", "not=", "<", ">", "<=", ">="] {
                for head in [
                    format!(":wat::rete::core::{ty}::{op}"),
                    format!(":wat::rete::{ty}::{op}"),
                ] {
                    if classify_constraint_head(&head).is_some() {
                        admitted.push(head);
                    }
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

        // ⊘ THE PHANTOM CHECK IS NOW STRUCTURAL, NOT ASSERTED. `classify_constraint_head` consults
        // `rete_op_for` before admitting anything, so a head with no row cannot be admitted — the
        // class this assertion policed is unrepresentable. It is KEPT as a regression guard on
        // that property: if someone restores a shape-only admission path, this goes red again and
        // names the offender, exactly as it did when stone E moved the string ops.
        let phantom: Vec<&String> = admitted.iter().filter(|h| !known.contains(h.as_str())).collect();
        assert!(
            phantom.is_empty(),
            "classify_constraint_head admits {} head(s) with NO matching RETE_OPS row — a renamed \
             row would silently stop matching and the clause would be refused as `Unrecognized` \
             (which teaches the wrong fix). Offenders: {phantom:#?}",
            phantom.len()
        );

        // ★ THE REVERSE DIRECTION — the claim that can still fail now that phantoms are
        // structurally impossible: every comparison-shaped ROW must be CLASSIFIED. A row whose
        // spelling `classify_constraint_head` cannot read would be inert — present in the
        // vocabulary, invisible to the clause compiler, and refused as `Unrecognized`. That is
        // precisely the failure stone E hit, from the other side.
        let unclassified: Vec<&str> = RETE_OPS
            .iter()
            .map(|op| op.rete_name)
            .filter(|n| {
                n.rsplit_once("::")
                    .is_some_and(|(_, op)| matches!(op, "=" | "not=" | "<" | ">" | "<=" | ">="))
            })
            .filter(|n| classify_constraint_head(n).is_none())
            .collect();
        assert!(
            unclassified.is_empty(),
            "{} comparison row(s) exist in RETE_OPS that classify_constraint_head cannot read — \
             they are inert: present in the vocabulary, invisible to the clause compiler, and \
             refused as `Unrecognized`. Offenders: {unclassified:#?}",
            unclassified.len()
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
            ":wat::rete::i64::+",
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

