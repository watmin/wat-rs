//! Fire loop: alpha/root/hash/production passes, leftover rematch, delta fixpoint.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

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

/// Split-borrow of the fire working-set. Token/Element are Copy; we cannot
/// hold `&mut WorkingMemory` while walking beta/alpha. Facts stay out of
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

fn acc_view(wm: &WorkingMemory) -> AccView<'_> {
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
/// against that type's alphas. Mirrors `wat/rete.wat:513-537` + `wat/rete.wat:489-508`.
/// A missing compiled cond refuses — do not walk `alpha_match_inner`.
pub(crate) fn alpha_pass(wm: &mut WorkingMemory, arm: &ReteArm) -> Result<(), EvalBreak> {
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
/// Mirrors `wat/rete.wat:544-621`.
pub(crate) fn root_join_pass(wm: &mut WorkingMemory) {
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
/// Mirrors `wat/rete.wat:629-650`. Returns -1 if not found.
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
    wm: &WorkingMemory,
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
    wm: &WorkingMemory,
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
    wm: &WorkingMemory,
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
    let mut index: HashMap<Vec<Value>, Vec<usize>> = HashMap::new();
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
/// Mirrors `wat/rete.wat` hash-join-pass (A1: a TestNode may parent a HashJoin).
pub(crate) fn hash_join_pass(wm: &mut WorkingMemory, arm: &ReteArm) -> Result<(), EvalBreak> {
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
/// Mirrors `wat/rete.wat:867-881` + `wat/rete.wat:828-865`.
pub(crate) fn production_pass(wm: &mut WorkingMemory, arm: &ReteArm, sym: &SymbolTable) -> Result<(), EvalBreak> {
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
            let slot_tables: Vec<Vec<Option<usize>>> = compiled_rhs
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
/// `fire-once'` evaluates its AST then delegates here. `fire-rules'` does **not**
/// re-run this; it calls `fire_fixpoint_delta` (or the stratified driver wrapping it).
/// Mirrors `fire-once` (`wat/rete.wat`): re-run-from-scratch each call (memories cleared).
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

fn harvest_query_memory(wm: &mut WorkingMemory) {
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

// ── Public entry: native fire-once' ──────────────────────────────────────────

/// `(:wat::rete::fire-once <session>) -> :wat::rete::Session`
///
/// Native Rust single-pass fire cycle: alpha → root-join → hash-join → production.
/// Observationally equivalent to the wat oracle's `fire-once`:
/// `query(fire-once' s, T) ≡ query(fire-once s, T)` for every type T.
///
/// Dispatch entry called from `runtime.rs:dispatch_keyword_head_value`.
/// Evaluates the single argument (must be `:wat::rete::Session`), runs the four passes
/// over the native `WorkingMemory`, and returns a frozen `Session` via `to_persistent`.
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
/// (`wat/rete.wat:940-955`).
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
/// (`wat/rete.wat:960-972`).
///
/// Used by the 7-strat-native stratified driver (`fire_rules_stratified`) — R18: the cross-stratum
/// derived-fact accumulation MUST value-dedup (mirrors the oracle's `merge-facts`,
/// `wat/rete.wat:1752`), not concat, or a fact produced by more than one stratum's
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
/// `fire-fixpoint` (`wat/rete.wat:991-998`) and `fire-rules` (`wat/rete.wat:1011-1018`).
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

/// Read the `rules` field (position 1) from a frozen Session Value. Mirrors `session_facts`
/// (position 5) — same field-reading convention as `to_transient` (`wat/rete.wat:124-131`
/// declaration order: network(0) rules(1) alpha-memory(2) beta-memory(3) production-memory(4)
/// facts(5) next-id(6)). Used by `eval_fire_rules_native` to read the rule set once, before
/// deciding fast-path vs stratified dispatch.
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
    for (name, produced, negated, consumed, bag) in deps {
        parts.push((
            synthetic_rule(name),
            produced.clone(),
            negated.clone(),
            consumed.clone(),
            bag.clone(),
        ));
    }
    let pn_only: Vec<RuleDeps> = parts
        .iter()
        .map(|(_, p, n, c, b)| (p.clone(), n.clone(), c.clone(), b.clone()))
        .collect();
    let type_strata = native_stratify(&pn_only)?;
    let mut max_s: i64 = 0;
    let mut rule_strata: Vec<i64> = Vec::with_capacity(parts.len());
    for (_, produced, negated, _consumed, _bag) in &parts {
        let s = native_rule_stratum(produced, negated, &type_strata);
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
pub(crate) enum GatherIndex {
    /// One join key: interned filler id (`DESIGN-STONE-gather-val-id`).
    UnaryId(FxHashMap<u32, Vec<usize>>),
    Nary(FxHashMap<Vec<Value>, Vec<usize>>),
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
    wm: &WorkingMemory,
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
        let mut index: FxHashMap<u32, Vec<usize>> = FxHashMap::default();
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
        let mut index: FxHashMap<Vec<Value>, Vec<usize>> = FxHashMap::default();
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
    wm: &WorkingMemory,
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
    wm: &WorkingMemory,
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
    wm: &WorkingMemory,
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
    wm: &'a mut WorkingMemory,
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

// ── P4b: delta-incremental fixpoint ──────────────────────────────────────────

/// Semi-naive delta fixpoint: persistent memories, per-round delta sets, linear depth.
///
/// Implements the algorithm from DESIGN-STONE-P4b-delta-fire.md:
/// - Memories (`wm.alpha`, `wm.beta`, `wm.production`) accumulate across rounds (never cleared).
/// - Each round propagates only `delta_facts` (the facts derived last round).
/// - Hash-join uses the semi-naive formula:
///   `Δbeta[J] = (Δbeta[P] ⋈ all wm.alpha[A]) ∪ (old_left[P] ⋈ Δalpha[A])`
///   where `old_left[P] = wm.beta[P]` before this round's root-join/hash-join appends.
/// - Terminates when `next_delta_facts` is empty (monotone-finite / datalog).
/// - Returns the persistent session with `facts = input` (same contract as P4a).
///
/// Observationally identical to a naive re-run fixpoint: same token multiset produced,
/// same `wm.production` multiset → identical `query` counts. O(depth²) → linear.
///
/// P6: the hash-join delta step uses persistent per-node `left_idx`/`right_idx`/`join_keys`
/// maintained incrementally across rounds (never rebuilt) — same observable result, O(1)
/// probe cost per match instead of O(W) rebuild per round per node.
/// Step-1 alpha activate for one fact. Shared by the seed worklist (`wm.facts`)
/// and later owned deltas (`DESIGN-STONE-setup-seen-once`).
pub(crate) struct AlphaHit<'a> {
    pub(crate) wm: &'a mut WorkingMemory,
    pub(crate) d_alpha: &'a mut AlphaDelta,
    pub(crate) alpha_tree: &'a crate::rete::alpha_tree::AlphaTree,
    pub(crate) compiled_conds: &'a HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    pub(crate) match_scratch: &'a mut SlotFrame,
    pub(crate) cand_scratch: &'a mut Vec<i64>,
    pub(crate) cond_key_ids: &'a CondKeyIds,
}

pub(crate) fn alpha_activate_fact(
    fact: &Value,
    fact_idx: u32,
    hit: &mut AlphaHit<'_>,
) -> Result<(), EvalBreak> {
    let (fact_class, fact_fields) = match fact {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            (a.class.as_ref(), a.fields.as_slice())
        }
        _ => return Ok(()),
    };
    hit.alpha_tree
        .candidates_into(fact_class, fact_fields, hit.cand_scratch);
    if hit.cand_scratch.is_empty() {
        return Ok(());
    }
    for aid in hit.cand_scratch.iter().copied() {
        let compiled = rematch_compiled(hit.compiled_conds, aid)?;
        let matched = crate::rete::compiled_cond::exec_compiled_with_key_ids(
            compiled,
            fact_fields,
            hit.match_scratch,
            &mut crate::rete::compiled_cond::BindIntern {
                keys: &mut hit.wm.bind_keys,
                vals: &mut hit.wm.bind_vals,
                ids: &mut hit.wm.bind_val_ids,
                pool: &mut hit.wm.bind_pool,
            },
            fact,
            hit.cond_key_ids.get(&aid).map(|v| v.as_slice()),
        );
        if let Some((off, len)) = matched {
            let el = make_element(fact_idx, off, len);
            let slot = {
                let v = hit.wm.alpha.entry(aid).or_default();
                v.push(el);
                v.len() - 1
            };
            hit.d_alpha.entry(aid).or_default().push(slot);
        }
    }
    Ok(())
}

/// Stamped Aggregates membership is the construction fingerprint
/// (`DESIGN-STONE-seen-identity-set`). `identity == 0` still stores `Value`.
pub(crate) fn seen_insert(ids: &mut FxHashSet<u64>, rest: &mut FxHashSet<Value>, v: &Value) -> bool {
    match v {
        Value::Aggregate(a) if a.identity() != 0 => ids.insert(a.identity()),
        _ => rest.insert(v.clone()),
    }
}

pub(crate) fn fire_fixpoint_delta(
    session: &Value,
    sym: &SymbolTable,
    mut support: Option<&mut HashMap<Value, (String, Value)>>,
) -> Result<Value, EvalBreak> {
    let __in = phase_start();
    let mut wm = to_transient(session)?;
    phase_end("IN: to_transient", __in);
    let __setup = phase_start();

    // Start with empty memories (staged session may carry stale state from prior calls).
    wm.alpha.clear();
    wm.beta.clear();
    wm.production.clear();
    wm.bind_pool.clear();
    wm.bind_keys.clear();
    wm.bind_vals.clear();
    wm.bind_val_ids.clear();
    wm.match_pool.clear();
    wm.derived_facts.clear();

    // `seen`: every fact ever in the working set. Seed with all input facts.
    // Mirrors `merge-facts`'s `contains?` guard — ensures each derived fact is processed once.
    // A HashSet (not Vec) so the membership check is O(1): with N derived facts, a Vec + `.contains`
    // is O(N) per check = O(N²) total (the fan-out blow-up); the set makes dedup O(N). Order does not
    // matter — RETE's final fact set is order-independent and the differential gates counts.
    // First worklist IS wm.facts. `seen` is filled once (one clone+hash per
    // input). Later rounds own a Vec of derived facts
    // (`DESIGN-STONE-setup-seen-once`).
    let input_facts: rpds::VectorSync<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => rpds::VectorSync::new_sync(),
    };
    wm.n_input = input_facts.len() as u32;
    wm.bind_pool
        .reserve(input_facts.len().saturating_mul(4));
    let __seen = phase_start();
    let __seen_alloc = phase_start();
    let mut seen_ids: FxHashSet<u64> =
        FxHashSet::with_capacity_and_hasher(input_facts.len(), Default::default());
    let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
    phase_end("  │  setup:seen:alloc", __seen_alloc);
    phase_end("  ├ setup:seen", __seen);
    let mut owned_delta: Vec<u32> = Vec::new();
    let mut seed_round = true;

    // Item 12 — the arm lives next to the network. Hit: skip lower/classify.
    // Miss: build once, intern under the network's rust identity. insert/clone
    // share that identity (facts overlay).
    let __arm = phase_start();
    let arm = rete_arm_get_or_build(&wm.network, &wm.rules, sym)?;
    phase_end("  ├ setup:arm", __arm);
    let node_ids = arm.node_ids.clone();
    let kind_ids = &arm.kind_ids;
    let compiled_conds = &arm.compiled_conds;
    let compiled_drivers = &arm.compiled_drivers;
    let compiled_wheres = &arm.compiled_wheres;
    let where_tree = &arm.where_tree;
    let compiled_acc_folds = &arm.compiled_acc_folds;
    let compiled_rhs_cache = &arm.compiled_rhs;
    let alpha_tree = &arm.alpha_tree;
    let feeding_alpha_of = &arm.feeding_alpha_of;
    let parents_of = &arm.parents_of;
    let beta_readers = &arm.beta_readers;

    // P6 — persistent join indexes, maintained ACROSS rounds (never rebuilt).
    // Keyed by HashJoinNode id J.
    // left_idx[J]:  key → Vec<Token>   (all left tokens seen so far for J)
    // right_idx[J]: key → Vec<Element> (all right elements seen so far for J)
    // join_keys[J]: the sorted shared-variable list (cached lazily on first use)
    let mut left_idx: HashMap<i64, HashMap<Vec<Value>, Vec<Token>>> = HashMap::new();
    let mut right_idx: HashMap<i64, HashMap<Vec<Value>, Vec<Element>>> = HashMap::new();
    let mut join_keys_cache: ProductionMemory = HashMap::new();
    // P6-for-gathers: persist across rounds, append d_alpha
    // (`DESIGN-STONE-persist-gather-across-rounds`). Not a Session field.
    let mut gather_cache: GatherCache = FxHashMap::default();

    // One scratch buffer, reused for every compiled-condition call this whole fire pass: sized
    // once to the largest `n_slots` any compiled alpha needs, so `exec_compiled`'s `clear` +
    // `resize` back up never reallocates after this point — the failure path it guards allocates
    // nothing (row 2 of the DESIGN-STONE's scorecard).
    let mut match_scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
    let mut cand_scratch: Vec<i64> = Vec::new();
    let mut cond_key_ids: CondKeyIds = HashMap::new();
    for (&id, c) in compiled_conds {
        cond_key_ids.insert(
            id,
            crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
        );
    }

    // A8 instrument: the round counter the census stamps its rows with (test-only).
    #[cfg(test)]
    let mut round_no: usize = 0;

    phase_end("SETUP: indexes", __setup);
    let __rounds = phase_start();
    loop {
        // ROUND LOOP's six named passes cover only ~60% of it on an accumulate workload (root-join
        // and hash-join do nothing there). These two marks bracket the loop's own scaffolding so
        // the remainder has a name instead of being inferred from a parent/child subtraction.
        let __pre = phase_start();
        // Per-round delta sets (new elements/tokens created THIS round).
        // Indices into this round's wm.alpha[aid] (DESIGN-STONE-delta-alpha-indices).
        let mut d_alpha: AlphaDelta = FxHashMap::default();
        let mut d_beta: BetaMemory = HashMap::new();

        phase_end("  ├ round:preamble", __pre);

        // ── 1. Alpha delta (type-indexed): each delta fact probes ONLY its type's alphas. ──
        #[cfg(test)]
        let this_round_in = if seed_round {
            input_facts.len()
        } else {
            owned_delta.len()
        };
        let __pt0 = phase_start();
        if seed_round {
            // Two pairs / fire, not per fact (`DESIGN-STONE-alpha-leftover-split`).
            let __seed = phase_start();
            for (i, fact) in input_facts.iter().enumerate() {
                // Fold `seen_insert` into this walk (`DESIGN-STONE-fold-seen-into-seed`).
                // Every input is in `seen` before production considers derived facts.
                seen_insert(&mut seen_ids, &mut seen_rest, fact);
                alpha_activate_fact(
                    fact,
                    i as u32,
                    &mut AlphaHit {
                        wm: &mut wm,
                        d_alpha: &mut d_alpha,
                        alpha_tree,
                        compiled_conds,
                        match_scratch: &mut match_scratch,
                        cand_scratch: &mut cand_scratch,
                        cond_key_ids: &cond_key_ids,
                    },
                )?;
            }
            phase_end("  ├ alpha:seed", __seed);
            seed_round = false;
        } else {
            let __delta = phase_start();
            for &idx in &owned_delta {
                let fact = fact_at(&wm.facts, &wm.derived_facts, wm.n_input, idx).clone();
                alpha_activate_fact(
                    &fact,
                    idx,
                    &mut AlphaHit {
                        wm: &mut wm,
                        d_alpha: &mut d_alpha,
                        alpha_tree,
                        compiled_conds,
                        match_scratch: &mut match_scratch,
                        cand_scratch: &mut cand_scratch,
                        cond_key_ids: &cond_key_ids,
                    },
                )?;
            }
            phase_end("  └ alpha:delta", __delta);
        }

        phase_end("alpha", __pt0);
        append_d_alpha(&mut gather_cache, &d_alpha, &wm);

        // ── 2. Root-join delta: seed tokens from NEW elements (d_alpha) only. ───
        let __pt1 = phase_start();
        for node_id in &kind_ids.alpha {
            // Group C: use &Value ref — no clone; kind_of/node_children take &Value.
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(node) != "AlphaNode" {
                continue;
            }
            // New this round: indices into wm.alpha[node_id].
            let new_idxs = match d_alpha.get(node_id) {
                Some(ix) if !ix.is_empty() => ix.as_slice(),
                _ => continue,
            };
            let child_ids = node_children(node);
            // node's last use is node_children above; wm.network borrow for `node` ends here (NLL).
            for child_id in &child_ids {
                // Group C: child_node ref — only used for kind_of; borrow ends before wm mutations.
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n,
                    None => continue,
                };
                if kind_of(child_node) != "RootJoinNode" {
                    continue;
                }
                for &ei in new_idxs {
                    let el = &wm.alpha[node_id][ei];
                    // Seed native Token: one matches edge (fact idx, alpha_id).
                    let tok = Token {
                        matches: push_match(&mut wm.match_pool, el.fact, *node_id),
                        binds: seed_token_binds(el),
                                    };
                    if beta_readers.contains(child_id) {
                        beta_written(*child_id, 1);
                        wm.beta.entry(*child_id).or_default().push(tok);
                    }
                    d_beta.entry(*child_id).or_default().push(tok);
                }
            }
        }

        phase_end("root-join", __pt1);

        // ── 3. Hash-join delta (ascending id — topological). ─────────────────────
        let __pt2 = phase_start();
        // P6 persistent-index algorithm (DESIGN-STONE-P6, 6-step ordering):
        //
        // For each parent P (Root/HashJoin) with HashJoinNode child J (feeding alpha A):
        //   dl = d_beta[P]  (Δleft:  tokens new this round at P)
        //   dr = d_alpha[A] (Δright: elements new this round at A)
        //
        //   Step 2: add dr → right_idx[J]   (right_idx now holds ALL right incl. this round's)
        //   Step 3: term1 = Δleft ⋈ all_right   (probe right_idx[J] with dl)
        //   Step 4: term2 = old_left ⋈ Δright   (probe left_idx[J] — still OLD — with dr)
        //   Step 5: add dl → left_idx[J]    (AFTER term2: left_idx now holds ALL left incl. this round's)
        //   Step 6: new tokens → wm.beta[J] + d_beta[J]
        //
        // Invariant: (Δleft×Δright) appears in term1 only (right_idx already has Δright at step 3);
        //            old_left×Δright appears in term2 only (left_idx lacks Δleft at step 4).
        //            No double-count, no miss — same semi-naive result as the keyed_join rebuild.
        // Dirty join-parents only (`DESIGN-STONE-dirty-join-parents`): left d_beta
        // or a HashJoin child whose feeding alpha has d_alpha. First-keying runs
        // the round the second side arrives (that delta is non-empty). Grow the
        // set as we emit so a middle join (J1→J2) is visited this round.
        let mut dirty_parents = seed_dirty_join_parents(
            &kind_ids.join_parent,
            &d_beta,
            &d_alpha,
            &arm.joins_fed_by,
            parents_of,
        );
        for node_id in &kind_ids.join_parent {
            if !dirty_parents.contains(node_id) {
                continue;
            }
            // Group C: use &Value ref (wm.network borrow) — no clone; kind_of/node_children take &Value.
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            let kind = kind_of(node);
            if kind != "RootJoinNode" && kind != "HashJoinNode" {
                continue;
            }

            let child_ids = node_children(node);
            // node's last use is node_children above; wm.network borrow for `node` ends here (NLL).
            for child_id in &child_ids {
                // Group C: child_node ref — only used for kind_of; borrow ends before wm mutations.
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n,
                    None => continue,
                };
                if kind_of(child_node) != "HashJoinNode" {
                    continue;
                }
                let alpha_id = feeding_alpha_of.get(child_id).copied().unwrap_or(-1);

                // Step 1: Ensure join_keys[J] is cached.
                // Compute from a sample token at P and a sample element at A (if both exist).
                // first_keying=true means J was previously skipped while one side was empty;
                // a one-time catch-up full-join is required to populate right_idx[J] from ALL
                // cumulative wm.alpha[alpha_id] (not just the current round's dr).
                let first_keying = if !join_keys_cache.contains_key(child_id) {
                    let sample_tok = wm.beta.get(node_id).and_then(|v| v.first());
                    // READ #1 of 2: one sample token, to derive this join's keys.
                    if sample_tok.is_some() {
                        beta_read(*node_id, 1);
                    }
                    let sample_el = wm.alpha.get(&alpha_id).and_then(|v| v.first());
                    match (sample_tok, sample_el) {
                        (Some(tok), Some(el)) => {
                            let el_b = element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                            let mut keys: Vec<Value> = bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds)
                                .iter()
                                .map(|(k, _)| k.clone())
                                .filter(|k| Bindings::get(&el_b, k).is_some())
                                .collect();
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
                            join_keys_cache.insert(*child_id, keys);
                            true // first keying: catch-up full-join needed
                        }
                        _ => {
                            // Neither side has data yet — skip this node for this round.
                            // The join_keys will be computed next round when both sides are populated.
                            continue;
                        }
                    }
                } else {
                    false
                };

                // Group C: borrow join_keys (pointer bump) instead of cloning (Vec alloc + copy).
                let jk: &[Value] = &join_keys_cache[child_id];

                // CATCH-UP (first keying only): J was skipped every prior round while one side
                // was empty, so right_idx[J] was never populated from those rounds' facts.
                // Rebuild from ALL cumulative wm.alpha[alpha_id] and wm.beta[parent], cross-join
                // fully, and build both indexes. Safe: J produced ZERO tokens before first keying
                // so there is nothing to double-count. On subsequent rounds the incremental
                // semi-naive path (steps 2–5 below) handles new arrivals correctly.
                //
                // Note: at this point in the round, steps 1 (alpha delta) and 2 (root-join delta)
                // have ALREADY run, so wm.alpha and wm.beta contain ALL cumulative data including
                // this round's new elements — the catch-up covers historical AND current-round facts.
                if first_keying {
                    // Clone to avoid split-borrow conflicts with later wm.beta/d_beta mutations.
                    let all_right: Vec<Element> =
                        wm.alpha.get(&alpha_id).cloned().unwrap_or_default();
                    let all_left: Vec<Token> = wm.beta.get(node_id).cloned().unwrap_or_default();
                    // READ #2 of 2: the parent's cumulative tokens, for the catch-up cross-join.
                    beta_read(*node_id, all_left.len() as u64);
                    // Build right_idx[J] from ALL cumulative right elements.
                    let __cri = phase_start();
                    {
                        let ridx = right_idx.entry(*child_id).or_default();
                        for el in &all_right {
                            let el_b = element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                            let k = key_of(&el_b, jk);
                            ridx.entry(k).or_default().push(*el);
                        }
                    }
                    phase_end("  ├ hj:catchup:right-idx", __cri);
                    // Reserve the 40k appends. Isolated unreserved extend paid
                    // G−E = 4.13 ms (`DESIGN-STONE-probe-gap-split`).
                    let n_join = match right_idx.get(child_id) {
                        Some(idx) if !idx.is_empty() && !all_right.is_empty() => all_left
                            .len()
                            .saturating_mul(all_right.len() / idx.len()),
                        _ => 0,
                    };
                    wm.bind_pool.reserve(n_join.saturating_mul(4));
                    wm.match_pool.reserve(n_join.saturating_mul(2));
                    // Full cross-join: every left token keyed against right_idx[J].
                    let __cpr = phase_start();
                    let mut new_tokens: Vec<Token> = Vec::with_capacity(n_join);
                    if let Some(ridx) = right_idx.get(child_id) {
                        for tok in &all_left {
                            let k = key_of(&bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds), jk);
                            if let Some(bucket) = ridx.get(&k) {
                                for el in bucket {
                                    if let Some(new_tok) = join_extend(
                                        tok,
                                        el,
                                        alpha_id,
                                        &mut FireCtx {
                                            compiled_conds,
                                            scratch: &mut match_scratch,
                                            pool: &mut wm.bind_pool,
                                            match_pool: &mut wm.match_pool,
                                            keys: &wm.bind_keys,
                                            vals: &wm.bind_vals,
                                            facts: &wm.facts,
                                            derived: &wm.derived_facts,
                                            n_input: wm.n_input,
                                        },
                                    )? {
                                        new_tokens.push(new_tok);
                                    }
                                }
                            }
                        }
                    }
                    phase_end("  ├ hj:catchup:probe", __cpr);
                    // Build left_idx[J] from ALL cumulative left tokens.
                    let __cli = phase_start();
                    {
                        let lidx = left_idx.entry(*child_id).or_default();
                        for tok in all_left {
                            let k = key_of(&bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds), jk);
                            lidx.entry(k).or_default().push(tok);
                        }
                    }
                    phase_end("  ├ hj:catchup:left-idx", __cli);
                    // Emit catch-up tokens into cumulative and delta memories.
                    let __cem = phase_start();
                    // `entry()` HOISTED out of the per-token loop: the key is constant, so the
                    // old form paid two map lookups per token (80,000 on the fanout cell) where
                    // two total will do. Correct regardless of the guard below.
                    if beta_readers.contains(child_id) {
                        beta_written(*child_id, new_tokens.len() as u64);
                        let beta = wm.beta.entry(*child_id).or_default();
                        beta.reserve(new_tokens.len());
                        for t in &new_tokens {
                            beta.push(*t);
                        }
                    }
                    let n_emit = new_tokens.len();
                    let delta = d_beta.entry(*child_id).or_default();
                    delta.reserve(n_emit);
                    for new_tok in new_tokens {
                        delta.push(new_tok);
                    }
                    if n_emit > 0 {
                        dirty_parents.insert(*child_id);
                    }
                    phase_end("  ├ hj:catchup:emit", __cem);
                    continue; // Skip incremental steps 2–5 for this round.
                }

                // Group C: borrow dl/dr slices — no Vec alloc per node per round.
                // NLL ends these borrows at their last use (step 5), before step 6 mutates d_beta.
                let dl: &[Token] = d_beta.get(node_id).map(Vec::as_slice).unwrap_or_default();
                let dr: &[usize] = d_alpha
                    .get(&alpha_id)
                    .map(Vec::as_slice)
                    .unwrap_or_default();

                // Skip if nothing new on either side.
                if dl.is_empty() && dr.is_empty() {
                    continue;
                }

                // Step 2: add Δright (dr) to right_idx[J] FIRST.
                // dr is indices into wm.alpha[A]; right_idx still owns Elements (P6).
                let __s2 = phase_start();
                {
                    let ridx = right_idx.entry(*child_id).or_default();
                    let right_mem = wm.alpha.get(&alpha_id).map(Vec::as_slice).unwrap_or(&[]);
                    for &ei in dr {
                        let el = &right_mem[ei];
                        let el_b = element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                        let k = key_of(&el_b, jk);
                        ridx.entry(k).or_default().push(*el);
                    }
                }
                phase_end("  ├ hj:step2-right-idx", __s2);

                // Step 3: term1 = Δleft ⋈ all_right (probe right_idx[J] — now includes Δright).
                // The mutable borrow from step 2 ended with that scope block; safe to borrow immutably.
                let __s3 = phase_start();
                let mut new_tokens: Vec<Token> = Vec::new();
                if !dl.is_empty() {
                    if let Some(ridx) = right_idx.get(child_id) {
                        for tok in dl {
                            let k = key_of(&bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds), jk);
                            if let Some(bucket) = ridx.get(&k) {
                                for el in bucket {
                                    if let Some(new_tok) = join_extend(
                                        tok,
                                        el,
                                        alpha_id,
                                        &mut FireCtx {
                                            compiled_conds,
                                            scratch: &mut match_scratch,
                                            pool: &mut wm.bind_pool,
                                            match_pool: &mut wm.match_pool,
                                            keys: &wm.bind_keys,
                                            vals: &wm.bind_vals,
                                            facts: &wm.facts,
                                            derived: &wm.derived_facts,
                                            n_input: wm.n_input,
                                        },
                                    )? {
                                        new_tokens.push(new_tok);
                                    }
                                }
                            }
                        }
                    }
                }
                phase_end("  ├ hj:step3-term1", __s3);

                // Step 4: term2 = old_left ⋈ Δright (probe left_idx[J] — still OLD, Δleft not yet added).
                // left_idx is a separate map from right_idx; no aliasing — safe immutable borrow.
                let __s4 = phase_start();
                if !dr.is_empty() {
                    if let Some(lidx) = left_idx.get(child_id) {
                        let right_mem =
                            wm.alpha.get(&alpha_id).map(Vec::as_slice).unwrap_or(&[]);
                        for &ei in dr {
                            let el = &right_mem[ei];
                            let el_b = element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                            let k = key_of(&el_b, jk);
                            if let Some(bucket) = lidx.get(&k) {
                                for tok in bucket {
                                    if let Some(new_tok) = join_extend(
                                        tok,
                                        el,
                                        alpha_id,
                                        &mut FireCtx {
                                            compiled_conds,
                                            scratch: &mut match_scratch,
                                            pool: &mut wm.bind_pool,
                                            match_pool: &mut wm.match_pool,
                                            keys: &wm.bind_keys,
                                            vals: &wm.bind_vals,
                                            facts: &wm.facts,
                                            derived: &wm.derived_facts,
                                            n_input: wm.n_input,
                                        },
                                    )? {
                                        new_tokens.push(new_tok);
                                    }
                                }
                            }
                        }
                    }
                }
                phase_end("  ├ hj:step4-term2", __s4);

                // Step 5: add Δleft (dl) to left_idx[J] AFTER term2 (no-double-count invariant).
                // dl is &[Token] — iterate directly.
                let __s5 = phase_start();
                {
                    let lidx = left_idx.entry(*child_id).or_default();
                    for tok in dl {
                        let k = key_of(&bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds), jk);
                        lidx.entry(k).or_default().push(*tok);
                    }
                }
                phase_end("  ├ hj:step5-left-idx", __s5);

                // Step 6: push new tokens to wm.beta[J] and d_beta[J].
                let __s6 = phase_start();
                // Same hoist + guard as the catch-up emit above.
                if beta_readers.contains(child_id) {
                    beta_written(*child_id, new_tokens.len() as u64);
                    let beta = wm.beta.entry(*child_id).or_default();
                    beta.reserve(new_tokens.len());
                    for t in &new_tokens {
                        beta.push(*t);
                    }
                }
                let n_emit = new_tokens.len();
                let delta = d_beta.entry(*child_id).or_default();
                delta.reserve(n_emit);
                for new_tok in new_tokens {
                    delta.push(new_tok);
                }
                if n_emit > 0 {
                    dirty_parents.insert(*child_id);
                }
                phase_end("  ├ hj:step6-emit", __s6);
            }
        }

        phase_end("hash-join", __pt2);

        // ── 3.25 Accumulate-pass (8-b): dispatch AccumulateNode. ────────────────
        let __pt3 = phase_start();
        // For each AccumulateNode (topological = ascending id order): for each NEW token
        // at the parent (d_beta[parent]), gather the token-compatible elements from the
        // FULL cumulative wm.alpha[from_alpha_id] (the aggregate needs all matching facts,
        // like 7-b negation), compute the aggregate in Rust (mirroring the wat acc::* folds),
        // and — if a value — extend the token with result-var → aggregate and push to
        // wm.beta[acc] (cumulative) + d_beta[acc] (new-this-round, consumed downstream).
        // min/max/mean on an empty gather → no value → drop the token.
        // Runs BEFORE the filter-pass so a :where on the result-var sees the binding.
        for node_id in &kind_ids.acc {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(node) != "AccumulateNode" {
                continue;
            }
            // AccumulateNode struct_form: id(0), result-var(1), acc-form(2), from-alpha-id(3), children(4).
            let (_, sf) = node_record(node).expect("accumulate-pass: node must be a Record");
            let result_var = match &sf[1] {
                Value::String(s) => Value::String(s.clone()),
                _ => continue, // malformed: skip
            };
            let Some(acc_fold) = compiled_acc_folds.get(node_id) else {
                return Err(RuntimeError::new(
                    crate::rust_caller_span!(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::rete::fire-rules".into(),
                        reason: format!(
                            "AccumulateNode {node_id} has no compiled fold — setup should have compiled it"
                        ),
                    },
                )
                .into());
            };
            let from_alpha_id: i64 = match &sf[3] {
                Value::i64(n) => *n,
                _ => continue, // malformed: skip
            };
            // NEW tokens at EVERY parent (clone to avoid the d_beta read/write borrow conflict).
            // Leading accumulate (Clara test-count): no parent — seed one empty token.
            // count/sum emit 0 on empty gather; min/max/mean drop the token.
            let pids = parents_of.get(node_id).map(|v| v.as_slice()).unwrap_or(&[]);
            let mut new_tokens: Vec<Token> = d_beta_from_parents(parents_of, &d_beta, *node_id);
            if new_tokens.is_empty() && pids.is_empty() {
                new_tokens = vec![Token {
                    matches: empty_span(),
                    binds: empty_span(),
                            }];
            }
            if new_tokens.is_empty() {
                continue;
            }
            // Derive the join-key tuple first (cheap: elements[0] + a sample-bindings
            // intersection) so the cache can be probed BEFORE paying for a snapshot clone or an
            // index build. Reads wm.alpha through a borrow, no clone yet.
            let __ix = phase_start();
            let join_keys = ensure_gather(
                &mut gather_cache,
                &wm,
                from_alpha_id,
                &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, new_tokens[0].binds),
            );
            phase_end("  ├ accum:index", __ix);
            // No clone — indices name this round's wm.alpha[id] (step 1 is done).
            let __sn = phase_start();
            let from_elements = alpha_elements(&wm.alpha, from_alpha_id);
            phase_end("  ├ accum:snapshot", __sn);
            let index = gather_cache
                .get(&(from_alpha_id, join_keys.clone()))
                .expect("ensure_gather just inserted");
            let from_compiled = rematch_compiled(compiled_conds, from_alpha_id).ok();
            let leftover = from_compiled
                .map(crate::rete::compiled_cond::CompiledCond::has_seed_cmp)
                .unwrap_or(false);
            let from_keys = from_compiled
                .map(|c| c.bind_keys())
                .unwrap_or_default();
            let operand_keys = acc_fold.operand_keys();
            let __fd = phase_start();
            for tok in new_tokens {
                let key = key_of(&bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds), &join_keys);
                let bucket: &[usize] = index.bucket(&key, &wm.bind_val_ids);
                let group_keys: Vec<Value> = from_keys
                    .iter()
                    .filter(|k| {
                        Bindings::get(&bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds), k).is_none()
                            && !operand_keys.iter().any(|o| o == *k)
                    })
                    .cloned()
                    .collect();
                // No leftover SeedCmp: the keyed bucket IS the gather (keyed-gather
                // contract). Rematch cannot reject a member or bind anything the
                // Element does not already hold. Count is len; value folds read a slot.
                if !leftover && group_keys.is_empty() {
                    if let Some(aggregate) =
                        fold_bucket(
                            acc_fold,
                            from_elements,
                            bucket,
                            sym,
                            &acc_view(&wm),
                        )?
                    {
                        let new_tok = token_assoc(
                            &tok,
                            result_var.clone(),
                            aggregate,
                            &mut wm.bind_keys,
                            &mut wm.bind_vals,
                            &mut wm.bind_val_ids,
                            &mut wm.bind_pool,
                        );
                        if beta_readers.contains(node_id) {
                            beta_written(*node_id, 1);
                            wm.beta.entry(*node_id).or_default().push(new_tok);
                        }
                        d_beta.entry(*node_id).or_default().push(new_tok);
                    }
                    continue;
                }
                // Gather the token-compatible :from elements (shared ?var agreement), in
                // alpha-memory insertion order (matches the wat foldl over from-els) — the
                // bucket's indices were pushed in that same order.
                let mut gathered: Vec<&Element> = Vec::new();
                if leftover {
                    for &i in bucket {
                        let el = &from_elements[i];
                        census_gather_visit();
                        let ok = match from_compiled {
                            Some(compiled) => fact_bindings_under(
                                fact_at(
                                    &wm.facts,
                                    &wm.derived_facts,
                                    wm.n_input,
                                    el.fact,
                                ),
                                &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                                compiled,
                                &mut match_scratch,
                            )
                            .is_some(),
                            None => {
                                let el_b = element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                                token_element_compatible(
                                    &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                                    &el_b,
                                )
                            }
                        };
                        if ok {
                            gathered.push(el);
                        }
                    }
                } else {
                    gathered.extend(bucket.iter().map(|&i| &from_elements[i]));
                }
                // One fold of the whole gather when the token already holds every
                // `:from` bind (or the `:from` binds none). Otherwise group by the
                // leftover binds; empty gather + leftover keys is not a bag-wide 0.
                let groups: Vec<(crate::value::pmap::PMap, Vec<&Element>)> = if group_keys
                    .is_empty()
                {
                    vec![(pmap_from_span(tok.binds, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool), gathered)]
                } else if gathered.is_empty() {
                    Vec::new()
                } else {
                    let mut order: Vec<Vec<(Value, Value)>> = Vec::new();
                    let mut buckets: HashMap<Vec<(Value, Value)>, Vec<&Element>> = HashMap::new();
                    for el in gathered {
                        let el_b = element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                        let proj = project_group_keys(&el_b, &group_keys);
                        buckets
                            .entry(proj.clone())
                            .or_insert_with(|| {
                                order.push(proj);
                                Vec::new()
                            })
                            .push(el);
                    }
                    order
                        .into_iter()
                        .map(|proj| {
                            let mut nb = pmap_from_span(tok.binds, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                            for (k, v) in &proj {
                                nb = nb.assoc(k.clone(), v.clone());
                            }
                            let els = buckets.remove(&proj).unwrap_or_default();
                            (nb, els)
                        })
                        .collect()
                };
                for (group_bindings, group_els) in groups {
                    if let Some(aggregate) =
                        accumulate_value(
                            acc_fold,
                            &group_els,
                            sym,
                            &acc_view(&wm),
                        )?
                    {
                        let new_bindings = group_bindings.assoc(result_var.clone(), aggregate);
                        let new_tok = Token {
                            matches: tok.matches,
                            binds: span_from_pairs(
                                &mut wm.bind_keys,
                                &mut wm.bind_vals,
                                &mut wm.bind_val_ids,
                                &mut wm.bind_pool,
                                new_bindings.iter().map(|(k, v)| (k.clone(), v.clone())),
                            ),
                                            };
                        if beta_readers.contains(node_id) {
                            beta_written(*node_id, 1);
                            wm.beta.entry(*node_id).or_default().push(new_tok);
                        }
                        d_beta.entry(*node_id).or_default().push(new_tok);
                    }
                }
            }
            phase_end("  └ accum:fold", __fd);
        }

        phase_end("accumulate", __pt3);

        // ── 3.5 Filter-pass (7-a unified): dispatch TestNode + NegationNode. ─────
        let __pt4 = phase_start();
        // For each TestNode or NegationNode (in topological = ascending id order):
        //   TestNode     → eval-test filter: pass the token iff expr evaluates true.
        //   NegationNode → negation filter: pass the un-extended token iff ZERO elements in
        //                  wm.alpha[neg_alpha_id] (the FULL cumulative alpha-memory) are
        //                  token-element-compatible with the token's bindings.
        // New tokens still come from d_beta[parent] (the delta); only the absence check
        // for NegationNode reads the full wm.alpha (populated in step 1 before this pass).
        // Passing tokens are pushed to wm.beta[node_id] (cumulative) and d_beta[node_id]
        // (new-this-round, consumed by production in step 4).
        let mut tests_done: HashSet<i64> = HashSet::new();
        for node_id in &kind_ids.filter {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            let kind = kind_of(node);
            if kind != "TestNode" && kind != "NegationNode" && kind != "ExistsNode" {
                continue;
            }
            let (_, sf) = node_record(node).expect("filter-pass: node must be a Record");
            // Clone the new-this-round tokens at EVERY parent to avoid a simultaneous
            // borrow conflict (reading d_beta[parent] while writing d_beta[*node_id]).
            // A Test/:not/:exists after condition `:or` has N parents.
            let pids = parents_of.get(node_id).map(|v| v.as_slice()).unwrap_or(&[]);
            let mut new_tokens: Vec<Token> = d_beta_from_parents(parents_of, &d_beta, *node_id);
            // Leading :not has no parent — Clara matches the empty world with one
            // empty-binding token. Do not seed when parents exist but produced nothing.
            if pids.is_empty() && kind == "NegationNode" {
                new_tokens = vec![Token {
                    matches: empty_span(),
                    binds: empty_span(),
                            }];
            }
            // Leading :exists: one token per DISTINCT inner binding (Clara
            // test-simple-exists — two Winds at MCI → one {?loc MCI}), not an
            // empty seed. Mid-chain exists still filters parent tokens below.
            if pids.is_empty() && kind == "ExistsNode" {
                let alpha_id: i64 = match &sf[1] {
                    Value::i64(n) => *n,
                    _ => continue,
                };
                let driver = driver_of(compiled_drivers, alpha_id)?;
                let mut seen = std::collections::HashSet::new();
                let exts: Vec<crate::value::pmap::PMap> = if matches!(driver, CondDriver::Leaf(_)) {
                    wm.alpha
                        .get(&alpha_id)
                        .into_iter()
                        .flatten()
                        .map(|el| {
                            let eb = element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                            crate::value::pmap::PMap::from_pairs(
                                eb.iter().map(|(k, v)| (k.clone(), v.clone())),
                            )
                        })
                        .collect()
                } else {
                    let empty = crate::value::pmap::PMap::new();
                    binding_extensions(
                        driver,
                        &wm,
                        &empty,
                        compiled_conds,
                        &mut match_scratch,
                        sym,
                        &mut gather_cache,
                    )?
                };
                for ext in exts {
                    if !seen.insert(ext.clone()) {
                        continue;
                    }
                    let tok = Token {
                        matches: empty_span(),
                        binds: span_from_pairs(
                            &mut wm.bind_keys,
                            &mut wm.bind_vals,
                            &mut wm.bind_val_ids,
                            &mut wm.bind_pool,
                            ext.iter().map(|(k, v)| (k.clone(), v.clone())),
                        ),
                                    };
                    if beta_readers.contains(node_id) {
                        beta_written(*node_id, 1);
                        wm.beta.entry(*node_id).or_default().push(tok);
                    }
                    d_beta.entry(*node_id).or_default().push(tok);
                }
                continue;
            }
            if new_tokens.is_empty() {
                continue;
            }
            if kind == "TestNode" {
                if tests_done.contains(node_id) {
                    continue;
                }
                // DESIGN-STONE-compiled-where Step 0 — capture the FIRST (expr, tokens) this loop
                // handles. Census only; production never reads `:expr`.
                #[cfg(test)]
                if let Value::wat__WatAST(ast) = &sf[1] {
                    capture_where_sample(ast.as_ref(), &new_tokens, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                }
                // Siblings that share this TestNode's parent set see the same token
                // stream — dispatch once through the where-tree (b).
                let pkey = sorted_parents_of(parents_of, *node_id);
                let mut sibs: Vec<i64> = Vec::new();
                for oid in &node_ids {
                    if tests_done.contains(oid) {
                        continue;
                    }
                    let Some(on) = get_node(&wm.network, *oid) else {
                        continue;
                    };
                    if kind_of(on) != "TestNode" {
                        continue;
                    }
                    if sorted_parents_of(parents_of, *oid) == pkey {
                        sibs.push(*oid);
                    }
                }
                dispatch_where_tests(
                    &sibs,
                    &new_tokens,
                    &mut WhereSink {
                        where_tree,
                        compiled_wheres,
                        beta_readers,
                        wm: &mut wm,
                        d_beta: &mut d_beta,
                        sym,
                    },
                )?;
                tests_done.extend(sibs);
            } else {
                // NegationNode / ExistsNode struct_form: id(0), <kind>-alpha-id(1), children(2).
                // Same gather as Acc: probe gather_cache for the token's join-key bucket.
                // Verdict inverts by kind: NegationNode passes iff ZERO compatible, ExistsNode
                // iff ≥1. The index is over FULL cumulative wm.alpha (step 1 ran first).
                // ExistsNode binds nothing and passes the token at most ONCE (no multiplicity).
                let is_exists = kind == "ExistsNode";
                let alpha_id: i64 = match &sf[1] {
                    Value::i64(n) => *n,
                    _ => continue, // malformed Negation/Exists node: skip
                };
                let driver = driver_of(compiled_drivers, alpha_id)?;
                for tok in new_tokens {
                    let any_compat = token_exists_under(
                        driver,
                        &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                        &wm,
                        compiled_conds,
                        &mut match_scratch,
                        sym,
                        &mut gather_cache,
                    )?;
                    // ExistsNode passes iff any-compat; NegationNode passes iff NOT any-compat.
                    let pass = if is_exists { any_compat } else { !any_compat };
                    if pass {
                        if beta_readers.contains(node_id) {
                            beta_written(*node_id, 1);
                            wm.beta.entry(*node_id).or_default().push(tok);
                        }
                        d_beta.entry(*node_id).or_default().push(tok);
                    }
                }
            }
        }

        phase_end("filter", __pt4);

        // ── 3.6 Join-after-filter (A1): HashJoin children of Test/Neg/Exists/Accum. ─
        // The P6 loop above only left-activates from Root/HashJoin. Compile will parent
        // a HashJoin on a mid-chain :where (Clara does; Join → Test → Join). Filter just
        // filled d_beta[test]; push those tokens across the next join. keyed_join against
        // the full alpha is the catch-up: this child produced nothing in step 3, so there
        // is nothing to double-count.
        let __pt36 = phase_start();
        let mut after_join_frontier: Vec<i64> = Vec::new();
        for node_id in &kind_ids.filter_or_acc {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            let kind = kind_of(node);
            if kind != "TestNode"
                && kind != "NegationNode"
                && kind != "ExistsNode"
                && kind != "AccumulateNode"
            {
                continue;
            }
            let new_tokens: Vec<Token> = match d_beta.get(node_id) {
                Some(ts) if !ts.is_empty() => ts.clone(),
                _ => continue,
            };
            let child_ids = node_children(node);
            for child_id in &child_ids {
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n,
                    None => continue,
                };
                if kind_of(child_node) != "HashJoinNode" {
                    continue;
                }
                let alpha_id = feeding_alpha_of.get(child_id).copied().unwrap_or(-1);
                let elements = match wm.alpha.get(&alpha_id) {
                    Some(els) if !els.is_empty() => els.as_slice(),
                    _ => continue,
                };
                let joined = keyed_join(
                    &new_tokens,
                    elements,
                    alpha_id,
                    &mut FireCtx {
                        compiled_conds,
                        scratch: &mut match_scratch,
                        pool: &mut wm.bind_pool,
                        match_pool: &mut wm.match_pool,
                        keys: &wm.bind_keys,
                        vals: &wm.bind_vals,
                        facts: &wm.facts,
                        derived: &wm.derived_facts,
                        n_input: wm.n_input,
                    },
                )?;
                if joined.is_empty() {
                    continue;
                }
                if beta_readers.contains(child_id) {
                    beta_written(*child_id, joined.len() as u64);
                    wm.beta
                        .entry(*child_id)
                        .or_default()
                        .extend(joined.iter().cloned());
                }
                d_beta.entry(*child_id).or_default().extend(joined);
                after_join_frontier.push(*child_id);
            }
        }
        phase_end("join-after-filter", __pt36);

        // ── 3.7 Filter-after-join: Test/Neg/Exists whose parent just got tokens
        // in 3.6 (trailing `:where` after a mid-chain `:where` + join). A1 only
        // left-activated HashJoin children of a Test. The trailing Test is a
        // *child* of that HashJoin; the first filter pass already finished
        // before 3.6 wrote d_beta[join]. Spec's topo emit covers it; native
        // must too. Loop: a Test may itself parent another HashJoin.
        let __pt37 = phase_start();
        let mut frontier = after_join_frontier;
        while !frontier.is_empty() {
            let mut next_frontier: Vec<i64> = Vec::new();
            for hj_id in frontier {
                let hj_node = match get_node(&wm.network, hj_id) {
                    Some(n) => n,
                    None => continue,
                };
                let filter_kids = node_children(hj_node);
                let mut tests_dispatched = false;
                for filter_id in filter_kids.iter().copied() {
                    let filter_node = match get_node(&wm.network, filter_id) {
                        Some(n) => n,
                        None => continue,
                    };
                    let fkind = kind_of(filter_node);
                    if fkind != "TestNode" && fkind != "NegationNode" && fkind != "ExistsNode" {
                        continue;
                    }
                    let new_tokens: Vec<Token> = match d_beta.get(&hj_id) {
                        Some(ts) if !ts.is_empty() => ts.clone(),
                        _ => continue,
                    };
                    if fkind == "TestNode" {
                        if !tests_dispatched {
                            let test_sibs: Vec<i64> = filter_kids
                                .iter()
                                .copied()
                                .filter(|id| {
                                    get_node(&wm.network, *id)
                                        .map(|n| kind_of(n) == "TestNode")
                                        .unwrap_or(false)
                                })
                                .collect();
                            dispatch_where_tests(
                                &test_sibs,
                                &new_tokens,
                                &mut WhereSink {
                                    where_tree,
                                    compiled_wheres,
                                    beta_readers,
                                    wm: &mut wm,
                                    d_beta: &mut d_beta,
                                    sym,
                                },
                            )?;
                            tests_dispatched = true;
                        }
                    } else {
                        let (_, sf) = match node_record(filter_node) {
                            Some(p) => p,
                            None => continue,
                        };
                        let is_exists = fkind == "ExistsNode";
                        let alpha_id: i64 = match &sf[1] {
                            Value::i64(n) => *n,
                            _ => continue,
                        };
                        if new_tokens.is_empty() {
                            continue;
                        }
                        let driver = driver_of(compiled_drivers, alpha_id)?;
                        for tok in new_tokens {
                            let any_compat = token_exists_under(
                                driver,
                                &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                                &wm,
                                compiled_conds,
                                &mut match_scratch,
                                sym,
                                &mut gather_cache,
                            )?;
                            let pass = if is_exists { any_compat } else { !any_compat };
                            if pass {
                                if beta_readers.contains(&filter_id) {
                                    beta_written(filter_id, 1);
                                    wm.beta.entry(filter_id).or_default().push(tok);
                                }
                                d_beta.entry(filter_id).or_default().push(tok);
                            }
                        }
                    }
                    // Walk children of this filter: HashJoin (3.6's grandchild) AND
                    // Test/Neg/Exists (Test→Test after join-after-filter — spoken
                    // two-temps: filter, join, filter, filter).
                    let mut chain: Vec<i64> = vec![filter_id];
                    while let Some(fid) = chain.pop() {
                        let fnode = match get_node(&wm.network, fid) {
                            Some(n) => n,
                            None => continue,
                        };
                        let parent_toks: Vec<Token> = match d_beta.get(&fid) {
                            Some(ts) if !ts.is_empty() => ts.clone(),
                            _ => continue,
                        };
                        let kids = node_children(fnode);
                        let test_sibs: Vec<i64> = kids
                            .iter()
                            .copied()
                            .filter(|id| {
                                get_node(&wm.network, *id)
                                    .map(|n| kind_of(n) == "TestNode")
                                    .unwrap_or(false)
                            })
                            .collect();
                        if !test_sibs.is_empty() {
                            dispatch_where_tests(
                                &test_sibs,
                                &parent_toks,
                                &mut WhereSink {
                                    where_tree,
                                    compiled_wheres,
                                    beta_readers,
                                    wm: &mut wm,
                                    d_beta: &mut d_beta,
                                    sym,
                                },
                            )?;
                            chain.extend(test_sibs);
                        }
                        for gc_id in kids {
                            let gc = match get_node(&wm.network, gc_id) {
                                Some(n) => n,
                                None => continue,
                            };
                            let gkind = kind_of(gc);
                            if gkind == "TestNode" {
                                continue;
                            }
                            if gkind == "HashJoinNode" {
                                let alpha_id = feeding_alpha_of.get(&gc_id).copied().unwrap_or(-1);
                                let elements = match wm.alpha.get(&alpha_id) {
                                    Some(els) if !els.is_empty() => els.as_slice(),
                                    _ => continue,
                                };
                                let joined = keyed_join(
                                    &parent_toks,
                                    elements,
                                    alpha_id,
                                    &mut FireCtx {
                                        compiled_conds,
                                        scratch: &mut match_scratch,
                                        pool: &mut wm.bind_pool,
                                        match_pool: &mut wm.match_pool,
                                        keys: &wm.bind_keys,
                                        vals: &wm.bind_vals,
                                        facts: &wm.facts,
                                        derived: &wm.derived_facts,
                                        n_input: wm.n_input,
                                    },
                                )?;
                                if joined.is_empty() {
                                    continue;
                                }
                                if beta_readers.contains(&gc_id) {
                                    beta_written(gc_id, joined.len() as u64);
                                    wm.beta
                                        .entry(gc_id)
                                        .or_default()
                                        .extend(joined.iter().cloned());
                                }
                                d_beta.entry(gc_id).or_default().extend(joined);
                                next_frontier.push(gc_id);
                                continue;
                            }
                            if gkind != "NegationNode" && gkind != "ExistsNode" {
                                continue;
                            }
                            let (_, gsf) = match node_record(gc) {
                                Some(p) => p,
                                None => continue,
                            };
                            let is_exists = gkind == "ExistsNode";
                            let alpha_id: i64 = match &gsf[1] {
                                Value::i64(n) => *n,
                                _ => continue,
                            };
                            let driver = driver_of(compiled_drivers, alpha_id)?;
                            for tok in &parent_toks {
                                let any_compat = token_exists_under(
                                    driver,
                                    &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                                    &wm,
                                    compiled_conds,
                                    &mut match_scratch,
                                    sym,
                                    &mut gather_cache,
                                )?;
                                let pass = if is_exists { any_compat } else { !any_compat };
                                if pass {
                                    if beta_readers.contains(&gc_id) {
                                        beta_written(gc_id, 1);
                                        wm.beta.entry(gc_id).or_default().push(*tok);
                                    }
                                    d_beta.entry(gc_id).or_default().push(*tok);
                                }
                            }
                            chain.push(gc_id);
                        }
                    }
                }
            }
            frontier = next_frontier;
        }
        phase_end("filter-after-join", __pt37);

        // ── 4. Production delta: fire production nodes on NEW tokens only. ────────
        let __pt5 = phase_start();
        let mut next_delta: Vec<u32> = Vec::new();
        for node_id in &kind_ids.prod {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(node) != "ProductionNode" {
                continue;
            }
            let (_, sf) = node_record(node).unwrap();
            let rule_name = match &sf[1] {
                Value::String(s) => s.as_str(),
                _ => continue,
            };
            // Production gate: rule name must be in this arm's compiled :then
            // (stratified slices pass a rules subset — a ProductionNode whose
            // rule is absent is inert).
            let compiled_rhs_forms = match compiled_rhs_cache.get(rule_name) {
                Some(forms) => forms,
                None => continue,
            };

            // Fire on NEW tokens at EVERY parent (condition `:or` has N).
            // Walk d_beta in place — production only reads bindings
            // (`DESIGN-STONE-prod-no-token-clone`).
            let Some(pids) = parents_of.get(node_id) else {
                continue;
            };

            for pid in pids {
                let Some(ts) = d_beta.get(pid) else {
                    continue;
                };
                if ts.is_empty() {
                    continue;
                }
                // `seen` grows by one entry per NEW derived fact, and hashbrown stores only 7-bit
                // control tags — it RE-HASHES every element on every resize. Reserve the exact
                // upper bound for this parent's tokens × RHS forms.
                seen_ids.reserve(ts.len().saturating_mul(compiled_rhs_forms.len()));

                let first = bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, ts[0].binds);
                let slot_tables: Vec<Vec<Option<usize>>> = compiled_rhs_forms
                    .iter()
                    .map(|c| crate::rete::compiled_rhs::rhs_bind_slots(c, &first))
                    .collect();
                for tok in ts {
                    let pairs = bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
                    for (compiled, slots) in compiled_rhs_forms.iter().zip(&slot_tables) {
                    let __prhs = phase_start();
                    let derived = crate::rete::compiled_rhs::exec_compiled_rhs_at(
                        compiled, pairs, slots, sym,
                    )?;
                    phase_end("  ├ prod:compiled-rhs", __prhs);
                    // Arc 278 — the LAST split probe. build_insert_fact's own four parts summed to
                    // ~18ms instrumented while `production` read ~51ms, so ~30ms lives OUTSIDE the
                    // function. This mark brackets the dedup-and-store block. One pair per
                    // derivation, same tax as the four inside — so these five are comparable to
                    // each other and to nothing else.
                    //
                    // It used to cost two full-aggregate hashes per derivation (`contains`, then
                    // `insert`) on top of the resize ladder; both are gone — `insert` alone reports
                    // newness, and the reserve above sizes the set once. Measured on the
                    // 40,000-pair fanout cell, 3 runs each: 610 -> 489 (kill the second hash)
                    // -> 244 (reserve) ns per derivation, ranges disjoint at every step.
                    // ~120-165 ns of what remains is this mark pair itself, so the block is at
                    // the instrument's resolution — measure something else before cutting here.
                    let __pd = phase_start();
                    census_count("prod:derivations");
                    // Dedup + termination guard: only propagate truly new facts.
                    if seen_insert(&mut seen_ids, &mut seen_rest, &derived) {
                        // P12a: record the support index (first-producer-wins; or_insert_with).
                        if let Some(ref mut idx) = support {
                            idx.entry(derived.clone()).or_insert_with(|| {
                                (
                                    rule_name.to_string(),
                                    native_token_to_value(*tok, &encode_view(&wm)),
                                )
                            });
                        }
                        wm.production
                            .entry(*node_id)
                            .or_default()
                            .push(derived.clone());
                        let idx = wm.n_input + wm.derived_facts.len() as u32;
                        wm.derived_facts.push(derived);
                        next_delta.push(idx);
                    }
                    phase_end("  ├ prod:dedup-store", __pd);
                    }
                }
            }
        }

        // ── A8 instrument: census this round BEFORE the terminate check. ─────────
        // Placed here so the row reflects the round's cumulative totals after all five passes,
        // and so the LAST round is recorded too (the break below would otherwise skip it).
        // `delta_facts` still holds this round's INPUT — it is not reassigned until after the
        // terminate check, so `.len()` here is what entered, not what leaves.
        #[cfg(test)]
        FIRE_CENSUS.with(|c| {
            let mut slot = c.borrow_mut();
            let rounds = match slot.as_mut() {
                Some(r) => r,
                None => return, // not armed — every other test in the suite pays nothing
            };
            let mut beta_by_node: Vec<(i64, &'static str, usize)> = Vec::new();
            let mut beta_tokens: usize = 0;
            let mut beta_token_matches: usize = 0;
            for node_id in &node_ids {
                let toks = match wm.beta.get(node_id) {
                    Some(t) if !t.is_empty() => t,
                    _ => continue,
                };
                let kind = match get_node(&wm.network, *node_id) {
                    Some(n) => census_kind(kind_of(n)),
                    None => "?",
                };
                beta_tokens += toks.len();
                beta_token_matches += toks.iter().map(|t| t.matches.len as usize).sum::<usize>();
                beta_by_node.push((*node_id, kind, toks.len()));
            }
            // Per-node DELTA, the same shape. Needed because the beta-readers guard
            // (DESIGN-STONE-beta-is-written-only-for-readers) stops materialising `wm.beta` for
            // nodes nothing reads — so a node whose beta is deliberately empty is now invisible
            // above, and any census reading of it would be an artifact of the guard rather than a
            // measurement of the join.
            //
            // This is the SAME quantity, not a weaker proxy: before the guard, every token was
            // pushed to `wm.beta[node]` and `d_beta[node]` by the same unconditional statement
            // pair, so `Σ over rounds |d_beta[node]| == |wm.beta[node]|` at end of fire, exactly.
            // `d_beta` is also the more honest instrument for "did this join re-run per rule?" —
            // it is what the node PRODUCED, where beta was a cumulative copy of the same tokens.
            let mut d_beta_by_node: Vec<(i64, &'static str, usize)> = Vec::new();
            for node_id in &node_ids {
                let toks = match d_beta.get(node_id) {
                    Some(t) if !t.is_empty() => t,
                    _ => continue,
                };
                let kind = match get_node(&wm.network, *node_id) {
                    Some(n) => census_kind(kind_of(n)),
                    None => "?",
                };
                d_beta_by_node.push((*node_id, kind, toks.len()));
            }
            rounds.push(RoundCensus {
                round: round_no,
                delta_facts_in: this_round_in,
                alpha_nodes: wm.alpha.values().filter(|v| !v.is_empty()).count(),
                alpha_elements: wm.alpha.values().map(Vec::len).sum(),
                beta_nodes: beta_by_node.len(),
                beta_tokens,
                beta_token_matches,
                d_beta_nodes: d_beta.values().filter(|v| !v.is_empty()).count(),
                d_beta_tokens: d_beta.values().map(Vec::len).sum(),
                left_idx_tokens: left_idx
                    .values()
                    .flat_map(|m| m.values())
                    .map(Vec::len)
                    .sum(),
                right_idx_elements: right_idx
                    .values()
                    .flat_map(|m| m.values())
                    .map(Vec::len)
                    .sum(),
                production_facts: wm.production.values().map(Vec::len).sum(),
                seen_facts: seen_ids.len() + seen_rest.len(),
                network_edges: node_ids
                    .iter()
                    .filter_map(|id| get_node(&wm.network, *id))
                    .map(|n| node_children(n).len())
                    .sum(),
                beta_by_node,
                d_beta_by_node,
            });
        });
        #[cfg(test)]
        {
            round_no += 1;
        }

        phase_end("production", __pt5);

        // ── 5. Terminate or loop. ─────────────────────────────────────────────────
        let __ep = phase_start();
        let __done = next_delta.is_empty();
        if !__done {
            owned_delta = next_delta;
        }
        phase_end("  └ round:epilogue", __ep);
        if __done {
            break;
        }
    }

    // Drop alpha elements before freeze — alpha is fire-scoped scratch, not session state.
    // The wat oracle's fire-rules-spec returns an EMPTY alpha (fire-stratified, rete.wat:1817),
    // so carrying one here is a divergence as well as a cost: both engines rebuild alpha from
    // `facts` every fire and never read a frozen one. It was ~31% of fire to serialize.
    // (fire_once_session deliberately keeps its alpha — it mirrors the oracle's fire-once,
    //  which does populate it.)
    // ── Binding-cardinality census (test-only) ───────────────────────────────────────────
    // The binding-representation stone rests on ONE premise: a binding map holds 1-2 entries,
    // so an rpds trie (heap alloc + Arc + hash + pointer-chase + dealloc) is paying trie prices
    // for a pair. If the real distribution is wide, a small-vec is WORSE and the stone inverts.
    // Measured on the LIVE population at end of fire — one walk, no hot-path instrumentation to
    // distort the very thing being measured.
    #[cfg(test)]
    {
        // Buckets are PER KIND. Element and Token have different operation profiles and are
        // getting different representations (DESIGN-STONE-element-bindings-array), so a combined
        // histogram cannot answer the question either of them asks. An earlier version of this
        // census shared one bucket set across both and a design doc then claimed it "separates
        // elements from tokens" — it separated only the totals.
        fn ebucket(n: usize) -> &'static str {
            match n {
                0 => "elem-card:0",
                1 => "elem-card:1",
                2 => "elem-card:2",
                3 => "elem-card:3",
                4 => "elem-card:4",
                5 => "elem-card:5",
                6..=7 => "elem-card:6-7",
                _ => "elem-card:8+",
            }
        }
        fn tbucket(n: usize) -> &'static str {
            match n {
                0 => "tok-card:0",
                1 => "tok-card:1",
                2 => "tok-card:2",
                3 => "tok-card:3",
                4 => "tok-card:4",
                5 => "tok-card:5",
                6..=7 => "tok-card:6-7",
                _ => "tok-card:8+",
            }
        }
        for els in wm.alpha.values() {
            for el in els {
                let b = element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                census_count(ebucket(b.len()));
                census_count("bind-card:ELEMENTS");
            }
        }
        for toks in wm.beta.values() {
            for t in toks {
                census_count(tbucket(t.binds.len as usize));
                census_count("bind-card:TOKENS");
            }
        }
    }

    harvest_query_memory(&mut wm);
    let __drop = phase_start();
    wm.alpha.clear();
    // Drop ephemeral beta tokens before freeze — derived facts live in production-memory.
    // (Re-generated on every fire; never read from a frozen Session's beta-memory by native fire.)
    wm.beta.clear();
    // Pairs last — Element spans must not dangle (`DESIGN-STONE-bind-pool`).
    wm.bind_pool.clear();
    wm.bind_keys.clear();
    wm.bind_vals.clear();
    wm.bind_val_ids.clear();
    wm.match_pool.clear();
    phase_end("  └ round:drop-memories", __drop);
    phase_end("ROUND LOOP", __rounds);

    // Return persistent session with facts = input (fire-rules contract).
    // The input facts are already in wm.facts (never modified during delta fire).
    let input_facts = wm.facts.clone();
    // The Value<->native conversions and the tail are OUTSIDE the round loop and were
    // never marked — the six phases covered only ~28% of fire, so everything apportioned
    // within them was apportioned within a quarter of the work.
    let __out = phase_start();
    let __res = Ok(session_with_facts(&to_persistent(wm), input_facts));
    phase_end("OUT: to_persistent", __out);
    __res
}

