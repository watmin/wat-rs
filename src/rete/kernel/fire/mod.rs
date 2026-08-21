//! Fire loop: alpha/root/hash/production passes, leftover rematch, delta fixpoint.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::ast::WatAST;
use crate::rete::compiled_cond::ValIntern;
use crate::rete::matcher::Bindings;
use crate::runtime::{
    EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
};
use crate::span::Span;
use crate::types::Nature;
use crate::value::value::AggregateValue;

use super::*;

/// Split-borrow of `FireSession`. Token/Element are Copy; we cannot
/// hold `&mut FireSession` while walking beta/alpha. Facts stay out of
/// the bind intern (`DESIGN-STONE-fact-as-index`).
pub(crate) struct FireCtx<'a> {
    pub(crate) compiled_conds: &'a HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    pub(crate) scratch: &'a mut SlotFrame,
    pub(crate) pool: &'a mut Vec<(u32, u32)>,
    pub(crate) match_pool: &'a mut Vec<(u32, i64)>,
    pub(crate) keys: &'a [Value],
    pub(crate) vals: &'a [Value],
    pub(crate) facts: &'a Value,
    pub(crate) derived: &'a [Value],
    pub(crate) n_input: u32,
}

/// Shared intern + fact store for accumulate folds (no pool mutation).
struct AccView<'a> {
    keys: &'a [Value],
    vals: &'a [Value],
    pool: &'a [(u32, u32)],
    facts: &'a Value,
    derived: &'a [Value],
    n_input: u32,
}

fn acc_view(wm: &FireSession) -> AccView<'_> {
    AccView {
        keys: &wm.bind_keys,
        vals: &wm.bind_vals,
        pool: &wm.bind_pool,
        facts: &wm.facts,
        derived: &wm.derived_facts,
        n_input: wm.n_input,
    }
}

// ── Pass 1: Alpha pass ────────────────────────────────────────────────────────

/// `activate-alpha` + `activate-fact` — type-index each fact, `exec_compiled`
/// against that type's alphas. Mirrors `wat/rete/oracle/pass.wat`.
/// A missing compiled cond refuses — do not walk `alpha_match_inner`.
pub(crate) fn alpha_pass(wm: &mut FireSession, arm: &InternedNetwork) -> Result<(), EvalBreak> {
    let mut match_scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
    let mut cand_scratch: Vec<i64> = Vec::new();
    let mut cond_key_ids: CondKeyIds = HashMap::new();
    for (&id, c) in &arm.compiled_conds {
        cond_key_ids.insert(
            id,
            crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
        );
    }

    let input = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => return Ok(()),
    };
    wm.n_input = input.len() as u32;

    for (i, fact) in input.iter().enumerate() {
        let (fact_class, fact_fields) = match fact {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                (a.class.as_ref(), a.fields.as_slice())
            }
            _ => continue,
        };
        arm.alpha_tree
            .candidates_into(fact_class, fact_fields, &mut cand_scratch);
        for aid in cand_scratch.iter().copied() {
            let compiled = rematch_compiled(&arm.compiled_conds, aid)?;
            let Some((off, len)) = crate::rete::compiled_cond::exec_compiled_with_key_ids(
                compiled,
                fact_fields,
                &mut match_scratch,
                &mut crate::rete::compiled_cond::BindIntern {
                    keys: &mut wm.bind_keys,
                    vals: &mut wm.bind_vals,
                    ids: &mut wm.bind_val_ids,
                    pool: &mut wm.bind_pool,
                },
                fact,
                cond_key_ids.get(&aid).map(|v| v.as_slice()),
            ) else {
                continue;
            };
            let el = make_element(i as u32, off, len);
            wm.alpha.entry(aid).or_default().push(el);
        }
    }
    Ok(())
}

// ── Pass 2: Root-join pass ────────────────────────────────────────────────────

/// `root-join-pass` / `seed-root-join-children` / `seed-token` / `append-token` —
/// for each AlphaNode with Elements, seed one Token per Element into each RootJoinNode child's beta.
/// Mirrors `wat/rete/oracle/pass.wat`.
pub(crate) fn root_join_pass(wm: &mut FireSession) {
    let node_ids = sorted_node_ids(&wm.network);

    for node_id in &node_ids {
        // Group C: use &Value ref from wm.network; borrow ends after node_children (NLL).
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != "AlphaNode" {
            continue;
        }
        let child_ids = node_children(node);
        // node's last use is node_children above; wm.network borrow ends here (NLL).

        // Group C: borrow elements from wm.alpha — wm.beta mutations below are on a different field.
        let elements = match wm.alpha.get(node_id) {
            Some(els) => els.as_slice(),
            None => continue, // no elements → skip
        };

        for child_id in &child_ids {
            // Group C: child_node ref — only used for kind_of; borrow ends before wm.beta mutation.
            let child_node = match get_node(&wm.network, *child_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(child_node) != "RootJoinNode" {
                continue;
            }
            // Seed one native Token per Element into beta[child_id].
            for el in elements {
                // Support edge: (fact idx, alpha-id). Mirrors seed-token (wat:544-551).
                // Share the Element span — no PMap (`DESIGN-STONE-token-bind-pool`).
                let tok = Token {
                    matches: push_match(&mut wm.match_pool, el.fact, *node_id),
                    binds: seed_token_binds(el),
                            };
                wm.beta.entry(*child_id).or_default().push(tok);
            }
        }
    }
}

// ── Pass 3: Hash-join pass ────────────────────────────────────────────────────

/// `alpha-feeding` — find the AlphaNode id whose `children` contains `hj_id`.
/// Mirrors `wat/rete/oracle/pass.wat`. Returns -1 if not found.
fn alpha_feeding(hj_id: i64, network: &Value) -> i64 {
    let node_ids: Vec<i64> = match network {
        Value::wat__core__PersistentMap(m) => m
            .keys()
            .into_iter()
            .filter_map(|k| if let Value::i64(n) = k { Some(n) } else { None })
            .collect(),
        _ => return -1,
    };
    for node_id in &node_ids {
        let node = match get_node(network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) == "AlphaNode" {
            let children = node_children(node);
            if children.contains(&hj_id) {
                return *node_id;
            }
        }
    }
    -1
}


pub(crate) fn driver_of(
    drivers: &HashMap<i64, CondDriver>,
    alpha_id: i64,
) -> Result<&CondDriver, EvalBreak> {
    drivers.get(&alpha_id).ok_or_else(|| {
        RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::fire-rules".into(),
                reason: format!(
                    "alpha {alpha_id} has no compiled driver — setup should have compiled every alpha"
                ),
            },
        )
        .into()
    })
}

