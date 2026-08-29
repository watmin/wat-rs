//! Native type-stratum numbering (`native_stratify` / `rule_produces` / `rule_negates`).
//! Dual of `wat/rete/oracle/stratify.wat`. The public `fire-rules` door lives in `fire/`.

use std::collections::HashMap;

use crate::ast::WatAST;
use crate::rete::clause::{classify_rete_clause, ReteClauseShape};
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};

// ── Arc 278 Stone 7-strat-native: STRATIFIED negation, native port ──────────────
//
// Faithful Rust port of the wat ORACLE's stratification (`wat/rete/oracle/stratify.wat`):
// `rule-produces` / `rule-negates` / `stratify-sweep` / `stratify-fix` / `rule-stratum` /
// `stratify` / `fire-stratified-loop` / `fire-stratified`. The oracle is the reference and
// does NOT change (`DESIGN-STONE-7strat-native.md`); this is a SEPARATE, self-contained Rust
// impl that moves in lockstep with it (the dual-impl doctrine — no `native?` flag anywhere).

/// A fact-form's type head, colon-stripped: `(:Type ...)` → `"Type"`.
/// Mirrors the inline `ast-name` + colon-strip done identically in both `rule-produces`
/// (`wat/rete/oracle/stratify.wat`) and `rule-negates` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn fact_type_head(fact_form: &WatAST) -> Option<String> {
    if let WatAST::List(items, _) = fact_form {
        let raw = match items.first() {
            Some(WatAST::Keyword(k, _)) => k.clone(),
            Some(WatAST::Symbol(s, _)) => s.as_str().to_string(),
            _ => return None,
        };
        return Some(raw.trim_start_matches(':').to_string());
    }
    None
}

/// Extract the produced type FQDNs from a Rule's RHS forms.
/// Arc 278 Stone A: each RHS form IS the fact-form directly (the `:wat::rete::insert` wrapper
/// is gone) — no more unwrapping a second child. Mirrors `rule-produces` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn rule_produces(rhs: &[WatAST], sym: &SymbolTable) -> Vec<String> {
    let mut out = Vec::new();
    for form in rhs {
        if let Some(name) = produced_type(form, sym) {
            out.push(name);
        }
    }
    out
}

/// Constructor head stays the class. A fn-headed `:then` produces its
/// declared return type (the fact `T` another rule can consume).
pub(crate) fn produced_type(form: &WatAST, sym: &SymbolTable) -> Option<String> {
    let head = fact_type_head(form)?;
    let path = if head.starts_with(':') {
        head.clone()
    } else {
        format!(":{head}")
    };
    if let Some(func) = sym.get(&path) {
        if let crate::types::TypeExpr::Path(p) = &func.ret_type {
            let t = p.trim_start_matches(':');
            if !t.is_empty() && !t.starts_with("wat::core::") {
                return Some(t.to_string());
            }
        }
    }
    Some(head)
}

/// Extract the negated type FQDNs from a Rule's LHS conditions.
/// `(:not <fact>)` and `(:not (:and/:or …))` both raise: the leaf types under
/// the combinator are the edges, not `"wat::rete::and"`. Walk via
/// `classify_rete_clause`. Positive `:exists` / accumulate / `:where` are not
/// negation edges (those are `rule_consumes`).
pub(crate) fn rule_negates(lhs: &[WatAST]) -> Vec<String> {
    let mut out = Vec::new();
    for form in lhs {
        negate_types(form, &mut out, false);
    }
    out
}

pub(crate) fn negate_types(form: &WatAST, out: &mut Vec<String>, under_not: bool) {
    match classify_rete_clause(form) {
        // A predicate NEGATES no fact type: it is an expression over bindings, never a pattern,
        // so it can neither be the thing a `not` excludes nor introduce one.
        ReteClauseShape::Predicate(_) => {}
        ReteClauseShape::Not(inner) => negate_types(inner, out, true),
        ReteClauseShape::And(xs) | ReteClauseShape::Or(xs) => {
            for x in xs {
                negate_types(x, out, under_not);
            }
        }
        ReteClauseShape::FactBind { type_head, .. } if under_not => {
            out.push(type_head.to_string());
        }
        ReteClauseShape::Unrecognized if under_not => {
            if let Some(name) = fact_type_head(form) {
                if !name.starts_with('?') && !name.starts_with("wat::rete::") {
                    out.push(name);
                }
            }
        }
        _ => {}
    }
}

/// The stratifier's dependency view of one rule.
/// `consumed` is task #94 — without it a rule that reads a higher-stratum fact sits too low.
/// `exists_and_from_types` is exists-inner / acc `:from` (+1 like negation when the type is derived).
#[derive(Clone, Debug)]
pub(crate) struct StratifyView {
    pub produced: Vec<String>,
    pub negated: Vec<String>,
    pub consumed: Vec<String>,
    pub exists_and_from_types: Vec<String>,
}

