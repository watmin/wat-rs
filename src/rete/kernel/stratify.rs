//! Native stratification and the public `fire-rules'` door.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ast::WatAST;
use crate::rete::matcher::{classify_rete_clause, ReteClauseShape};
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::span::Span;
use crate::types::Nature;
use crate::value::value::AggregateValue;

use super::{
    collect_derived, dedupe_filter_children, driver_leaf_ids, fire_fixpoint_delta,
    fire_once_session, fire_rules_from_deps, get_node, kind_of, merge_facts, network_has_production,
    network_identity, node_children, node_kind_label, node_record, refuse_export_without_arm,
    rete_arm_get_or_build, rete_arm_intern, rete_arm_lookup, rules_lack_ast, session_facts,
    session_names, session_network, session_rules, session_with_facts, sorted_node_ids,
    subset_rete_arm, ParentsOf,
};

// ── Arc 278 Stone 7-strat-native: STRATIFIED negation, native port ──────────────
//
// Faithful Rust port of the wat ORACLE's stratification (`wat/rete.wat:1543-1800`):
// `rule-produces` / `rule-negates` / `stratify-sweep` / `stratify-fix` / `rule-stratum` /
// `stratify` / `fire-stratified-loop` / `fire-stratified`. The oracle is the reference and
// does NOT change (`DESIGN-STONE-7strat-native.md`); this is a SEPARATE, self-contained Rust
// impl that moves in lockstep with it (the dual-impl doctrine — no `native?` flag anywhere).

/// A fact-form's type head, colon-stripped: `(:Type ...)` → `"Type"`.
/// Mirrors the inline `ast-name` + colon-strip done identically in both `rule-produces`
/// (`wat/rete.wat:1558-1562`) and `rule-negates` (`wat/rete.wat:1586-1589`).
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
/// is gone) — no more unwrapping a second child. Mirrors `rule-produces` (`wat/rete.wat`).
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

/// The fact types a rule reads POSITIVELY (task #94 — the input the stratifier never had).
///
/// Correct stratification needs BOTH `stratum(r) >= stratum(p)` for positively-used `p` and
/// `stratum(r) > stratum(p)` for negated `p`. Only the second existed, so a rule consuming a
/// fact produced in a HIGHER stratum was left LOWER, fired before its input existed, and never
/// re-fired. `:not` / `:where` are not positive reads. `:exists` inner and accumulate
/// `:from` ARE — they were dropped as engine-form prefixes and the `:from` head
/// leaked as `"?n"`. Walk via `classify_rete_clause`.
/// The stratifier's dependency view of one rule.
/// `consumed` is task #94 — without it a rule that reads a higher-stratum fact sits too low.
/// `bag` is exists-inner / acc `:from` (+1 like negation when the type is derived).
#[derive(Clone, Debug)]
pub(crate) struct StratifyView {
    pub produced: Vec<String>,
    pub negated: Vec<String>,
    pub consumed: Vec<String>,
    pub bag: Vec<String>,
}

/// A compiled rule paired with its stratify view.
#[derive(Clone)]
pub(crate) struct RuleParts {
    pub rule: Value,
    pub produced: Vec<String>,
    pub negated: Vec<String>,
    pub consumed: Vec<String>,
    pub bag: Vec<String>,
}