/// Binding maps that satisfy a compiled combinator driver under `seed`.
fn binding_extensions(
    driver: &CondDriver,
    wm: &FireSession,
    seed: &crate::value::pmap::PMap,
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    scratch: &mut SlotFrame,
    sym: &SymbolTable,
    gather_cache: &mut GatherCache,
) -> Result<Vec<crate::value::pmap::PMap>, EvalBreak> {
    match driver {
        CondDriver::And(kids) => {
            let mut exts = vec![seed.clone()];
            for kid in kids {
                let mut next = Vec::new();
                for ext in &exts {
                    next.extend(binding_extensions(
                        kid,
                        wm,
                        ext,
                        compiled_conds,
                        scratch,
                        sym,
                        gather_cache,
                    )?);
                }
                exts = next;
                if exts.is_empty() {
                    break;
                }
            }
            Ok(exts)
        }
        CondDriver::Or(kids) => {
            let mut out = Vec::new();
            for kid in kids {
                out.extend(binding_extensions(
                    kid,
                    wm,
                    seed,
                    compiled_conds,
                    scratch,
                    sym,
                    gather_cache,
                )?);
            }
            Ok(out)
        }
        CondDriver::Where(program) => {
            match crate::rete::expr_ir::exec_where(
                program,
                seed,
                sym,
                &crate::rust_caller_span!(),
            ) {
                Ok(true) => Ok(vec![seed.clone()]),
                _ => Ok(vec![]),
            }
        }
        CondDriver::Not(inner) => {
            if exists_cond_under(inner, wm, seed, compiled_conds, scratch, sym, gather_cache)? {
                Ok(vec![])
            } else {
                Ok(vec![seed.clone()])
            }
        }
        CondDriver::Exists(inner) => {
            if exists_cond_under(inner, wm, seed, compiled_conds, scratch, sym, gather_cache)? {
                Ok(vec![seed.clone()])
            } else {
                Ok(vec![])
            }
        }
        CondDriver::Leaf(alpha_id) => {
            let compiled = rematch_compiled(compiled_conds, *alpha_id)?;
            Ok(seeded_bindings_keyed(
                gather_cache,
                wm,
                *alpha_id,
                seed,
                compiled,
                scratch,
            ))
        }
    }
}

pub(crate) fn rematch_compiled(
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    alpha_id: i64,
) -> Result<&crate::rete::compiled_cond::CompiledCond, EvalBreak> {
    compiled_conds.get(&alpha_id).ok_or_else(|| {
        RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::fire-rules".into(),
                reason: format!(
                    "alpha {alpha_id} has no compiled cond — setup should have compiled every fact-shaped alpha"
                ),
            },
        )
        .into()
    })
}

fn fact_bindings_under<B: Bindings + ?Sized>(
    fact: &Value,
    seed: &B,
    compiled: &crate::rete::compiled_cond::CompiledCond,
    scratch: &mut SlotFrame,
) -> Option<crate::value::pmap::PMap> {
    let fact_fields = match fact {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice(),
        _ => return None,
    };
    let pairs =
        crate::rete::compiled_cond::exec_compiled_under(compiled, fact_fields, scratch, seed)?;
    let pairs = crate::rete::compiled_cond::attach_fact(compiled, fact, pairs);
    // Unify with the seed. A Bind of `?c` compiles as a first-write (the cond
    // in isolation does not know the left token already bound `?c`). Conflict
    // is no match — same as `alpha_match_inner_seeded` / Clara join. Overwrite
    // is how `where-not-and-bound` row 3 accepted Temp.c ≠ Cold.c.
    // Leftover rematch only: materialize a PMap. The fire path does not
    // keep it (`DESIGN-STONE-token-bind-pool`).
    let mut pm = crate::value::pmap::PMap::from_pairs(
        seed.iter().map(|(k, v)| (k.clone(), v.clone())),
    );
    for (k, v) in pairs.iter() {
        match pm.get(k) {
            Some(existing) if existing != v => return None,
            Some(_) => {}
            None => pm = pm.assoc(k.clone(), v.clone()),
        }
    }
    Some(pm)
}





fn token_exists_under<B: Bindings + ?Sized>(
    driver: &CondDriver,
    tok: &B,
    wm: &FireSession,
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    scratch: &mut SlotFrame,
    sym: &SymbolTable,
    gather_cache: &mut GatherCache,
) -> Result<bool, EvalBreak> {
    match driver {
        CondDriver::Leaf(alpha_id) => {
            let compiled = rematch_compiled(compiled_conds, *alpha_id)?;
            Ok(any_seeded_keyed(
                gather_cache,
                wm,
                *alpha_id,
                tok,
                compiled,
                scratch,
            ))
        }
        other => {
            let seed = crate::value::pmap::PMap::from_pairs(
                tok.iter().map(|(k, v)| (k.clone(), v.clone())),
            );
            exists_cond_under(
                other,
                wm,
                &seed,
                compiled_conds,
                scratch,
                sym,
                gather_cache,
            )
        }
    }
}

