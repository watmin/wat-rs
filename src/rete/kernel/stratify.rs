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
//     ⚠ AND THE HOLE IS NARROWER THAN THIS PARAGRAPH FIRST CLAIMED — investigated 2026-08-27,
//     immediately after writing it. To slip past this verifier a fn-headed `:then` must MINT a
//     novel fact, and three attempts at one were each refused by a DIFFERENT pre-existing fence:
//
//         fn body computes with `i64::+`            -> then-item-fence: "is not total"
//         fn body computes with the total fallback  -> then-item-fence: "is not a rete primitive"
//         fn body CONSTRUCTS a record at all        -> purity: "`kwargs-construct` is not pure"
//           (`probe_arc278_then_user_forms_userfn.wat`'s own header records that last one, and it
//            is why that fixture EXTRACTS an existing fact rather than building one)
//
//     An extracting fn cannot mint: it returns a fact already in the accumulated set, so it cannot
//     produce an unbounded stream of distinct facts. `then-item-fence` also already walks the fn's
//     BODY for admitted ops — which is most of the analysis this paragraph proposed writing.
//
//     THE HONEST CLAIM, then: no exploit found, and the shape is guarded by adjacent fences —
//     which is NOT the same as proven impossible, and is not stated as such. What this verifier
//     proves is the absence of ONE unbounded-derivation shape. The runtime round cap still stands
//     behind both this and Export's missing AST.

/// Every type the rule's `:then` derives, paired with the rule's `:when` types — the edge
/// `consumed -> produced` this rule contributes to the derivation graph.
struct RuleEdge {
    name: String,
    produced: Vec<String>,
    consumed: Vec<String>,
    /// The `:then` form that computes rather than copies, if any, with its own span for the error.
    computed: Option<(String, crate::span::Span)>,
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
        let computed = rhs
            .iter()
            .find(|f| then_form_computes(f) || rete_fn_body_mints(f, sym))
            .map(|f| {
                (
                    fact_type_head(f).unwrap_or_else(|| String::from("<unknown>")),
                    f.span().clone(),
                )
            });
        edges.push(RuleEdge {
            name,
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
        if cyclic {
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