/// A compiled rule paired with its stratify view.
#[derive(Clone)]
pub(crate) struct RuleParts {
    pub rule: Value,
    pub view: StratifyView,
}

/// The fact types a rule reads POSITIVELY (task #94 — the input the stratifier never had).
///
/// Correct stratification needs BOTH `stratum(r) >= stratum(p)` for positively-used `p` and
/// `stratum(r) > stratum(p)` for negated `p`. Only the second existed, so a rule consuming a
/// fact produced in a HIGHER stratum was left LOWER, fired before its input existed, and never
/// re-fired. `:not` / `:where` are not positive reads. `:exists` inner and accumulate
/// `:from` ARE — they were dropped as engine-form prefixes and the `:from` head
/// leaked as `"?n"`. Walk via `classify_rete_clause`.
pub(crate) fn rule_consumes(lhs: &[WatAST]) -> Vec<String> {
    let mut out = Vec::new();
    for form in lhs {
        consume_types(form, &mut out);
    }
    out
}

/// Exists-inner and accumulate `:from` types. Stratify +1 (closed bag).
pub(crate) fn rule_bag_consumes(lhs: &[WatAST]) -> Vec<String> {
    let mut out = Vec::new();
    for form in lhs {
        bag_types(form, &mut out);
    }
    out
}

pub(crate) fn bag_types(form: &WatAST, out: &mut Vec<String>) {
    match classify_rete_clause(form) {
        ReteClauseShape::Exists(inner) => consume_types(inner, out),
        ReteClauseShape::Accumulate { from, .. } => consume_types(from, out),
        ReteClauseShape::And(xs) | ReteClauseShape::Or(xs) => {
            for x in xs {
                bag_types(x, out);
            }
        }
        _ => {}
    }
}

pub(crate) fn consume_types(form: &WatAST, out: &mut Vec<String>) {
    match classify_rete_clause(form) {
        // A predicate CONSUMES no fact type: it is an expression over bindings this condition
        // already made, never a pattern reaching into another rule's derivations. It contributes
        // nothing to the produces->consumes graph stratification is built from.
        ReteClauseShape::Predicate(_) => {}
        ReteClauseShape::Exists(inner) => consume_types(inner, out),
        ReteClauseShape::Accumulate { from, .. } => consume_types(from, out),
        ReteClauseShape::And(xs) | ReteClauseShape::Or(xs) => {
            for x in xs {
                consume_types(x, out);
            }
        }
        ReteClauseShape::FactBind { type_head, .. } => {
            out.push(type_head.to_string());
        }
        ReteClauseShape::Not(_)
        | ReteClauseShape::Where(_)
        | ReteClauseShape::Bind { .. }
        | ReteClauseShape::Constraint { .. } => {}
        ReteClauseShape::Unrecognized => {
            if let Some(name) = fact_type_head(form) {
                if !name.starts_with('?') {
                    out.push(name);
                }
            }
        }
    }
}

/// One sweep over all rules' (produced, negated, consumed) triples, raising `type_strata` entries.
/// For each rule: `required = max(stratum[n]+1 for n in negated, default 0)`; for each produced
/// type `p`: `stratum[p] = max(stratum[p], required)`. Returns `true` iff any stratum rose.
/// Mirrors `stratify-sweep` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn native_stratify_sweep(rule_parts: &[StratifyView], type_strata: &mut HashMap<String, i64>) -> bool {
    let mut changed = false;
    for view in rule_parts {
        let mut required = 0i64;
        for n in &view.negated {
            let v = *type_strata.get(n).unwrap_or(&0) + 1;
            if v > required {
                required = v;
            }
        }
        // exists / acc :from of a type THIS SET derives: +1 (closed bag).
        // Inserted-only bag types stay +0 so the unstratified path survives.
        // A rule that both produces and bags `b` (userfn-head gather that
        // returns the same type) is a self-cycle — do not count it as derived.
        for b in &view.exists_and_from_types {
            let derived = rule_parts.iter().any(|other| {
                other.produced.iter().any(|t| t == b) && !other.exists_and_from_types.iter().any(|t| t == b)
            });
            let v = *type_strata.get(b).unwrap_or(&0) + i64::from(derived);
            if v > required {
                required = v;
            }
        }
        // req-pos: a positive consumer may share its input's stratum but never sit BELOW it.
        // NOT +1 — same-stratum forward chaining is ordinary and must stay allowed.
        for c in &view.consumed {
            let v = *type_strata.get(c).unwrap_or(&0);
            if v > required {
                required = v;
            }
        }
        for p in &view.produced {
            let cur = *type_strata.get(p).unwrap_or(&0);
            if required > cur {
                type_strata.insert(p.clone(), required);
                changed = true;
            }
        }
    }
    changed
}