fn exists_cond_under(
    driver: &CondDriver,
    wm: &FireSession,
    seed: &crate::value::pmap::PMap,
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    scratch: &mut SlotFrame,
    sym: &SymbolTable,
    gather_cache: &mut GatherCache,
) -> Result<bool, EvalBreak> {
    match driver {
        CondDriver::And(_) => Ok(!binding_extensions(
            driver,
            wm,
            seed,
            compiled_conds,
            scratch,
            sym,
            gather_cache,
        )?
        .is_empty()),
        CondDriver::Or(kids) => {
            for k in kids {
                if exists_cond_under(k, wm, seed, compiled_conds, scratch, sym, gather_cache)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        CondDriver::Where(program) => Ok(crate::rete::expr_ir::exec_where(
            program,
            seed,
            sym,
            &crate::rust_caller_span!(),
        )
        .unwrap_or(false)),
        CondDriver::Not(inner) => Ok(!exists_cond_under(
            inner,
            wm,
            seed,
            compiled_conds,
            scratch,
            sym,
            gather_cache,
        )?),
        CondDriver::Exists(inner) => exists_cond_under(
            inner,
            wm,
            seed,
            compiled_conds,
            scratch,
            sym,
            gather_cache,
        ),
        CondDriver::Leaf(leaf_id) => {
            let compiled = rematch_compiled(compiled_conds, *leaf_id)?;
            Ok(any_seeded_keyed(
                gather_cache,
                wm,
                *leaf_id,
                seed,
                compiled,
                scratch,
            ))
        }
    }
}


/// Shared-variable agreement: if a key is in both maps with a DIFFERENT value → false.
/// A variable only on one side never conflicts. The keyed hash-join does not call this
/// in the hot path; the NegationNode / ExistsNode gather still does.
fn token_element_compatible<B: Bindings + ?Sized, E: Bindings + ?Sized>(
    tok_bindings: &B,
    el_bindings: &E,
) -> bool {
    for (k, e_val) in el_bindings.iter() {
        if let Some(t_val) = tok_bindings.get(k) {
            if t_val != e_val {
                return false;
            }
        }
        // Key only in element bindings → no conflict (compatible).
    }
    true
}

/// Merge an Element's fact and bindings into a native `Token`.
/// Concat left match-span + `(el_fact, alpha_id)` into `match_pool`; concat left
/// bind-span + right-only pairs into `bind_pool`. Duplicate keys on the right are
/// skipped (a repeated `?v` bind within one condition is an equality CONSTRAINT
/// in `eval_clause`, never a second pair).
pub(crate) fn extend_token(
    tok: &Token,
    el_fact: u32,
    el_span: BindSpan,
    alpha_id: i64,
    pool: &mut Vec<(u32, u32)>,
    match_pool: &mut Vec<(u32, i64)>,
) -> Token {
    // Concat left edges + the new fact idx. Do not hold a slice across push
    // (`DESIGN-STONE-match-pool`, `DESIGN-STONE-match-pool-fact-as-index`).
    let mo = tok.matches.off as usize;
    let mn = tok.matches.len as usize;
    let mstart = match_pool.len();
    for i in 0..mn {
        let e = match_pool[mo + i];
        match_pool.push(e);
    }
    match_pool.push((el_fact, alpha_id));
    // Concat left + right-only into the bind pool. Do not hold a slice across push
    // (`DESIGN-STONE-token-bind-pool`, `DESIGN-STONE-bind-value-intern`).
    let lo = tok.binds.off as usize;
    let ln = tok.binds.len as usize;
    let eo = el_span.off as usize;
    let en = el_span.len as usize;
    let start = pool.len();
    for i in 0..ln {
        let p = pool[lo + i];
        pool.push(p);
    }
    for i in 0..en {
        let (k, v) = pool[eo + i];
        let already = (start..start + ln).any(|j| pool[j].0 == k);
        if !already {
            pool.push((k, v));
        }
    }
    Token {
        matches: BindSpan {
            off: mstart as u32,
            len: (match_pool.len() - mstart) as u16,
        },
        binds: BindSpan {
            off: start as u32,
            len: (pool.len() - start) as u16,
        },
    }
}

/// Share the Element's bind-span as the new Token's binds.
/// Root-join is the one place a Token is born from an Element; every other
/// Token is produced by `extend_token`, which concats spans in `bind_pool`.
fn seed_token_binds(el: &Element) -> BindSpan {
    el.binds
}

fn token_assoc(
    tok: &Token,
    k: Value,
    v: Value,
    keys: &mut Vec<Value>,
    vals: &mut Vec<Value>,
    ids: &mut crate::rete::compiled_cond::ValIntern,
    pool: &mut Vec<(u32, u32)>,
) -> Token {
    let kid = intern_key(keys, &k);
    let vid = intern_val(vals, ids, v);
    let pairs: Vec<(u32, u32)> = pool_slice(pool, tok.binds).to_vec();
    let start = pool.len();
    let mut found = false;
    for (ek, ev) in pairs {
        if ek == kid {
            pool.push((ek, vid));
            found = true;
        } else {
            pool.push((ek, ev));
        }
    }
    if !found {
        pool.push((kid, vid));
    }
    Token {
        matches: tok.matches,
        binds: BindSpan {
            off: start as u32,
            len: (pool.len() - start) as u16,
        },
    }
}

/// Keyed hash-join helper (P3 — shared by batch `hash_join_pass` and delta `fire_fixpoint_delta`).
///
/// Joins `left_tokens` (native `Token`) against `right_elements` (Value Element Records) using the
/// keyed index-and-probe strategy. Returns the new extended tokens produced by the join. If either
/// slice is empty, returns an empty Vec (no join possible). `alpha_id` is recorded in each new
/// token's matches vec.
///
/// The join_keys (sorted intersection of token/element binding keys) are derived from the
/// first element of each slice — callers must guarantee both slices are non-empty.
pub(crate) fn join_extend(
    tok: &Token,
    el: &Element,
    alpha_id: i64,
    ctx: &mut FireCtx<'_>,
) -> Result<Option<Token>, EvalBreak> {
    let compiled = rematch_compiled(ctx.compiled_conds, alpha_id)?;
    // No leftover SeedCmp: the keyed bucket is the join (same contract as
    // fold-the-wall). Rematch cannot reject a member (`DESIGN-STONE-join-extend-no-leftover`).
    if compiled.has_seed_cmp()
        && fact_bindings_under(
            fact_at(ctx.facts, ctx.derived, ctx.n_input, el.fact),
            &bind_view(ctx.keys, ctx.vals, ctx.pool, tok.binds),
            compiled,
            ctx.scratch,
        )
        .is_none()
    {
        return Ok(None);
    }
    Ok(Some(extend_token(
        tok,
        el.fact,
        el.binds,
        alpha_id,
        ctx.pool,
        ctx.match_pool,
    )))
}

fn keyed_join(
    left_tokens: &[Token],
    right_elements: &[Element],
    alpha_id: i64,
    ctx: &mut FireCtx<'_>,
) -> Result<Vec<Token>, EvalBreak> {
    if left_tokens.is_empty() || right_elements.is_empty() {
        return Ok(vec![]);
    }

    // Step 1: compute join_keys = sorted shared variable names (intersection of binding key-sets).
    let join_keys: Vec<Value> = {
        let sample_tok_bindings = bind_view(ctx.keys, ctx.vals, ctx.pool, left_tokens[0].binds);
        let sample_el_bindings = element_fact_bindings(&right_elements[0], ctx.keys, ctx.vals, ctx.pool);
        let mut keys: Vec<Value> = sample_tok_bindings
            .iter()
            .map(|(k, _)| k.clone())
            .filter(|k| Bindings::get(&sample_el_bindings, k).is_some())
            .collect();
        // Binding keys are Value::String (variable names like "?loc").
        // Sort by their string content for a stable canonical order.
        keys.sort_by(|a, b| {
            let a_str = match a {
                Value::String(s) => s.as_str(),
                _ => "",
            };
            let b_str = match b {
                Value::String(s) => s.as_str(),
                _ => "",
            };
            a_str.cmp(b_str)
        });
        keys
    };

    // Step 2: index RIGHT (elements) by join-key-value tuple.
    let mut index: JoinKeyMap<usize> = HashMap::new();
    for (i, el) in right_elements.iter().enumerate() {
        let el_bindings = element_fact_bindings(el, ctx.keys, ctx.vals, ctx.pool);
        let key: Vec<Value> = join_keys
            .iter()
            .map(|k| {
                Bindings::get(&el_bindings, k)
                    .cloned()
                    .expect("keyed_join: join key missing from element bindings")
            })
            .collect();
        index.entry(key).or_default().push(i);
    }

    // Step 3: probe with each LEFT (token).
    let mut out: Vec<Token> = Vec::new();
    for tok in left_tokens {
        let probe_key: Vec<Value> = join_keys
            .iter()
            .map(|k| {
                Bindings::get(&bind_view(ctx.keys, ctx.vals, ctx.pool, tok.binds), k)
                    .cloned()
                    .expect("keyed_join: join key missing from token bindings")
            })
            .collect();
        if let Some(bucket) = index.get(&probe_key) {
            for &el_idx in bucket {
                if let Some(new_tok) = join_extend(tok, &right_elements[el_idx], alpha_id, ctx)? {
                    out.push(new_tok);
                }
            }
        }
    }
    Ok(out)
}

/// `hash-join-pass` / `cross-join-node` — propagate tokens from a left-parent to
/// its HashJoinNode children, in ascending node-id order (topological).
/// Left parents: RootJoin / HashJoin / Test / Negation / Exists / Accumulate.
/// Mirrors `wat/rete/oracle/pass.wat` hash-join-pass (A1: a TestNode may parent a HashJoin).
pub(crate) fn hash_join_pass(wm: &mut FireSession, arm: &InternedNetwork) -> Result<(), EvalBreak> {
    let node_ids = &arm.node_ids;
    let mut match_scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
    let mut ctx = FireCtx {
        compiled_conds: &arm.compiled_conds,
        scratch: &mut match_scratch,
        pool: &mut wm.bind_pool,
        match_pool: &mut wm.match_pool,
        keys: &wm.bind_keys,
        vals: &wm.bind_vals,
        facts: &wm.facts,
        derived: &wm.derived_facts,
        n_input: wm.n_input,
    };

    for node_id in node_ids {
        // Group C: use &Value ref from wm.network; borrow ends after node_children (NLL).
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        let kind = kind_of(node);
        if kind != "RootJoinNode"
            && kind != "HashJoinNode"
            && kind != "TestNode"
            && kind != "NegationNode"
            && kind != "ExistsNode"
            && kind != "AccumulateNode"
        {
            continue;
        }
        let child_ids = node_children(node);
        // node's last use is node_children above; wm.network borrow for `node` ends here (NLL).

        // tokens must remain a clone: wm.beta[node_id] is read here, wm.beta[child_id] is
        // mutated below — Rust cannot prove key disjointness, so the borrow would conflict.
        // With native Token the clone copies the Vec<Token> (cheap Vec of structs).
        let tokens: Vec<Token> = match wm.beta.get(node_id) {
            Some(ts) => ts.clone(),
            None => continue, // no tokens → skip
        };
        for child_id in &child_ids {
            // Group C: child_node ref — only used for kind_of; borrow ends before wm mutations.
            let child_node = match get_node(&wm.network, *child_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(child_node) != "HashJoinNode" {
                continue;
            }
            // Find the feeding alpha for this HashJoinNode.
            let alpha_id = alpha_feeding(*child_id, &wm.network);
            // Group C: borrow elements from wm.alpha — wm.beta mutations are on a different field.
            let elements = match wm.alpha.get(&alpha_id) {
                Some(els) => els.as_slice(),
                None => continue, // no right-side elements → skip
            };
            let new_tokens = keyed_join(&tokens, elements, alpha_id, &mut ctx)?;
            for new_tok in new_tokens {
                wm.beta.entry(*child_id).or_default().push(new_tok);
            }
        }
    }
    Ok(())
}

// ── Pass 4: Production pass ───────────────────────────────────────────────────

/// Delta tokens at every non-alpha parent of `node_id`. Condition `:or` leaves
/// N terminals; a later Test/:not/:exists/accum must see all of them.
fn d_beta_from_parents(
    parents_of: &ParentsOf,
    d_beta: &BetaMemory,
    node_id: i64,
) -> Vec<Token> {
    let mut out = Vec::new();
    if let Some(pids) = parents_of.get(&node_id) {
        for pid in pids {
            if let Some(ts) = d_beta.get(pid) {
                out.extend(ts.iter().cloned());
            }
        }
    }
    out
}

fn node_parents(child_id: i64, network: &Value) -> Vec<i64> {
    let node_ids: Vec<i64> = match network {
        Value::wat__core__PersistentMap(m) => m
            .keys()
            .into_iter()
            .filter_map(|k| if let Value::i64(n) = k { Some(n) } else { None })
            .collect(),
        _ => return vec![],
    };
    let mut out = Vec::new();
    for node_id in &node_ids {
        let node = match get_node(network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if node_children(node).contains(&child_id) {
            out.push(*node_id);
        }
    }
    out
}

/// `production-pass` / `fire-production` — for each ProductionNode, find its parent's beta tokens,
/// for each token × each compiled `:then` form, `exec_compiled_rhs`, push to `production[prod_id]`.
/// Mirrors `wat/rete/oracle/pass.wat`.
pub(crate) fn production_pass(wm: &mut FireSession, arm: &InternedNetwork, sym: &SymbolTable) -> Result<(), EvalBreak> {
    let node_ids = &arm.node_ids;

    for node_id in node_ids {
        // Group C: use &Value ref from wm.network; borrow ends after rule_name extraction (NLL).
        // wm.production mutations below are on a different field — no conflict.
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != "ProductionNode" {
            continue;
        }
        // ProductionNode: id(0), rule-name(1)
        let (_, sf) = node_record(node).unwrap();
        let rule_name = match &sf[1] {
            Value::String(s) => s.as_str(),
            _ => continue,
        };

        let Some(compiled_rhs) = arm.compiled_rhs.get(rule_name) else {
            continue;
        };

        // All non-alpha parents (condition `:or` wires N arm terminals to one production).
        // Slots from the first token of THIS parent — `:or` arms may not share layout
        // (`DESIGN-STONE-rhs-bind-slot`).
        for pid in node_parents(*node_id, &wm.network) {
            let Some(ts) = wm.beta.get(&pid) else {
                continue;
            };
            if ts.is_empty() {
                continue;
            }
            let first = bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, ts[0].binds);
            let slot_tables: crate::rete::compiled_rhs::RhsSlotTables = compiled_rhs
                .iter()
                .map(|c| crate::rete::compiled_rhs::rhs_bind_slots(c, &first))
                .collect();
            for tok in ts {
                let pairs = bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
                for (compiled, slots) in compiled_rhs.iter().zip(&slot_tables) {
                    let derived = crate::rete::compiled_rhs::exec_compiled_rhs_at(
                        compiled, pairs, slots, sym,
                    )?;
                    wm.production.entry(*node_id).or_default().push(derived);
                }
            }
        }
    }
    Ok(())
}

// ── Pure single-pass fn (extracted for fixpoint reuse) ───────────────────────

/// Pure single-pass fire: `to_transient` → clear memories → four passes → `to_persistent`.
///
/// Public `fire-once` evaluates its AST then delegates here. `fire-rules` does
/// **not** re-run this; it calls `fire_fixpoint_delta` (or the stratified
/// driver wrapping it). Mirrors `fire-once$oracle` (`wat/rete/oracle/fire.wat`):
/// re-run-from-scratch each call (memories cleared).
pub(crate) fn fire_once_session(session: &Value, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let mut wm = to_transient(session)?;
    let rules_empty = matches!(&wm.rules, Value::wat__core__PersistentVector(pv) if pv.is_empty());
    if rules_empty
        && rete_arm_lookup(network_identity(&wm.network).unwrap_or(0)).is_none()
        && network_has_production(&wm.network)
    {
        return Err(refuse_export_without_arm(
            ":wat::rete::fire-once",
            &crate::rust_caller_span!(),
        ));
    }

    // Clear memories — re-run-from-scratch.
    wm.alpha.clear();
    wm.beta.clear();
    wm.production.clear();
    wm.bind_pool.clear();
    wm.bind_keys.clear();
    wm.bind_vals.clear();
    wm.bind_val_ids.clear();
    wm.match_pool.clear();
    wm.derived_facts.clear();

    let arm = rete_arm_get_or_build(&wm.network, &wm.rules, sym)?;

    // Four passes (alpha → root-join → hash-join → production).
    alpha_pass(&mut wm, &arm)?;
    root_join_pass(&mut wm);
    hash_join_pass(&mut wm, &arm)?;
    production_pass(&mut wm, &arm, sym)?;

    harvest_query_memory(&mut wm);
    // Drop ephemeral beta tokens before freeze — derived facts live in production-memory.
    // (Re-generated on every fire; never read from a frozen Session's beta-memory by native fire.)
    wm.beta.clear();
    Ok(to_persistent(wm))
}

fn harvest_query_memory(wm: &mut FireSession) {
    wm.query.clear();
    let node_ids = sorted_node_ids(&wm.network);
    for node_id in node_ids {
        let node = match get_node(&wm.network, node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != "QueryNode" {
            continue;
        }
        let (_, sf) = match node_record(node) {
            Some(p) => p,
            None => continue,
        };
        let qname = match &sf[1] {
            Value::String(s) => s.as_ref().clone(),
            _ => continue,
        };
        let mut maps: Vec<crate::value::pmap::PMap> = Vec::new();
        for pid in node_parents(node_id, &wm.network) {
            if let Some(ts) = wm.beta.get(&pid) {
                maps.extend(ts.iter().map(|t| pmap_from_span(t.binds, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool)));
            }
        }
        wm.query.insert(qname, maps);
    }
}

// ── Public entry: native fire-once ───────────────────────────────────────────

/// `(:wat::rete::fire-once <session>) -> :wat::rete::Session`
///
/// Native Rust single-pass fire cycle: alpha → root-join → hash-join → production.
/// Observationally equivalent to the wat oracle's `fire-once$oracle`.
///
/// Dispatch entry called from `runtime.rs:dispatch_keyword_head_value`.
/// Evaluates the single argument (must be `:wat::rete::Session`), runs the four passes
/// over the native `FireSession`, and returns a frozen `Session` via `to_persistent`.
pub(crate) fn eval_fire_once_native(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::fire-once";
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

    // Evaluate the session argument, then delegate to the pure single-pass fn.
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    fire_once_session(&session, sym)
}

// ── Cascade fixpoint helpers (P4a) ───────────────────────────────────────────

/// Flatten `production-memory`'s per-node `PV<Record>` values into one `Vec<Value>`.
///
/// `production-memory` is a `PersistentMap<node-id, PV<Record>>`. The outer pass visits
/// each node's PV; the inner pass collects each Record. Mirrors `collect-derived`
/// (`wat/rete/oracle/fire.wat`).
///
/// Used by the 7-strat-native stratified driver (`fire_rules_stratified`) to collect
/// each stratum's derived facts.
pub(crate) fn collect_derived(production_pm: &Value) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    if let Value::wat__core__PersistentMap(m) = production_pm {
        for (_k, v) in m.iter() {
            if let Value::wat__core__PersistentVector(pv) = v {
                for fact in pv.iter() {
                    out.push(fact.clone());
                }
            }
        }
    }
    out
}

/// Fold `derived` facts into the existing `facts` PersistentVector, conj-ing ONLY facts
/// not already present (structural `==` dedup).
///
/// The dedup is the termination guard: if every derived fact is already in `facts`, the
/// result length equals `facts` length → the fixpoint loop exits. Re-adding a present
/// fact would grow `facts` every round and spin forever. Mirrors `merge-facts`
/// (`wat/rete/oracle/fire.wat`).
///
/// Used by the 7-strat-native stratified driver (`fire_rules_stratified`) — R18: the cross-stratum
/// derived-fact accumulation MUST value-dedup (mirrors the oracle's `merge-facts`,
/// `wat/rete/oracle/fire.wat`), not concat, or a fact produced by more than one stratum's
/// query is double-counted.
///
/// P9 perf: membership is checked via a `HashSet` mirror of `pv`'s contents, not a linear
/// `.any()` scan — the former was O(len(pv)) PER derived fact (O(n²) over a stratum-chain
/// run, since `fire_rules_stratified` calls this once per stratum with `pv` = the whole
/// accumulated closure so far), the exact quadratic blow-up behind the `[7,3000]`-class hang.
/// `Value: Hash + Eq` already (the round-loop's own `seen: HashSet<Value>` dedup, above, uses
/// the same property) — same value-dedup semantics, same push_back order, O(len(pv) +
/// len(derived)) instead.
pub(crate) fn merge_facts(facts_pv: &Value, derived: &[Value]) -> Value {
    // Start with a clone of the existing PV.
    let mut pv: rpds::VectorSync<Value> = match facts_pv {
        Value::wat__core__PersistentVector(v) => v.clone(),
        _ => rpds::VectorSync::new_sync(),
    };
    let mut present: std::collections::HashSet<Value> = pv.iter().cloned().collect();
    for fact in derived {
        // Conj only if not already present (structural equality, now O(1) amortized).
        if present.insert(fact.clone()) {
            pv.push_back_mut(fact.clone());
        }
    }
    Value::wat__core__PersistentVector(pv)
}

/// Rebuild a frozen Session from a fired session, replacing only the `facts` field.
///
/// Used in the fixpoint to carry `new_facts` into the next round and in `eval_fire_rules_native`
/// to restore `facts = input` before returning. Mirrors the Session reconstruction in
/// `fire-fixpoint` (`wat/rete/oracle/fire.wat`) and `fire-rules` (`wat/rete/oracle/fire.wat`).
pub(crate) fn session_with_facts(fired: &Value, new_facts: Value) -> Value {
    match fired {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let sf = a.fields.as_slice();
            Value::Aggregate(Arc::new(AggregateValue::record_arc(
                a.class.clone(),
                a.names.clone(),
                Arc::new(vec![
                    sf[0].clone(), // network
                    sf[1].clone(), // rules
                    sf[2].clone(), // alpha-memory
                    sf[3].clone(), // beta-memory
                    sf[4].clone(), // production-memory
                    new_facts,     // facts (replaced)
                    sf[6].clone(), // next-id
                    if sf.len() > 7 {
                        sf[7].clone()
                    } else {
                        Value::wat__core__PersistentMap(crate::value::pmap::PMap::new())
                    },
                ]),
            )))
        }
        // Should never happen — callers pass only a Session; pass through unchanged.
        other => other.clone(),
    }
}

/// Read the `facts` field (position 5) from a frozen Session Value.
///
/// Used by the 7-strat-native stratified driver (`fire_rules_stratified`) to read a
/// session's current fact set (both the original input session and each stratum's fired
/// sub-session).
pub(crate) fn session_facts(session: &Value) -> Value {
    match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice()[5].clone(),
        _ => Value::wat__core__PersistentVector(rpds::VectorSync::new_sync()),
    }
}

/// Read the `network` field (position 0) from a frozen Session Value.
/// Declaration order: network(0) rules(1) … facts(5) next-id(6) query-memory(7).
pub(crate) fn session_network(session: &Value) -> Option<&Value> {
    match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice().first(),
        _ => None,
    }
}