impl RuleParts {
    pub(crate) fn view(&self) -> StratifyView {
        StratifyView {
            produced: self.produced.clone(),
            negated: self.negated.clone(),
            consumed: self.consumed.clone(),
            bag: self.bag.clone(),
        }
    }
}

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
/// Mirrors `stratify-sweep` (`wat/rete.wat:1599-1646`).
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
        for b in &view.bag {
            let derived = rule_parts.iter().any(|other| {
                other.produced.iter().any(|t| t == b) && !other.bag.iter().any(|t| t == b)
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
/// oracle raises. Mirrors `stratify-fix` (`wat/rete.wat:1651-1667`).
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
/// a stratifiable set — same bound the oracle uses). Mirrors `stratify` (`wat/rete.wat:1707-1713`).
pub(crate) fn native_stratify(rule_parts: &[StratifyView]) -> Result<HashMap<String, i64>, EvalBreak> {
    let bound = rule_parts.len() as i64 + 1;
    native_stratify_fix(rule_parts, HashMap::new(), bound)
}

/// A single rule's stratum given the final type-strata:
/// `max(max strata[p] for produced p, max strata[n]+1 for negated n)`.
/// Mirrors `rule-stratum` (`wat/rete.wat:1671-1702`).
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

/// Native stratified fire drive — port of `fire-stratified-loop` + `fire-stratified`
/// (`wat/rete.wat:1724-1800`), wrapped the way `fire-rules-spec` wraps `fire-stratified`
/// (`wat/rete.wat:1808-1820`: reset `facts = input` on the final result).
///
/// P9 — fused network, no per-stratum recompile: `session` is *already* a `compile`d Session
/// for the FULL rule set (`compile` is called once by the caller — e.g. `strat-neg.wat`'s
/// `(:wat::rete::compile rules)` — before `fire-rules` ever runs), so its `network`(0)/
/// `next-id`(6) already contain every stratum's alpha/join/filter/production chain. Per
/// stratum `[0..=max_s]` ascending, this now reuses THAT SAME network/next-id verbatim
/// (zero recompiles — was one wat-interpreted `:wat::rete::compile` call per stratum),
/// varying only `rules`(1) (filtered to this stratum's rule subset) and `facts`(5) (the
/// accumulated closure from lower strata), fires it to its own fixpoint, and value-dedup-
/// merges (`merge_facts` — R18, NOT concat) the newly derived facts into the running
/// accumulator.
///
/// Why sharing the full network across strata is still correct (no "shared-alpha
/// duplicate-edge" regression, `wat/rete.wat:1772-1775`): `fire_fixpoint_delta` gates
/// PRODUCTION firing by `rule_rhs_cache`, built ONLY from the `rules` field passed in
/// (kernel.rs `fire_fixpoint_delta`, the `rule_rhs_cache.get(rule_name)` `None => continue`
/// skip) — a ProductionNode whose owning rule name is absent from this stratum's `rules`
/// subset can NEVER derive a fact this call, no matter what the shared network's higher-
/// stratum join/negation chains compute incidentally this round (e.g. a shared `Item` alpha
/// feeding every stratum's first condition). And no memory persists ACROSS the stratum
/// boundary either — each call is still a fully FRESH round-loop (`fire_fixpoint_delta`
/// clears alpha/beta/production at its own top, seeded only from this call's `facts`), so a
/// higher stratum's `:not` reading its dependency's alpha before that dependency is complete
/// can only ever reach a name-gated-off production — inert, never observable. The one thing
/// that must still hold (and does, by construction): strata fire in ascending order, each to
/// its own fixpoint, before `acc_facts` advances — i.e. exactly the invariant the loop below
/// already enforced before this change.
pub(crate) fn fire_rules_stratified(
    session: &Value,
    parts: &[RuleParts],
    rule_strata: &[i64],
    max_s: i64,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let input_facts = session_facts(session);

    // The already-compiled network + next-id, shared verbatim across every stratum below.
    let (network, next_id, class, names) = match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let sf = a.fields.as_slice();
            (
                sf[0].clone(),
                sf[6].clone(),
                a.class.clone(),
                a.names.clone(),
            )
        }
        _ => (
            Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
            Value::i64(0),
            // Unreachable in practice — callers only ever pass a compiled Session — but keep
            // a harmless placeholder class rather than panicking on a malformed input.
            Arc::<str>::from("wat::rete::Session"),
            session_names(),
        ),
    };

    // ── Precompute (ONCE, not per stratum) the graph indexes needed to SLICE the shared
    // network down to just one stratum's own rule chain(s) each iteration, native (no
    // wat-interpretation) and O(network size) total, not O(network size × strata).
    //
    // Why slicing is still necessary even though production is already name-gated (see doc
    // comment above): `fire_fixpoint_delta` tours EVERY node in `sorted_node_ids(&wm.network)`
    // every round, including every OTHER stratum's Negation/Exists/Accumulate/join nodes, each
    // doing real work (alpha-memory clones, compatibility scans) regardless of whether its own
    // production is gated off. Handing it the FULL multi-stratum network on every one of the
    // `max_s+1` calls turned the per-stratum cost from "this stratum's own tiny chain" (today's
    // per-stratum-recompile behavior) into "the WHOLE network, every stratum" — an O(strata ×
    // total-nodes) regression measured directly: `[5,500]` went 44ms → ~180ms before this slice
    // was added. Slicing restores the "small per-stratum network" cost profile while still
    // skipping the wat-interpreted recompile.
    //
    // `rev_children`: child-id → parent-ids, built by inverting every node's forward
    // `node_children()` edge once. Lets a backward walk from a stratum's ProductionNode(s)
    // find every upstream Alpha/Join/Test/Negation/Exists/Accumulate node that feeds it.
    // `production_id_by_rule`: rule-name → its ProductionNode id (compile mints exactly one
    // ProductionNode per rule — kernel.rs `rule_produces`/`compile-rule`, wat/rete.wat:781-784).
    let all_ids = sorted_node_ids(&network);
    let mut rev_children: ParentsOf = HashMap::new();
    let mut production_id_by_rule: HashMap<String, i64> = HashMap::new();
    for id in &all_ids {
        let node = match get_node(&network, *id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) == "ProductionNode" {
            if let Some((_, sf)) = node_record(node) {
                if let Value::String(rname) = &sf[1] {
                    production_id_by_rule.insert(rname.to_string(), *id);
                }
            }
        }
        for child in node_children(node) {
            rev_children.entry(child).or_default().push(*id);
        }
    }
    // A Negation/Exists/Accumulate node's own tested-fact-type alpha is a REFERENCE field
    // (`negated-alpha-id` / `exists-alpha-id` / `from-alpha-id`), not a forward `children` edge
    // — `rev_children` alone never reaches it, so the backward walk below follows this
    // reference explicitly wherever it meets one of these three node kinds. Missing this would
    // silently slice the referenced alpha OUT of the sub-network, leaving it permanently empty
    // and making every negation vacuously pass (STOP-1 class bug — caught before shipping).
    //
    // `mint-leaf-alphas` is the same class one level down: a combinator inner
    // (`:not` of `:and` of Wind+Temp) mints one dummy alpha for the wrapper
    // (the reference field) plus an orphan AlphaNode per fact-shaped leaf.
    // Those leaves have no children edge and no id field. Dropping them
    // forced a WM-scan fallback; rematch now refuses. Follow the cond.
    let ref_alpha_of = |node: &Value| -> Option<i64> {
        let (fqdn, sf) = node_record(node)?;
        match node_kind_label(fqdn) {
            "NegationNode" | "ExistsNode" => match &sf[1] {
                Value::i64(n) => Some(*n),
                _ => None,
            },
            "AccumulateNode" => match &sf[3] {
                Value::i64(n) => Some(*n),
                _ => None,
            },
            _ => None,
        }
    };

    let orig_rules = match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice()[1].clone(),
        _ => Value::wat__core__PersistentVector(rpds::VectorSync::new_sync()),
    };
    let full_arm = rete_arm_get_or_build(&network, &orig_rules, sym)?;
    let slice_drivers = &full_arm.compiled_drivers;

    let mut acc_facts: Value = input_facts.clone();
    let mut acc_derived: Vec<Value> = Vec::new();

    for s in 0..=max_s {
        // Filter the original typed rule set to this stratum (same filter the oracle's
        // fire-stratified-loop applies, `wat/rete.wat:1735-1738`) — this IS the production
        // gate (see doc comment above): only these rules' ProductionNodes may fire this call.
        let mut stratum_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        let mut active_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
        let mut frontier: Vec<i64> = Vec::new();
        let mut stratum_rule_names: HashSet<String> = HashSet::new();
        for (part, stratum) in parts.iter().zip(rule_strata.iter()) {
            if *stratum == s {
                stratum_pv.push_back_mut(part.rule.clone());
                if let Some((_, rsf)) = node_record(&part.rule) {
                    if let Value::String(rname) = &rsf[0] {
                        stratum_rule_names.insert(rname.to_string());
                        if let Some(&pid) = production_id_by_rule.get(rname.as_str()) {
                            if active_ids.insert(pid) {
                                frontier.push(pid);
                            }
                        }
                    }
                }
            }
        }
        let stratum_rules = Value::wat__core__PersistentVector(stratum_pv);

        // Backward closure: from this stratum's ProductionNode id(s), follow `rev_children`
        // (upstream via the forward-graph edges), `ref_alpha_of` (upstream via a
        // Negation/Exists/Accumulate node's own tested alpha reference), and
        // `mint_leaf_alpha_ids` (orphan fact-shaped leaves of a combinator ref
        // alpha) until no new node is discovered.
        while let Some(id) = frontier.pop() {
            if let Some(parents) = rev_children.get(&id) {
                for &p in parents {
                    if active_ids.insert(p) {
                        frontier.push(p);
                    }
                }
            }
            if let Some(node) = get_node(&network, id) {
                if let Some(alpha_id) = ref_alpha_of(node) {
                    if active_ids.insert(alpha_id) {
                        frontier.push(alpha_id);
                    }
                }
                if kind_of(node) == "AlphaNode" {
                    if let Some(d) = slice_drivers.get(&id) {
                        for leaf_id in driver_leaf_ids(d) {
                            if leaf_id != id && active_ids.insert(leaf_id) {
                                frontier.push(leaf_id);
                            }
                        }
                    }
                }
            }
        }

        // Slice the shared network down to just `active_ids` — same node Records, no
        // recompile — EXCEPT each retained node's own `children` field is rewritten
        // de-duplicated + `active_ids`-filtered (`dedupe_filter_children`): the ORIGINAL
        // wat-compiled network, built once for every rule together, can carry a shared
        // Alpha/RootJoin node whose `children` list has ONE entry PER RULE that shares that
        // first condition (the doc-commented `wat/rete.wat:1772-1775` shared-alpha hazard) —
        // measured directly: with strat-neg's shared `Item` first condition across 6 rules,
        // the shared root-join's un-deduped children list produced 6 tokens per fact instead
        // of 1 (`beta[rootjoin] == 6000` for 1000 facts), a real N× per-round blow-up (never
        // a WRONG final fact — `seen: HashSet<Value>` still dedups at production — but a
        // measured perf regression this fix removes). Cured entirely on this native COPY;
        // the oracle's own `network` Value is never mutated.
        let sliced_network = match &network {
            Value::wat__core__PersistentMap(m) => {
                let mut nm = rpds::HashTrieMapSync::new_sync();
                for id in &active_ids {
                    if let Some(v) = m.get(&Value::i64(*id)) {
                        nm.insert_mut(Value::i64(*id), dedupe_filter_children(v, &active_ids));
                    }
                }
                // Never wrap a built trie directly — choose the arm by size.
                Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(nm))
            }
            other => other.clone(),
        };

        let slice_arm = subset_rete_arm(
            &full_arm,
            &active_ids,
            &stratum_rule_names,
            &sliced_network,
        );
        if let Some(id) = network_identity(&sliced_network) {
            rete_arm_intern(id, &slice_arm);
        }

        // Reuse the ALREADY-compiled (now stratum-sliced) network + next-id (no
        // `invoke_wat_compile` call); fresh empty alpha/beta/production memories (same
        // "fresh per stratum" semantics as before); facts = the accumulated closure from
        // lower strata.
        let sub_sess = Value::Aggregate(Arc::new(AggregateValue::record_arc(
            class.clone(),
            names.clone(),
            Arc::new(vec![
                sliced_network,
                stratum_rules,
                Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
                Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
                Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
                acc_facts.clone(),
                next_id.clone(),
            ]),
        )));

        let fired = fire_fixpoint_delta(&sub_sess, sym, None)?;

        // Collect this stratum's derived facts from its production-memory (position 4).
        // NOTE: unlike the oracle's bare `fire-fixpoint` (whose `Session/facts` is left as the
        // full input∪derived closure, `wat/rete.wat:1754`), native `fire_fixpoint_delta` already
        // resets `facts = input` internally (its own fire-rules-shaped contract) — so `fired`'s
        // facts field equals the seed, not the closure. Reconstruct the closure explicitly below.
        let production_pm = match &fired {
            Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice()[4].clone(),
            _ => Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
        };
        let new_derived = collect_derived(&production_pm);

        // acc_facts := this stratum's post-fixpoint closure (seed ∪ new_derived), for the next
        // stratum's `:not` to see — the value the oracle gets for free by reading
        // `(:wat::rete::Session/facts fired)` (`wat/rete.wat:1754`).
        acc_facts = merge_facts(&acc_facts, &new_derived);

        // acc_derived := value-dedup union across strata (mirrors `merge-facts`, R18 — NOT concat).
        let acc_derived_pv: rpds::VectorSync<Value> = acc_derived.iter().cloned().collect();
        let merged = merge_facts(
            &Value::wat__core__PersistentVector(acc_derived_pv),
            &new_derived,
        );
        acc_derived = match merged {
            Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
            _ => acc_derived,
        };
    }

    // Pack derived facts into production-memory {0: acc_derived} (mirrors fire-stratified's
    // `fprod-m`, wat/rete.wat:1792) and reset facts = input (mirrors fire-rules-spec's outer
    // wrap, wat/rete.wat:1808-1820). network/rules/next-id preserved from the ORIGINAL input
    // session; alpha-memory/beta-memory reset to empty (mirrors fire-stratified's Session
    // constructor, wat/rete.wat:1793-1800).
    let mut prod_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
    for d in &acc_derived {
        prod_pv.push_back_mut(d.clone());
    }
    let prod_pm = crate::value::pmap::PMap::from_pairs([(
        Value::i64(0),
        Value::wat__core__PersistentVector(prod_pv),
    )]);

    // Oracle fire-stratified does a throwaway fire-once on the ORIGINAL network + closed
    // facts so QueryNodes (absent from per-stratum slices) fill query-memory.
    let q_fired = fire_once_session(&session_with_facts(session, acc_facts.clone()), sym)?;
    let qmem = match &q_fired {
        Value::Aggregate(a) if a.nature != Nature::Struct && a.fields.len() > 7 => {
            a.fields.as_slice()[7].clone()
        }
        _ => Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
    };

    match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let sf = a.fields.as_slice();
            Ok(Value::Aggregate(Arc::new(AggregateValue::record_arc(
                a.class.clone(),
                a.names.clone(),
                Arc::new(vec![
                    sf[0].clone(),                                                    // network (original)
                    sf[1].clone(), // rules (original)
                    Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()), // alpha-memory
                    Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()), // beta-memory
                    Value::wat__core__PersistentMap(prod_pm), // production-memory
                    input_facts,                              // facts = input
                    sf[6].clone(),                            // next-id (original)
                    qmem,
                ]),
            ))))
        }
        other => Ok(other.clone()),
    }
}