/// Recursive fixpoint for stratification: sweeps until converged or `remaining` runs out.
/// A negation cycle (non-terminating strata) raises the same "not stratifiable" error the
/// oracle raises. Mirrors `stratify-fix` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn native_stratify_fix(
    rule_parts: &[StratifyView],
    mut type_strata: HashMap<String, i64>,
    mut remaining: i64,
) -> Result<HashMap<String, i64>, EvalBreak> {
    loop {
        let changed = native_stratify_sweep(rule_parts, &mut type_strata);
        if !changed {
            return Ok(type_strata);
        }
        if remaining <= 0 {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::rete::fire-rules".into(),
                    reason: "stratify: negation cycle detected — rule set is not stratifiable"
                        .into(),
                },
            )
            .into());
        }
        remaining -= 1;
    }
}

/// Compute the type→stratum map for a rule set (`length(rules)+1` sweeps is always enough for
/// a stratifiable set — same bound the oracle uses). Mirrors `stratify` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn native_stratify(rule_parts: &[StratifyView]) -> Result<HashMap<String, i64>, EvalBreak> {
    let bound = rule_parts.len() as i64 + 1;
    native_stratify_fix(rule_parts, HashMap::new(), bound)
}

/// A single rule's stratum given the final type-strata:
/// `max(max strata[p] for produced p, max strata[n]+1 for negated n)`.
/// Mirrors `rule-stratum` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn native_rule_stratum(
    produced: &[String],
    negated: &[String],
    type_strata: &HashMap<String, i64>,
) -> i64 {
    let from_p = produced
        .iter()
        .map(|p| *type_strata.get(p).unwrap_or(&0))
        .max()
        .unwrap_or(0);
    let from_n = negated
        .iter()
        .map(|n| *type_strata.get(n).unwrap_or(&0) + 1)
        .max()
        .unwrap_or(0);
    from_p.max(from_n)
}