// ── Public entry: native fire-rules-explain' ─────────────────────────────────

/// `(:wat::rete::fire-rules-explain <session>) -> :wat::rete::Explained`
///
/// P12a: OPT-IN diagnostic fire. Runs the EXACT same delta fixpoint as `fire-rules'` but
/// additionally records, for each derived fact, the token that produced it (and the rule name).
/// Returns `Explained { session, support }` — `session` is the same frozen Session the fast path
/// produces; `support` is a `PersistentMap<derived-fact, Support>`.
///
/// The fast `fire-rules'` / `fire-rules-spec` are byte-for-byte behaviorally identical — this is
/// purely additive (the `None`-param path is unchanged; the `Some`-param path adds provenance).
pub(crate) fn eval_fire_rules_explain(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::fire-rules-explain";
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

    // Evaluate the session argument (mirrors eval_fire_rules_native).
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();

    // Run the fixpoint with the support index recording enabled.
    let mut idx: HashMap<Value, (String, Value)> = HashMap::new();
    let session_out = fire_fixpoint_delta(&session, sym, Some(&mut idx))?;

    // Build the support PersistentMap: derived-fact → Support{rule, token_value}.
    let mut support_pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (derived_fact, (rule_name, token_value)) in idx {
        let support_value = Value::Aggregate(Arc::new(AggregateValue::record(
            (*support_class_fqdn()).clone(),
            support_names(),
            Arc::new(vec![Value::String(Arc::new(rule_name)), token_value]),
        )));
        support_pm.insert_mut(derived_fact, support_value);
    }

    // Build Explained { session, support }.
    let explained = Value::Aggregate(Arc::new(AggregateValue::record(
        (*explained_class_fqdn()).clone(),
        explained_names(),
        Arc::new(vec![
            session_out,
            // Never wrap a built trie directly — choose the arm by size.
            Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(support_pm)),
        ]),
    )));

    Ok(explained)
}
