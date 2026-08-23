//! Native `fire-rules` door: stratify-or-delta choice, per-stratum drive.
//! Dual of `wat/rete/oracle/fire.wat` fire-stratified / fire-rules$oracle.
//! Numbering lives in `kernel/stratify.rs`.

use std::collections::{HashMap, HashSet};

use crate::ast::WatAST;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};

use super::*;

/// Native stratified fire drive — port of `fire-stratified-loop` + `fire-stratified`
/// (`wat/rete/oracle/fire.wat`), wrapped the way `fire-rules$oracle` wraps `fire-stratified`
/// (reset `facts = input` on the final result).
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
/// duplicate-edge" regression, `wat/rete/oracle/fire.wat`): `fire_fixpoint_delta` gates
/// PRODUCTION firing by `rule_rhs_cache`, built ONLY from the `rules` field passed in
/// (kernel/ `fire_fixpoint_delta`, the `rule_rhs_cache.get(rule_name)` `None => continue`
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
    mut support: Option<&mut ExplainSupport>,
) -> Result<Value, EvalBreak> {
    let input_facts = session_facts(session);

    // The already-compiled network + next-id, shared verbatim across every stratum below.
    let network = session_network(session)
        .cloned()
        .unwrap_or_else(|| Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()));
    let next_id = session_named_field(session, "next-id")
        .cloned()
        .unwrap_or(Value::i64(0));

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
    // ProductionNode per rule — kernel/ `rule_produces`/`compile-rule`, wat/rete/oracle/fire.wat).
    let all_ids = sorted_node_ids(&network);
    let mut rev_children: ParentsOf = HashMap::new();
    let mut production_id_by_rule: HashMap<String, i64> = HashMap::new();
    for id in &all_ids {
        let node = match get_node(&network, *id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) == NodeKind::Production {
            if let Some(rname) = node_named_string(node, "rule-name") {
                production_id_by_rule.insert(rname.to_string(), *id);
            }
        }
        for child in node_children(node) {
            rev_children.entry(child).or_default().push(*id);
        }
    }
    // Negation/Exists/Accumulate tested-alpha is a REFERENCE field, not a children
    // edge — `close_upstream` follows `node_ref_alpha_id` plus combinator leaf alphas.

    let orig_rules = session_rules(session);
    let full_arm = rete_arm_get_or_build(&network, &orig_rules, sym)?;
    let slice_drivers = &full_arm.compiled_drivers;

    let mut acc_facts: Value = input_facts.clone();
    let mut acc_derived: Vec<Value> = Vec::new();
    let mut acc_derived_set: HashSet<Value> = HashSet::new();

    for s in 0..=max_s {
        // Filter the original typed rule set to this stratum (same filter the oracle's
        // fire-stratified-loop applies, `wat/rete/oracle/fire.wat`) — this IS the production
        // gate (see doc comment above): only these rules' ProductionNodes may fire this call.
        let mut stratum_pv: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
        let mut active_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
        let mut frontier: Vec<i64> = Vec::new();
        let mut stratum_rule_names: HashSet<String> = HashSet::new();
        for (part, stratum) in parts.iter().zip(rule_strata.iter()) {
            if *stratum == s {
                stratum_pv.push_back_mut(part.rule.clone());
                if let Some(rname) = rule_name_of(&part.rule) {
                    stratum_rule_names.insert(rname.clone());
                    if let Some(&pid) = production_id_by_rule.get(rname.as_str()) {
                        if active_ids.insert(pid) {
                            frontier.push(pid);
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
        close_upstream(
            &network,
            &rev_children,
            slice_drivers,
            &mut active_ids,
            &mut frontier,
        );

        // Slice the shared network down to just `active_ids` — same node Records, no
        // recompile — EXCEPT each retained node's own `children` field is rewritten
        // de-duplicated + `active_ids`-filtered (`dedupe_filter_children`): the ORIGINAL
        // wat-compiled network, built once for every rule together, can carry a shared
        // Alpha/RootJoin node whose `children` list has ONE entry PER RULE that shares that
        // first condition (the doc-commented `wat/rete/oracle/fire.wat` shared-alpha hazard) —
        // measured directly: with strat-neg's shared `Item` first condition across 6 rules,
        // the shared root-join's un-deduped children list produced 6 tokens per fact instead
        // of 1 (`beta[rootjoin] == 6000` for 1000 facts), a real N× per-round blow-up (never
        // a WRONG final fact — `seen: HashSet<Value>` still dedups at production — but a
        // measured perf regression this fix removes). Cured entirely on this native COPY;
        // the oracle's own `network` Value is never mutated.
        let sliced_network = slice_active_network(&network, &active_ids);

        let slice_arm =
            subset_rete_arm(&full_arm, &active_ids, &stratum_rule_names, &sliced_network);

        // Reuse the ALREADY-compiled (now stratum-sliced) network + next-id (no
        // `invoke_wat_compile` call); fresh empty alpha/beta/production memories (same
        // "fresh per stratum" semantics as before); facts = the accumulated closure from
        // lower strata. The slice arm is a value on this pass — not interned
        // (slice rust_identity is not the connection Session; `release-session`
        // would never see it).
        let empty_pm = Value::wat__core__PersistentMap(crate::value::pmap::PMap::new());
        let sub_sess = session_with_fields(
            session,
            &[
                ("network", sliced_network),
                ("rules", stratum_rules),
                ("alpha-memory", empty_pm.clone()),
                ("beta-memory", empty_pm.clone()),
                ("production-memory", empty_pm),
                ("facts", acc_facts.clone()),
                ("next-id", next_id.clone()),
            ],
        );

        let fired = fire_fixpoint_delta_armed(
            &sub_sess,
            sym,
            support.as_deref_mut(),
            Some(slice_arm),
            FireKind::Rules,
        )?;

        // Collect this stratum's derived facts from its production-memory (position 4).
        // NOTE: unlike the oracle's bare `fire-fixpoint` (whose `Session/facts` is left as the
        // full input∪derived closure, `wat/rete/oracle/fire.wat`), native `fire_fixpoint_delta` already
        // resets `facts = input` internally (its own fire-rules-shaped contract) — so `fired`'s
        // facts field equals the seed, not the closure. Reconstruct the closure explicitly below.
        let production_pm = session_named_field(&fired, "production-memory")
            .cloned()
            .unwrap_or_else(|| Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()));
        let new_derived = collect_derived(&production_pm);

        // acc_facts := this stratum's post-fixpoint closure (seed ∪ new_derived), for the next
        // stratum's `:not` to see — the value the oracle gets for free by reading
        // `(:wat::rete::Session/facts fired)` (`wat/rete/oracle/fire.wat`).
        acc_facts = merge_facts(&acc_facts, &new_derived);

        // acc_derived := value-dedup union across strata (mirrors `merge-facts`, R18 — NOT concat).
        for d in &new_derived {
            if acc_derived_set.insert(d.clone()) {
                acc_derived.push(d.clone());
            }
        }
    }

    // Pack derived facts into production-memory {0: acc_derived} (mirrors fire-stratified's
    // `fprod-m`, oracle/fire.wat) and reset facts = input (mirrors fire-rules$oracle's outer
    // wrap). network/rules/next-id preserved from the ORIGINAL input
    // session; alpha-memory/beta-memory reset to empty (mirrors fire-stratified's Session
    // constructor, wat/rete/oracle/fire.wat).
    let mut prod_pv: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
    for d in &acc_derived {
        prod_pv.push_back_mut(d.clone());
    }
    let prod_pm = crate::value::pmap::PMap::from_pairs([(
        Value::i64(0),
        Value::wat__core__PersistentVector(prod_pv),
    )]);

    // QueryNodes sit off the production chain, so stratum slices never contain them.
    // Oracle pays a throwaway fire-once on the FULL network + closed facts
    // (`wat/rete/oracle/fire.wat` q-seed). Native class-scan queries harvest
    // input ∪ derived in place; constrained queries still seed the QueryNode
    // reverse-closure once — same answers, no second pass of the stratified
    // join/negation chains.
    let qmem = if full_arm.kind_ids.query.is_empty() {
        Value::wat__core__PersistentMap(crate::value::pmap::PMap::new())
    } else {
        harvest_stratified_queries(
            session,
            &full_arm,
            &rev_children,
            &acc_facts,
            &acc_derived,
            sym,
        )?
    };

    let empty_pm = Value::wat__core__PersistentMap(crate::value::pmap::PMap::new());
    Ok(session_with_fields(
        session,
        &[
            ("alpha-memory", empty_pm.clone()),
            ("beta-memory", empty_pm),
            (
                "production-memory",
                Value::wat__core__PersistentMap(prod_pm),
            ),
            ("facts", input_facts),
            ("query-memory", qmem),
        ],
    ))
}

/// Reverse-close from `frontier`. Parent edges (`rev_children`) are not enough:
/// Negation/Exists/Accumulate tested-alpha is a reference field, not a children
/// edge — missing it slices the tested type OUT and every negation vacuously
/// passes. Combinator `:not`/`:and`/`:or` mints orphan leaf alphas; follow
/// `driver_leaf_ids` or rematch refuses.
fn close_upstream(
    network: &Value,
    rev_children: &ParentsOf,
    drivers: &HashMap<i64, CondDriver>,
    active_ids: &mut HashSet<i64>,
    frontier: &mut Vec<i64>,
) {
    while let Some(id) = frontier.pop() {
        if let Some(parents) = rev_children.get(&id) {
            for &p in parents {
                if active_ids.insert(p) {
                    frontier.push(p);
                }
            }
        }
        if let Some(node) = get_node(network, id) {
            if let Some(alpha_id) = node_ref_alpha_id(node) {
                if active_ids.insert(alpha_id) {
                    frontier.push(alpha_id);
                }
            }
            if kind_of(node) == NodeKind::Alpha {
                if let Some(d) = drivers.get(&id) {
                    for leaf_id in driver_leaf_ids(d) {
                        if leaf_id != id && active_ids.insert(leaf_id) {
                            frontier.push(leaf_id);
                        }
                    }
                }
            }
        }
    }
}

fn slice_active_network(network: &Value, active_ids: &HashSet<i64>) -> Value {
    match network {
        Value::wat__core__PersistentMap(m) => {
            let mut nm = rpds::HashTrieMapSync::new_sync();
            for id in active_ids {
                if let Some(v) = m.get(&Value::i64(*id)) {
                    nm.insert_mut(Value::i64(*id), dedupe_filter_children(v, active_ids));
                }
            }
            Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(nm))
        }
        other => other.clone(),
    }
}

/// True when every query is fed by a class-scan alpha. Occupancy seed
/// of a harvest Once is then theater (`DESIGN-STONE-strat-neg-harvest-once`).
fn class_scans_cover_queries(
    arm: &InternedNetwork,
    scans: &HashMap<i64, QueryClassScan>,
) -> bool {
    if arm.kind_ids.query.is_empty() {
        return false;
    }
    arm.kind_ids.query.iter().all(|qid| {
        let pids = arm
            .parents_of
            .get(qid)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        pids.iter().any(|pid| {
            arm.feeding_alpha_of
                .get(pid)
                .is_some_and(|aid| scans.contains_key(aid))
        })
    })
}

/// Fill query-memory after stratified fire. QueryNodes are absent from production
/// slices. Class-scan queries harvest input ∪ derived in place; constrained
/// queries still seed the QueryNode reverse-closure once.
fn harvest_stratified_queries(
    session: &Value,
    full_arm: &InternedNetwork,
    rev_children: &ParentsOf,
    acc_facts: &Value,
    acc_derived: &[Value],
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let Some(network) = session_network(session) else {
        return Ok(Value::wat__core__PersistentMap(
            crate::value::pmap::PMap::new(),
        ));
    };
    let scans = query_class_scans(full_arm, network);
    if class_scans_cover_queries(full_arm, &scans) {
        let mut wm = to_transient_for_fire(session)?;
        wm.derived_facts.extend(acc_derived.iter().cloned());
        let wanted: HashSet<&str> = scans.values().map(|s| s.class.as_str()).collect();
        if let Value::wat__core__PersistentVector(pv) = &wm.facts {
            wm.input_has_scan_class = pv.iter().any(|f| match f {
                Value::Aggregate(a) if a.nature != crate::types::Nature::Struct => {
                    wanted.contains(a.class.as_ref())
                }
                _ => false,
            });
        }
        let __hq = phase_start();
        harvest_query_memory(&mut wm, full_arm, &scans);
        phase_end("  ├ harvest:query", __hq);
        return Ok(query_memory_to_pm(std::mem::take(&mut wm.query)));
    }
    let next_id = session_named_field(session, "next-id")
        .cloned()
        .unwrap_or(Value::i64(0));
    let mut q_ids: HashSet<i64> = HashSet::new();
    let mut q_front: Vec<i64> = Vec::new();
    for &qid in &full_arm.kind_ids.query {
        if q_ids.insert(qid) {
            q_front.push(qid);
        }
    }
    close_upstream(
        network,
        rev_children,
        &full_arm.compiled_drivers,
        &mut q_ids,
        &mut q_front,
    );
    let q_net = slice_active_network(network, &q_ids);
    let q_arm = subset_rete_arm(full_arm, &q_ids, &HashSet::new(), &q_net);
    let empty_pm = Value::wat__core__PersistentMap(crate::value::pmap::PMap::new());
    let empty_rules = Value::wat__core__PersistentVector(crate::value::pvec::PVec::new());
    let q_sess = session_with_fields(
        session,
        &[
            ("network", q_net),
            ("rules", empty_rules),
            ("alpha-memory", empty_pm.clone()),
            ("beta-memory", empty_pm.clone()),
            ("production-memory", empty_pm),
            ("facts", acc_facts.clone()),
            ("next-id", next_id),
        ],
    );
    let q_fired = fire_fixpoint_delta_armed(&q_sess, sym, None, Some(q_arm), FireKind::Once)?;
    Ok(session_named_field(&q_fired, "query-memory")
        .cloned()
        .unwrap_or_else(|| Value::wat__core__PersistentMap(crate::value::pmap::PMap::new())))
}

// ── Public entry: native fire-rules ──────────────────────────────────────────

/// `(:wat::rete::fire-rules <session>) -> :wat::rete::Session`
///
/// Native cascade fixpoint. Delegates to `fire_fixpoint_delta` (semi-naive),
/// or to `fire_rules_stratified` when a rule negates a same-or-lower-stratum type.
/// Restores `facts = input` before returning.
///
/// Observationally equivalent to the wat oracle's `fire-rules$oracle` on AST
/// Sessions (non-empty Rule/Query forms). Export is native-only: the oracle
/// refuses an imported Export (`wat/rete.wat` bounds Export as "Native fire
/// only"; `wat/rete/oracle/fire.wat` fire-rules$oracle / fire-once$oracle).
/// An imported Export has empty rules AST; stratify reads interned `arm.rule_deps`.
/// A production network without those deps refuses (`refuse_export_without_arm`).
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
    fire_rules_on_session(&session, sym, None)
}

/// Native cascade fixpoint on an already-evaluated Session. Shared by
/// `fire-rules` and `fire-rules-explain` so explain cannot skip stratify.
pub(crate) fn fire_rules_on_session(
    session: &Value,
    sym: &SymbolTable,
    support: Option<&mut ExplainSupport>,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::fire-rules";
    // 7-strat-native: read the rule set once and compute each rule's stratum (port of the
    // oracle's stratify: produces/negates/sweep/fix/rule-stratum). `max_s == 0` means no rule
    // negates a type any rule in the SAME OR LOWER stratum produces — i.e. no negation-over-
    // derived — so the fast unstratified path is observationally identical and MUST stay the
    // one taken (byte-identical to today, zero perf cost for the 99% non-stratified case).
    //
    // An imported Export has empty rules AST. Stratify inputs live on the interned arm
    // (`rule_deps`). Without them, max_s is 0 and negation-over-derived lies.
    let rules_value = session_rules(session);
    let rules: Vec<Value> = match &rules_value {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    };
    if rules_lack_ast(&rules) {
        if let Some(net) = session_network(session) {
            if let Some(id) = network_identity(net) {
                if let Some(arm) = rete_arm_lookup(id) {
                    if !arm.rule_deps.is_empty() {
                        return fire_rules_from_deps(session, &arm.rule_deps, sym, support);
                    }
                }
            }
            if network_has_production(net) {
                return Err(refuse_export_without_arm(OP, &crate::rust_caller_span!()));
            }
        }
    }
    // rune:temperare(simplicity-win) — AST sessions re-walk lhs/rhs to learn max_s; interned
    // arm.rule_deps is the Export door. n rules is small.
    let mut parts: Vec<RuleParts> = Vec::with_capacity(rules.len());
    for r in &rules {
        if rule_named_field(r, "name").is_none() {
            continue;
        }
        let lhs = rule_asts_field(r, "lhs");
        let rhs = rule_asts_field(r, "rhs");
        let produced = rule_produces(&rhs, sym);
        let negated = rule_negates(&lhs);
        let consumed = rule_consumes(&lhs);
        let exists_and_from_types = rule_bag_consumes(&lhs);
        parts.push(RuleParts {
            rule: r.clone(),
            view: StratifyView {
                produced,
                negated,
                consumed,
                exists_and_from_types,
            },
        });
    }

    let pn_only: Vec<StratifyView> = parts.iter().map(|p| p.view.clone()).collect();
    let type_strata = native_stratify(&pn_only)?;

    let mut max_s: i64 = 0;
    let mut rule_strata: Vec<i64> = Vec::with_capacity(parts.len());
    for part in &parts {
        let s = native_rule_stratum(&part.view.produced, &part.view.negated, &type_strata);
        rule_strata.push(s);
        if s > max_s {
            max_s = s;
        }
    }

    if max_s == 0 {
        // Fast path — P4b: semi-naive delta fixpoint (input_facts restore is
        // done inside). Explain threads `Some(support)` through the same door.
        return fire_fixpoint_delta(session, sym, support);
    }

    // Stratified drive — port of fire-stratified-loop, bottom→top.
    fire_rules_stratified(session, &parts, &rule_strata, max_s, sym, support)
}