// ── THE TERMINATION VERIFIER ────────────────────────────────────────────────────────────────
//
// `RETE-OPEN-WORK.md` § 4.2, and the builder's framing: rete should be like the kernel's eBPF
// verifier. It already IS, for everything except termination — `validate_rete_rules` refuses
// unregistered fact types, unrecognised clause shapes, unreal field-refs, non-rete constraints and
// unconsumed `:not` binds, and `stratify` above refuses un-stratifiable sets outright. Termination
// was the hole, and it was not theoretical: 11 lines of legal wat killed the process on
// `memory allocation of 545259536 bytes failed`, with no wat error and no rule named.
//
// THE RULE. Datalog terminates because its fact domain is FINITE — every head value comes from the
// body, so no rule can mint a value that was not already there. That property is RANGE
// RESTRICTION. A `:then` that COMPUTES a value breaks it, and inside a derivation CYCLE that means
// a structurally novel fact every round, forever. So:
//
//     a COMPUTED head inside a positive produces->consumes CYCLE is refused, named, at compile.
//
// eBPF refuses an unbounded loop; this refuses an unbounded derivation. Outside a cycle a computed
// head is FINE and stays legal — `(:Celsius :c (- ?f 32))` derives once and stops, which is why
// the check is about the cycle and not about arithmetic.
//
// ★ NO ESCAPE HATCH, by two builder rulings. A `rune:` marker was proposed and refused ("no magic
// comments"), then a data form on the `Rule` record — `Termination::Asserted [why <- String]` —
// was refused in turn: *"so.... we allow users to make mistakes that they own?... their strings
// are their reason for themselves?"* Correct: an author's string is not a proof, and taking one as
// a termination guarantee would mint exactly the unchecked exemption `excusare` exists to hunt.
// With no opt-out there is nothing to declare, so `Rule` needs no new field and its 60 hand-built
// construction sites are untouched. If a bounded pattern must exist later, the answer is a FORM
// the verifier can CHECK — eBPF's `bpf_loop()` move, the bound as a verified argument — never a
// promise it must trust.
//
// WHERE IT RUNS, AND WHY NOT THE `defrule` WALL. At `arm-session`, which `compile-all` calls for
// every session. The freeze-time wall sees DECLARED rules only; rules built at runtime as `Rule`
// values (both differential fuzzers do this) bypass it entirely. `compile-all` is the one door
// every rule passes, which is the same reason the declaration had to be data rather than a comment.
//
// WHAT IT CANNOT SEE, stated rather than left as a silent hole:
//   - An imported Export carries no rule AST (`rules_lack_ast`), so there is nothing to analyse.
//     That is where the runtime round cap keeps earning its place — the path where static proof is
//     unavailable, rather than a general apology for not having proof.
//   - A GUARDED COUNTER is refused THOUGH IT TERMINATES — the third hole, named here 2026-08-28
//     after an integration-branch report pointed out it was recorded in exactly one place: the
//     prose header of an unrelated fixture (`probe_arc278_fixpoint_round_cap_deep.wat`), where
//     nothing would ever find it again.
//
//         N(k+1) :- N(k), (where (< ?k 500))      -> REFUSED. Terminates at k=500.
//
//     The cyclicity test below is purely STRUCTURAL — reachability over fact-type edges — and does
//     not read the `where` fence. Proving this one terminates needs monotonicity analysis plus
//     comparison-direction reasoning against a literal, which the first cut deliberately punted.
//     The refusal is therefore correct BY THIS VERIFIER'S OWN CLAIM ("proves the absence of ONE
//     unbounded-derivation shape"), and the cost is that a bounded counter — the first thing most
//     people write in recursive Datalog-with-arithmetic — meets a hard compile refusal.
//
//     ⛔ THE ANSWER IS NOT AN ESCAPE HATCH; both were already refused above, and both rulings
//     stand. It is a FORM THE VERIFIER CAN CHECK — eBPF's `bpf_loop()` posture, the bound as an
//     argument it READS rather than a promise it trusts. Tracked, with the open design questions,
//     at `RETE-OPEN-WORK.md` § "The order" item 9. Zero programs in the corpus trip this today,
//     which is exactly when the class is cheapest to widen.
//
//   - A fn-headed `:then` is opaque: `(:my::mk-fact ?k)` may compute anything inside `mk-fact`,
//     so a cycle through one is NOT proven terminating. It is nonetheless ADMITTED, and that is a
//     deliberate narrowing rather than an oversight.
//
//     The first cut refused it, on the reasoning that "cannot see inside" and "proved safe" are
//     different claims — which is true. The FLOOR measured what that costs: `:then` fn-heads are a
//     shipped, deliberate feature (arc 278 Stone B widened an item's head to "a fn whose declared
//     return type is a fact type"), and `probe_arc278_then_user_forms` exercises exactly a cyclic
//     one. Refusing it would delete a working capability on a guess, to close a hole nobody has
//     fallen into, while the shape actually MEASURED to kill the process — a computed ARGUMENT —
//     is refused either way.
//
//     ⛔ WHAT THIS PARAGRAPH USED TO SAY WAS WRONG, AND IT CONTRADICTED `rete_fn_body_mints`'s own
//     doc-comment twenty lines below it. Struck 2026-08-28, re-measured by driving all three rows.
//
//     It read: "the hole is narrower than this paragraph first claimed — three attempts to mint
//     were each refused by a DIFFERENT pre-existing fence", offering a table of three fences and
//     concluding "no exploit found, the shape is guarded by adjacent fences." Two of its three
//     rows were false, and the conclusion was superseded the same day it was written:
//
//         computes with core `i64::+`      -> "is not total".  TRUE, and a genuine BODY fence.
//         computes with the total fallback -> "is not a rete primitive".  MIS-ATTRIBUTED. That
//           refusal names the FN (`:bc::mk-next`), not the body's op — it is Law A refusing a fn
//           that was never declared `:wat::rete::core::defn`. All three attempts used a plain
//           `:wat::core::defn`, so the table measured ONE door three times and read it as three.
//         constructs a record at all       -> "`kwargs-construct` is not pure".  FALSE today. A
//           rete defn whose body is `(:bc::N :k (…::i64::+ k 1 :undefined 0))` is ADMITTED — it
//           declares clean and reaches the cyclicity check below. Driven 2026-08-28.
//
//     THE EXPLOIT EXISTS AND WAS RUN. Declare the fn `:wat::rete::core::defn` and the minting body
//     compiles clean and derives forever — measured 2026-08-27, which is exactly what
//     `rete_fn_body_mints` (below) was written to catch and what
//     `probe_arc278_termination_fn_head.wat` now pins. The paragraph survived because its three
//     probes failed for an unrelated reason and it read that as safety: a refusal is evidence
//     about the door you knocked on, not about the room behind it.
//
//     THE HONEST CLAIM, then: the minting shape is CAUGHT — by `rete_fn_body_mints`, which walks
//     the fn body for a constructor carrying a computed argument — not merely fenced off by
//     accident. What is still not proven is the general case: a fn body can compute in ways that
//     walk does not model. What this verifier proves is the absence of ONE unbounded-derivation
//     shape. The runtime round cap still stands behind both this and Export's missing AST.