pub(crate) fn network_has_production(network: &Value) -> bool {
    sorted_node_ids(network)
        .iter()
        .any(|&id| get_node(network, id).is_some_and(|n| kind_of(n) == "ProductionNode"))
}

pub(crate) fn refuse_export_without_arm(op: &'static str, span: &Span) -> EvalBreak {
    RuntimeError::new(
        span.clone(),
        RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: "cannot consume an Export without interned arm — empty rules, live network"
                .into(),
        },
    )
    .into()
}

pub(crate) fn rules_lack_ast(rules: &[Value]) -> bool {
    if rules.is_empty() {
        return true;
    }
    rules.iter().all(|r| match node_record(r) {
        Some((_, sf)) if sf.len() > 1 => match &sf[1] {
            Value::wat__core__PersistentVector(pv) => {
                !pv.iter().any(|x| matches!(x, Value::wat__WatAST(_)))
            }
            _ => true,
        },
        _ => true,
    })
}

pub(crate) fn synthetic_rule(name: &str) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::rete::Rule".into(),
        crate::value::value::names_arc_from_static(RULE_FIELDS),
        Arc::new(vec![
            Value::String(Arc::new(name.to_string())),
            Value::wat__core__PersistentVector(rpds::VectorSync::new_sync()),
            Value::wat__core__PersistentVector(rpds::VectorSync::new_sync()),
        ]),
    )))
}