// ── Public entry: native fire-rules' ─────────────────────────────────────────

/// `(:wat::rete::fire-rules <session>) -> :wat::rete::Session`
///
/// Native cascade fixpoint. Delegates to `fire_fixpoint_delta` (semi-naive),
/// or to `fire_rules_stratified` when a rule negates a same-or-lower-stratum type.
/// Restores `facts = input` before returning.
///
/// Observationally equivalent to the wat oracle's `fire-rules`:
/// `query(fire-rules' s, T) ≡ query(fire-rules s, T)` for every type T.
pub(crate) fn eval_fire_rules_native(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::fire-rules";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }

    // Evaluate the session argument.
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();

    // 7-strat-native: read the rule set once and compute each rule's stratum (port of the
    // oracle's stratify: produces/negates/sweep/fix/rule-stratum). `max_s == 0` means no rule
    // negates a type any rule in the SAME OR LOWER stratum produces — i.e. no negation-over-
    // derived — so the fast unstratified path is observationally identical and MUST stay the
    // one taken (byte-identical to today, zero perf cost for the 99% non-stratified case).
    //
    // An imported Export has empty rules AST. Stratify inputs live on the interned arm
    // (`rule_deps`). Without them, max_s is 0 and negation-over-derived lies.
    let rules_value = session_rules(&session);
    let rules: Vec<Value> = match &rules_value {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    };
    if rules_lack_ast(&rules) {
        if let Some(net) = session_network(&session) {
            if let Some(id) = network_identity(net) {
                if let Some(arm) = rete_arm_lookup(id) {
                    if !arm.rule_deps.is_empty() {
                        return fire_rules_from_deps(&session, &arm.rule_deps, sym);
                    }
                }
            }
            if network_has_production(net) {
                return Err(refuse_export_without_arm(OP, list_span));
            }
        }
    }
    let mut parts: Vec<RuleParts> = Vec::with_capacity(rules.len());
    for r in &rules {
        let (_, rsf) = match node_record(r) {
            Some(x) => x,
            None => continue,
        };
        let to_asts = |v: &Value| -> Vec<WatAST> {
            match v {
                Value::wat__core__PersistentVector(pv) => pv
                    .iter()
                    .filter_map(|x| match x {
                        Value::wat__WatAST(ast) => Some((**ast).clone()),
                        _ => None,
                    })
                    .collect(),
                _ => vec![],
            }
        };
        let lhs = to_asts(&rsf[1]);
        let rhs = to_asts(&rsf[2]);
        let produced = rule_produces(&rhs, sym);
        let negated = rule_negates(&lhs);
        let consumed = rule_consumes(&lhs);
        let bag = rule_bag_consumes(&lhs);
        parts.push(RuleParts {
            rule: r.clone(),
            produced,
            negated,
            consumed,
            bag,
        });
    }

    let pn_only: Vec<StratifyView> = parts.iter().map(RuleParts::view).collect();
    let type_strata = native_stratify(&pn_only)?;

    let mut max_s: i64 = 0;
    let mut rule_strata: Vec<i64> = Vec::with_capacity(parts.len());
    for part in &parts {
        let s = native_rule_stratum(&part.produced, &part.negated, &type_strata);
        rule_strata.push(s);
        if s > max_s {
            max_s = s;
        }
    }

    if max_s == 0 {
        // UNCHANGED fast path — P4b: run the semi-naive delta fixpoint (input_facts restore is
        // done inside). Pass None — the fast path records no support index (zero behavior change).
        return fire_fixpoint_delta(&session, sym, None);
    }

    // Stratified drive — port of fire-stratified-loop, bottom→top.
    fire_rules_stratified(&session, &parts, &rule_strata, max_s, sym)
}