/// Every type the rule's `:then` derives, paired with the rule's `:when` types — the edge
/// `consumed -> produced` this rule contributes to the derivation graph.
struct RuleEdge {
    name: String,
    produced: Vec<String>,
    consumed: Vec<String>,
    /// The `:then` form that computes rather than copies, if any, with its own span for the error.
    computed: Option<(String, crate::span::Span)>,
    /// `Some(n)` when that form's COMPUTED fields are all finite-typed and their product is `n`,
    /// under [`MAX_PROVABLE_FACT_POPULATION`]. A cycle through such a form terminates after at
    /// most `n` distinct facts, so it is admitted. `None` is "this analysis cannot bound it" —
    /// never "unbounded"; see [`domain_cardinality`].
    provably_finite: Option<u128>,
}

/// Does a rete fn's BODY construct a fact from a computed value?
///
/// THE HOLE THIS CLOSES, and it was demonstrated before it was fixed. The item-level check below
/// inspects the `:then` ITEM only — so `(:my::bump ?n)` reads as "all arguments are bound
/// variables" and passes, while `:my::bump`'s body does
/// `(:my::N :k (:wat::rete::core::i64::+ (:my::N/k n) 1 :undefined 0))` and mints a novel fact
/// every round. Measured 2026-08-27: it compiled clean and ran to the round cap.
///
/// The `:then` head must be a RETE fn (`:wat::rete::core::defn`) to be admitted at all — a plain
/// `:wat::core::defn` is refused by `then-item-fence`'s Law A conjunct as "not a rete primitive".
/// That door is why three earlier attempts at this exploit failed for the WRONG reason and briefly
/// convinced me the hole was already guarded; it was not.
///
/// WHAT COUNTS AS A CONSTRUCTOR INSIDE THE BODY: a sub-form whose head does NOT start with
/// `:wat::` — i.e. a user type or user fn, rather than a rete/core OP. That distinction matters:
/// `probe_arc278_then_user_forms_userfn.wat`'s admitted fn is
/// `(:wat::rete::core::PersistentVector/first rs :undefined (:tf::Rate :count 0))`, whose OUTER
/// form is an op with a List argument — "computed" by the item-level test — but whose only
/// constructor, `(:tf::Rate :count 0)`, takes a literal. Testing the outer form would refuse a
/// legitimate extraction; testing constructors admits it and still refuses the minting one.
fn rete_fn_body_mints(form: &WatAST, sym: &SymbolTable) -> bool {
    let Some(head) = fact_type_head(form) else {
        return false;
    };
    let path = if head.starts_with(':') { head } else { format!(":{head}") };
    let Some(func) = sym.get(&path) else {
        return false;
    };
    // NOT gated on `func.rete.is_some()`. That guard was defensive and it silently disarmed this
    // whole check: the stamp is applied at rete-defn registration (`purity.rs`) but
    // `freeze/env.rs` documents a path that rebuilds a fresh `Function` and DROPS it. The guard
    // bought nothing either way — a non-rete fn cannot be a `:then` head at all, because
    // `then-item-fence`'s Law A conjunct refuses it as "not a rete primitive".
    let crate::value::environment::FunctionBody::Wat(body) = &func.body else {
        return false;
    };
    body_constructs_computed(body)
}

/// Walk a fn body for a CONSTRUCTOR form carrying a computed argument.
fn body_constructs_computed(ast: &WatAST) -> bool {
    if let WatAST::List(items, _) = ast {
        if let Some(head) = items.first().and_then(|h| match h {
            WatAST::Keyword(k, _) => Some(k.as_str()),
            _ => None,
        }) {
            // CONSTRUCTOR POSITIONS. A user type or user fn (`:my::N`, `:my::mk`) is one — but so
            // are the two DESUGARED heads, and missing them silently disarmed this whole check:
            // `(:my::N :k <expr>)` is kwargs SUGAR, and by the time it is a stored fn body the
            // macro has rewritten it to `(:wat::core::kwargs-construct :my::N :k <expr>)`, whose
            // head starts with `:wat::` and so read as an "op" to skip. The exploit compiled clean
            // with `computed=None` until this line named them.
            //
            // These are the same two heads `purity.rs`'s KNOWN_UNREVIEWED ratchet lists as the
            // surfaces every record-construction sugar bottoms out in — which is why there are
            // exactly two, and why a third would show up there first.
            let is_constructor = !head.starts_with(":wat::")
                || head == ":wat::core::kwargs-construct"
                || head == ":wat::core::aggregate-new";
            if is_constructor && then_form_computes(ast) {
                return true;
            }
        }
        return items.iter().any(body_constructs_computed);
    }
    false
}