pub(crate) fn fire_rules_from_deps(
    session: &Value,
    deps: &[RuleDep],
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let mut parts: Vec<RuleParts> = Vec::with_capacity(deps.len());
    for d in deps {
        parts.push(RuleParts {
            rule: synthetic_rule(&d.name),
            produced: d.produced.clone(),
            negated: d.negated.clone(),
            consumed: d.consumed.clone(),
            bag: d.bag.clone(),
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
        return fire_fixpoint_delta(session, sym, None);
    }
    fire_rules_stratified(session, &parts, &rule_strata, max_s, sym)
}

pub(crate) fn session_rules(session: &Value) -> Value {
    match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice()[1].clone(),
        _ => Value::wat__core__PersistentVector(rpds::VectorSync::new_sync()),
    }
}

// ── key_of helper ────────────────────────────────────────────────────────────

/// Extract a join key tuple from a bindings map given the pre-computed `join_keys` list.
///
/// `join_keys` is the sorted list of shared variable names (same tuple `keyed_join` computes).
/// Returns `Vec<Value>` of the bound values in key order. For empty `join_keys` (cartesian
/// product case) returns `vec![]` — all tokens/elements share the single empty-key bucket.
///
/// Panics if a join key is absent from `bindings` (structurally impossible in a well-formed
/// rete network; all shared variables must be bound before this node is reached).
pub(crate) fn key_of<B: Bindings + ?Sized>(bindings: &B, join_keys: &[Value]) -> Vec<Value> {
    join_keys
        .iter()
        .map(|k| {
            bindings
                .get(k)
                .cloned()
                .unwrap_or_else(|| panic!("key_of: join key {:?} missing from bindings", k))
        })
        .collect()
}

/// Derive the join-key tuple shared between `sample_bindings` and `elements` — the cheap half of
/// `gather_index` (step 1 of the `keyed_join` (`:779-834`) shape): a sorted intersection of
/// `sample_bindings`' keys and a sample element's keys, string-sorted for a stable canonical
/// order, derived from `elements[0]` when non-empty. An empty `elements` slice yields `[]`.
///
/// Split out from the index build so a cache lookup can key on `(alpha_id, join_keys)` *before*
/// paying for the expensive half (`build_gather_index`) — the gather-index cache's ordering
/// constraint (`DESIGN-STONE-gather-index-cache.md`).
fn gather_join_keys<B: Bindings + ?Sized>(
    sample_bindings: &B,
    elements: &[Element],
    bind_keys: &[Value],
    vals: &[Value],
    pool: &[(u32, u32)],
) -> Vec<Value> {
    if elements.is_empty() {
        return Vec::new();
    }
    let sample_el_bindings = element_fact_bindings(&elements[0], bind_keys, vals, pool);
    let mut keys: Vec<Value> = sample_bindings
        .iter()
        .map(|(k, _)| k)
        .filter(|k| Bindings::get(&sample_el_bindings, k).is_some())
        .cloned()
        .collect();
    // Binding keys are Value::String (variable names like "?loc").
    // Sort by their string content for a stable canonical order.
    keys.sort_by(|a, b| {
        let a_str = match a {
            Value::String(s) => s.as_str(),
            _ => "",
        };
        let b_str = match b {
            Value::String(s) => s.as_str(),
            _ => "",
        };
        a_str.cmp(b_str)
    });
    keys
}

/// Join-key → element indices (bucket), as built by `build_gather_index`.
/// Unary when `join_keys.len() == 1` — no one-element `Vec` (`DESIGN-STONE-gather-unary-index`).
type GatherUnary = FxHashMap<u32, Vec<usize>>;
type GatherNary = FxHashMap<Vec<Value>, Vec<usize>>;

pub(crate) enum GatherIndex {
    /// One join key: interned filler id (`DESIGN-STONE-gather-val-id`).
    UnaryId(GatherUnary),
    Nary(GatherNary),
}

impl GatherIndex {
    fn bucket(&self, key: &[Value], val_ids: &ValIntern) -> &[usize] {
        match self {
            Self::UnaryId(m) => key
                .first()
                .and_then(|k| val_ids.get(k))
                .and_then(|vid| m.get(&vid))
                .map_or(&[], Vec::as_slice),
            Self::Nary(m) => m.get(key).map_or(&[], Vec::as_slice),
        }
    }

    /// Push new alpha indices into existing buckets (`DESIGN-STONE-persist-gather-across-rounds`).
    /// New ids are `>=` the previous length (alpha only appends). Foldl order holds.
    fn append(
        &mut self,
        new_idxs: &[usize],
        elements: &[Element],
        join_keys: &[Value],
        bind_keys: &[Value],
        vals: &[Value],
        pool: &[(u32, u32)],
    ) {
        if new_idxs.is_empty() || elements.is_empty() || join_keys.is_empty() {
            return;
        }
        match self {
            Self::UnaryId(m) => {
                let Some(kid) = bind_keys
                    .iter()
                    .position(|k| k == &join_keys[0])
                    .map(|i| i as u32)
                else {
                    return;
                };
                for &i in new_idxs {
                    let pairs = pool_slice(pool, elements[i].binds);
                    if let Some((_, vid)) = pairs.iter().find(|(k, _)| *k == kid) {
                        m.entry(*vid).or_default().push(i);
                    }
                }
            }
            Self::Nary(m) => {
                for &i in new_idxs {
                    let el_bindings = element_fact_bindings(&elements[i], bind_keys, vals, pool);
                    let key = key_of(&el_bindings, join_keys);
                    m.entry(key).or_default().push(i);
                }
            }
        }
    }
}

/// Fire-scoped cache: `(alpha_id, join_keys) -> index`. Buckets are indices
/// into `wm.alpha[alpha_id]` (`DESIGN-STONE-gather-no-snapshot`).
/// Persists across rounds; `append` takes `d_alpha`
/// (`DESIGN-STONE-persist-gather-across-rounds`). Not a Session field.
type GatherCache = FxHashMap<(i64, Vec<Value>), GatherIndex>;

fn append_d_alpha(
    cache: &mut GatherCache,
    d_alpha: &AlphaDelta,
    wm: &FireSession,
) {
    for ((aid, join_keys), idx) in cache.iter_mut() {
        let Some(news) = d_alpha.get(aid) else {
            continue;
        };
        let els = alpha_elements(&wm.alpha, *aid);
        idx.append(
            news,
            els,
            join_keys,
            &wm.bind_keys,
            &wm.bind_vals,
            &wm.bind_pool,
        );
    }
}

fn alpha_elements(alpha: &AlphaMemory, alpha_id: i64) -> &[Element] {
    alpha.get(&alpha_id).map(Vec::as_slice).unwrap_or(&[])
}

/// Build the bucket index over `elements` for a given `join_keys` tuple — the expensive half of
/// `gather_index` (the full scan). Buckets hold element *indices* in iteration order, matching
/// `keyed_join`'s right-index and the wat oracle's foldl order.
///
/// Panics only via `key_of` if an element's bindings lack a derived join key — structurally
/// impossible for a well-formed network (every element at one alpha node shares a binding
/// key-set, the same guarantee `keyed_join` already rests on).
pub(crate) fn build_gather_index(
    elements: &[Element],
    join_keys: &[Value],
    bind_keys: &[Value],
    vals: &[Value],
    pool: &[(u32, u32)],
) -> GatherIndex {
    if join_keys.len() == 1 {
        let mut index: GatherUnary = FxHashMap::default();
        if let Some(kid) = bind_keys
            .iter()
            .position(|k| k == &join_keys[0])
            .map(|i| i as u32)
        {
            for (i, el) in elements.iter().enumerate() {
                let pairs = pool_slice(pool, el.binds);
                if let Some((_, vid)) = pairs.iter().find(|(k, _)| *k == kid) {
                    index.entry(*vid).or_default().push(i);
                }
            }
        }
        GatherIndex::UnaryId(index)
    } else {
        let mut index: GatherNary = FxHashMap::default();
        for (i, el) in elements.iter().enumerate() {
            let el_bindings = element_fact_bindings(el, bind_keys, vals, pool);
            let key = key_of(&el_bindings, join_keys);
            index.entry(key).or_default().push(i);
        }
        GatherIndex::Nary(index)
    }
}

/// Get-or-build the fire-scoped gather index for `alpha_id` under `sample`'s shared keys.
/// Acc, Negation, and Exists all miss through here so one pair is built once per fire
/// and appended across rounds (`DESIGN-STONE-persist-gather-across-rounds`).
fn ensure_gather<B: Bindings + ?Sized>(
    cache: &mut GatherCache,
    wm: &FireSession,
    alpha_id: i64,
    sample: &B,
) -> Vec<Value> {
    let els = alpha_elements(&wm.alpha, alpha_id);
    let join_keys = gather_join_keys(sample, els, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
    let cache_key = (alpha_id, join_keys.clone());
    cache.entry(cache_key).or_insert_with(|| {
        census_count("accum:index-builds");
        census_count_n("accum:index-elements", els.len() as u64);
        build_gather_index(els, &join_keys, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool)
    });
    join_keys
}

/// Exists/Not Leaf: probe the token's bucket. Empty bucket is absence (contract clause 2).
fn any_seeded_keyed<B: Bindings + ?Sized>(
    cache: &mut GatherCache,
    wm: &FireSession,
    alpha_id: i64,
    seed: &B,
    compiled: &crate::rete::compiled_cond::CompiledCond,
    scratch: &mut SlotFrame,
) -> bool {
    let join_keys = ensure_gather(cache, wm, alpha_id, seed);
    let key = key_of(seed, &join_keys);
    let index = cache
        .get(&(alpha_id, join_keys))
        .expect("ensure_gather just inserted");
    let elements = alpha_elements(&wm.alpha, alpha_id);
    let bucket = index.bucket(&key, &wm.bind_val_ids);
    bucket.iter().any(|&i| {
        census_gather_visit();
        fact_bindings_under(
            fact_at(&wm.facts, &wm.derived_facts, wm.n_input, elements[i].fact),
            seed,
            compiled,
            scratch,
        )
        .is_some()
    })
}

/// Leftover rematch / combinator Leaf: every matching binding in the token's bucket.
fn seeded_bindings_keyed(
    cache: &mut GatherCache,
    wm: &FireSession,
    alpha_id: i64,
    seed: &crate::value::pmap::PMap,
    compiled: &crate::rete::compiled_cond::CompiledCond,
    scratch: &mut SlotFrame,
) -> Vec<crate::value::pmap::PMap> {
    let join_keys = ensure_gather(cache, wm, alpha_id, seed);
    let key = key_of(seed, &join_keys);
    let index = cache
        .get(&(alpha_id, join_keys))
        .expect("ensure_gather just inserted");
    let elements = alpha_elements(&wm.alpha, alpha_id);
    let bucket = index.bucket(&key, &wm.bind_val_ids);
    bucket
        .iter()
        .filter_map(|&i| {
            census_gather_visit();
            fact_bindings_under(
                fact_at(&wm.facts, &wm.derived_facts, wm.n_input, elements[i].fact),
                seed,
                compiled,
                scratch,
            )
        })
        .collect()
}

// ── Accumulate folds (8-b) — native mirrors of the wat acc::* fold library ────

/// Read an element's bound `?var` value as an i64 (the value-folds' arg).
/// Mirrors `(Option/expect (PersistentMap/get (Element/bindings e) var) ...)`.
/// Panics on an unbound var or a non-i64 value (a compile-time-impossible shape).
fn acc_var_i64(
    el: &Element,
    var: &Value,
    bind_keys: &[Value],
    vals: &[Value],
    pool: &[(u32, u32)],
) -> i64 {
    let bindings = element_fact_bindings(el, bind_keys, vals, pool);
    match Bindings::get(&bindings, var) {
        Some(Value::i64(n)) => *n,
        Some(other) => panic!("accumulate: var bound to non-i64 {other:?}"),
        None => panic!("accumulate: var {var:?} unbound in element bindings"),
    }
}

/// Slot of `var` on the first bucket element. Empty bucket → None (count/sum
/// emit identity; min/max/mean drop). Derived from a live Element, never stored
/// on the interned `AccFold` (`DESIGN-STONE-accum-fold-the-wall`).
fn operand_slot(
    elements: &[Element],
    bucket: &[usize],
    var: &Value,
    bind_keys: &[Value],
    pool: &[(u32, u32)],
) -> Option<usize> {
    let &i = bucket.first()?;
    pool_slice(pool, elements[i].binds)
        .iter()
        .position(|(id, _)| bind_keys.get(*id as usize) == Some(var))
}

fn slot_i64(el: &Element, slot: usize, vals: &[Value], pool: &[(u32, u32)]) -> i64 {
    match pool_slice(pool, el.binds).get(slot) {
        Some((_, vid)) => match vals.get(*vid as usize) {
            Some(Value::i64(n)) => *n,
            Some(other) => panic!("accumulate: slot bound to non-i64 {other:?}"),
            None => panic!("accumulate: slot {slot} filler id {vid} missing"),
        },
        None => panic!("accumulate: slot {slot} missing in element bindings"),
    }
}

/// Fold a keyed bucket with no leftover `SeedCmp`. The bucket IS the gather
/// (join-key equality ≡ `token_element_compatible`). Count is `len`; value
/// folds read `bindings[slot]`.
fn fold_bucket(
    fold: &AccFold,
    elements: &[Element],
    bucket: &[usize],
    sym: &SymbolTable,
    view: &AccView<'_>,
) -> Result<Option<Value>, EvalBreak> {
    match fold {
        AccFold::Count => Ok(Some(Value::i64(bucket.len() as i64))),
        AccFold::Sum(var) => {
            let Some(slot) = operand_slot(elements, bucket, var, view.keys, view.pool) else {
                return Ok(Some(Value::i64(0)));
            };
            let s: i64 = bucket
                .iter()
                .map(|&i| {
                    census_gather_visit();
                    slot_i64(&elements[i], slot, view.vals, view.pool)
                })
                .sum();
            Ok(Some(Value::i64(s)))
        }
        AccFold::Min(var) => {
            let Some(slot) = operand_slot(elements, bucket, var, view.keys, view.pool) else {
                return Ok(None);
            };
            let mut acc: Option<i64> = None;
            for &i in bucket {
                census_gather_visit();
                let v = slot_i64(&elements[i], slot, view.vals, view.pool);
                acc = Some(match acc {
                    Some(cur) if cur <= v => cur,
                    _ => v,
                });
            }
            Ok(acc.map(Value::i64))
        }
        AccFold::Max(var) => {
            let Some(slot) = operand_slot(elements, bucket, var, view.keys, view.pool) else {
                return Ok(None);
            };
            let mut acc: Option<i64> = None;
            for &i in bucket {
                census_gather_visit();
                let v = slot_i64(&elements[i], slot, view.vals, view.pool);
                acc = Some(match acc {
                    Some(cur) if cur >= v => cur,
                    _ => v,
                });
            }
            Ok(acc.map(Value::i64))
        }
        AccFold::Mean(var) => {
            let Some(slot) = operand_slot(elements, bucket, var, view.keys, view.pool) else {
                return Ok(None);
            };
            let n = bucket.len() as i64;
            let s: i64 = bucket
                .iter()
                .map(|&i| {
                    census_gather_visit();
                    slot_i64(&elements[i], slot, view.vals, view.pool)
                })
                .sum();
            Ok(Some(Value::i64(s / n)))
        }
        AccFold::Distinct(_)
        | AccFold::All
        | AccFold::GroupBy(_)
        | AccFold::User { .. } => {
            let gathered: Vec<&Element> = bucket.iter().map(|&i| &elements[i]).collect();
            accumulate_value(fold, &gathered, sym, view)
        }
    }
}

fn project_group_keys<B: Bindings + ?Sized>(
    el_bindings: &B,
    keys: &[Value],
) -> Vec<(Value, Value)> {
    keys.iter()
        .filter_map(|k| el_bindings.get(k).map(|v| (k.clone(), v.clone())))
        .collect()
}

fn accumulate_value(
    fold: &AccFold,
    gathered: &[&Element],
    sym: &SymbolTable,
    view: &AccView<'_>,
) -> Result<Option<Value>, EvalBreak> {
    Ok(match fold {
        AccFold::Count => Some(Value::i64(gathered.len() as i64)),
        AccFold::Sum(var) => {
            let s: i64 = gathered.iter().map(|el| acc_var_i64(el, var, view.keys, view.vals, view.pool)).sum();
            Some(Value::i64(s))
        }
        AccFold::Min(var) => {
            // None seed; first element sets it, subsequent narrow with `<`. Empty → None.
            let mut acc: Option<i64> = None;
            for el in gathered {
                let v = acc_var_i64(el, var, view.keys, view.vals, view.pool);
                acc = Some(match acc {
                    Some(cur) => {
                        if v < cur {
                            v
                        } else {
                            cur
                        }
                    }
                    None => v,
                });
            }
            acc.map(Value::i64)
        }
        AccFold::Max(var) => {
            let mut acc: Option<i64> = None;
            for el in gathered {
                let v = acc_var_i64(el, var, view.keys, view.vals, view.pool);
                acc = Some(match acc {
                    Some(cur) => {
                        if v > cur {
                            v
                        } else {
                            cur
                        }
                    }
                    None => v,
                });
            }
            acc.map(Value::i64)
        }
        AccFold::Mean(var) => {
            let n = gathered.len() as i64;
            if n == 0 {
                None
            } else {
                let s: i64 = gathered.iter().map(|el| acc_var_i64(el, var, view.keys, view.vals, view.pool)).sum();
                Some(Value::i64(s / n))
            }
        }
        AccFold::Distinct(var) => {
            let mut seen: HashSet<i64> = HashSet::new();
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for el in gathered {
                let n = acc_var_i64(el, var, view.keys, view.vals, view.pool);
                if seen.insert(n) {
                    pv.push_back_mut(Value::i64(n));
                }
            }
            Some(Value::wat__core__PersistentVector(pv))
        }
        AccFold::All => {
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for el in gathered {
                pv.push_back_mut(fact_at(view.facts, view.derived, view.n_input, el.fact).clone());
            }
            Some(Value::wat__core__PersistentVector(pv))
        }
        AccFold::GroupBy(var) => {
            let mut groups: HashMap<i64, rpds::VectorSync<Value>> = HashMap::new();
            for el in gathered {
                let fact = fact_at(view.facts, view.derived, view.n_input, el.fact).clone();
                let k = acc_var_i64(el, var, view.keys, view.vals, view.pool);
                groups
                    .entry(k)
                    .or_insert_with(rpds::VectorSync::new_sync)
                    .push_back_mut(fact);
            }
            Some(Value::wat__core__PersistentMap(
                crate::value::pmap::PMap::from_pairs(groups.into_iter().map(|(k, pv)| {
                    (Value::i64(k), Value::wat__core__PersistentVector(pv))
                })),
            ))
        }
        AccFold::User { var, program } => {
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for el in gathered {
                pv.push_back_mut(Value::i64(acc_var_i64(el, var, view.keys, view.vals, view.pool)));
            }
            let gathered_pv = Value::wat__core__PersistentVector(pv);
            Some(crate::rete::expr_ir::exec_call(
                program,
                &[gathered_pv],
                sym,
                &crate::rust_caller_span!(),
            )?)
        }
    })
}

fn exec_stashed_where(
    programs: &HashMap<i64, crate::rete::expr_ir::Program>,
    node_id: i64,
    bindings: &(impl Bindings + ?Sized),
    sym: &SymbolTable,
) -> Result<bool, EvalBreak> {
    let Some(program) = programs.get(&node_id) else {
        return Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::fire-rules".into(),
                reason: format!(
                    "TestNode {node_id} has no compiled where — compile-condition should have refused"
                ),
            },
        )
        .into());
    };
    crate::rete::expr_ir::exec_where(program, bindings, sym, &program.span)
}