/// A `:then` fact-form COMPUTES rather than copies when any argument is itself a call.
///
/// `(:N :k ?k)` copies a bound variable — range-restricted, finite domain, terminates.
/// `(:N :k (:wat::rete::core::i64::+ ?k 1 :undefined 0))` computes — the domain is now unbounded.
/// A nested CONSTRUCTOR counts too and is not a special case: `(:N :k (:Wrap ?k))` wraps one layer
/// deeper every round, which is the same unbounded structure by a different route.
///
/// Field-name keywords in kwargs position are `Keyword`, values are `Symbol`/literal/`List`, so
/// "any `List` argument" reads both call shapes without having to know which one it is looking at.
fn then_form_computes(form: &WatAST) -> bool {
    let WatAST::List(items, _) = form else {
        return false;
    };
    items.iter().skip(1).any(|a| matches!(a, WatAST::List(..)))
}

/// The largest fact population this analysis will certify as finite.
///
/// A finite domain still has to FIT. Twenty `bool` fields is 2^20 facts — provably terminating and
/// an allocator abort all the same, which is the exact failure this item exists to stop. So the
/// product of cardinalities is capped, and over the cap the rule is refused as "too large to
/// prove" rather than admitted on a technicality.
///
/// ── WHY 1_000_000, MEASURED ────────────────────────────────────────────────────────────────────
///
/// ⚠ **This was 10_000 for one commit, chosen to "match `DEFAULT_MAX_FIRE_ROUNDS`". That was
/// symmetry, not evidence, and the two answer different questions** — a round cap bounds WORK per
/// fire, this bounds STATE. Conflating them is the same mistake the eBPF comparison exists to
/// avoid: the kernel bounds a program's per-invocation work tightly (33 tail calls, a 512-byte
/// stack) while the maps it reads are enormous (`with_max_entries(5_000_000)` in our own XDP
/// scrubber). Small step budget, large state budget. 10_000 borrowed the wrong one.
///
/// **And it was below a workload we run on every push.** The grid's `fanout` axis derives
/// **40_000** facts at its top rung (`run-all.sh`, ladder `10000|20000|40000`) and is now a CI job.
/// Refusing to CERTIFY a population four times smaller than something the suite materialises
/// unbounded is incoherent — it withholds a proof from the safe case while permitting the larger
/// one unproven.
///
/// **Measured 2026-08-29** — insert N facts, derive N, read peak child RSS:
///
/// ```text
///   N=0        50_460 KB   (bare runtime)
///   N=20_000   68_968 KB
///   N=100_000 171_864 KB
///   N=400_000 548_704 KB      -> 498 MB over baseline for 800_000 facts
/// ```
///
/// **~600 bytes per fact**, stable across three sizes (474 / 622 / 623 B). That covers the fact,
/// its alpha memory, the token and the index entries — not a bare struct.
///
/// So 1_000_000 facts is **~600 MB worst case**, 25x the largest legitimate population in our own
/// corpus. It is a real commitment and it is stated rather than implied: the cap only bites on a
/// cycle whose finite fields MULTIPLY to something large (five `defenum`s of 16 variants is ~1M),
/// which is a genuine multi-dimensional state machine and not an accident.
///
/// **What would change it.** Raise it once a runtime STATE ceiling exists (item 8's other strike) —
/// exhaustion becomes catchable, so certifying more costs less. Lower it if 600 MB is too generous
/// a default — **but never below the corpus's own 40_000**, or this constant is again refusing to
/// prove what the suite already runs. A knob is deliberately NOT added: `dim_count` and
/// `max_fire_rounds` are tunable because they trade DEEP against DIVERGENT with no single right
/// answer, and this one has a measured floor and a measured cost. If it ever bites a real program,
/// that is the evidence for making it configurable — not before.
const MAX_PROVABLE_FACT_POPULATION: u128 = 1_000_000;

/// How many values can inhabit `ty`? `None` = not finite, or not finitely KNOWABLE here.
///
/// ⛔ `None` MEANS ONE THING ONLY: "this analysis cannot bound it." It is never "unbounded" as a
/// fact about the world — an `i64` field constrained by a `where` fence may well be finite, and
/// saying so is the fence half of this item, deliberately not attempted. Conflating the two is
/// this arc's most-repeated defect, so the name and this note keep them apart.
fn domain_cardinality(ty: &crate::types::TypeExpr, types: &crate::types::TypeEnv) -> Option<u128> {
    let crate::types::TypeExpr::Path(p) = ty else {
        // Parametric / fn / tuple types: a container's population is its element type's raised to
        // an unbounded length. Not finite, and not this strike's business.
        return None;
    };
    match p.as_str() {
        // The only primitive small enough to enumerate. `i64`/`f64`/`String`/`keyword` are 2^64 and
        // up — infeasible by exhaustion, which is why the fence half stays punted.
        ":wat::core::bool" => Some(2),
        _ => match types.get(p)? {
            // A `defenum`'s inhabitants are its variants — but ONLY when every variant is a unit.
            // A payload-carrying variant's population is its payload's, which reintroduces the
            // whole question one level down; refusing here keeps the analysis one level deep.
            crate::types::TypeDef::Enum(e) => e
                .variants
                .iter()
                .all(|v| matches!(v, crate::types::EnumVariant::Unit(_)))
                .then_some(e.variants.len() as u128),
            _ => None,
        },
    }
}

/// Is every COMPUTED field of this `:then` form finite-typed, with a population under the cap?
///
/// ── WHY THIS ADMITS WHAT THE CYCLICITY TEST REFUSES ──────────────────────────────────────────
///
/// Range restriction is a SYNTACTIC property — the head's value came from the body. Finiteness is a
/// TYPE property. The cyclicity test measures the first, so a fact domain of TWO is refused exactly
/// as an unbounded `i64` counter is: `(:F :flag (not ?flag))` converges after two rounds and was
/// refused for the life of the check. Measured 2026-08-29, `bool` → converges at 2, enum(3) → 2,
/// guarded i64 → 501, unguarded i64 → allocator abort.
///
/// COPIED fields are not examined and do not need to be: a copied value comes from a matched fact,
/// so it is range-restricted by construction — that is the property the whole check is built on.
/// Only a COMPUTED field can introduce a value that was not already present.
///
/// ⚠ CONSTRUCTOR-HEADED FORMS ONLY. A fn-headed `:then` hides its constructor inside the fn body
/// (`rete_fn_body_mints`), so the fields are not visible here and it stays refused. Widening to it
/// means walking the body, which is a different strike.
fn computed_fields_are_provably_finite(form: &WatAST, sym: &SymbolTable) -> Option<u128> {
    let types = sym.types()?;
    let head = fact_type_head(form)?;
    let key = format!(":{}", head.trim_start_matches(':'));
    let crate::types::TypeDef::Aggregate(def) = types.get(&key)? else {
        return None;
    };
    let WatAST::List(items, _) = form else { return None };

    // ⚠ THE `:then` FORM IS POSITIONAL BY THE TIME IT REACHES HERE, not kwargs.
    // `(:fd::F :flag (not ?b))` in source arrives as `(:fd::F (not ?b))` — `reorder_then_kwargs`
    // (`validate.rs`) has already normalised the kwargs into DECLARATION order and dropped the
    // keywords. So `items[1..]` maps 1:1 onto `def.fields`, and reading it as `:field value` pairs
    // (which is what the source looks like) finds no keyword and bails. Measured, not assumed: the
    // first cut walked pairs and silently refused everything it was written to admit.
    if items.len() != def.fields.len() + 1 {
        // Arity disagrees with the declaration — a shape this walk does not understand. An
        // unrecognized shape is NOT proof of finiteness.
        return None;
    }
    let mut population: u128 = 1;
    let mut saw_computed = false;
    for (idx, value) in items.iter().enumerate().skip(1) {
        // A COMPUTED field is a call; anything else is a literal or a bound variable, and a bound
        // variable is range-restricted by construction — the property the whole check rests on.
        if !matches!(value, WatAST::List(..)) {
            continue;
        }
        saw_computed = true;
        let (_, ty) = &def.fields[idx - 1];
        let card = domain_cardinality(ty, types)?;
        population = population.checked_mul(card)?;
        if population > MAX_PROVABLE_FACT_POPULATION {
            return None;
        }
    }
    saw_computed.then_some(population)
}