/// Mut sink for one where-dispatch. Named so the fire loop does not grow
/// an 8-arg helper (validate.rs `ClauseCtx` — a struct, not an allow).
struct WhereSink<'a> {
    where_tree: &'a crate::rete::where_tree::WhereTree,
    compiled_wheres: &'a HashMap<i64, crate::rete::expr_ir::Program>,
    beta_readers: &'a HashSet<i64>,
    wm: &'a mut FireSession,
    d_beta: &'a mut BetaMemory,
    sym: &'a SymbolTable,
}

/// (b) — eval only the TestNodes the where-tree says this token could still pass.
/// `exec_where` stays the authority. Tree miss → skip the eval (over-approx only).
fn dispatch_where_tests(
    tids: &[i64],
    tokens: &[Token],
    sink: &mut WhereSink<'_>,
) -> Result<(), EvalBreak> {
    if tids.is_empty() || tokens.is_empty() {
        return Ok(());
    }
    let use_tree = tids.iter().any(|id| sink.where_tree.covers(*id));
    if use_tree {
        let span = crate::rust_caller_span!();
        for tok in tokens {
            let binds = bind_view(&sink.wm.bind_keys, &sink.wm.bind_vals, &sink.wm.bind_pool, tok.binds);
            let cands = sink.where_tree.candidates(&binds, &span);
            let proven: HashSet<i64> = cands.proven.into_iter().collect();
            let maybe: HashSet<i64> = cands.maybe.into_iter().collect();
            for &tid in tids {
                // Uncovered ids are not in the tree — always eval.
                // Covered + miss is a proven fail (or a raise we suppress).
                if sink.where_tree.covers(tid) && !proven.contains(&tid) && !maybe.contains(&tid) {
                    continue;
                }
                if proven.contains(&tid) && sink.where_tree.is_pure_cmp(tid) {
                    census_count("filter:test-reuse");
                    census_count("filter:test-pass");
                    if sink.beta_readers.contains(&tid) {
                        beta_written(tid, 1);
                        sink.wm.beta.entry(tid).or_default().push(*tok);
                    }
                    sink.d_beta.entry(tid).or_default().push(*tok);
                    continue;
                }
                census_count("filter:test-evals");
                if exec_stashed_where(sink.compiled_wheres, tid, &binds, sink.sym)? {
                    census_count("filter:test-pass");
                    if sink.beta_readers.contains(&tid) {
                        beta_written(tid, 1);
                        sink.wm.beta.entry(tid).or_default().push(*tok);
                    }
                    sink.d_beta.entry(tid).or_default().push(*tok);
                }
            }
        }
    } else {
        for &tid in tids {
            for tok in tokens {
                census_count("filter:test-evals");
                if exec_stashed_where(
                    sink.compiled_wheres,
                    tid,
                    &bind_view(&sink.wm.bind_keys, &sink.wm.bind_vals, &sink.wm.bind_pool, tok.binds),
                    sink.sym,
                )? {
                    census_count("filter:test-pass");
                    if sink.beta_readers.contains(&tid) {
                        beta_written(tid, 1);
                        sink.wm.beta.entry(tid).or_default().push(*tok);
                    }
                    sink.d_beta.entry(tid).or_default().push(*tok);
                }
            }
        }
    }
    Ok(())
}

fn sorted_parents_of(parents_of: &ParentsOf, id: i64) -> Vec<i64> {
    let mut p = parents_of.get(&id).cloned().unwrap_or_default();
    p.sort_unstable();
    p
}

mod delta;
pub(crate) use delta::*;