/// Refuse a rule set that cannot be proven to terminate.
///
/// See the doctrine block above. Called from `arm-session`, so it covers every rule that reaches
/// `compile-all` — declared or built at runtime.
pub(crate) fn refuse_non_terminating(
    rules: &Value,
    sym: &SymbolTable,
) -> Result<(), crate::runtime::EvalBreak> {
    let Value::wat__core__PersistentVector(pv) = rules else {
        return Ok(());
    };
    let mut edges: Vec<RuleEdge> = Vec::new();
    for r in pv.iter() {
        let Some(name) = crate::rete::kernel::session::rule_named_field(r, "name") else {
            continue;
        };
        let name = match name {
            Value::String(s) => s.to_string(),
            _ => String::from("<unnamed>"),
        };
        let lhs = crate::rete::kernel::session::rule_asts_field(r, "lhs");
        let rhs = crate::rete::kernel::session::rule_asts_field(r, "rhs");
        // An imported Export carries no AST — nothing to analyse, and saying so is the honest
        // outcome rather than passing it as proven.
        if lhs.is_empty() && rhs.is_empty() {
            continue;
        }
        // `produced_type`, NOT `fact_type_head` — the two disagree on exactly one shape and the
        // diagnostic is what pays. A fn-headed `:then` has the FUNCTION as its raw head, so
        // `fact_type_head` yields `:bc::mk-next` while `produced` (which resolves the return type
        // through `sym`) yields `:bc::N`. The message then reads "derives `:bc::mk-next` … and
        // `:bc::mk-next` feeds back into this rule's own `:when`" — naming a function that appears
        // nowhere in the `:when`, sending the reader hunting for a fact type that does not exist.
        // The DETECTION was always right; only this one string was drawn from the wrong well.
        // Driven 2026-08-28 (`RETE-OPEN-WORK` item 9, defect 2). One resolver, used by both fields.
        let computed = rhs
            .iter()
            .find(|f| then_form_computes(f) || rete_fn_body_mints(f, sym))
            .map(|f| {
                (
                    produced_type(f, sym).unwrap_or_else(|| String::from("<unknown>")),
                    f.span().clone(),
                )
            });
        let provably_finite = rhs
            .iter()
            .find(|f| then_form_computes(f) || rete_fn_body_mints(f, sym))
            .and_then(|f| computed_fields_are_provably_finite(f, sym));
        edges.push(RuleEdge {
            name,
            provably_finite,
            produced: rule_produces(&rhs, sym),
            consumed: rule_consumes(&lhs),
            computed,
        });
    }
    if edges.iter().all(|e| e.computed.is_none()) {
        // The overwhelmingly common case: nothing computes, so no cycle can be unbounded and the
        // graph never has to be built. Measured 2026-08-27: 371 of 381 corpus rules take this exit.
        return Ok(());
    }

    // ── the derivation graph, consumed -> produced ────────────────────────────────────────────
    // Transitive closure by repeated relaxation. Types number in the tens, and this runs once per
    // `compile-all` rather than per fire, so the simple fixpoint is the right shape — and it is
    // itself bounded, which would be an embarrassing place to loop forever.
    let mut reach: HashMap<String, Vec<String>> = HashMap::new();
    for e in &edges {
        for c in &e.consumed {
            let to = reach.entry(c.clone()).or_default();
            for p in &e.produced {
                if !to.contains(p) {
                    to.push(p.clone());
                }
            }
        }
    }
    loop {
        let mut grew = false;
        let keys: Vec<String> = reach.keys().cloned().collect();
        for k in keys {
            let current = reach.get(&k).cloned().unwrap_or_default();
            let mut add: Vec<String> = Vec::new();
            for t in &current {
                if let Some(next) = reach.get(t) {
                    for n in next {
                        if !current.contains(n) && !add.contains(n) {
                            add.push(n.clone());
                        }
                    }
                }
            }
            if !add.is_empty() {
                grew = true;
                reach.entry(k).or_default().extend(add);
            }
        }
        if !grew {
            break;
        }
    }

    for e in &edges {
        let Some((fact_type, span)) = &e.computed else {
            continue;
        };
        // In a cycle iff something this rule PRODUCES can reach something it CONSUMES — then
        // produced -> ... -> consumed -> produced closes the loop through this very rule. The
        // `p == c` case (a rule reading and deriving one type) is covered: `reach[p]` contains `p`
        // via this rule's own edge.
        let cyclic = e.produced.iter().any(|p| {
            e.consumed.contains(p)
                || reach
                    .get(p)
                    .is_some_and(|rs| rs.iter().any(|t| e.consumed.contains(t)))
        });
        // ── THE FINITE-DOMAIN ADMISSION ─────────────────────────────────────────────────────
        //
        // A cycle whose computed fields are all finite-typed cannot mint an unbounded stream of
        // facts: the fact population is the product of those cardinalities, dedup bites at that
        // count, and the fixpoint converges. Driven 2026-08-29 with the check disarmed —
        // `(:F :flag (not ?flag))` converged at exactly 2, the product.
        //
        // This is eBPF's `[u32; 16]` bound COMPUTED rather than declared: the ceiling comes from
        // the shape of the state, and nothing is trusted. It is not an escape hatch — the author
        // asserts nothing and there is nothing to assert. A type either has finitely many
        // inhabitants or it does not.
        //
        // ⛔ IT DOES NOT WIDEN THE `i64` AXIS, which is where the danger measured. An `i64`
        // computed field is `None` here and stays refused, so the shape that reaches an allocator
        // abort in 6.2s is untouched by this admission.
        if cyclic {
            if let Some(population) = e.provably_finite {
                let _ = population; // the bound is the PROOF; nothing downstream needs the number yet
                continue;
            }
            return Err(crate::runtime::RuntimeError::new(
                span.clone(),
                crate::runtime::RuntimeErrorKind::RuleSetMayNotTerminate {
                    rule: e.name.clone(),
                    fact_type: fact_type.clone(),
                },
            )
            .into());
        }
    }
    Ok(())
}
